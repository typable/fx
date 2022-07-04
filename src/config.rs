use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::Error;
use crate::Result;
use crate::APP_NAME;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    // The default app for opening files
    pub default: Option<String>,
    // The apps used for different file extensions
    pub apps: Option<HashMap<String, Vec<String>>>,
}

impl Config {
    pub fn acquire() -> Result<Self> {
        match config_path() {
            Some(config_path) => match fs::read_to_string(config_path) {
                Ok(raw) => match toml::from_str(&raw) {
                    Ok(config) => Ok(config),
                    Err(err) => {
                        return Err(Error::new(&format!("Invalid config file! Reason: {}", err)))
                    }
                },
                Err(_) => Ok(Config::default()),
            },
            None => Err(Error::new("Unable to determine config path!")),
        }
    }
    // Get app for file extension
    pub fn get_app(&self, file_ext: &str) -> Option<String> {
        if self.apps.is_some() {
            for (app, exts) in self.apps.as_ref().unwrap() {
                if exts.contains(&file_ext.to_string().to_lowercase()) {
                    return Some(app.clone());
                }
            }
        }
        self.default.clone()
    }
}

fn config_path() -> Option<PathBuf> {
    match dirs::config_dir() {
        Some(mut config_dir) => {
            config_dir.push(APP_NAME);
            config_dir.push("config.toml");
            Some(config_dir)
        }
        None => None,
    }
}
