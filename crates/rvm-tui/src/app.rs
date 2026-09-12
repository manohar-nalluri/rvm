use rvm_core::diff::DiffLine;
use rvm_core::Workspace;
use rvm_tracker::status::BranchSummary;
use rvm_types::Commit;

/// Active panel in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    BranchList,
    MainView,
    JobDetails,
}

/// Which main view is displayed in the center panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    BranchExplorer,
    ResumePreview,
    DiffView,
    JobBoard,
    CommitLog,
    AiPanel,
    ValidationReport,
}

/// Application state for the TUI.
pub struct App {
    pub running: bool,
    pub active_panel: ActivePanel,
    pub main_view: MainView,
    pub selected_branch_index: usize,
    pub branches: Vec<String>,
    pub current_branch: String,
    pub show_job_panel: bool,
    pub command_mode: bool,
    pub command_input: String,
    pub status_message: String,

    // Cached view data
    pub tex_content: String,
    pub diff_lines: Vec<DiffLine>,
    pub commits: Vec<Commit>,
    pub job_summaries: Vec<BranchSummary>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_panel: ActivePanel::BranchList,
            main_view: MainView::BranchExplorer,
            selected_branch_index: 0,
            branches: Vec::new(),
            current_branch: String::new(),
            show_job_panel: true,
            command_mode: false,
            command_input: String::new(),
            status_message: String::new(),
            tex_content: String::new(),
            diff_lines: Vec::new(),
            commits: Vec::new(),
            job_summaries: Vec::new(),
        }
    }

    /// Load state from a workspace.
    pub fn load_from_workspace(&mut self, ws: &Workspace) -> anyhow::Result<()> {
        self.current_branch = ws.current_branch()?;
        self.branches = rvm_core::branch::list(ws)?;
        self.refresh_view_data(ws);
        Ok(())
    }

    /// Refresh cached data for the current view.
    pub fn refresh_view_data(&mut self, ws: &Workspace) {
        match self.main_view {
            MainView::ResumePreview => {
                self.tex_content = ws
                    .find_tex_file()
                    .and_then(|p| std::fs::read_to_string(p).map_err(Into::into))
                    .unwrap_or_default();
            }
            MainView::DiffView => {
                let snapshot_path = ws
                    .branches_dir()
                    .join(&self.current_branch)
                    .join("snapshot.tex");
                let snapshot = std::fs::read_to_string(snapshot_path).unwrap_or_default();
                let working = ws
                    .find_tex_file()
                    .and_then(|p| std::fs::read_to_string(p).map_err(Into::into))
                    .unwrap_or_default();
                self.diff_lines = rvm_core::diff::compute(&snapshot, &working);
            }
            MainView::CommitLog => {
                self.commits =
                    rvm_core::commit::load_history(ws, &self.current_branch).unwrap_or_default();
            }
            MainView::JobBoard => {
                let dashboard =
                    rvm_tracker::status::build_dashboard(&ws.branches_dir()).unwrap_or_default();
                self.job_summaries = dashboard.by_status.into_values().flatten().collect();
            }
            _ => {}
        }
    }

    /// Execute a command from the command palette.
    pub fn execute_command(&mut self, ws: &Workspace) {
        let cmd = self.command_input.trim().to_string();
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first().copied() {
            Some("log") => {
                self.main_view = MainView::CommitLog;
                self.refresh_view_data(ws);
                self.status_message = "Switched to commit log".to_string();
            }
            Some("diff") => {
                self.main_view = MainView::DiffView;
                self.refresh_view_data(ws);
                self.status_message = "Switched to diff view".to_string();
            }
            Some("preview") => {
                self.main_view = MainView::ResumePreview;
                self.refresh_view_data(ws);
                self.status_message = "Switched to preview".to_string();
            }
            Some("jobs") => {
                self.main_view = MainView::JobBoard;
                self.refresh_view_data(ws);
                self.status_message = "Switched to job board".to_string();
            }
            Some("checkout") => {
                if let Some(branch_name) = parts.get(1) {
                    match rvm_core::branch::checkout(ws, branch_name) {
                        Ok(()) => {
                            self.current_branch = branch_name.to_string();
                            self.status_message =
                                format!("Switched to branch '{}'", branch_name);
                            self.refresh_view_data(ws);
                        }
                        Err(e) => {
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                } else {
                    self.status_message = "Usage: checkout <branch>".to_string();
                }
            }
            Some("q") | Some("quit") => {
                self.running = false;
            }
            Some(other) => {
                self.status_message = format!("Unknown command: {}", other);
            }
            None => {}
        }
    }

    /// Cycle to the next main view.
    pub fn next_view(&mut self) {
        self.main_view = match self.main_view {
            MainView::BranchExplorer => MainView::ResumePreview,
            MainView::ResumePreview => MainView::DiffView,
            MainView::DiffView => MainView::JobBoard,
            MainView::JobBoard => MainView::CommitLog,
            MainView::CommitLog => MainView::AiPanel,
            MainView::AiPanel => MainView::ValidationReport,
            MainView::ValidationReport => MainView::BranchExplorer,
        };
    }

    /// Cycle active panel.
    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::BranchList => ActivePanel::MainView,
            ActivePanel::MainView => {
                if self.show_job_panel {
                    ActivePanel::JobDetails
                } else {
                    ActivePanel::BranchList
                }
            }
            ActivePanel::JobDetails => ActivePanel::BranchList,
        };
    }

    /// Move selection up in branch list.
    pub fn select_prev(&mut self) {
        if self.selected_branch_index > 0 {
            self.selected_branch_index -= 1;
        }
    }

    /// Move selection down in branch list.
    pub fn select_next(&mut self) {
        if self.selected_branch_index < self.branches.len().saturating_sub(1) {
            self.selected_branch_index += 1;
        }
    }

    /// Toggle the right-side job details panel.
    pub fn toggle_job_panel(&mut self) {
        self.show_job_panel = !self.show_job_panel;
    }

    /// Enter command palette mode.
    pub fn enter_command_mode(&mut self) {
        self.command_mode = true;
        self.command_input.clear();
    }

    /// Exit command palette mode.
    pub fn exit_command_mode(&mut self) {
        self.command_mode = false;
        self.command_input.clear();
    }
}
