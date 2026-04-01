pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    terminal.set_handler(|event| {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('v') && key.modifiers.contains(Modifier::CONTROL) {
                // Ignore Ctrl+V when the Log Level panel is open
                return;
            }
        }
    });
    Ok(terminal)
