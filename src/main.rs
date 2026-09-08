mod clicker;
mod config;
mod listener;
mod state;
mod tui;

use std::sync::{Arc, Mutex};

use clap::{Parser, ValueEnum};

use state::{AppState, CPS_MIN, MouseBtn};

/// TUI-based autoclicker for Windows and Linux (X11).
#[derive(Parser, Debug)]
#[command(name = "oxiclick", version, about)]
struct Cli {
    /// Clicks per second (0.1 – 100.0).
    #[arg(long)]
    cps: Option<f64>,

    /// Mouse button to click.
    #[arg(long, value_enum)]
    button: Option<ButtonArg>,

    /// Jitter in milliseconds (0 = off).
    #[arg(long)]
    jitter: Option<u64>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ButtonArg {
    Left,
    Right,
    Middle,
}

impl From<ButtonArg> for MouseBtn {
    fn from(b: ButtonArg) -> Self {
        match b {
            ButtonArg::Left => MouseBtn::Left,
            ButtonArg::Right => MouseBtn::Right,
            ButtonArg::Middle => MouseBtn::Middle,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let mut initial = AppState::default();

    let loaded = config::load();
    if let Some(cfg) = loaded.as_ref() {
        initial.cps = cfg.cps;
        initial.button = cfg.button;
        initial.jitter_ms = cfg.jitter_ms;
        initial.fix_position = cfg.fix_position;
        initial.hotkey = cfg.hotkey;
    }

    if let Some(cps) = cli.cps {
        initial.cps = cps.max(CPS_MIN);
    }
    if let Some(button) = cli.button {
        initial.button = button.into();
    }
    if let Some(jitter) = cli.jitter {
        initial.jitter_ms = jitter;
    }
    initial.log(if loaded.is_some() {
        "OxiClick started (config loaded)"
    } else {
        "OxiClick started"
    });

    let state = Arc::new(Mutex::new(initial));

    {
        let state = Arc::clone(&state);
        std::thread::spawn(move || clicker::run(state));
    }

    // rdev::listen blocks forever, so it needs its own thread.
    {
        let state = Arc::clone(&state);
        std::thread::spawn(move || listener::run(state));
    }

    if let Err(err) = tui::run(state) {
        eprintln!("TUI error: {err}");
    }
}
