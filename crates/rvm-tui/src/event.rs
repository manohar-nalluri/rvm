use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActivePanel, App};

/// Poll for terminal events with a timeout.
pub fn poll(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Handle a key event and update app state accordingly.
/// Returns true if view data should be refreshed.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Quit on Ctrl+C
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.running = false;
        return false;
    }

    // Command palette mode
    if app.command_mode {
        match key.code {
            KeyCode::Esc => app.exit_command_mode(),
            KeyCode::Enter => {
                // Signal to caller that command should be executed
                return true;
            }
            KeyCode::Backspace => {
                app.command_input.pop();
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            _ => {}
        }
        return false;
    }

    // Normal mode - vim-style navigation
    let mut needs_refresh = false;
    match key.code {
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Tab => app.next_panel(),
        KeyCode::Char('v') => {
            app.next_view();
            needs_refresh = true;
        }
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('h') | KeyCode::Left => {
            app.active_panel = ActivePanel::BranchList;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.active_panel = ActivePanel::MainView;
        }
        KeyCode::Char('p') => app.toggle_job_panel(),
        KeyCode::Char('1') => {
            app.main_view = crate::app::MainView::BranchExplorer;
            needs_refresh = true;
        }
        KeyCode::Char('2') => {
            app.main_view = crate::app::MainView::ResumePreview;
            needs_refresh = true;
        }
        KeyCode::Char('3') => {
            app.main_view = crate::app::MainView::DiffView;
            needs_refresh = true;
        }
        KeyCode::Char('4') => {
            app.main_view = crate::app::MainView::JobBoard;
            needs_refresh = true;
        }
        KeyCode::Char('5') => {
            app.main_view = crate::app::MainView::CommitLog;
            needs_refresh = true;
        }
        _ => {}
    }

    needs_refresh
}
