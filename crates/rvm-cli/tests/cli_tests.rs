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
    // Pre-release 0.0.1: `rvm --version` must report the workspace package version.
    rvm_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(concat!("rvm ", env!("CARGO_PKG_VERSION"))));
}

#[test]
fn test_version_short_flag() {
    rvm_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.0.1"));
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
