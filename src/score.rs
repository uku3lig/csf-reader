use anyhow::{anyhow, bail, Result};

#[derive(Debug, Clone)]
pub struct Score {
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone)]
pub struct Measure {
    pub commands: Vec<MeasureCommand>,
}

#[derive(Debug, Clone)]
pub enum MeasureCommand {
    DisplayCommand(DisplayCommand),
    Command(Command),
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    DataDisplayCommand(String),
    InlineDisplayCommand(String),
}

#[derive(Debug, Copy, Clone)]
pub enum Command {
    MoveTo(i32, i32),
    ZIndex(i32),
    FlipVertical(bool),
}

impl Score {
    pub fn from_str(s: &str, data_names: &[String]) -> Result<Self> {
        parse_score(s, data_names)
    }
}

fn parse_score(lines: &str, data_names: &[String]) -> Result<Score> {
    let mut measures = Vec::new();
    for measure in lines.split("---") {
        measures.push(parse_measure(measure, data_names)?);
    }

    Ok(Score { measures })
}

fn parse_measure(lines: &str, data_names: &[String]) -> Result<Measure> {
    let mut commands = Vec::new();
    for line in lines.lines() {
        if line.starts_with('/') || line.trim().is_empty() {
            continue;
        } else if line.starts_with('#') {
            commands.push(MeasureCommand::Command(parse_command(line)?));
        } else {
            commands.push(MeasureCommand::DisplayCommand(parse_display_command(
                line, data_names,
            )?));
        }
    }

    Ok(Measure { commands })
}

fn parse_display_command(line: &str, data_names: &[String]) -> Result<DisplayCommand> {
    if let Some(line) = line.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Ok(DisplayCommand::InlineDisplayCommand(line.to_string()))
    } else if data_names.contains(&line.into()) {
        Ok(DisplayCommand::DataDisplayCommand(line.to_string()))
    } else {
        Ok(DisplayCommand::InlineDisplayCommand(line.to_string()))
    }
}

fn parse_command(line: &str) -> Result<Command> {
    let line = &line[1..]; // strip initial `#`
    let mut parts = line.split_whitespace();
    let command = parts.next().ok_or(anyhow!("empty command"))?;

    match command {
        "MOVETO" => {
            let x = parts.next().ok_or(anyhow!("empty x position"))?.parse()?;
            let y = parts.next().ok_or(anyhow!("empty y position"))?.parse()?;
            Ok(Command::MoveTo(x, y))
        }
        "ZINDEX" => {
            let z = parts.next().ok_or(anyhow!("empty z index"))?.parse()?;
            Ok(Command::ZIndex(z))
        }
        "FLIP" => {
            if parts.next().ok_or(anyhow!("empty flip direction"))? == "vertical" {
                let b = match parts.next().ok_or(anyhow!("empty flip value"))? {
                    "on" => true,
                    "off" => false,
                    _ => bail!("invalid flip value"),
                };
                Ok(Command::FlipVertical(b))
            } else {
                bail!("invalid flip command");
            }
        }
        _ => bail!("unknown command: {}", command),
    }
}
