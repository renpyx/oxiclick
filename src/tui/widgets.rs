use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::state::{AppState, Field, RecordingState};

pub fn draw(f: &mut Frame, state: &AppState) {
    let outer = Block::default().borders(Borders::ALL).title(" OxiClick ");
    let inner = outer.inner(f.size());
    f.render_widget(outer, f.size());

    let chunks = Layout::vertical([
        Constraint::Length(1), // status
        Constraint::Length(6), // settings
        Constraint::Min(3),    // log
        Constraint::Length(1), // help line
    ])
    .split(inner);

    draw_status(f, chunks[0], state);
    draw_settings(f, chunks[1], state);
    draw_log(f, chunks[2], state);
    draw_help(f, chunks[3]);
}

fn draw_status(f: &mut Frame, area: Rect, state: &AppState) {
    let (symbol, label, color) = if state.active {
        ("▶", "ACTIVE  ", Color::Green)
    } else {
        ("■", "INACTIVE", Color::DarkGray)
    };
    let line = Line::from(vec![
        Span::raw(" Status:  "),
        Span::styled(
            format!("{symbol} {label}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("   Clicks: {}", state.click_count)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_settings(f: &mut Frame, area: Rect, state: &AppState) {
    let pos_label = match state.fix_position {
        Some((x, y)) => format!("locked ({x}, {y})"),
        None => "free".to_string(),
    };

    let hotkey_label = if state.recording == RecordingState::WaitingForInput {
        "⏺ Recording... (press key/mouse button)".to_string()
    } else {
        state.hotkey.label()
    };

    let rows = [
        (Field::Cps, "CPS", format!("{:.1}", state.cps)),
        (Field::Button, "Button", state.button.label().to_string()),
        (Field::Jitter, "Jitter", format!("{} ms", state.jitter_ms)),
        (Field::Position, "Position", pos_label),
        (Field::Hotkey, "Toggle Key", hotkey_label),
    ];

    let items: Vec<ListItem> = rows
        .iter()
        .map(|(field, name, value)| {
            let focused = state.focus == *field;
            let marker = if focused { "›" } else { " " };

            let editing = focused && state.edit_buffer.is_some();
            let display = match &state.edit_buffer {
                Some(buf) if focused => format!("{buf}_"),
                _ => value.clone(),
            };

            let style = if editing {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!(" {marker} {name:<13} [ {display} ]"),
                style,
            )))
        })
        .collect();

    f.render_widget(List::new(items), area);
}

fn draw_log(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default().borders(Borders::TOP).title(" Log ");
    let inner_height = block.inner(area).height as usize;

    let items: Vec<ListItem> = state
        .status_log
        .iter()
        .rev()
        .take(inner_height)
        .rev()
        .map(|line| ListItem::new(Line::from(format!(" > {line}"))))
        .collect();

    f.render_widget(List::new(items).block(block), area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = " Hotkey Toggle · ↑↓ Field · ←→ Value · E Edit · R Record · Q Quit";
    f.render_widget(
        Paragraph::new(Span::styled(help, Style::default().fg(Color::DarkGray))),
        area,
    );
}
