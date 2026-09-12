use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{branch_explorer, commit_log, diff_view, job_board, resume_preview};
use crate::app::{ActivePanel, App, MainView};

/// Draw the full TUI layout.
pub fn draw(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    // Main content: left panel + center + optional right panel
    let mut constraints = vec![
        Constraint::Length(28), // branch list
        Constraint::Min(40),   // main view
    ];
    if app.show_job_panel {
        constraints.push(Constraint::Length(32)); // job details
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(outer[0]);

    // Left panel: branch list
    let branch_block = Block::default()
        .title(" Branches ")
        .borders(Borders::ALL)
        .border_style(if app.active_panel == ActivePanel::BranchList {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    branch_explorer::draw_branch_list(frame, columns[0], app, branch_block);

    // Center panel: dispatch to the active view
    let active_style = if app.active_panel == ActivePanel::MainView {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    match app.main_view {
        MainView::ResumePreview => {
            resume_preview::draw(frame, columns[1], &app.tex_content);
        }
        MainView::DiffView => {
            diff_view::draw(frame, columns[1], &app.diff_lines);
        }
        MainView::CommitLog => {
            commit_log::draw(frame, columns[1], &app.commits);
        }
        MainView::JobBoard => {
            job_board::draw(frame, columns[1], &app.job_summaries);
        }
        _ => {
            // BranchExplorer, AiPanel, ValidationReport - show placeholder
            let view_title = match app.main_view {
                MainView::BranchExplorer => " Explorer ",
                MainView::AiPanel => " AI ",
                MainView::ValidationReport => " Validation ",
                _ => unreachable!(),
            };

            let content = match app.main_view {
                MainView::BranchExplorer => format!(
                    "Branch: {}\nBranches: {}\n\nUse j/k to navigate branches, Enter to select.\nPress v to cycle views, 1-5 for direct access.",
                    app.current_branch,
                    app.branches.len()
                ),
                MainView::AiPanel => "AI-powered operations require Claude Code runtime.\n\nAvailable commands:\n  :tailor  - Tailor resume to JD\n  :score   - Score resume against JD\n  :cover   - Generate cover letter".to_string(),
                MainView::ValidationReport => "Run validation after compiling your resume.\n\nChecks:\n  - Page count limit\n  - Required sections\n  - ATS compatibility\n  - Bullet point quality".to_string(),
                _ => unreachable!(),
            };

            let block = Block::default()
                .title(view_title)
                .borders(Borders::ALL)
                .border_style(active_style);

            let paragraph = Paragraph::new(content)
                .block(block)
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, columns[1]);
        }
    }

    // Right panel: job details (if visible)
    if app.show_job_panel && columns.len() > 2 {
        let job_block = Block::default()
            .title(" Job Details ")
            .borders(Borders::ALL)
            .border_style(if app.active_panel == ActivePanel::JobDetails {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        frame.render_widget(job_block, columns[2]);
    }

    // Bottom status bar
    let status = if app.command_mode {
        format!(":{}", app.command_input)
    } else if !app.status_message.is_empty() {
        format!(" {}", app.status_message)
    } else {
        format!(
            " {} | {} | Press : for commands, q to quit",
            app.current_branch,
            match app.main_view {
                MainView::BranchExplorer => "Explorer",
                MainView::ResumePreview => "Preview",
                MainView::DiffView => "Diff",
                MainView::JobBoard => "Jobs",
                MainView::CommitLog => "Log",
                MainView::AiPanel => "AI",
                MainView::ValidationReport => "Validation",
            }
        )
    };

    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(status_bar, outer[1]);
}
