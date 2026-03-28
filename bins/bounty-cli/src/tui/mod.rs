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

pub struct ButtonStyle {
    pub style: Style,
    pub hovered_style: Style,
}

impl ButtonStyle {
    pub fn new() -> Self {
        let style = Style::default().fg(Color::White).bg(Color::Rgb { r: 22, g: 22, b: 22 });
        let hovered_style = Style::default().fg(Color::White).bg(Color::Rgb { r: 40, g: 40, b: 40 });
        Self { style, hovered_style }
    }

    pub fn destructive() -> Self {
        let style = Style::default().fg(Color::White).bg(Color::Rgb { r: 150, g: 0, b: 0 });
        let hovered_style = Style::default().fg(Color::White).bg(Color::Rgb { r: 200, g: 0, b: 0 });
        Self { style, hovered_style }
    }

    pub fn minimize() -> Self {
        let style = Style::default().fg(Color::White).bg(Color::Rgb { r: 22, g: 22, b: 22 });
        let hovered_style = Style::default().fg(Color::White).bg(Color::Rgb { r: 40, g: 40, b: 40 });
        Self { style, hovered_style }
    }

    pub fn close() -> Self {
        let style = Style::default().fg(Color::White).bg(Color::Rgb { r: 100, g: 0, b: 0 });
        let hovered_style = Style::default().fg(Color::White).bg(Color::Rgb { r: 150, g: 0, b: 0 });
        Self { style, hovered_style }
    }
}

pub struct Button {
    pub text: String,
    pub style: ButtonStyle,
}

impl Button {
    pub fn new(text: String) -> Self {
        let style = ButtonStyle::new();
        Self { text, style }
    }

    pub fn destructive(text: String) -> Self {
        let style = ButtonStyle::destructive();
        Self { text, style }
    }

    pub fn minimize(text: String) -> Self {
        let style = ButtonStyle::minimize();
        Self { text, style }
    }

    pub fn close(text: String) -> Self {
        let style = ButtonStyle::close();
        Self { text, style }
    }
}

pub fn draw_button(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, button: &Button, area: Rect) -> Result<()> {
    let (x, y) = (area.x, area.y);
    let (width, height) = (area.width, area.height);
    let text = Paragraph::new(button.text.clone())
        .style(button.style.style)
        .block(Block::default().borders(Borders::NONE));
    terminal.draw(|f| f.render_widget(text, area))?;
    Ok(())
}

pub fn draw_destructive_button(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, button: &Button, area: Rect) -> Result<()> {
    let (x, y) = (area.x, area.y);
    let (width, height) = (area.width, area.height);
    let text = Paragraph::new(button.text.clone())
        .style(button.style.hovered_style)
        .block(Block::default().borders(Borders::NONE));
    terminal.draw(|f| f.render_widget(text, area))?;
    Ok(())
}