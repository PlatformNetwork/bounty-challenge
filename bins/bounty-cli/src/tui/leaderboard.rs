impl App {
    original_entries: Vec<LeaderboardEntry>,
    entries: Vec<LeaderboardEntry>,
    scroll_offset: usize,
    error: Option<String>,
    filter: Option<String>,
}

fn apply_filter(app: &mut App, filter: Option<String>) {
    if let Some(filter) = filter {
        app.entries = app.original_entries.iter().filter(|e| {
            e.hotkey.contains(&filter) || e.github.contains(&filter)
        }).cloned().collect();
    } else {
        app.entries = app.original_entries.clone();
    }
}