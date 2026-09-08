use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{AppState, HotkeyInput, MouseBtn};

/// Runtime status (active, click_count, log) is deliberately not persisted.
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cps: f64,
    pub button: MouseBtn,
    pub jitter_ms: u64,
    pub fix_position: Option<(i32, i32)>,
    pub hotkey: HotkeyInput,
}

fn config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let mut path = PathBuf::from(home);
    path.push(".oxiclick");
    Some(path)
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|mut p| {
        p.push("config.json");
        p
    })
}

pub fn load() -> Option<Config> {
    let data = fs::read_to_string(config_path()?).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save(state: &AppState) -> io::Result<()> {
    let cfg = Config {
        cps: state.cps,
        button: state.button,
        jitter_ms: state.jitter_ms,
        fix_position: state.fix_position,
        hotkey: state.hotkey,
    };

    let dir = config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home directory"))?;
    fs::create_dir_all(&dir)?;

    let data = serde_json::to_string_pretty(&cfg)?;
    fs::write(dir.join("config.json"), data)
}
