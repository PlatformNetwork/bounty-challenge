pub mod leaderboard;
pub mod stats;
pub mod weights;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn handle_events(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, is_resizing: &mut bool) -> Result<()> {
    loop {
        if let event::Event::Mouse(event) = event::read()? {
            match event.kind {
                event::MouseEventKind::Moved => {
                    // Handle mouse move event
                }
                event::MouseEventKind::Pressed(_) => {
                    *is_resizing = true;
                    // Handle mouse press event
                }
                event::MouseEventKind::Released(_) => {
                    *is_resizing = false;
                    // Handle mouse release event
                }
                _ => {}
            }
        }
    }
}