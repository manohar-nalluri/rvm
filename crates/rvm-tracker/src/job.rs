use std::path::Path;

use rvm_types::{ApplicationStatus, JobMetadata, RvmResult, StatusEvent};

/// Load job metadata for a branch.
pub fn load(branches_dir: &Path, branch_name: &str) -> RvmResult<JobMetadata> {
    let path = branches_dir.join(branch_name).join("job.json");
    if !path.exists() {
        return Ok(JobMetadata::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Save job metadata for a branch.
pub fn save(branches_dir: &Path, branch_name: &str, metadata: &JobMetadata) -> RvmResult<()> {
    let dir = branches_dir.join(branch_name);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("job.json");
    std::fs::write(path, serde_json::to_string_pretty(metadata)?)?;
    Ok(())
}

/// Update the application status and record the event in the timeline.
pub fn update_status(
    branches_dir: &Path,
    branch_name: &str,
    new_status: ApplicationStatus,
    notes: Option<String>,
) -> RvmResult<()> {
    let mut metadata = load(branches_dir, branch_name)?;

    let event = StatusEvent {
        status: new_status.clone(),
        timestamp: chrono::Utc::now(),
        notes,
    };

    metadata.status = new_status;
    metadata.events.push(event);

    save(branches_dir, branch_name, &metadata)?;
    Ok(())
}

/// Load all job metadata across all branches.
pub fn load_all(branches_dir: &Path) -> RvmResult<Vec<(String, JobMetadata)>> {
    let mut results = Vec::new();

    if !branches_dir.exists() {
        return Ok(results);
    }

    for entry in std::fs::read_dir(branches_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = load(branches_dir, &name)?;
            results.push((name, metadata));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_returns_default() {
        let tmp = TempDir::new().unwrap();
        let meta = load(tmp.path(), "nonexistent").unwrap();
        assert_eq!(meta.status, ApplicationStatus::Draft);
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut meta = JobMetadata::default();
        meta.company = Some("Acme".to_string());
        meta.role = Some("Engineer".to_string());
        save(tmp.path(), "test-branch", &meta).unwrap();

        let loaded = load(tmp.path(), "test-branch").unwrap();
        assert_eq!(loaded.company.as_deref(), Some("Acme"));
        assert_eq!(loaded.role.as_deref(), Some("Engineer"));
    }

    #[test]
    fn test_update_status() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("test-branch")).unwrap();
        update_status(tmp.path(), "test-branch", ApplicationStatus::Applied, None).unwrap();

        let loaded = load(tmp.path(), "test-branch").unwrap();
        assert_eq!(loaded.status, ApplicationStatus::Applied);
        assert_eq!(loaded.events.len(), 1);
    }

    #[test]
    fn test_load_all() {
        let tmp = TempDir::new().unwrap();
        let mut meta1 = JobMetadata::default();
        meta1.company = Some("Company A".to_string());
        save(tmp.path(), "branch-a", &meta1).unwrap();

        let mut meta2 = JobMetadata::default();
        meta2.company = Some("Company B".to_string());
        save(tmp.path(), "branch-b", &meta2).unwrap();

        let all = load_all(tmp.path()).unwrap();
        assert_eq!(all.len(), 2);
    }
}
