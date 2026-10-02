//! End-to-end coverage for the `.docx` that shadows the PDF.
//!
//! `tectonic` and `pandoc` are both stubbed on `PATH` for most of these, so the
//! tests assert RVM's own behaviour — one file per branch, rebuilt on commit and
//! on checkout, reported rather than fatal when the converter is missing —
//! without depending on a real TeX installation. The two tests that need a real
//! toolchain skip themselves when it is absent, which is what
//! `ai_tailor_tests.rs` already does for tectonic.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn rvm_cmd() -> Command {
    Command::cargo_bin("rvm").unwrap()
}

const MAIN_TEX: &str = r#"\documentclass{article}
\begin{document}
\section*{Experience}
Main branch content.
\end{document}
"#;

const FEATURE_TEX: &str = r#"\documentclass{article}
\begin{document}
\section*{Experience}
Feature branch content.
\end{document}
"#;

fn write_script(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// A `tectonic` stub that writes `<stem>.pdf` next to the source.
///
/// It echoes the source into the artifact, so a test can tell which branch's
/// content a PDF was built from.
fn write_fake_tectonic(bin: &Path) -> PathBuf {
    std::fs::create_dir_all(bin).unwrap();
    write_script(
        &bin.join("tectonic"),
        r#"#!/bin/sh
tex=""
outdir=""
while [ $# -gt 0 ]; do
  case "$1" in
    --outdir) shift; outdir="$1" ;;
    *.tex) tex="$1" ;;
  esac
  shift
done
stem=$(basename "$tex" .tex)
printf 'pdf:%s' "$(cat "$tex")" > "$outdir/$stem.pdf"
"#,
    );
    bin.to_path_buf()
}

/// A `pandoc` stub that writes whatever `--output` names, echoing its input.
fn write_fake_pandoc(bin: &Path) -> PathBuf {
    std::fs::create_dir_all(bin).unwrap();
    write_script(
        &bin.join("pandoc"),
        r#"#!/bin/sh
in="$1"
out=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--output" ]; then shift; out="$1"; fi
  shift
done
printf 'docx:%s' "$(cat "$in")" > "$out"
"#,
    );
    bin.to_path_buf()
}

fn rvm_cmd_with_path(bin: &Path) -> Command {
    let mut cmd = rvm_cmd();
    let path = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{}", bin.display(), path));
    cmd
}

/// Leave `main` unprotected (integration tests have no terminal to type a
/// system password into) and set the two compiler switches under test.
fn configure(dir: &Path, auto_compile: bool, docx: bool) {
    std::fs::write(
        dir.join("rvm.toml"),
        format!(
            "[protection]\nprotected_branches = []\n\n\
             [compiler]\nengine = \"tectonic\"\nauto_compile = {auto_compile}\ndocx = {docx}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join(".rvm/config"),
        r#"{"protection":{"protected_branches":[]}}"#,
    )
    .unwrap();
}

fn init(dir: &Path) {
    rvm_cmd().arg("init").current_dir(dir).assert().success();
}

fn commit(dir: &Path, message: &str) {
    rvm_cmd()
        .args(["commit", "-m", message])
        .current_dir(dir)
        .assert()
        .success();
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap_or_else(|e| panic!("cannot read {name}: {e}"))
}

fn count_extension(dir: &Path, extension: &str) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .map(|ext| ext == extension)
                .unwrap_or(false)
        })
        .count()
}

fn tectonic_available() -> bool {
    std::process::Command::new("tectonic")
        .arg("--version")
        .output()
        .is_ok()
}

fn pandoc_available() -> bool {
    std::process::Command::new("pandoc")
        .arg("--version")
        .output()
        .is_ok()
}

/// Build a workspace with `main` and `feature` committed, compiling disabled so
/// the setup itself needs no toolchain.
fn workspace_with_two_branches() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let bin = write_fake_tectonic(&tmp.path().join("fake-tectonic"));
    write_fake_pandoc(&tmp.path().join("fake-tectonic"));

    init(tmp.path());
    configure(tmp.path(), false, true);

    std::fs::write(tmp.path().join("resume.tex"), MAIN_TEX).unwrap();
    commit(tmp.path(), "main resume");

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
    std::fs::write(tmp.path().join("resume.tex"), FEATURE_TEX).unwrap();
    commit(tmp.path(), "feature resume");

    (tmp, bin)
}

