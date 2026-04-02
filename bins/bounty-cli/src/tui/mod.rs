pub mod leaderboard;
pub mod stats;
pub mod weights;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub struct AuxiliaryActivityBar {
    position: String,
}

impl AuxiliaryActivityBar {
    pub fn new(position: String) -> Self {
        Self { position }
    }

    pub fn update_position(&mut self, position: String) {
        self.position = position;
    }

    pub fn get_activity_bar_style(&self) -> Style {
        let mut style = Style::default();
        if self.position == "right" {
            style = style.fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD);
        } else if self.position == "left" {
            style = style.fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD);
        }
        style
    }
}

pub fn draw_auxiliary_activity_bar(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    activity_bar: &mut AuxiliaryActivityBar,
) -> Result<()> {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());
    let activity_bar_style = activity_bar.get_activity_bar_style();
    let activity_bar_text = if activity_bar.position == "right" {
        "Right"
    } else {
        "Left"
    };
    let border_style = if activity_bar.position == "right" {
        Borders::RIGHT
    } else {
        Borders::LEFT
    };
    f.render_widget(
        Paragraph::new(activity_bar_text)
            .style(activity_bar_style)
            .block(Block::default().borders(border_style)),
        chunks[0],
    )?;
    Ok(())
}