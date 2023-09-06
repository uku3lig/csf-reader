use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct CsfMeta {
    #[serde(rename = "BPM")]
    pub bpm: usize,
    #[serde(rename = "AudioFilePath")]
    pub audio_file_path: String,
    #[serde(rename = "AudioOffset")]
    pub audio_offset: f32,
}

#[non_exhaustive]
pub struct CsfRoot {
    pub root: PathBuf,
    pub meta: CsfMeta,
}

impl CsfRoot {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let meta = std::fs::read_to_string(root.join("meta.yaml"))?;
        let meta: CsfMeta = serde_yaml::from_str(meta.as_str())?;

        let data = root.join("data");
        if !data.is_dir() {
            anyhow::bail!("data directory does not exist or is not a directory");
        }

        let scores = root.join("scores");
        if !scores.is_dir() {
            anyhow::bail!("scores directory does not exist or is not a directory");
        }

        let audio_path = root.join(&meta.audio_file_path);
        if !audio_path.is_file() {
            anyhow::bail!("audio file does not exist or is not a file");
        }

        Ok(Self { root, meta })
    }

    pub fn get_audio_path(&self) -> PathBuf {
        self.root.join(&self.meta.audio_file_path)
    }

    pub fn find_data(&self, name: &str) -> std::io::Result<String> {
        let path = self.root.join("data").join(name);

        std::fs::read_to_string(path)
    }
}
