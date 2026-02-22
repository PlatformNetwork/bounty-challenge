use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::rpc::rpc_call;

struct WeightEntry {
    hotkey: String,
    weight: f64,
}

fn parse_weights(data: &Value) -> Vec<WeightEntry> {
    let body = data.get("body").unwrap_or(data);

    if let Some(obj) = body.as_object() {
        let mut entries: Vec<WeightEntry> = obj
            .iter()
            .filter_map(|(k, v)| {
                v.as_f64().map(|w| {
                    let short = if k.len() > 14 {
                        format!("{}...", &k[..14])
                    } else {
                        k.clone()
                    };
                    WeightEntry {
                        hotkey: short,
                        weight: w,
                    }
                })
            })
            .collect();
        entries.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        return entries;
    }

    if let Some(arr) = body.as_array() {
        let mut entries: Vec<WeightEntry> = arr
            .iter()
            .filter_map(|e| {
                let hotkey = e.get("hotkey").and_then(|v| v.as_str())?;
                let weight = e.get("weight").and_then(|v| v.as_f64())?;
                let short = if hotkey.len() > 14 {
                    format!("{}...", &hotkey[..14])
                } else {
                    hotkey.to_string()
                };
                Some(WeightEntry {
                    hotkey: short,
                    weight,
                })
            })
            .collect();
        entries.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        return entries;
    }

    vec![]
}

fn ui(frame: &mut Frame, entries: &[WeightEntry], scroll: usize, error: &Option<String>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Hotkey"),
        Cell::from("Weight"),
        Cell::from("Bar"),
    ])
    .style(Style::default().fg(Color::Yellow).bold());

    let max_weight = entries.iter().map(|e| e.weight).fold(0.0_f64, f64::max);

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .skip(scroll)
        .map(|(i, e)| {
            let bar_len = if max_weight > 0.0 {
                ((e.weight / max_weight) * 30.0) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_len);
            Row::new(vec![
                Cell::from((i + 1).to_string()),
                Cell::from(e.hotkey.clone()),
                Cell::from(format!("{:.6}", e.weight)),
                Cell::from(Span::styled(bar, Style::default().fg(Color::Green))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(17),
        Constraint::Length(12),
        Constraint::Min(10),
    ];

    let title = if let Some(ref err) = error {
        format!(" Weights — ERROR: {} ", err)
    } else {
        format!(" Weights — {} miners ", entries.len())
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        );

    frame.render_widget(table, chunks[0]);

    let help = Paragraph::new(" ↑/↓ scroll  |  q/Esc quit  |  auto-refresh 5s")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[1]);
}

pub async fn run(rpc_url: &str) -> Result<()> {
    let mut terminal = super::setup_terminal()?;
    let mut entries: Vec<WeightEntry> = vec![];
    let mut scroll: usize = 0;
    let mut error: Option<String> = None;
    let mut last_fetch = Instant::now() - Duration::from_secs(10);

    loop {
        if last_fetch.elapsed() >= Duration::from_secs(5) {
            match rpc_call(rpc_url, "GET", "/get_weights", None).await {
                Ok(data) => {
                    entries = parse_weights(&data);
                    error = None;
                }
                Err(e) => error = Some(e.to_string()),
            }
            last_fetch = Instant::now();
        }

        terminal.draw(|f| ui(f, &entries, scroll, &error))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up | KeyCode::Char('k') => {
                            scroll = scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if scroll + 1 < entries.len() {
                                scroll += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    super::restore_terminal(&mut terminal)?;
    Ok(())
}
