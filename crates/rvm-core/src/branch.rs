use rvm_types::{Branch, RvmError, RvmResult};

use crate::auth::Authenticator;
use crate::workspace::Workspace;

/// List all branch names (non-archived).
pub fn list(ws: &Workspace) -> RvmResult<Vec<String>> {
    let branches_dir = ws.branches_dir();
    let mut names = Vec::new();

    for entry in std::fs::read_dir(&branches_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let branch = load(ws, entry.file_name().to_string_lossy().as_ref())?;
            if !branch.metadata.archived {
                names.push(branch.metadata.name);
            }
        }
    }

    names.sort();
    Ok(names)
}

/// Load a branch by name.
pub fn load(ws: &Workspace, name: &str) -> RvmResult<Branch> {
    let path = ws.branches_dir().join(name).join("branch.json");
    if !path.exists() {
        return Err(RvmError::BranchNotFound(name.to_string()));
    }
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Save a branch.
pub fn save(ws: &Workspace, branch: &Branch) -> RvmResult<()> {
    let dir = ws.branches_dir().join(&branch.metadata.name);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("branch.json");
    std::fs::write(path, serde_json::to_string_pretty(branch)?)?;
    Ok(())
}

/// Create a new branch forked from the current branch.
pub fn create(ws: &Workspace, name: &str) -> RvmResult<Branch> {
    let current = ws.current_branch()?;
    create_from(ws, name, &current)
}

/// Create a new branch forked from an explicit base branch.
///
/// Forking does not touch the base branch at all, so this is safe to call with
/// a protected branch such as `main`. It also does not move HEAD, which matters
/// for automation: a batch of tailored branches must not disturb whatever the
/// operator currently has checked out.
pub fn create_from(ws: &Workspace, name: &str, base: &str) -> RvmResult<Branch> {
    let dir = ws.branches_dir().join(name);
    if dir.exists() {
        return Err(RvmError::BranchAlreadyExists(name.to_string()));
    }

    let base_branch = load(ws, base)?;
    if base_branch.metadata.archived {
        return Err(RvmError::Other(format!(
            "Cannot branch from archived branch '{}'.",
            base
        )));
    }

    let mut new_branch = Branch::new(name.to_string(), Some(base.to_string()));
    new_branch.head = base_branch.head.clone();

    std::fs::create_dir_all(&dir)?;

    // Copy the latest snapshot if it exists
    let base_snapshot = ws.branches_dir().join(base).join("snapshot.tex");
    if base_snapshot.exists() {
        std::fs::copy(&base_snapshot, dir.join("snapshot.tex"))?;
    }

    // Copy commit history so merge can find common ancestors
    let base_commits = ws.branches_dir().join(base).join("commits.json");
    if base_commits.exists() {
        std::fs::copy(&base_commits, dir.join("commits.json"))?;
    }

    save(ws, &new_branch)?;

    tracing::info!("Created branch '{}' from '{}'", name, base);
    Ok(new_branch)
}

/// Switch to a branch, updating the working .tex file.
pub fn checkout(ws: &Workspace, name: &str) -> RvmResult<()> {
    let branch = load(ws, name)?;
    if branch.metadata.archived {
        return Err(RvmError::Other(format!(
            "Branch '{}' is archived. Unarchive it first.",
            name
        )));
    }

    // Restore snapshot to working file
    let snapshot_path = ws.branches_dir().join(name).join("snapshot.tex");
    if snapshot_path.exists() {
        // Write to the existing .tex file, or fall back to resume.tex for new workspaces
        let tex_path = ws.find_tex_file().unwrap_or_else(|_| ws.root().join("resume.tex"));
        std::fs::copy(&snapshot_path, tex_path)?;
    }

    ws.set_head(name)?;
    tracing::info!("Switched to branch '{}'", name);
    Ok(())
}

/// Archive a branch (soft delete).
///
/// Requires authentication when the branch is protected.
pub fn archive(ws: &Workspace, name: &str, auth: &dyn Authenticator) -> RvmResult<()> {
    if name == "main" {
        return Err(RvmError::Other("Cannot archive the main branch".to_string()));
    }

    crate::guard::ensure_mutable(ws, name, auth)?;

    let mut branch = load(ws, name)?;
    branch.metadata.archived = true;
    save(ws, &branch)?;

    tracing::info!("Archived branch '{}'", name);
    Ok(())
}

/// Delete a branch directory entirely.
///
/// Requires authentication when the branch is protected.
pub fn delete(ws: &Workspace, name: &str, auth: &dyn Authenticator) -> RvmResult<()> {
    if name == "main" {
        return Err(RvmError::Other("Cannot delete the main branch".to_string()));
    }

    crate::guard::ensure_mutable(ws, name, auth)?;

    let current = ws.current_branch()?;
    if current == name {
        return Err(RvmError::Other(
            "Cannot delete the currently checked out branch".to_string(),
        ));
    }

    let dir = ws.branches_dir().join(name);
    if !dir.exists() {
        return Err(RvmError::BranchNotFound(name.to_string()));
    }

    std::fs::remove_dir_all(dir)?;
    tracing::info!("Deleted branch '{}'", name);
    Ok(())
}

/// Check if a branch exists.
pub fn exists(ws: &Workspace, name: &str) -> bool {
    ws.branches_dir().join(name).join("branch.json").exists()
}

/// List all branch names (including archived).
pub fn list_all(ws: &Workspace) -> RvmResult<Vec<String>> {
    let branches_dir = ws.branches_dir();
    let mut names = Vec::new();

    for entry in std::fs::read_dir(&branches_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if branches_dir.join(&name).join("branch.json").exists() {
                names.push(name);
            }
        }
    }

    names.sort();
    Ok(names)
}

