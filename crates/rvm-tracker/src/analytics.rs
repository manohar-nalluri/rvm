use std::collections::HashMap;
use std::path::Path;

use chrono::{Datelike, Utc};
use rvm_types::{ApplicationStatus, RvmResult};

use crate::job;

/// Application analytics summary.
#[derive(Debug, Default)]
pub struct AnalyticsSummary {
    pub total_applications: usize,
    pub applications_this_month: usize,
    pub response_rate: f64,
    pub avg_days_to_response: Option<f64>,
    pub status_counts: HashMap<String, usize>,
}

/// Generate analytics from all tracked applications.
pub fn generate(branches_dir: &Path) -> RvmResult<AnalyticsSummary> {
    let all = job::load_all(branches_dir)?;
    let now = Utc::now();
    let mut summary = AnalyticsSummary::default();

    let mut responded = 0usize;
    let mut applied_count = 0usize;
    let mut total_response_days = 0i64;
    let mut response_count = 0usize;

    for (_, meta) in &all {
        summary.total_applications += 1;

        // Count by status
        let status_key = meta.status.to_string();
        *summary.status_counts.entry(status_key).or_default() += 1;

        // Applications this month
        if let Some(date) = meta.applied_date {
            if date.month() == now.month() && date.year() == now.year() {
                summary.applications_this_month += 1;
            }
            applied_count += 1;
        }

        // Response rate calculation
        match meta.status {
            ApplicationStatus::Draft => {}
            ApplicationStatus::Applied | ApplicationStatus::Ghosted => {}
            _ => {
                responded += 1;
                // Calculate time to first response
                if let Some(applied) = meta.applied_date {
                    if let Some(first_response) = meta.events.iter().find(|e| {
                        !matches!(
                            e.status,
                            ApplicationStatus::Draft | ApplicationStatus::Applied
                        )
                    }) {
                        let days = (first_response.timestamp - applied).num_days();
                        total_response_days += days;
                        response_count += 1;
                    }
                }
            }
        }
    }

    if applied_count > 0 {
        summary.response_rate = responded as f64 / applied_count as f64 * 100.0;
    }

    if response_count > 0 {
        summary.avg_days_to_response =
            Some(total_response_days as f64 / response_count as f64);
    }

    Ok(summary)
}
