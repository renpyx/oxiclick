use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

pub const LOG_CAPACITY: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    WaitingForInput,
}

/// A key or a mouse button (including side buttons, via `rdev::Button::Unknown`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HotkeyInput {
    Key(rdev::Key),
    MouseButton(rdev::Button),
}

impl HotkeyInput {
    pub fn label(&self) -> String {
        match self {
            HotkeyInput::Key(k) => format!("{k:?}"),
            HotkeyInput::MouseButton(b) => match b {
                rdev::Button::Left => "Mouse Left".to_string(),
                rdev::Button::Right => "Mouse Right".to_string(),
                rdev::Button::Middle => "Mouse Middle".to_string(),
                rdev::Button::Unknown(n) => format!("Mouse Button{n}"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseBtn {
    Left,
    Right,
    Middle,
}

impl MouseBtn {
    pub fn label(&self) -> &'static str {
        match self {
            MouseBtn::Left => "Left",
            MouseBtn::Right => "Right",
            MouseBtn::Middle => "Middle",
        }
    }

    pub fn to_enigo(self) -> enigo::Button {
        match self {
            MouseBtn::Left => enigo::Button::Left,
            MouseBtn::Right => enigo::Button::Right,
            MouseBtn::Middle => enigo::Button::Middle,
        }
    }

    pub fn next(self) -> Self {
        match self {
            MouseBtn::Left => MouseBtn::Middle,
            MouseBtn::Middle => MouseBtn::Right,
            MouseBtn::Right => MouseBtn::Left,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            MouseBtn::Left => MouseBtn::Right,
            MouseBtn::Right => MouseBtn::Middle,
            MouseBtn::Middle => MouseBtn::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Cps,
    Button,
    Jitter,
    Position,
    Hotkey,
}

impl Field {
    pub const ALL: [Field; 5] = [
        Field::Cps,
        Field::Button,
        Field::Jitter,
        Field::Position,
        Field::Hotkey,
    ];

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&f| f == self).unwrap();
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&f| f == self).unwrap();
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Guards against division by ~0 in the clicker sleep; no upper limit.
pub const CPS_MIN: f64 = 0.1;

pub struct AppState {
    pub active: bool,
    pub cps: f64,
    pub button: MouseBtn,
    pub jitter_ms: u64,
    pub fix_position: Option<(i32, i32)>,
    pub hotkey: HotkeyInput,
    pub recording: RecordingState,
    pub click_count: u64,
    pub status_log: VecDeque<String>,
    pub should_quit: bool,
    pub focus: Field,
    /// `None` = not in edit mode.
    pub edit_buffer: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active: false,
            cps: 10.0,
            button: MouseBtn::Left,
            jitter_ms: 0,
            fix_position: None,
            hotkey: HotkeyInput::Key(rdev::Key::F6),
            recording: RecordingState::Idle,
            click_count: 0,
            status_log: VecDeque::with_capacity(LOG_CAPACITY),
            should_quit: false,
            focus: Field::Cps,
            edit_buffer: None,
        }
    }
}

impl AppState {
    pub fn log(&mut self, msg: impl Into<String>) {
        let line = format!("{}  {}", now_hms(), msg.into());
        if self.status_log.len() >= LOG_CAPACITY {
            self.status_log.pop_front();
        }
        self.status_log.push_back(line);
    }
}

/// No external time crate.
fn now_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    format!("{h:02}:{m:02}:{s:02}")
}
