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

    write_commit(ws, &branch_name, message)
}

/// Create a commit on `branch_name` from an explicit snapshot.
///
/// Used by AI tailoring, which generates a resume that was never written to the
/// operator's working file and therefore cannot be read back from it. The guard
/// applies exactly as it does to [`create`], so this cannot be used to land on
/// a protected branch such as `main`.
pub fn create_with_content(
    ws: &Workspace,
    branch_name: &str,
    message: &str,
    tex_content: &str,
    auth: &dyn Authenticator,
) -> RvmResult<Commit> {
    crate::guard::ensure_mutable(ws, branch_name, auth)?;
    write_commit_content(ws, branch_name, message, tex_content)
}

/// Minimum length accepted for an abbreviated commit hash.
pub const MIN_ABBREV: usize = 4;

/// Resolve a user-supplied reference to a commit on `branch_name`.
///
/// Accepts `HEAD`/`latest`, a full commit hash, or an unambiguous abbreviated
/// hash prefix (at least [`MIN_ABBREV`] hex characters). Abbreviations that
/// match more than one commit are rejected rather than guessed at.
pub fn resolve(ws: &Workspace, branch_name: &str, reference: &str) -> RvmResult<Commit> {
    let history = load_history(ws, branch_name)?;
    let reference = reference.trim();

    if reference.eq_ignore_ascii_case("HEAD") || reference.eq_ignore_ascii_case("latest") {
        return history.last().cloned().ok_or(RvmError::NoCommits);
    }

    // A full hash always wins, even if it happens to be a valid prefix of
    // nothing else.
    if let Some(found) = history
        .iter()
        .find(|c| c.hash.0.eq_ignore_ascii_case(reference))
    {
        return Ok(found.clone());
    }

    let lowered = reference.to_ascii_lowercase();
    let looks_like_hash =
        lowered.len() >= MIN_ABBREV && lowered.chars().all(|c| c.is_ascii_hexdigit());

    if looks_like_hash {
        let matches: Vec<&Commit> = history
            .iter()
            .filter(|c| c.hash.0.starts_with(&lowered))
            .collect();
        match matches.len() {
            1 => return Ok(matches[0].clone()),
            0 => {}
            n => {
                return Err(RvmError::AmbiguousCommit {
                    reference: reference.to_string(),
                    matches: n,
                })
            }
        }
    }

    Err(RvmError::CommitNotFound(reference.to_string()))
}

/// Restore `branch_name`'s working file to the content of an earlier commit and
/// record that as a new commit.
///
/// This is a **forward** restore: history is never rewritten, so the restore is
/// itself an ordinary commit that can be undone by restoring again. That keeps
/// recovery safe, which matters most for the case this exists for — undoing
/// unwanted changes on a protected branch, where a destructive reset would
/// throw away the evidence of what went wrong.
///
/// Returns the commit that was restored from, and the new commit created.
pub fn restore(
    ws: &Workspace,
    branch_name: &str,
    reference: &str,
    auth: &dyn Authenticator,
) -> RvmResult<(Commit, Commit)> {
    // Authenticate once here, then use the unguarded write path so the operator
    // is not prompted a second time by `create`.
    crate::guard::ensure_mutable(ws, branch_name, auth)?;

    let target = resolve(ws, branch_name, reference)?;

    let tex_path = ws.find_tex_file()?;
    let current = std::fs::read_to_string(&tex_path)?;

    if current == target.snapshot_tex {
        return Err(RvmError::Other(format!(
            "working file already matches {}; nothing to restore",
            target.hash.short()
        )));
    }

    std::fs::write(&tex_path, &target.snapshot_tex)?;

    let summary = target.message.lines().next().unwrap_or("").trim();
    let summary = if summary.chars().count() > 60 {
        format!("{}...", summary.chars().take(60).collect::<String>())
    } else {
        summary.to_string()
    };

    let message = format!("Restore to {}: {}", target.hash.short(), summary);
    let commit = write_commit(ws, branch_name, &message)?;

    tracing::info!(
        "Restored branch '{}' to {} (new commit {})",
        branch_name,
        target.hash.short(),
        commit.hash.short()
    );

    Ok((target, commit))
}

/// Write a commit on `branch_name` without consulting the branch guard.
///
/// Callers are responsible for authorisation; see [`create`] and [`restore`].
fn write_commit(ws: &Workspace, branch_name: &str, message: &str) -> RvmResult<Commit> {
    // Read the current working file (auto-discovers the single .tex file)
    let tex_path = ws.find_tex_file()?;
    let tex_content = std::fs::read_to_string(&tex_path)?;
    write_commit_content(ws, branch_name, message, &tex_content)
}

