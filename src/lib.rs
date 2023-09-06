mod score;

use crate::score::Score;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct CsfMeta {
    #[serde(rename = "BPM")]
    pub bpm: usize,
    #[serde(rename = "AudioFilePath")]
    pub audio_file_path: String,
    #[serde(rename = "AudioOffsetSec")]
    pub audio_offset: f32,
}

#[non_exhaustive]
pub struct CsfRoot {
    pub root: PathBuf,
    pub meta: CsfMeta,
    pub scores: Vec<Score>,
    pub data: HashMap<String, String>,
}

impl CsfRoot {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let meta = std::fs::read_to_string(root.join("meta.yaml"))?;
        let meta: CsfMeta = serde_yaml::from_str(meta.as_str())?;

        let data = root.join("data");
        if !data.is_dir() {
            anyhow::bail!("data directory does not exist or is not a directory");
        }

        let scores_path = root.join("scores");
        let scores = if !scores_path.is_dir() {
            anyhow::bail!("scores directory does not exist or is not a directory");
        } else {
            let data_names = get_data_names(&root)?;
            let mut scores = Vec::new();

            for path in walk_dir(&scores_path)? {
                let score_str = std::fs::read_to_string(path)?;
                let score = Score::from_str(score_str.as_str(), &data_names)?;
                scores.push(score);
            }

            scores
        };

        let audio_path = root.join(&meta.audio_file_path);
        if !audio_path.is_file() {
            anyhow::bail!("audio file does not exist or is not a file");
        }

        Ok(Self {
            root,
            meta,
            scores,
            data: HashMap::new(),
        })
    }

    pub fn new_eager(root: PathBuf) -> anyhow::Result<Self> {
        let mut this = Self::new(root)?;
        this.data = this.load_all_data()?;
        Ok(this)
    }

    pub fn get_audio_path(&self) -> PathBuf {
        self.root.join(&self.meta.audio_file_path)
    }

    pub fn find_data(&self, name: &str) -> std::io::Result<String> {
        match self.data.get(name) {
            Some(data) => Ok(data.clone()),
            None => {
                let path = self.root.join("data").join(name);
                std::fs::read_to_string(path)
            }
        }
    }

    pub fn load_all_data(&self) -> anyhow::Result<HashMap<String, String>> {
        let data_path = self.root.join("data");
        let mut data = HashMap::new();

        for path in walk_dir(&data_path)? {
            let name = path.strip_prefix(&data_path)?.to_str().unwrap().to_string();
            let data_str = std::fs::read_to_string(path)?;
            data.insert(name, data_str);
        }

        Ok(data)
    }
}

fn walk_dir(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            paths.extend(walk_dir(&path)?);
        } else {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn get_data_names(root: &Path) -> anyhow::Result<Vec<String>> {
    let data_path = root.join("data");
    let mut names = Vec::new();

    for path in walk_dir(&data_path)? {
        let name = path.strip_prefix(&data_path)?;
        let name = name.to_str().unwrap();
        names.push(name.to_string());
    }

    Ok(names)
}
