use chrono::{DateTime, Utc};
use rvm_types::{ApplicationStatus, RvmResult};
use std::path::Path;

use crate::job;

/// A unified timeline event across all branches.
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub branch_name: String,
    pub company: Option<String>,
    pub status: ApplicationStatus,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Build a unified timeline of all status changes across all branches,
/// sorted chronologically (most recent first).
pub fn build_timeline(branches_dir: &Path) -> RvmResult<Vec<TimelineEntry>> {
    let all = job::load_all(branches_dir)?;
    let mut timeline = Vec::new();

    for (name, meta) in all {
        for event in &meta.events {
            timeline.push(TimelineEntry {
                branch_name: name.clone(),
                company: meta.company.clone(),
                status: event.status.clone(),
                timestamp: event.timestamp,
                notes: event.notes.clone(),
            });
        }
    }

    // Sort by timestamp descending (most recent first)
    timeline.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(timeline)
}
