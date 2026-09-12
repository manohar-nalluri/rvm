use std::io;
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use rvm_core::Workspace;
use rvm_tui::app::App;
use rvm_tui::event;
use rvm_tui::ui;

pub fn execute() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    app.load_from_workspace(&ws)?;

    // Main loop
    let tick_rate = Duration::from_millis(16); // ~60fps
    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if let Some(crossterm::event::Event::Key(key)) = event::poll(tick_rate)? {
            let needs_refresh = event::handle_key(&mut app, key);
            if app.command_mode && needs_refresh {
                // Command entered - execute it
                app.execute_command(&ws);
                app.exit_command_mode();
            } else if needs_refresh {
                app.refresh_view_data(&ws);
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
