//! `rvm branch --delete` — removing a branch, and refusing to remove the
//! protected ones without a terminal to authenticate at.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

#[allow(deprecated)]
fn rvm_cmd() -> Command {
    Command::cargo_bin("rvm").unwrap()
}

/// Protection lives in two files; integration tests cannot type a password, so
/// they write both directly, exactly as `rvm branch --protect` would.
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

fn setup(dir: &Path) {
    rvm_cmd().arg("init").current_dir(dir).assert().success();
    std::fs::write(dir.join("resume.tex"), "\\documentclass{article}\n").unwrap();
}

#[test]
fn test_delete_removes_an_unprotected_branch() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);

    rvm_cmd()
        .args(["branch", "scratch"])
        .current_dir(dir)
        .assert()
        .success();
    assert!(dir.join(".rvm/branches/scratch").exists());

    rvm_cmd()
        .args(["branch", "--delete", "scratch"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch 'scratch'"));

    assert!(!dir.join(".rvm/branches/scratch").exists());
    // The branch is gone from the listing too.
    rvm_cmd()
        .args(["branch", "--list"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("scratch").not());
}

#[test]
fn test_delete_of_main_is_refused() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);

    rvm_cmd()
        .args(["branch", "--delete", "main"])
        .current_dir(dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot delete the main branch"));

    assert!(dir.join(".rvm/branches/main").exists());
}

#[test]
fn test_delete_of_unknown_branch_errors() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);

    rvm_cmd()
        .args(["branch", "--delete", "ghost"])
        .current_dir(dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Branch 'ghost' not found"));
}

#[test]
fn test_delete_of_a_protected_branch_needs_authentication() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);

    rvm_cmd()
        .args(["branch", "release"])
        .current_dir(dir)
        .assert()
        .success();
    set_protection(dir, &["main", "release"]);

    // No TTY here, so the refusal is the only correct outcome — and it must
    // happen before anything is removed.
    rvm_cmd()
        .args(["branch", "--delete", "release"])
        .current_dir(dir)
        .assert()
        .failure();

    assert!(
        dir.join(".rvm/branches/release").exists(),
        "a refused delete must leave the branch in place"
    );
}

#[test]
fn test_delete_of_the_checked_out_branch_is_refused() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);

    rvm_cmd()
        .args(["branch", "wip"])
        .current_dir(dir)
        .assert()
        .success();
    rvm_cmd()
        .args(["checkout", "wip"])
        .current_dir(dir)
        .assert()
        .success();

    rvm_cmd()
        .args(["branch", "--delete", "wip"])
        .current_dir(dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("currently checked out"));
}
