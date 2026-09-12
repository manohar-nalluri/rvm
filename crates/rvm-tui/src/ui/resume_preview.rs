use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Draw a syntax-highlighted preview of the .tex content.
pub fn draw(frame: &mut Frame, area: Rect, tex_content: &str) {
    let block = Block::default()
        .title(" Resume Preview ")
        .borders(Borders::ALL);

    // Basic syntax highlighting for LaTeX
    let lines: Vec<Line> = tex_content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let style = if trimmed.starts_with('%') {
                Style::default().fg(Color::DarkGray) // comments
            } else if trimmed.starts_with('\\') {
                Style::default().fg(Color::Cyan) // commands
            } else if trimmed.contains("\\begin{") || trimmed.contains("\\end{") {
                Style::default().fg(Color::Yellow) // environments
            } else {
                Style::default()
            };
            Line::styled(line.to_string(), style)
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
