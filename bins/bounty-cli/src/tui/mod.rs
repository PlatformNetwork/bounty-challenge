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

pub struct AuxiliarySidebarProps {
    pub on_active_view_change: Box<dyn Fn(Option<String>) -> () + 'static>,
}

impl AuxiliarySidebarProps {
    pub fn new(on_active_view_change: impl Fn(Option<String>) -> () + 'static) -> Self {
        Self {
            on_active_view_change: Box::new(on_active_view_change),
        }
    }
}

pub struct AuxiliarySidebar {
    props: AuxiliarySidebarProps,
    active_view_id: Option<String>,
}

impl AuxiliarySidebar {
    pub fn new(props: AuxiliarySidebarProps) -> Self {
        Self {
            props,
            active_view_id: None,
        }
    }

    pub fn toggle(&mut self, view_id: String) {
        if self.active_view_id == Some(view_id.clone()) {
            self.active_view_id = None;
            (self.props.on_active_view_change)(None);
        } else {
            self.active_view_id = Some(view_id.clone());
            (self.props.on_active_view_change)(Some(view_id));
        }
    }
}