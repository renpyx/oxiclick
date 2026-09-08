# OxiClick

A fast, configurable TUI autoclicker for **Linux (X11)** and **Windows**, written in Rust.

- Terminal UI (ratatui) with live configuration while running
- Global toggle hotkey - recordable at runtime (keyboard **or** mouse side buttons)
- Adjustable clicks per second (CPS) with no upper limit, plus direct value entry
- Optional jitter for less robotic timing
- Optional fixed click position
- Settings persisted across restarts in `~/.oxiclick/config.json`

---

## Motivation

Seit ich unter Linux (KDE, X11) unterwegs bin, hatte ich keinen brauchbaren
Autoclicker mehr zur Hand. Eine GUI fand ich für so ein simples Tool unnötig,
daher ist eine TUI daraus geworden.

Das Projekt war für mich in erster Linie ein Ausprobieren von Rust - ein
großer Teil der Implementierung ist KI-unterstützt entstanden. Im Zuge dessen
habe ich mich entschieden, den Fokus stattdessen auf Go zu legen: Der
Arbeitsmarkt dafür ist in meiner Region deutlich größer, gerade im
DevOps-Umfeld, wo Go klar besser positioniert ist als Rust.

---

## Controls

| Key | Action |
|---|---|
| `↑` / `↓` | Select field |
| `Tab` / `Shift+Tab` | Select field (down / up) |
| `←` / `→` | Change value of the selected field (CPS ±0.5, Jitter ±5) |
| `E` | Edit selected numeric field directly (CPS / Jitter) - type a value, `Enter` to apply, `Esc` to cancel |
| `R` | Record a new toggle hotkey (press any key or mouse button) |
| `Q` | Quit |
| *toggle hotkey* | Start/stop clicking (default **F6**) |

The toggle hotkey works globally, even when the terminal is not focused.

---

## Requirements

- Rust toolchain (edition 2024) - install via [rustup](https://rustup.rs)

### Linux (X11)

`enigo` and `rdev` link against X11 system libraries. Install the development packages
before building:

```bash
sudo apt install libxi-dev libxtst-dev libxdo-dev libx11-dev
```

| Package | Needed by |
|---|---|
| `libxi-dev` | rdev / x11 (`xi.pc`) |
| `libxtst-dev` | XTest (rdev) |
| `libxdo-dev` | enigo (mouse control) |
| `libx11-dev` | base X11 |

> Note: This targets **X11**. Under a Wayland session the global listener / synthetic
> clicks may not work as expected.

### Windows

No extra system libraries are required - `enigo` and `rdev` use the Win32 API directly.
Just install Rust (the MSVC toolchain is recommended) and build.

---

## Build

```bash
# Debug build
cargo build

# Optimized release build → target/release/oxiclick (oxiclick.exe on Windows)
cargo build --release
```

The build process is the same on both platforms; only the Linux system libraries above
are an extra prerequisite.

---

## Install

### Linux

**Option A - `cargo install` (simplest):**

```bash
cargo install --path .
```

Installs to `~/.cargo/bin/oxiclick`, which is already on your `PATH` if you use rustup.

**Option B - into `~/.local/bin`:**

```bash
cargo build --release
install -Dm755 target/release/oxiclick ~/.local/bin/oxiclick
```

Make sure `~/.local/bin` is on your `PATH` (add `export PATH="$HOME/.local/bin:$PATH"` to
`~/.bashrc` if needed).

### Windows

```powershell
cargo install --path .
```

...or copy `target\release\oxiclick.exe` anywhere on your `PATH`.

---

## Usage

```bash
oxiclick                          # start with saved/default settings
oxiclick --cps 15 --button left   # override starting values
oxiclick --jitter 20              # 20 ms jitter
```

CLI flags override the saved configuration for that session.

| Flag | Description |
|---|---|
| `--cps <f64>` | Clicks per second (min 0.1, no upper limit) |
| `--button <left\|right\|middle>` | Mouse button to click |
| `--jitter <ms>` | Random jitter in milliseconds (0 = off) |

---

## Configuration

Settings are stored as JSON at:

- **Linux:** `~/.oxiclick/config.json`
- **Windows:** `%USERPROFILE%\.oxiclick\config.json`

Persisted: CPS, mouse button, jitter, fixed position, and the toggle hotkey. The file is
written automatically whenever you change a value or record a new hotkey, and loaded on
startup.

---

## License

MIT - see [LICENSE](LICENSE).
