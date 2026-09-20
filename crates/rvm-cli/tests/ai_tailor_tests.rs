//! End-to-end coverage for `rvm ai tailor`.
//!
//! No test here reaches a model. The AI binary is a shell script planted through
//! `RVM_AI_CLI`, so the suite stays offline, deterministic and fast — and the
//! prompt is never sent anywhere.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const SAMPLE_TEX: &str = r#"\documentclass{article}
\begin{document}
%-------------------- HEADER / CONTACT --------------------
\section{Experience}
Original experience body that the model is expected to rewrite entirely.
\section{Education}
Original education body.
\section{Skills}
Python, PostgreSQL
\end{document}
"#;

/// What the stub model "returns": a full, long-enough body that keeps every
/// required section, so it survives `tailor::lint`.
const STUB_BODY: &str = r#"%-------------------- HEADER / CONTACT --------------------
\section{Experience}
\textbf{Python} and \textbf{Rust} engineer who built and shipped backend services for real users.
\section{Education}
Bachelor of Technology in Computer Science.
\section{Skills}
Python, Rust, C, C++, PostgreSQL"#;

// `cargo_bin` is deprecated upstream in favour of a macro that needs a newer
// assert_cmd; this file mirrors the existing `cli_tests.rs` helper on purpose.
#[allow(deprecated)]
fn rvm_cmd() -> Command {
    Command::cargo_bin("rvm").unwrap()
}

/// Mirror of the helper in `cli_tests.rs`: protection is stored in two files and
/// integration tests have no terminal to authenticate with.
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
    std::fs::write(dir.join("resume.tex"), SAMPLE_TEX).unwrap();
}

/// Commit the base resume on `main`, then re-protect it. Everything after this
/// runs against a workspace where writing to `main` would need a password.
fn commit_base(dir: &Path) {
    set_protection(dir, &[]);
    rvm_cmd()
        .args(["commit", "-m", "base resume"])
        .current_dir(dir)
        .assert()
        .success();
    set_protection(dir, &["main"]);
}

fn write_jd(dir: &Path) -> PathBuf {
    let path = dir.join("acme-jd.txt");
    std::fs::write(
        &path,
        "Company: Acme Corp\nRole: Backend Engineer\n\nWe build distributed systems in Python and Rust.\n",
    )
    .unwrap();
    path
}

fn stub_cli(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("stub-agy.sh");
    std::fs::write(&path, format!("#!/bin/sh\ncat <<'RVM_STUB_EOF'\n{body}\nRVM_STUB_EOF\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

fn failing_cli(dir: &Path) -> PathBuf {
    let path = dir.join("failing-agy.sh");
    std::fs::write(&path, "#!/bin/sh\necho 'model exploded' >&2\nexit 3\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

fn commits(dir: &Path, branch: &str) -> usize {
    let path = dir.join(".rvm/branches").join(branch).join("commits.json");
    if !path.exists() {
        return 0;
    }
    let raw = std::fs::read_to_string(path).unwrap();
    serde_json::from_str::<Vec<serde_json::Value>>(&raw).unwrap().len()
}

fn head_of(dir: &Path, branch: &str) -> Option<String> {
    let path = dir.join(".rvm/branches").join(branch).join("branch.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value.get("head")?.as_str().map(|s| s.to_string())
}

fn current_branch(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".rvm/HEAD")).unwrap().trim().to_string()
}

fn tectonic_available() -> bool {
    std::process::Command::new("tectonic")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------

#[test]
fn test_tailor_creates_a_branch_and_leaves_main_untouched() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);

    let main_head_before = head_of(dir, "main");
    let main_commits_before = commits(dir, "main");
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args([
            "ai",
            "tailor",
            "--jd",
            jd.to_str().unwrap(),
            "--company",
            "Acme Corp",
            "--role",
            "Backend Engineer",
            "--branch",
            "acme-backend",
            "--no-compile",
            "--json",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();

    // `main` is protected and has no terminal here: any write to it would have
    // failed the command outright.
    assert_eq!(head_of(dir, "main"), main_head_before);
    assert_eq!(commits(dir, "main"), main_commits_before);

    // The tailored resume landed on its own branch. Forking copies the base
    // history, so the branch carries main's commit plus the tailored one.
    assert_eq!(commits(dir, "acme-backend"), 2);
    let snapshot =
        std::fs::read_to_string(dir.join(".rvm/branches/acme-backend/snapshot.tex")).unwrap();
    assert!(snapshot.contains("\\textbf{Python}"));
    // The operator's preamble is preserved byte for byte, but the old body is gone.
    assert!(snapshot.starts_with("\\documentclass{article}\n\\begin{document}\n"));
    assert!(!snapshot.contains("Original experience body"));

    // Job metadata travelled with the branch.
    let job = std::fs::read_to_string(dir.join(".rvm/branches/acme-backend/job.json")).unwrap();
    assert!(job.contains("Acme Corp"));
    assert!(job.contains("distributed systems in Python"));
}

#[test]
fn test_tailor_json_payload_describes_the_result() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);

    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    let output = rvm_cmd()
        .args([
            "ai",
            "tailor",
            "--jd",
            jd.to_str().unwrap(),
            "--branch",
            "acme-backend",
            "--no-compile",
            "--json",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Exactly one line of JSON on stdout, so a caller can parse it blindly.
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "stdout was: {stdout}");

    let payload: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(payload["ok"], serde_json::Value::Bool(true));
    assert_eq!(payload["branch"], "acme-backend");
    assert_eq!(payload["base"], "main");
    assert_eq!(payload["compiled"], serde_json::Value::Bool(false));
    assert!(payload["commit"].as_str().unwrap().len() >= 7);
    assert!(payload["keywords"]
        .as_array()
        .unwrap()
        .iter()
        .any(|k| k.as_str().unwrap().eq_ignore_ascii_case("python")));
}

#[test]
fn test_tailor_switches_branch_by_default_and_stays_put_with_no_checkout() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-a", "--no-compile",
            "--no-checkout",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();
    assert_eq!(current_branch(dir), "main", "--no-checkout must not move HEAD");

    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-b", "--no-compile",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();
    assert_eq!(current_branch(dir), "acme-b");
    // The working file now holds the tailored resume.
    let working = std::fs::read_to_string(dir.join("resume.tex")).unwrap();
    assert!(working.contains("\\textbf{Python}"));
}

#[test]
fn test_tailor_reports_a_failing_model_and_creates_no_branch() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = failing_cli(dir);
    let jd = write_jd(dir);

    let output = rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-fail", "--no-compile",
            "--json",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(payload["ok"], serde_json::Value::Bool(false));
    assert!(payload["error"].as_str().unwrap().contains("model exploded"));
    assert!(!dir.join(".rvm/branches/acme-fail").exists());
}

#[test]
fn test_tailor_refuses_a_base_branch_without_a_snapshot() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args(["ai", "tailor", "--jd", jd.to_str().unwrap()])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no committed snapshot"));
}

#[test]
fn test_tailor_without_force_refuses_to_clobber_an_existing_branch() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-dup",
            "--no-compile", "--no-checkout",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();

    // Without --force the second run must refuse rather than silently fork.
    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-dup", "--no-compile",
            "--no-checkout",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    assert_eq!(commits(dir, "acme-dup"), 2);
}

