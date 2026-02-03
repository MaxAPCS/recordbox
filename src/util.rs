use config::{Config, ConfigError};
use std::fs;

use crate::autotag;

pub(crate) struct Configuration {
    config: Config,
    pub(crate) metadatasources: autotag::MetadataSources,
}

impl Configuration {
    pub(crate) fn open() -> Result<Self, ConfigError> {
        Ok(Self {
            config: Config::builder()
                .add_source(config::File::with_name("config"))
                .add_source(config::Environment::with_prefix("RECORDBOX"))
                .build()?,
            metadatasources: autotag::MetadataSources::default(),
        })
    }

    pub(crate) fn get_directory(&self, dir: &str) -> Result<std::path::PathBuf, String> {
        match self.config.get_string(dir) {
            Err(_) => Err(format!("Directory '{}' unset", dir)),
            Ok(path) => match fs::canonicalize(&path) {
                Err(_) => Err(format!("Directory '{}' does not exist", dir)),
                Ok(path) if !path.is_dir() => Err(format!("Directory '{}' is of wrong type", dir)),
                Ok(path) => Ok(path),
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Metadata {
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) artists: Vec<String>,
    pub(crate) album: Option<String>,
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) genres: Vec<String>,
    pub(crate) lyrics: Option<String>,
}

impl From<mp4ameta::Tag> for Metadata {
    fn from(value: mp4ameta::Tag) -> Self {
        Self {
            title: value.title().map(|a| a.to_string()),
            artists: value.artists().map(|a| a.to_string()).collect(),
            album: value.album().map(|a| a.to_string()),
            date: value.year().map(|a| a.to_string()),
            genres: value.genres().map(|a| a.to_string()).collect(),
            lyrics: value.lyrics().map(|a| a.to_string()),
        }
    }
}

impl Metadata {
    pub(crate) fn apply(self, tag: &mut mp4ameta::Tag) {
        if let Some(title) = self.title {
            tag.set_title(title);
        }
        if !self.artists.is_empty() {
            tag.set_artists(self.artists);
        }
        if let Some(album) = self.album {
            tag.set_album(album);
        }
        if let Some(date) = self.date {
            tag.set_year(date);
        }
        if !self.genres.is_empty() {
            tag.set_genres(self.genres);
        }
        if let Some(lyrics) = self.lyrics {
            tag.set_lyrics(lyrics);
        }
    }
}
