mod widgets;

use std::io::{self, Stdout};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use enigo::{Enigo, Mouse, Settings};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::state::{AppState, CPS_MIN, Field, RecordingState};

pub fn run(state: Arc<Mutex<AppState>>) -> io::Result<()> {
    let mut terminal = setup()?;
    let res = event_loop(&mut terminal, &state);
    restore(&mut terminal)?;
    res
}

fn setup() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &Arc<Mutex<AppState>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            if let Ok(s) = state.lock() {
                widgets::draw(f, &s);
            }
        })?;

        if state.lock().map(|s| s.should_quit).unwrap_or(true) {
            return Ok(());
        }

        // ~60 fps.
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(key.code, state);
                }
            }
        }
    }
}

fn handle_key(code: KeyCode, state: &Arc<Mutex<AppState>>) {
    let mut s = match state.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    if s.edit_buffer.is_some() {
        handle_edit_key(&mut s, code);
        return;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => s.should_quit = true,
        KeyCode::Up => s.focus = s.focus.prev(),
        KeyCode::Down | KeyCode::Tab => s.focus = s.focus.next(),
        KeyCode::BackTab => s.focus = s.focus.prev(),
        KeyCode::Left => {
            adjust(&mut s, -1);
            persist(&mut s);
        }
        KeyCode::Right => {
            adjust(&mut s, 1);
            persist(&mut s);
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if matches!(s.focus, Field::Cps | Field::Jitter) {
                s.edit_buffer = Some(String::new());
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            s.recording = RecordingState::WaitingForInput;
            s.focus = Field::Hotkey;
            s.log("Recording started - press key/mouse button");
        }
        _ => {}
    }
}

fn handle_edit_key(s: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if let Some(buf) = s.edit_buffer.as_mut() {
                buf.push(c);
            }
        }
        // Only for CPS, and only once.
        KeyCode::Char('.') if s.focus == Field::Cps => {
            if let Some(buf) = s.edit_buffer.as_mut() {
                if !buf.contains('.') {
                    buf.push('.');
                }
            }
        }
        KeyCode::Backspace => {
            if let Some(buf) = s.edit_buffer.as_mut() {
                buf.pop();
            }
        }
        KeyCode::Enter => commit_edit(s),
        KeyCode::Esc => s.edit_buffer = None,
        _ => {}
    }
}

fn commit_edit(s: &mut AppState) {
    let buf = match s.edit_buffer.take() {
        Some(b) => b,
        None => return,
    };
    if buf.is_empty() {
        return;
    }

    match s.focus {
        Field::Cps => {
            if let Ok(v) = buf.parse::<f64>() {
                s.cps = v.max(CPS_MIN);
                persist(s);
            }
        }
        Field::Jitter => {
            if let Ok(v) = buf.parse::<u64>() {
                s.jitter_ms = v;
                persist(s);
            }
        }
        _ => {}
    }
}

fn persist(s: &mut AppState) {
    if let Err(err) = crate::config::save(s) {
        s.log(format!("Config save failed: {err}"));
    }
}

fn adjust(s: &mut AppState, dir: i32) {
    match s.focus {
        Field::Cps => {
            s.cps = (s.cps + 0.5 * dir as f64).max(CPS_MIN);
        }
        Field::Button => {
            s.button = if dir > 0 { s.button.next() } else { s.button.prev() };
        }
        Field::Jitter => {
            let step = 5i64 * dir as i64;
            s.jitter_ms = (s.jitter_ms as i64 + step).max(0) as u64;
        }
        Field::Position => {
            if s.fix_position.is_some() {
                s.fix_position = None;
                s.log("Position released");
            } else {
                match current_mouse_pos() {
                    Some((x, y)) => {
                        s.fix_position = Some((x, y));
                        s.log(format!("Position locked ({x}, {y})"));
                    }
                    None => s.log("Could not read mouse position"),
                }
            }
        }
        // Hotkey is set via recording (R), not ↑/↓.
        Field::Hotkey => {}
    }
}

fn current_mouse_pos() -> Option<(i32, i32)> {
    let enigo = Enigo::new(&Settings::default()).ok()?;
    enigo.location().ok()
}