#[test]
fn test_tailor_force_replaces_an_existing_branch() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-dup",
            "--no-compile", "--no-checkout",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();
    assert_eq!(commits(dir, "acme-dup"), 2);

    rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-dup", "--no-compile",
            "--no-checkout", "--force",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();

    // Replaced, not appended to: a forced re-run still carries exactly the
    // base commit plus one tailored commit, not three.
    assert_eq!(commits(dir, "acme-dup"), 2);
}

#[test]
fn test_tailor_derives_a_branch_name_from_company_and_role() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    rvm_cmd()
        .args([
            "ai",
            "tailor",
            "--jd",
            jd.to_str().unwrap(),
            "--company",
            "NXP Semiconductors",
            "--role",
            "Software Engineer, Modelzoo",
            "--no-compile",
            "--no-checkout",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Tailored branch: nxp-semiconductors-software-engineer-modelzoo",
        ));
}

#[test]
fn test_tailor_compiles_and_enforces_the_page_limit_when_tectonic_is_present() {
    if !tectonic_available() {
        eprintln!("skipping: tectonic is not on PATH");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);
    let jd = write_jd(dir);

    let output = rvm_cmd()
        .args([
            "ai", "tailor", "--jd", jd.to_str().unwrap(), "--branch", "acme-pdf", "--json",
        ])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_str(String::from_utf8(output.stdout).unwrap().trim()).unwrap();
    assert_eq!(payload["compiled"], serde_json::Value::Bool(true));
    assert_eq!(payload["pages"], 1);
    assert_eq!(payload["checked_out"], serde_json::Value::Bool(true));
    // The compiled PDF is kept beside the branch snapshot and in the workspace.
    assert!(dir.join(".rvm/branches/acme-pdf/snapshot.pdf").exists());
    assert!(dir.join("resume.pdf").exists());
}

#[test]
fn test_tailor_falls_back_to_the_branch_job_description() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    setup(dir);
    commit_base(dir);
    let stub = stub_cli(dir, STUB_BODY);

    // No --jd at all: the JD comes from `.rvm/branches/<current>/job.json`.
    let job_dir = dir.join(".rvm/branches/main");
    std::fs::create_dir_all(&job_dir).unwrap();
    std::fs::write(
        job_dir.join("job.json"),
        r#"{"company":"Acme Corp","role":"Backend Engineer","jd_text":"Python and Rust distributed systems"}"#,
    )
    .unwrap();

    rvm_cmd()
        .args(["ai", "tailor", "--branch", "acme-from-job", "--no-compile", "--no-checkout"])
        .env("RVM_AI_CLI", &stub)
        .current_dir(dir)
        .assert()
        .success();

    let job =
        std::fs::read_to_string(dir.join(".rvm/branches/acme-from-job/job.json")).unwrap();
    assert!(job.contains("Python and Rust distributed systems"));
}
