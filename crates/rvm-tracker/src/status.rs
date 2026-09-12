use std::collections::HashMap;
use std::path::Path;

use chrono::Utc;
use rvm_types::{ApplicationStatus, RvmResult};

use crate::job;

/// Summary of all applications grouped by status.
#[derive(Debug, Default)]
pub struct StatusDashboard {
    pub by_status: HashMap<String, Vec<BranchSummary>>,
    pub total: usize,
    pub active: usize,
}

/// Summary info for a single branch in the dashboard.
#[derive(Debug, Clone)]
pub struct BranchSummary {
    pub branch_name: String,
    pub company: String,
    pub role: String,
    pub status: ApplicationStatus,
    pub days_since_activity: i64,
    pub has_deadline: bool,
    pub deadline_passed: bool,
}

/// Build the status dashboard from all branches.
pub fn build_dashboard(branches_dir: &Path) -> RvmResult<StatusDashboard> {
    let all = job::load_all(branches_dir)?;
    let now = Utc::now();
    let mut dashboard = StatusDashboard::default();

    for (name, meta) in &all {
        let last_activity = meta
            .events
            .last()
            .map(|e| e.timestamp)
            .or(meta.applied_date)
            .unwrap_or(now);

        let days_since = (now - last_activity).num_days();

        let deadline_passed = meta
            .deadline
            .map(|d| d < now)
            .unwrap_or(false);

        let summary = BranchSummary {
            branch_name: name.clone(),
            company: meta.company.clone().unwrap_or_else(|| "—".to_string()),
            role: meta.role.clone().unwrap_or_else(|| "—".to_string()),
            status: meta.status.clone(),
            days_since_activity: days_since,
            has_deadline: meta.deadline.is_some(),
            deadline_passed,
        };

        let status_key = meta.status.to_string();
        dashboard
            .by_status
            .entry(status_key)
            .or_default()
            .push(summary);

        dashboard.total += 1;

        match meta.status {
            ApplicationStatus::Rejected
            | ApplicationStatus::Ghosted
            | ApplicationStatus::Withdrawn
            | ApplicationStatus::Accepted => {}
            _ => dashboard.active += 1,
        }
    }

    Ok(dashboard)
}
