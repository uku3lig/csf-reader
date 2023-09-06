use crate::CsfRoot;
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub fn play(root: &CsfRoot) -> anyhow::Result<()> {
    let delay = Duration::from_secs_f32(root.meta.audio_offset);
    let sec_per_beat = Duration::from_secs_f32(60.0 / root.meta.bpm as f32);

    let _stream = play_audio(root)?;
    std::thread::sleep(delay);
    println!("animation starts");

    for i in 0..80 {
        std::thread::sleep(sec_per_beat);
        println!("beat {}", i);
    }

    Ok(())
}

fn play_audio(root: &CsfRoot) -> anyhow::Result<OutputStream> {
    let audio_path = root.root.join(&root.meta.audio_file_path);
    let file = File::open(audio_path)?;
    let source = Decoder::new(BufReader::new(file))?;

    let (stream, stream_handle) = OutputStream::try_default()?;
    stream_handle.play_raw(source.convert_samples())?;

    Ok(stream)
}
