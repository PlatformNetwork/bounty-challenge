use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::rpc::rpc_call;

struct StatsData {
    total_bounties: u64,
    active_miners: u64,
    validator_count: u64,
    total_issues: u64,
}

impl Default for StatsData {
    fn default() -> Self {
        Self {
            total_bounties: 0,
            active_miners: 0,
            validator_count: 0,
            total_issues: 0,
        }
    }
}

fn parse_stats(data: &Value) -> StatsData {
    let body = data.get("body").unwrap_or(data);
    StatsData {
        total_bounties: body
            .get("total_bounties")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        active_miners: body
            .get("active_miners")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        validator_count: body
            .get("validator_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_issues: body
            .get("total_issues")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    }
}

fn stat_block<'a>(label: &'a str, value: u64, color: Color) -> Paragraph<'a> {
    let text = vec![
        Line::from(Span::styled(
            label,
            Style::default().fg(Color::DarkGray).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            value.to_string(),
            Style::default().fg(color).bold(),
        )),
    ];
    Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color)),
    )
}

fn ui(frame: &mut Frame, stats: &StatsData, error: &Option<String>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_text = if let Some(ref err) = error {
        format!(" Challenge Stats — ERROR: {} ", err)
    } else {
        " Challenge Stats ".to_string()
    };
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).bold())
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, outer[0]);

    let grid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(outer[1]);

    frame.render_widget(
        stat_block("Total Bounties", stats.total_bounties, Color::Green),
        grid[0],
    );
    frame.render_widget(
        stat_block("Active Miners", stats.active_miners, Color::Yellow),
        grid[1],
    );
    frame.render_widget(
        stat_block("Validators", stats.validator_count, Color::Cyan),
        grid[2],
    );
    frame.render_widget(
        stat_block("Total Issues", stats.total_issues, Color::Magenta),
        grid[3],
    );

    let help = Paragraph::new(" q/Esc quit  |  auto-refresh 5s")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, outer[2]);
}

pub async fn run(rpc_url: &str) -> Result<()> {
    let mut terminal = super::setup_terminal()?;
    let mut stats = StatsData::default();
    let mut error: Option<String> = None;
    let mut last_fetch = Instant::now() - Duration::from_secs(10);

    loop {
        if last_fetch.elapsed() >= Duration::from_secs(5) {
            match rpc_call(rpc_url, "GET", "/stats", None).await {
                Ok(data) => {
                    stats = parse_stats(&data);
                    error = None;
                }
                Err(e) => error = Some(e.to_string()),
            }
            last_fetch = Instant::now();
        }

        terminal.draw(|f| ui(f, &stats, &error))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    break;
                }
            }
        }
    }

    super::restore_terminal(&mut terminal)?;
    Ok(())
}