/// List only archived branch names.
pub fn list_archived(ws: &Workspace) -> RvmResult<Vec<String>> {
    let branches_dir = ws.branches_dir();
    let mut names = Vec::new();

    for entry in std::fs::read_dir(&branches_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let branch = load(ws, entry.file_name().to_string_lossy().as_ref())?;
            if branch.metadata.archived {
                names.push(branch.metadata.name);
            }
        }
    }

    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AllowAuthenticator, DenyAuthenticator};
    use tempfile::TempDir;

    fn setup_workspace() -> (TempDir, Workspace) {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        (tmp, ws)
    }

    #[test]
    fn test_create_and_list_branch() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "feature").unwrap();
        let branches = list(&ws).unwrap();
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"feature".to_string()));
    }

    #[test]
    fn test_create_duplicate_branch_fails() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "test").unwrap();
        assert!(create(&ws, "test").is_err());
    }

    #[test]
    fn test_checkout_branch() {
        let (tmp, ws) = setup_workspace();
        std::fs::write(tmp.path().join("resume.tex"), "content").unwrap();
        create(&ws, "dev").unwrap();
        checkout(&ws, "dev").unwrap();
        assert_eq!(ws.current_branch().unwrap(), "dev");
    }

    #[test]
    fn test_archive_and_list_archived() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "old-branch").unwrap();
        archive(&ws, "old-branch", &AllowAuthenticator).unwrap();
        let archived = list_archived(&ws).unwrap();
        assert!(archived.contains(&"old-branch".to_string()));
        let active = list(&ws).unwrap();
        assert!(!active.contains(&"old-branch".to_string()));
    }

    #[test]
    fn test_cannot_archive_main() {
        let (_tmp, ws) = setup_workspace();
        assert!(archive(&ws, "main", &AllowAuthenticator).is_err());
    }

    #[test]
    fn test_archive_of_protected_branch_requires_authentication() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "release").unwrap();
        crate::config::set_protected(&ws, "release", true).unwrap();

        assert!(archive(&ws, "release", &DenyAuthenticator).is_err());
        // Still active: the refusal happened before any write.
        assert!(list(&ws).unwrap().contains(&"release".to_string()));

        archive(&ws, "release", &AllowAuthenticator).unwrap();
        assert!(list_archived(&ws).unwrap().contains(&"release".to_string()));
    }

    #[test]
    fn test_delete_branch() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "deleteme").unwrap();
        delete(&ws, "deleteme", &AllowAuthenticator).unwrap();
        assert!(!exists(&ws, "deleteme"));
    }

    #[test]
    fn test_cannot_delete_main() {
        let (_tmp, ws) = setup_workspace();
        assert!(delete(&ws, "main", &AllowAuthenticator).is_err());
    }

    #[test]
    fn test_delete_of_protected_branch_requires_authentication() {
        let (_tmp, ws) = setup_workspace();
        create(&ws, "keepme").unwrap();
        crate::config::set_protected(&ws, "keepme", true).unwrap();

        assert!(delete(&ws, "keepme", &DenyAuthenticator).is_err());
        assert!(exists(&ws, "keepme"), "refusal must not delete the branch");

        delete(&ws, "keepme", &AllowAuthenticator).unwrap();
        assert!(!exists(&ws, "keepme"));
    }

    #[test]
    fn test_branch_copies_commits() {
        let (tmp, ws) = setup_workspace();
        std::fs::write(tmp.path().join("resume.tex"), "test content").unwrap();
        crate::commit::create(&ws, "initial", &AllowAuthenticator).unwrap();
        create(&ws, "feature").unwrap();
        let commits_path = ws.branches_dir().join("feature").join("commits.json");
        assert!(commits_path.exists());
    }
}
