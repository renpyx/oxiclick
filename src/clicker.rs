use std::sync::{Arc, Mutex};
use std::time::Duration;

use enigo::{Coordinate, Direction, Enigo, Mouse, Settings};
use rand::Rng;

use crate::state::AppState;

pub fn run(state: Arc<Mutex<AppState>>) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(err) => {
            if let Ok(mut s) = state.lock() {
                s.log(format!("Enigo init failed: {err}"));
            }
            return;
        }
    };

    let mut rng = rand::thread_rng();

    loop {
        let (active, cps, button, jitter_ms, fix_position, should_quit) = {
            let s = match state.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            (
                s.active,
                s.cps,
                s.button,
                s.jitter_ms,
                s.fix_position,
                s.should_quit,
            )
        };

        if should_quit {
            return;
        }

        if !active {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        if let Some((x, y)) = fix_position {
            if let Err(err) = enigo.move_mouse(x, y, Coordinate::Abs) {
                if let Ok(mut s) = state.lock() {
                    s.log(format!("move_mouse error: {err}"));
                }
            }
        }

        match enigo.button(button.to_enigo(), Direction::Click) {
            Ok(()) => {
                if let Ok(mut s) = state.lock() {
                    s.click_count += 1;
                }
            }
            Err(err) => {
                if let Ok(mut s) = state.lock() {
                    s.log(format!("Click error: {err}"));
                }
            }
        }

        std::thread::sleep(sleep_duration(cps, jitter_ms, &mut rng));
    }
}

fn sleep_duration<R: Rng>(cps: f64, jitter_ms: u64, rng: &mut R) -> Duration {
    let base_ms = 1000.0 / cps.max(0.001);
    let delta_ms = if jitter_ms == 0 {
        0.0
    } else {
        let j = jitter_ms as f64;
        rng.gen_range(-j..=j)
    };
    let total = (base_ms + delta_ms).max(1.0);
    Duration::from_secs_f64(total / 1000.0)
}
