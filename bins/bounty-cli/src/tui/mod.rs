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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

pub struct AuxiliaryBar {
    visible: bool,
    on_visibility_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
    initial_run: Arc<AtomicBool>,
}

impl AuxiliaryBar {
    pub fn new() -> Self {
        Self {
            visible: false,
            on_visibility_change: None,
            initial_run: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn set_on_visibility_change(&mut self, callback: impl Fn(bool) + Send + Sync + 'static) {
        self.on_visibility_change = Some(Box::new(callback));
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            if let Some(callback) = &self.on_visibility_change {
                if !*self.initial_run.load(Ordering::SeqCst) {
                    callback(visible);
                } else {
                    self.initial_run.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}