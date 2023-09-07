use crate::score::{Command, DisplayCommand, MeasureCommand, Score};
use crate::CsfRoot;
use anyhow::Context;
use crossterm::event::Event;
use crossterm::{event, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use rodio::{Decoder, OutputStream, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use tokio::time::Instant;

pub struct IndexedScore {
    pub measures: Vec<DisplayMeasure>,
}

pub struct DisplayMeasure {
    pub items: Vec<DisplayItem>,
}

pub struct DisplayItem {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub content: String,
}

impl IndexedScore {
    fn from_score(root: &CsfRoot, score: &Score) -> Self {
        let mut flip_vertical = false;
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;

        let mut measures = Vec::new();

        for measure in &score.measures {
            let mut items = Vec::new();

            for command in measure.commands.iter() {
                match command {
                    MeasureCommand::Command(command) => match command {
                        Command::MoveTo(x_dest, y_dest) => {
                            x = *x_dest;
                            y = *y_dest;
                        }
                        Command::ZIndex(z_dest) => {
                            z = *z_dest;
                        }
                        Command::FlipVertical(v) => {
                            flip_vertical = *v;
                        }
                    },
                    MeasureCommand::DisplayCommand(display_command) => {
                        let content = match display_command {
                            DisplayCommand::DataDisplayCommand(data_name) => {
                                root.find_data(data_name).unwrap_or(data_name.clone())
                            }
                            DisplayCommand::InlineDisplayCommand(data) => data.clone(),
                        };

                        let content = if flip_vertical {
                            flip_string_vert(content)
                        } else {
                            content
                        };

                        items.push(DisplayItem { x, y, z, content });
                    }
                }
            }

            measures.push(DisplayMeasure { items });
        }

        Self { measures }
    }
}

impl DisplayItem {
    fn offset_content_lines(&self) -> Vec<String> {
        let lines = self
            .content
            .lines()
            .map(|line| format!("{}{}", " ".repeat(self.x as usize), line))
            .collect::<Vec<_>>();

        let content = format!("{}{}", "\n".repeat(self.y as usize), lines.join("\n"));
        content.lines().map(String::from).collect()
    }
}

pub fn play_sync(root: CsfRoot) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { play(root).await })
}

pub async fn play(root: CsfRoot) -> anyhow::Result<()> {
    let delay = Duration::from_secs_f32(root.meta.audio_offset);
    let sec_per_measure = 60.0 / root.meta.bpm as f32 * 4.0;

    let measure_count = root
        .scores
        .iter()
        .map(|s| s.measures.len())
        .max()
        .context("no measures found")?;

    let scores = root
        .scores
        .iter()
        .map(|s| IndexedScore::from_score(&root, s))
        .collect::<Vec<_>>();

    let mut terminal = {
        let mut stdout = std::io::stdout();
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All)
        )?;
        Terminal::new(CrosstermBackend::new(stdout))?
    };

    let framerate = Duration::from_secs_f32(1.0 / 60.0);
    let start = Instant::now() + delay;
    let mut interval = tokio::time::interval_at(start, framerate);

    let _stream = play_audio(&root)?;

    loop {
        interval.tick().await;

        let current_measure = (Instant::now() - start).as_secs_f32() / sec_per_measure;
        let measure_index = current_measure as usize;

        if measure_index > measure_count {
            break;
        }

        let mut items = scores
            .iter()
            .filter_map(|score| score.measures.get(measure_index))
            .filter_map(|measure| {
                if measure.items.is_empty() {
                    return None;
                }

                let item_index = measure.items.len() as f32 * (current_measure % 1.0);
                let item_index = item_index.floor() as usize;

                measure.items.get(item_index)
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            continue;
        }

        items.sort_by_key(|item| item.z);
        let content = flatten_items(items);

        terminal.draw(|frame| {
            let paragraph = Paragraph::new(content);
            frame.render_widget(paragraph, frame.size());
        })?;

        if event::poll(Duration::from_millis(0)).context("event poll failed")? {
            if let Event::Key(_) = event::read().context("event read failed")? {
                break;
            }
        }
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn flip_string_vert(content: String) -> String {
    let mut lines = content.lines().collect::<Vec<_>>();
    lines.reverse();
    lines.join("\n")
}

// "prints" every item on top of each other, keeping transparency in mind
fn flatten_items(items: Vec<&DisplayItem>) -> String {
    let mut items = VecDeque::from(items);

    let mut lines = if let Some(first) = items.pop_front() {
        first.offset_content_lines()
    } else {
        return String::new();
    };

    for item in items {
        let item_lines = item.offset_content_lines();
        for (i, item_line) in item_lines.iter().enumerate() {
            match lines.get_mut(i) {
                Some(line) => *line = overlap_strings(line, item_line),
                None => lines.push(item_line.clone()),
            }
        }
    }

    lines.join("\n")
}

fn overlap_strings(orig: &str, second: &str) -> String {
    if second.trim().is_empty() {
        return orig.into();
    }

    if orig.trim().is_empty() {
        return second.into();
    }

    let mut orig_chars = orig.chars().collect::<Vec<_>>();
    let second_chars = second.chars().collect::<Vec<_>>();

    for (i, second_char) in second_chars.iter().enumerate() {
        match orig_chars.get_mut(i) {
            Some(orig_char) => {
                if *second_char != ' ' {
                    *orig_char = *second_char
                }
            }
            None => orig_chars.push(*second_char),
        }
    }

    orig_chars.into_iter().collect()
}

fn play_audio(root: &CsfRoot) -> anyhow::Result<OutputStream> {
    let audio_path = root.root.join(&root.meta.audio_file_path);
    let file = File::open(audio_path)?;
    let source = Decoder::new(BufReader::new(file))?;

    let (stream, stream_handle) = OutputStream::try_default()?;
    stream_handle.play_raw(source.convert_samples())?;

    Ok(stream)
}
