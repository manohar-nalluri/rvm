use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use rvm_core::diff::{DiffLine, DiffTag};

/// Draw a diff view showing additions and deletions.
pub fn draw(frame: &mut Frame, area: Rect, diff_lines: &[DiffLine]) {
    let block = Block::default()
        .title(" Diff View ")
        .borders(Borders::ALL);

    let lines: Vec<Line> = diff_lines
        .iter()
        .map(|dl| {
            let (prefix, style) = match dl.tag {
                DiffTag::Equal => (" ", Style::default()),
                DiffTag::Insert => ("+", Style::default().fg(Color::Green)),
                DiffTag::Delete => ("-", Style::default().fg(Color::Red)),
            };
            Line::styled(
                format!("{} {}", prefix, dl.content.trim_end()),
                style,
            )
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