/// Write a commit from an explicit snapshot instead of the working file.
///
/// Both commit paths funnel through here so `commits.json`, `snapshot.tex` and
/// the branch HEAD can never drift apart.
fn write_commit_content(
    ws: &Workspace,
    branch_name: &str,
    message: &str,
    tex_content: &str,
) -> RvmResult<Commit> {
    let mut branch_data = branch::load(ws, branch_name)?;

    // Create the commit
    let commit = Commit::new(
        branch_data.head.clone(),
        message.to_string(),
        tex_content.to_string(),
    );

    // Save commit to history
    let commits_path = ws
        .branches_dir()
        .join(branch_name)
        .join("commits.json");
    let mut history = load_history(ws, branch_name)?;
    history.push(commit.clone());
    std::fs::write(&commits_path, serde_json::to_string_pretty(&history)?)?;

    // Update snapshot
    let snapshot_path = ws
        .branches_dir()
        .join(branch_name)
        .join("snapshot.tex");
    std::fs::write(snapshot_path, tex_content)?;

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

    // --- resolve -----------------------------------------------------------

    #[test]
    fn test_resolve_accepts_full_hash() {
        let (_tmp, ws) = setup();
        let c = create(&ws, "one", &AllowAuthenticator).unwrap();
        let found = resolve(&ws, "main", &c.hash.0).unwrap();
        assert_eq!(found.hash, c.hash);
    }

    #[test]
    fn test_resolve_accepts_unambiguous_prefix() {
        let (_tmp, ws) = setup();
        let c = create(&ws, "one", &AllowAuthenticator).unwrap();
        let prefix = &c.hash.0[..8];
        let found = resolve(&ws, "main", prefix).unwrap();
        assert_eq!(found.hash, c.hash);
    }

    #[test]
    fn test_resolve_is_case_insensitive() {
        let (_tmp, ws) = setup();
        let c = create(&ws, "one", &AllowAuthenticator).unwrap();
        let upper = c.hash.0.to_uppercase();
        assert_eq!(resolve(&ws, "main", &upper).unwrap().hash, c.hash);
    }

    #[test]
    fn test_resolve_head_returns_tip() {
        let (_tmp, ws) = setup();
        create(&ws, "one", &AllowAuthenticator).unwrap();
        let second = create(&ws, "two", &AllowAuthenticator).unwrap();
        assert_eq!(resolve(&ws, "main", "HEAD").unwrap().hash, second.hash);
        assert_eq!(resolve(&ws, "main", "head").unwrap().hash, second.hash);
    }

    #[test]
    fn test_resolve_head_on_empty_branch_errors() {
        let (_tmp, ws) = setup();
        assert!(matches!(
            resolve(&ws, "main", "HEAD").unwrap_err(),
            RvmError::NoCommits
        ));
    }

    #[test]
    fn test_resolve_unknown_reference_errors() {
        let (_tmp, ws) = setup();
        create(&ws, "one", &AllowAuthenticator).unwrap();
        assert!(matches!(
            resolve(&ws, "main", "deadbeef").unwrap_err(),
            RvmError::CommitNotFound(_)
        ));
        // A prefix below the minimum length is not even treated as a hash.
        assert!(matches!(
            resolve(&ws, "main", "ab").unwrap_err(),
            RvmError::CommitNotFound(_)
        ));
    }

    // --- restore -----------------------------------------------------------

    #[test]
    fn test_restore_brings_content_back_as_a_new_commit() {
        let (tmp, ws) = setup();
        std::fs::write(tmp.path().join("resume.tex"), "GOOD\n").unwrap();
        let good = create(&ws, "good state", &AllowAuthenticator).unwrap();

        std::fs::write(tmp.path().join("resume.tex"), "POLLUTED\n").unwrap();
        create(&ws, "bad state", &AllowAuthenticator).unwrap();

        let (target, new_commit) =
            restore(&ws, "main", &good.hash.0[..8], &AllowAuthenticator).unwrap();

        assert_eq!(target.hash, good.hash);
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
            "GOOD\n"
        );
        // History grew rather than being rewritten: nothing is lost.
        let history = load_history(&ws, "main").unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history.last().unwrap().hash, new_commit.hash);
        assert_eq!(branch::load(&ws, "main").unwrap().head, Some(new_commit.hash));
        assert!(new_commit.message.contains("Restore to"));
    }

    #[test]
    fn test_restore_of_identical_content_is_refused() {
        let (_tmp, ws) = setup();
        let c = create(&ws, "one", &AllowAuthenticator).unwrap();
        // Working file already matches, so there is nothing to do.
        let err = restore(&ws, "main", &c.hash.0[..8], &AllowAuthenticator).unwrap_err();
        assert!(format!("{err}").contains("nothing to restore"));
        assert_eq!(load_history(&ws, "main").unwrap().len(), 1);
    }

    #[test]
    fn test_restore_on_protected_branch_requires_auth() {
        let (tmp, ws) = setup();
        std::fs::write(tmp.path().join("resume.tex"), "GOOD\n").unwrap();
        let good = create(&ws, "good", &AllowAuthenticator).unwrap();
        std::fs::write(tmp.path().join("resume.tex"), "POLLUTED\n").unwrap();
        create(&ws, "bad", &AllowAuthenticator).unwrap();
        let before = std::fs::read_to_string(commits_json_path(&ws, "main")).unwrap();

        // A non-interactive agent cannot reach the restore path either.
        assert!(restore(&ws, "main", &good.hash.0[..8], &DenyAuthenticator).is_err());

        assert_eq!(
            std::fs::read_to_string(commits_json_path(&ws, "main")).unwrap(),
            before,
            "a refused restore must not touch history"
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
            "POLLUTED\n",
            "a refused restore must not touch the working file"
        );
    }

    fn commits_json_path(ws: &Workspace, branch: &str) -> std::path::PathBuf {
        ws.branches_dir().join(branch).join("commits.json")
    }
}
