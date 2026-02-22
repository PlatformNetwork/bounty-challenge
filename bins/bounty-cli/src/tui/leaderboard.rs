use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::rpc::rpc_call;

struct LeaderboardEntry {
    rank: u64,
    hotkey: String,
    github: String,
    net_points: f64,
    valid: u64,
    invalid: u64,
    stars: u64,
    weight: f64,
}

struct App {
    entries: Vec<LeaderboardEntry>,
    scroll_offset: usize,
    error: Option<String>,
}

fn parse_entries(data: &Value) -> Vec<LeaderboardEntry> {
    let body = data.get("body").unwrap_or(data);
    let arr = match body.as_array() {
        Some(a) => a,
        None => return vec![],
    };

    arr.iter()
        .map(|e| {
            let hotkey = e.get("hotkey").and_then(|v| v.as_str()).unwrap_or("?");
            let hotkey_short = if hotkey.len() > 14 {
                format!("{}...", &hotkey[..14])
            } else {
                hotkey.to_string()
            };
            LeaderboardEntry {
                rank: e.get("rank").and_then(|v| v.as_u64()).unwrap_or(0),
                hotkey: hotkey_short,
                github: e
                    .get("github_username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string(),
                net_points: e.get("net_points").and_then(|v| v.as_f64()).unwrap_or(0.0),
                valid: e.get("valid_issues").and_then(|v| v.as_u64()).unwrap_or(0),
                invalid: e
                    .get("invalid_issues")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                stars: e.get("star_count").and_then(|v| v.as_u64()).unwrap_or(0),
                weight: e.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            }
        })
        .collect()
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    let header = Row::new(vec![
        Cell::from("Rank"),
        Cell::from("Hotkey"),
        Cell::from("GitHub"),
        Cell::from("Net Pts"),
        Cell::from("Valid"),
        Cell::from("Invalid"),
        Cell::from("Stars"),
        Cell::from("Weight"),
    ])
    .style(Style::default().fg(Color::Yellow).bold())
    .height(1);

    let rows: Vec<Row> = app
        .entries
        .iter()
        .skip(app.scroll_offset)
        .map(|e| {
            Row::new(vec![
                Cell::from(e.rank.to_string()),
                Cell::from(e.hotkey.clone()),
                Cell::from(e.github.clone()),
                Cell::from(format!("{:.2}", e.net_points)),
                Cell::from(e.valid.to_string()),
                Cell::from(e.invalid.to_string()),
                Cell::from(e.stars.to_string()),
                Cell::from(format!("{:.4}", e.weight)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(17),
        Constraint::Length(18),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(10),
    ];

    let title = if let Some(ref err) = app.error {
        format!(" Leaderboard — ERROR: {} ", err)
    } else {
        format!(" Leaderboard — {} miners ", app.entries.len())
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(table, chunks[0]);

    let help = Paragraph::new(" ↑/↓ scroll  |  q/Esc quit  |  auto-refresh 5s")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[1]);
}

pub async fn run(rpc_url: &str) -> Result<()> {
    let mut terminal = super::setup_terminal()?;
    let mut app = App {
        entries: vec![],
        scroll_offset: 0,
        error: None,
    };

    let mut last_fetch = Instant::now() - Duration::from_secs(10);
    let refresh_interval = Duration::from_secs(5);

    loop {
        if last_fetch.elapsed() >= refresh_interval {
            match rpc_call(rpc_url, "GET", "/leaderboard", None).await {
                Ok(data) => {
                    app.entries = parse_entries(&data);
                    app.error = None;
                }
                Err(e) => app.error = Some(e.to_string()),
            }
            last_fetch = Instant::now();
        }

        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.scroll_offset = app.scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.scroll_offset + 1 < app.entries.len() {
                                app.scroll_offset += 1;
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
