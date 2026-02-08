use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::game::types::Statistics;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub ai_level: i32,
    pub animation_speed: f64,
    // Optional persisted window geometry (may be absent on first run or unsupported platforms)
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ai_level: 2,
            animation_speed: 0.2,
            window_width: None,
            window_height: None,
        }
    }
}

fn project_config_dir() -> Option<PathBuf> {
    // Use application-specific qualifiers; these determine platform default locations.
    ProjectDirs::from("io.github", "laserlicht", "TowerOops").map(|p| p.config_dir().to_path_buf())
}

fn ensure_config_dir() -> io::Result<PathBuf> {
    if let Some(dir) = project_config_dir() {
        fs::create_dir_all(&dir)?;
        Ok(dir)
    } else {
        // Fallback to current directory
        Ok(std::env::current_dir()?)
    }
}

fn settings_path() -> io::Result<PathBuf> {
    let mut p = ensure_config_dir()?;
    p.push("settings.json");
    Ok(p)
}

fn statistics_path() -> io::Result<PathBuf> {
    let mut p = ensure_config_dir()?;
    p.push("statistics.json");
    Ok(p)
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    if let Ok(p) = path {
        if p.is_file() {
            match File::open(&p).and_then(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                let cfg: Settings = serde_json::from_str(&s)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(cfg)
            }) {
                Ok(cfg) => return cfg,
                Err(_) => return Settings::default(),
            }
        }
    }
    Settings::default()
}

pub fn save_settings(s: &Settings) -> io::Result<()> {
    let p = settings_path()?;
    let data =
        serde_json::to_string_pretty(s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut f = File::create(&p)?;
    f.write_all(data.as_bytes())?;
    Ok(())
}

pub fn load_statistics() -> Statistics {
    let path = statistics_path();
    if let Ok(p) = path {
        if p.is_file() {
            match File::open(&p).and_then(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                let st: Statistics = serde_json::from_str(&s)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(st)
            }) {
                Ok(st) => return st,
                Err(_) => return Statistics::default(),
            }
        }
    }
    Statistics::default()
}

pub fn save_statistics(st: &Statistics) -> io::Result<()> {
    let p = statistics_path()?;
    let data =
        serde_json::to_string_pretty(st).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut f = File::create(&p)?;
    f.write_all(data.as_bytes())?;
    Ok(())
}