#[test]
fn test_checkout_rebuilds_both_artifacts_for_the_branch() {
    let (tmp, bin) = workspace_with_two_branches();
    let dir = tmp.path();

    // Only now is compiling on checkout enabled, which is the behaviour under
    // test: switching branches must refresh the PDF *and* the DOCX.
    configure(dir, true, true);

    rvm_cmd_with_path(&bin)
        .args(["checkout", "main"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Compiled:"))
        .stdout(predicate::str::contains("DOCX:"));

    assert!(read(dir, "resume.pdf").contains("Main branch content"));
    assert!(read(dir, "resume.docx").contains("Main branch content"));

    rvm_cmd_with_path(&bin)
        .args(["checkout", "feature"])
        .current_dir(dir)
        .assert()
        .success();

    // The same two files were overwritten, not joined by new ones.
    assert!(read(dir, "resume.pdf").contains("Feature branch content"));
    assert!(read(dir, "resume.docx").contains("Feature branch content"));
    assert_eq!(count_extension(dir, "docx"), 1);
    assert_eq!(count_extension(dir, "pdf"), 1);
}

#[test]
fn test_docx_false_suppresses_the_docx() {
    let (tmp, bin) = workspace_with_two_branches();
    let dir = tmp.path();
    configure(dir, true, false);

    rvm_cmd_with_path(&bin)
        .args(["checkout", "main"])
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Compiled:"))
        .stdout(predicate::str::contains("DOCX:").not());

    assert!(dir.join("resume.pdf").exists());
    assert!(!dir.join("resume.docx").exists());
}

#[test]
fn test_missing_pandoc_is_reported_but_does_not_fail_the_checkout() {
    let (tmp, bin) = workspace_with_two_branches();
    let dir = tmp.path();
    configure(dir, true, true);

    let mut cmd = rvm_cmd_with_path(&bin);
    cmd.env("RVM_PANDOC", "/nonexistent/pandoc-for-rvm-tests");
    cmd.args(["checkout", "main"])
        .current_dir(dir)
        .assert()
        .success()
        .stderr(predicate::str::contains("DOCX skipped"))
        .stderr(predicate::str::contains("brew install pandoc"));

    // The PDF is the artifact that gates the work, so it must still be there.
    assert!(dir.join("resume.pdf").exists());
    assert!(!dir.join("resume.docx").exists());
}

#[test]
fn test_commit_writes_a_docx_beside_the_pdf() {
    if !tectonic_available() {
        eprintln!("skipping: tectonic is not on PATH");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let bin = write_fake_pandoc(&tmp.path().join("fake-pandoc"));
    init(tmp.path());
    configure(tmp.path(), true, true);
    std::fs::write(tmp.path().join("resume.tex"), MAIN_TEX).unwrap();

    rvm_cmd_with_path(&bin)
        .args(["commit", "-m", "initial"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("DOCX:"));

    assert!(read(tmp.path(), "resume.docx").contains("Main branch content"));
    assert!(tmp.path().join("resume.pdf").exists());
}

#[test]
fn test_export_bundles_the_docx() {
    let (tmp, bin) = workspace_with_two_branches();
    let dir = tmp.path();

    // Put a DOCX in the working tree the way commit or checkout would.
    configure(dir, true, true);
    rvm_cmd_with_path(&bin)
        .args(["checkout", "main"])
        .current_dir(dir)
        .assert()
        .success();
    assert!(dir.join("resume.docx").exists());

    // Some portals accept Word and not PDF, which is the whole reason the DOCX
    // is kept — so it has to travel with the bundle.
    rvm_cmd()
        .arg("export")
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("resume.docx"));

    assert!(
        dir.join("export").join("main").join("resume.docx").exists(),
        "docx missing from the export directory"
    );
}

#[test]
fn test_real_pandoc_emits_a_docx_package() {
    if !tectonic_available() || !pandoc_available() {
        eprintln!("skipping: tectonic and pandoc are both required");
        return;
    }

    let tmp = TempDir::new().unwrap();
    init(tmp.path());
    configure(tmp.path(), true, true);
    std::fs::write(tmp.path().join("resume.tex"), MAIN_TEX).unwrap();

    rvm_cmd()
        .args(["commit", "-m", "initial"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // A .docx is a ZIP package, so it must start with the local file header.
    let bytes = std::fs::read(tmp.path().join("resume.docx")).unwrap();
    assert!(bytes.starts_with(b"PK"), "not a ZIP container");
    assert!(
        bytes.len() > 1000,
        "suspiciously small docx: {}",
        bytes.len()
    );
}
