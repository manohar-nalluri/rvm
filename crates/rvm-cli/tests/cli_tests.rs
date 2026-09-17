use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn rvm_cmd() -> Command {
    Command::cargo_bin("rvm").unwrap()
}

/// Simulate an operator changing branch protection from a trusted shell.
///
/// Integration tests run without a terminal, so they cannot type a system
/// password. Writing both config files directly is the same thing
/// `rvm branch --protect/--unprotect` would do after authenticating.
fn set_protection(dir: &Path, branches: &[&str]) {
    let toml_list = branches
        .iter()
        .map(|b| format!("\"{b}\""))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        dir.join("rvm.toml"),
        format!("[protection]\nprotected_branches = [{toml_list}]\n"),
    )
    .unwrap();

    let json_list = branches
        .iter()
        .map(|b| format!("\"{b}\""))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(
        dir.join(".rvm/config"),
        format!(r#"{{"protection":{{"protected_branches":[{json_list}]}}}}"#),
    )
    .unwrap();
}

/// Unprotect `main`, for tests that are not about protection and need to
/// commit on `main` without a terminal.
fn unprotect_main(dir: &Path) {
    set_protection(dir, &[]);
}

const SAMPLE_TEX: &str = r#"\documentclass{article}
\begin{document}
\section*{Experience}
Test resume content.
\end{document}
"#;

fn init_workspace(dir: &Path) {
    rvm_cmd()
        .arg("init")
        .current_dir(dir)
        .assert()
        .success();
}

fn commits_json(dir: &Path, branch: &str) -> std::path::PathBuf {
    dir.join(".rvm").join("branches").join(branch).join("commits.json")
}

#[test]
fn test_init_creates_workspace() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized RVM workspace"));

    assert!(tmp.path().join(".rvm").exists());
    assert!(tmp.path().join(".rvm/HEAD").exists());
    assert!(tmp.path().join(".rvm/branches/main").exists());
    assert!(tmp.path().join("rvm.toml").exists());
}

