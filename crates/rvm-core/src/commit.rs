use rvm_types::{Commit, CommitHash, RvmError, RvmResult};

use crate::auth::Authenticator;
use crate::branch;
use crate::workspace::Workspace;

/// Create a new commit on the current branch.
///
/// Fails with an authentication error before touching any state if the current
/// branch is protected and `auth` does not authenticate.
pub fn create(ws: &Workspace, message: &str, auth: &dyn Authenticator) -> RvmResult<Commit> {
    let branch_name = ws.current_branch()?;

    // Gate first: nothing is written until the operator is authorised.
    crate::guard::ensure_mutable(ws, &branch_name, auth)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AllowAuthenticator, DenyAuthenticator};
    use tempfile::TempDir;

    fn setup() -> (TempDir, Workspace) {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        std::fs::write(tmp.path().join("resume.tex"), "base content\n").unwrap();
        (tmp, ws)
    }

    #[test]
    fn test_commit_is_refused_on_protected_main() {
        let (_tmp, ws) = setup();

        let err = create(&ws, "should not land", &DenyAuthenticator).unwrap_err();
        assert!(matches!(
            err,
            RvmError::AuthenticationUnavailable(_) | RvmError::AuthenticationFailed(_)
        ));

        // Fail closed: no commit, no snapshot, HEAD untouched.
        assert!(load_history(&ws, "main").unwrap().is_empty());
        assert!(branch::load(&ws, "main").unwrap().head.is_none());
        assert!(!ws
            .branches_dir()
            .join("main")
            .join("snapshot.tex")
            .exists());
    }

    #[test]
    fn test_commit_succeeds_on_protected_main_when_authenticated() {
        let (_tmp, ws) = setup();
        let commit = create(&ws, "authorised", &AllowAuthenticator).unwrap();
        assert_eq!(load_history(&ws, "main").unwrap().len(), 1);
        assert_eq!(branch::load(&ws, "main").unwrap().head, Some(commit.hash));
    }

    #[test]
    fn test_commit_on_unprotected_branch_needs_no_authentication() {
        let (_tmp, ws) = setup();
        branch::create(&ws, "feature").unwrap();
        ws.set_head("feature").unwrap();

        // Even a deny-everything authenticator must not block an unprotected
        // branch; the guard short-circuits before consulting it.
        create(&ws, "feature work", &DenyAuthenticator).unwrap();
        assert_eq!(load_history(&ws, "feature").unwrap().len(), 1);
    }

    #[test]
    fn test_unprotecting_main_allows_unauthenticated_commit() {
        let (_tmp, ws) = setup();
        crate::config::set_protected(&ws, "main", false).unwrap();
        create(&ws, "after unprotect", &DenyAuthenticator).unwrap();
        assert_eq!(load_history(&ws, "main").unwrap().len(), 1);
    }
}
