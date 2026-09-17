use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn rvm_cmd() -> Command {
    Command::cargo_bin("rvm").unwrap()
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
    std::fs::write(
        tmp.path().join("resume.tex"),
        r#"\documentclass{article}
\begin{document}
\section*{Experience}
Test resume content.
\end{document}
"#,
    )
    .unwrap();

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
