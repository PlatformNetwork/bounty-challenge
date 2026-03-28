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

pub struct TuiState {
    pub minimize_button_hovered: bool,
}

impl Default for TuiState {
    fn default() -> Self {
        TuiState {
            minimize_button_hovered: false,
        }
    }
}

pub fn render_minimize_button(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut TuiState) -> Result<()> {
    let button_style = if state.minimize_button_hovered {
        Style::default().fg(Color::White).bg(Color::Gray)
    } else {
        Style::default().fg(Color::White).bg(Color::Transparent)
    };

    let button = Button::new("Minimize")
        .style(button_style);

    terminal.draw(|f| {
        f.render_widget(button, Rect::new(0, 0, 10, 1));
    })?;

    Ok(())
}

pub fn handle_minimize_button_click(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut TuiState) -> Result<()> {
    // Handle minimize button click logic here
    Ok(())
}