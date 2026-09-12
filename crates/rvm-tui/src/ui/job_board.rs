use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use rvm_tracker::status::BranchSummary;

/// Draw a kanban-style job board.
pub fn draw(frame: &mut Frame, area: Rect, summaries: &[BranchSummary]) {
    let block = Block::default()
        .title(" Job Board ")
        .borders(Borders::ALL);

    let header = Row::new(vec![
        Cell::from("Branch").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Company").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Role").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Days").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = summaries
        .iter()
        .map(|s| {
            let status_style = match s.status {
                rvm_types::ApplicationStatus::Offered => Style::default().fg(Color::Green),
                rvm_types::ApplicationStatus::Rejected => Style::default().fg(Color::Red),
                rvm_types::ApplicationStatus::Interviewing => Style::default().fg(Color::Yellow),
                rvm_types::ApplicationStatus::Applied => Style::default().fg(Color::Cyan),
                rvm_types::ApplicationStatus::Ghosted => Style::default().fg(Color::DarkGray),
                _ => Style::default(),
            };

            Row::new(vec![
                Cell::from(s.branch_name.clone()),
                Cell::from(s.company.clone()),
                Cell::from(s.role.clone()),
                Cell::from(s.status.to_string()).style(status_style),
                Cell::from(s.days_since_activity.to_string()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(table, area);
}
