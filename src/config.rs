use config::{Config, ConfigError};
use std::fs;

pub(crate) struct Configuration {
    config: Config,
}

impl Configuration {
    pub(crate) fn open() -> Result<Self, ConfigError> {
        Ok(Self {
            config: Config::builder()
                .add_source(config::File::with_name("config"))
                .add_source(config::Environment::with_prefix("RECORDBOX"))
                .build()?,
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
