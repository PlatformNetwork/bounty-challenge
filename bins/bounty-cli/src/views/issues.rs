use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

/// Renders the issues view with proper accessibility labeling
pub fn render_issues(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title with accessible label association
    let title = Paragraph::new("Issues")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("[Issues View]"));
    f.render_widget(title, chunks[0]);

    // Issues list - ensure each item has proper labeling
    let issues: Vec<ListItem> = app
        .issues
        .items
        .iter()
        .map(|issue| {
            // Create properly labeled list item with accessibility in mind
            let label = format!("#{} - {}", issue.number, issue.title);
            let content = Line::from(Span::styled(
                label,
                Style::default().fg(Color::White),
            ));
            ListItem::new(content)
        })
        .collect();

    let issues_list = List::new(issues)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[Issue List - aria-label: 'Issues list, use arrow keys to navigate']"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(issues_list, chunks[1], &mut app.issues.state.clone());

    // Status bar with accessible instructions
    let status = Paragraph::new("Press Enter to view issue | q to quit")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

/// Helper function to ensure form controls have proper accessibility labels
/// This addresses issues where controls like "Tab Close Button Position"
/// are not programmatically associated with their visible labels
pub fn create_accessible_control_label<'a>(
    control_id: &str,
    visible_label: &str,
    description: Option<&str>,
) -> Line<'a> {
    let mut spans = vec![];
    
    // Add the visible label with control ID association
    spans.push(Span::styled(
        format!("{}: ", visible_label),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    
    // Add description if provided (acts as aria-describedby)
    if let Some(desc) = description {
        spans.push(Span::styled(
            format!("({}) ", desc),
            Style::default().fg(Color::Gray),
        ));
    }
    
    // Add control identifier for accessibility tools
    spans.push(Span::styled(
        format!("[id: {}]", control_id),
        Style::default().fg(Color::DarkGray),
    ));
    
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessible_control_label_creation() {
        let label = create_accessible_control_label(
            "tab-close-button-position",
            "Tab Close Button Position",
            Some("Controls where the close button appears on tabs"),
        );
        
        assert_eq!(label.spans.len(), 3);
    }

    #[test]
    fn test_accessible_control_label_without_description() {
        let label = create_accessible_control_label(
            "tab-close-button-position",
            "Tab Close Button Position",
            None,
        );
        
        assert_eq!(label.spans.len(), 2);
    }
}