#[test]
fn test_init_fails_if_already_initialized() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn test_branch_and_checkout() {
    let tmp = TempDir::new().unwrap();

    // Init
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    // Create a minimal resume.tex so we can commit
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    // This test is about branch/checkout, not protection.
    unprotect_main(tmp.path());

    // Commit
    rvm_cmd()
        .args(["commit", "-m", "Initial resume"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Create branch
    rvm_cmd()
        .args(["branch", "google-swe"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created branch 'google-swe'"));

    // Checkout
    rvm_cmd()
        .args(["checkout", "google-swe"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to branch 'google-swe'"));
}

#[test]
fn test_log_empty_branch() {
    let tmp = TempDir::new().unwrap();

    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .arg("log")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No commits"));
}

#[test]
fn test_help_output() {
    rvm_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("resume versions"));
}

#[test]
fn test_branch_list() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    // List branches (should show main)
    rvm_cmd()
        .args(["branch", "--list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_branch_list_default() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    // `rvm branch` with no args should also list
    rvm_cmd()
        .arg("branch")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_diff_no_changes() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .arg("diff")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn test_export() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    std::fs::write(
        tmp.path().join("resume.tex"),
        r#"\documentclass{article}
\begin{document}
Hello
\end{document}
"#,
    )
    .unwrap();

    // This test is about export, not protection.
    unprotect_main(tmp.path());

    rvm_cmd()
        .args(["commit", "-m", "first"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .arg("export")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported branch"));
}

#[test]
fn test_merge_clean() {
    let tmp = TempDir::new().unwrap();

    // Init and commit on main
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    std::fs::write(tmp.path().join("resume.tex"), "Base content\n").unwrap();

    // This test is about merging, not protection.
    unprotect_main(tmp.path());

    rvm_cmd()
        .args(["commit", "-m", "base"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Create branch and modify
    rvm_cmd()
        .args(["branch", "feature"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["checkout", "feature"])
        .current_dir(tmp.path())
        .assert()
        .success();

    std::fs::write(tmp.path().join("resume.tex"), "Feature content\n").unwrap();

    rvm_cmd()
        .args(["commit", "-m", "feature change"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Switch back to main and merge
    rvm_cmd()
        .args(["checkout", "main"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["merge", "feature"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("cleanly"));
}

#[test]
fn test_archive_and_list_archived() {
    let tmp = TempDir::new().unwrap();
    rvm_cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["branch", "old-branch"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["archive", "old-branch"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["branch", "--archived"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("old-branch"));
}

#[test]
fn test_version_flag_reports_package_version() {
    // `rvm --version` must report the workspace package version. Derived from
    // CARGO_PKG_VERSION so a version bump does not require editing this test.
    rvm_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(concat!(
            "rvm ",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn test_version_short_flag() {
    rvm_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(concat!(
            "rvm ",
            env!("CARGO_PKG_VERSION")
        )));
}

// ---------------------------------------------------------------------------
// Branch protection
// ---------------------------------------------------------------------------

#[test]
fn test_main_is_protected_by_default() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    rvm_cmd()
        .args(["branch", "--list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main [protected]"));
}

#[test]
fn test_commit_on_protected_main_is_refused_without_a_terminal() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    // This is the core guarantee: an agent with no terminal cannot commit to
    // the protected main branch.
    rvm_cmd()
        .args(["commit", "-m", "agent should not land this"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));

    // It must fail closed: nothing recorded, HEAD untouched.
    assert!(
        !commits_json(tmp.path(), "main").exists(),
        "a refused commit must not create history"
    );
    assert_eq!(
        std::fs::read_to_string(tmp.path().join(".rvm/HEAD")).unwrap(),
        "main"
    );
    assert!(tmp.path().join("resume.tex").exists());
}

#[test]
fn test_refused_commit_leaves_existing_history_intact() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    // Establish one authorised commit as the operator.
    unprotect_main(tmp.path());
    rvm_cmd()
        .args(["commit", "-m", "operator commit"])
        .current_dir(tmp.path())
        .assert()
        .success();
    let before = std::fs::read_to_string(commits_json(tmp.path(), "main")).unwrap();

    // Re-protect, then let the agent try.
    set_protection(tmp.path(), &["main"]);
    rvm_cmd()
        .args(["commit", "-m", "agent commit"])
        .current_dir(tmp.path())
        .assert()
        .failure();

    let after = std::fs::read_to_string(commits_json(tmp.path(), "main")).unwrap();
    assert_eq!(before, after, "history must be byte-identical after a refusal");
}

#[test]
fn test_commit_on_unprotected_branch_needs_no_password() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    // Only main is protected by default; feature branches stay frictionless so
    // the agent can do its job.
    rvm_cmd()
        .args(["branch", "google-swe"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("google-swe"));

    rvm_cmd()
        .args(["checkout", "google-swe"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["commit", "-m", "tailored for google"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[google-swe]"));
}

#[test]
fn test_unprotect_of_main_requires_authentication() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    rvm_cmd()
        .args(["branch", "--unprotect", "main"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));

    // Still protected afterwards.
    rvm_cmd()
        .args(["branch", "--list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main [protected]"));
}

#[test]
fn test_protect_of_unknown_branch_errors() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    rvm_cmd()
        .args(["branch", "--protect", "does-not-exist"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_merge_into_protected_main_is_refused() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    unprotect_main(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), "Base content\n").unwrap();

    rvm_cmd()
        .args(["commit", "-m", "base"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["branch", "feature"])
        .current_dir(tmp.path())
        .assert()
        .success();
    rvm_cmd()
        .args(["checkout", "feature"])
        .current_dir(tmp.path())
        .assert()
        .success();

    std::fs::write(tmp.path().join("resume.tex"), "Feature content\n").unwrap();
    rvm_cmd()
        .args(["commit", "-m", "feature change"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["checkout", "main"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // The operator re-protects main; merging in would otherwise be a way to
    // land changes on a protected branch without ever calling commit.
    set_protection(tmp.path(), &["main"]);

    rvm_cmd()
        .args(["merge", "feature"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));

    assert_eq!(
        std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
        "Base content\n",
        "a refused merge must not rewrite the working file"
    );
}

#[test]
fn test_archive_of_protected_branch_is_refused() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    unprotect_main(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    rvm_cmd()
        .args(["branch", "release"])
        .current_dir(tmp.path())
        .assert()
        .success();

    set_protection(tmp.path(), &["main", "release"]);

    rvm_cmd()
        .args(["archive", "release"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));

    // Still active.
    rvm_cmd()
        .args(["branch", "--list"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("release [protected]"));
}

#[test]
fn test_archive_of_unprotected_branch_still_works() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    unprotect_main(tmp.path());

    rvm_cmd()
        .args(["branch", "scratch"])
        .current_dir(tmp.path())
        .assert()
        .success();

    rvm_cmd()
        .args(["archive", "scratch"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived branch 'scratch'"));
}

#[test]
fn test_protection_applies_to_workspace_predating_the_feature() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    std::fs::write(tmp.path().join("resume.tex"), SAMPLE_TEX).unwrap();

    // Simulate an existing workspace: config files with no protection section,
    // exactly as written by the previous release.
    std::fs::write(
        tmp.path().join("rvm.toml"),
        "[compiler]\nengine = \"tectonic\"\nauto_compile = true\nwatch_mode = false\n",
    )
    .unwrap();
    std::fs::write(tmp.path().join(".rvm/config"), "{}").unwrap();

    rvm_cmd()
        .args(["commit", "-m", "legacy workspace"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));
}

// ---------------------------------------------------------------------------
// show / restore / diff against commits
// ---------------------------------------------------------------------------

/// Commit hashes on a branch, in history order.
///
/// Parsed with serde rather than by scanning for hex runs: every commit also
/// stores a `parent` hash, so a naive scan double-counts.
fn commit_hashes(dir: &Path, branch: &str) -> Vec<String> {
    let raw = std::fs::read_to_string(commits_json(dir, branch)).unwrap();
    let parsed: Vec<rvm_types::Commit> = serde_json::from_str(&raw).unwrap();
    parsed.into_iter().map(|c| c.hash.0).collect()
}

/// Build a workspace with two commits on an unprotected `main`.
fn workspace_with_two_revisions(dir: &Path) -> Vec<String> {
    init_workspace(dir);
    unprotect_main(dir);
    std::fs::write(dir.join("resume.tex"), "GOOD CONTENT\n").unwrap();
    rvm_cmd()
        .args(["commit", "-m", "good state"])
        .current_dir(dir)
        .assert()
        .success();
    std::fs::write(dir.join("resume.tex"), "POLLUTED CONTENT\n").unwrap();
    rvm_cmd()
        .args(["commit", "-m", "polluted state"])
        .current_dir(dir)
        .assert()
        .success();
    commit_hashes(dir, "main")
}

#[test]
fn test_show_prints_previous_commit_content() {
    let tmp = TempDir::new().unwrap();
    let hashes = workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["show", &hashes[0][..8]])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("GOOD CONTENT"))
        .stdout(predicate::str::contains("POLLUTED CONTENT").not());
}

#[test]
fn test_show_head_prints_latest_content() {
    let tmp = TempDir::new().unwrap();
    workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["show", "HEAD"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("POLLUTED CONTENT"));
}

#[test]
fn test_show_unknown_reference_fails() {
    let tmp = TempDir::new().unwrap();
    workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["show", "deadbeef"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_diff_against_a_commit_actually_diffs() {
    // Regression test: `rvm diff <hash>` used to look for a *branch* of that
    // name, find nothing, and print the entire resume as added lines while
    // still exiting 0.
    let tmp = TempDir::new().unwrap();
    let hashes = workspace_with_two_revisions(tmp.path());

    let output = rvm_cmd()
        .args(["diff", &hashes[0][..8]])
        .current_dir(tmp.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&output);

    assert!(
        !text.contains("@@ -0,0"),
        "diff reported the whole file as new instead of resolving the commit:\n{text}"
    );
    assert!(
        text.contains("-GOOD CONTENT") && text.contains("+POLLUTED CONTENT"),
        "expected a real hunk between the two revisions, got:\n{text}"
    );
}

#[test]
fn test_diff_unknown_reference_fails_instead_of_lying() {
    let tmp = TempDir::new().unwrap();
    workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["diff", "not-a-real-branch-or-commit"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a branch"));
}

#[test]
fn test_restore_brings_back_an_older_revision() {
    let tmp = TempDir::new().unwrap();
    let hashes = workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["restore", &hashes[0][..8]])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored"))
        .stdout(predicate::str::contains("History was not rewritten"));

    assert_eq!(
        std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
        "GOOD CONTENT\n"
    );
    // A forward restore: two originals plus the restore commit.
    assert_eq!(commit_hashes(tmp.path(), "main").len(), 3);
    assert_eq!(
        std::fs::read_to_string(tmp.path().join(".rvm/branches/main/snapshot.tex")).unwrap(),
        "GOOD CONTENT\n"
    );
}

#[test]
fn test_restore_is_reversible() {
    let tmp = TempDir::new().unwrap();
    let hashes = workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["restore", &hashes[0][..8]])
        .current_dir(tmp.path())
        .assert()
        .success();

    // The polluted revision was never erased, so it can be restored back.
    rvm_cmd()
        .args(["restore", &hashes[1][..8]])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
        "POLLUTED CONTENT\n"
    );
}

#[test]
fn test_restore_refuses_when_content_already_matches() {
    let tmp = TempDir::new().unwrap();
    workspace_with_two_revisions(tmp.path());

    rvm_cmd()
        .args(["restore", "HEAD"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing to restore"));

    assert_eq!(commit_hashes(tmp.path(), "main").len(), 2);
}

#[test]
fn test_restore_on_protected_main_is_refused() {
    let tmp = TempDir::new().unwrap();
    let hashes = workspace_with_two_revisions(tmp.path());

    // Re-protect main: an agent must not be able to restore either.
    set_protection(tmp.path(), &["main"]);

    rvm_cmd()
        .args(["restore", &hashes[0][..8]])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("protected"));

    assert_eq!(
        std::fs::read_to_string(tmp.path().join("resume.tex")).unwrap(),
        "POLLUTED CONTENT\n",
        "a refused restore must not touch the working file"
    );
    assert_eq!(commit_hashes(tmp.path(), "main").len(), 2);
}
