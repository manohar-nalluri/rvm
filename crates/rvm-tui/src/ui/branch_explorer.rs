use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, ListState};

use crate::app::App;

pub fn draw_branch_list(frame: &mut Frame, area: Rect, app: &App, block: Block) {
    let items: Vec<ListItem> = app
        .branches
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let prefix = if *name == app.current_branch {
                "* "
            } else {
                "  "
            };
            let style = if i == app.selected_branch_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if *name == app.current_branch {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected_branch_index));
    frame.render_stateful_widget(list, area, &mut state);
}
