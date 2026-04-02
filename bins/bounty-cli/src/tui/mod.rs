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
use structopt::StructOpt;

#[derive(StructOpt)]
struct FeedbackOptions {
    #[structopt(short = "s", long = "session")]
    session: Option<String>,
    #[structopt()]
    message: String,
}

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

pub fn run_feedback(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let options = FeedbackOptions::from_args(&args[1..]);
    let message = options.message;
    let session = options.session;

    if let Some(session_id) = session {
        // Save feedback with session_id
        println!("Feedback saved with session_id: {}", session_id);
        // Save JSON with session_id and message
        // ...
    } else {
        // Save feedback without session_id
        println!("Feedback saved without session_id");
        // Save JSON with message only
        // ...
    }

    Ok(())
}