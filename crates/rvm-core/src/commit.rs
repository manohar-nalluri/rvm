use rvm_types::{Commit, CommitHash, RvmError, RvmResult};

use crate::branch;
use crate::workspace::Workspace;

/// Create a new commit on the current branch.
pub fn create(ws: &Workspace, message: &str) -> RvmResult<Commit> {
    let branch_name = ws.current_branch()?;
    let mut branch_data = branch::load(ws, &branch_name)?;

    // Read the current working file (auto-discovers the single .tex file)
    let tex_path = ws.find_tex_file()?;
    let tex_content = std::fs::read_to_string(&tex_path)?;

    // Create the commit
    let commit = Commit::new(branch_data.head.clone(), message.to_string(), tex_content.clone());

    // Save commit to history
    let commits_path = ws
        .branches_dir()
        .join(&branch_name)
        .join("commits.json");
    let mut history = load_history(ws, &branch_name)?;
    history.push(commit.clone());
    std::fs::write(&commits_path, serde_json::to_string_pretty(&history)?)?;

    // Update snapshot
    let snapshot_path = ws
        .branches_dir()
        .join(&branch_name)
        .join("snapshot.tex");
    std::fs::write(snapshot_path, &tex_content)?;

    // Update branch HEAD
    branch_data.head = Some(commit.hash.clone());
    branch::save(ws, &branch_data)?;

    tracing::info!(
        "Committed {} on branch '{}'",
        commit.hash.short(),
        branch_name
    );

    Ok(commit)
}

/// Load commit history for a branch.
pub fn load_history(ws: &Workspace, branch_name: &str) -> RvmResult<Vec<Commit>> {
    let path = ws
        .branches_dir()
        .join(branch_name)
        .join("commits.json");

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Get a specific commit by hash.
pub fn get(ws: &Workspace, branch_name: &str, hash: &CommitHash) -> RvmResult<Commit> {
    let history = load_history(ws, branch_name)?;
    history
        .into_iter()
        .find(|c| c.hash == *hash)
        .ok_or_else(|| RvmError::Other(format!("Commit {} not found", hash)))
}

/// Get the latest commit on a branch.
pub fn latest(ws: &Workspace, branch_name: &str) -> RvmResult<Option<Commit>> {
    let history = load_history(ws, branch_name)?;
    Ok(history.into_iter().last())
}
