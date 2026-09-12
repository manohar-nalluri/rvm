use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use rvm_types::Commit;

/// Draw the commit log for the current branch.
pub fn draw(frame: &mut Frame, area: Rect, commits: &[Commit]) {
    let block = Block::default()
        .title(" Commit Log ")
        .borders(Borders::ALL);

    let items: Vec<ListItem> = commits
        .iter()
        .rev() // Most recent first
        .map(|c| {
            let time = c.timestamp.format("%Y-%m-%d %H:%M");
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        c.hash.short().to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        c.message.clone(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::styled(
                    format!("  {}", time),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
