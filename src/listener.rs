use std::sync::{Arc, Mutex};

use rdev::{Event, EventType};

use crate::state::{AppState, HotkeyInput, RecordingState};

pub fn run(state: Arc<Mutex<AppState>>) {
    let cb_state = Arc::clone(&state);
    let callback = move |event: Event| {
        let input = match event.event_type {
            EventType::KeyPress(key) => HotkeyInput::Key(key),
            EventType::ButtonPress(button) => HotkeyInput::MouseButton(button),
            _ => return,
        };

        let mut s = match cb_state.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        match s.recording {
            RecordingState::WaitingForInput => {
                s.hotkey = input;
                s.recording = RecordingState::Idle;
                let label = input.label();
                s.log(format!("Hotkey → {label}"));
                if let Err(err) = crate::config::save(&s) {
                    s.log(format!("Config save failed: {err}"));
                }
            }
            RecordingState::Idle => {
                if input == s.hotkey {
                    s.active = !s.active;
                    let on = s.active;
                    let label = s.hotkey.label();
                    s.log(format!(
                        "{} ({label})",
                        if on { "ON " } else { "OFF" }
                    ));
                }
            }
        }
    };

    if let Err(err) = rdev::listen(callback) {
        if let Ok(mut s) = state.lock() {
            s.log(format!("Listener error: {err:?}"));
        }
    }
}
