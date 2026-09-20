//! Single-shot client for the external agent CLI that performs the model call.
//!
//! RVM never speaks HTTP to a model and never holds an API key. It shells out
//! to a CLI that already owns the operator's credentials, which keeps secrets
//! out of this process's argv, environment and memory — the same reasoning that
//! puts branch authentication in `dscl` rather than collecting a password here.
//!
//! `std` has no way to wait on a child with a deadline, so the wait is a poll
//! loop that kills the child on expiry. A wedged model call must not hang a
//! batch of fifty resumes.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use rvm_types::{RvmError, RvmResult};

/// Providers whose argv dialect this client knows how to build.
pub const SUPPORTED_PROVIDERS: &[&str] = &["antigravity_cli"];

/// Environment variable that overrides `ai.cli_path`.
///
/// Exists so an operator (or a test harness) can point RVM at a different
/// binary without editing a committed `rvm.toml`.
pub const CLI_ENV: &str = "RVM_AI_CLI";

pub struct CliClient {
    provider: String,
    binary: String,
    model: String,
    timeout: Duration,
}

impl CliClient {
    pub fn new(provider: &str, binary: &str, model: &str, timeout_seconds: u64) -> Self {
        Self {
            provider: provider.to_string(),
            binary: binary.to_string(),
            model: model.to_string(),
            timeout: Duration::from_secs(timeout_seconds.max(1)),
        }
    }

    /// Resolve the binary: explicit config first, then the environment, then
    /// the provider's conventional name on `PATH`.
    pub fn resolve_binary(configured: &str) -> String {
        if !configured.trim().is_empty() {
            return configured.trim().to_string();
        }
        match std::env::var(CLI_ENV) {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => default_binary().to_string(),
        }
    }

    fn argv(&self, prompt: &str) -> RvmResult<Vec<String>> {
        match self.provider.as_str() {
            "antigravity" | "antigravity_cli" | "agy" => Ok(vec![
                self.binary.clone(),
                "-p".to_string(),
                prompt.to_string(),
                "--model".to_string(),
                self.model.clone(),
                // Skills and slash commands would expand inside the prompt and
                // change what the model is actually asked to do.
                "--disable-slash-commands".to_string(),
            ]),
            other => Err(RvmError::AiError(format!(
                "Unknown AI provider '{}'. Supported: {}.",
                other,
                SUPPORTED_PROVIDERS.join(", ")
            ))),
        }
    }

    /// Run one prompt to completion and return the model's stdout.
    ///
    /// Both pipes are drained on their own threads before waiting: the CLI is
    /// chatty on stderr, and a pipe that fills up would block the child forever
    /// while the poll loop waits for it to exit.
    pub fn complete(&self, prompt: &str) -> RvmResult<String> {
        let argv = self.argv(prompt)?;
        let program = argv[0].clone();

        let mut child = Command::new(&program)
            .args(&argv[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => RvmError::AiError(format!(
                    "AI CLI '{program}' not found. Install it, or set ai.cli_path in rvm.toml \
                     (or the {CLI_ENV} environment variable)."
                )),
                _ => RvmError::AiError(format!("Failed to run '{program}': {e}")),
            })?;

        let out_pipe = child
            .stdout
            .take()
            .ok_or_else(|| RvmError::AiError("AI CLI stdout was not captured".to_string()))?;
        let err_pipe = child
            .stderr
            .take()
            .ok_or_else(|| RvmError::AiError("AI CLI stderr was not captured".to_string()))?;

        let out_thread = std::thread::spawn(move || drain(out_pipe));
        let err_thread = std::thread::spawn(move || drain(err_pipe));

        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        let _ = out_thread.join();
                        let _ = err_thread.join();
                        return Err(RvmError::AiError(format!(
                            "AI CLI '{}' exceeded the {}s timeout.",
                            self.binary,
                            self.timeout.as_secs()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    let _ = child.kill();
                    return Err(RvmError::AiError(format!(
                        "Failed to wait for '{}': {e}",
                        self.binary
                    )));
                }
            }
        };

        let stdout = out_thread.join().unwrap_or_default();
        let stderr = err_thread.join().unwrap_or_default();

        if !status.success() {
            return Err(RvmError::AiError(format!(
                "AI CLI '{}' exited with {}: {}",
                self.binary,
                status,
                tail(&stderr, 300)
            )));
        }

        if stdout.trim().is_empty() {
            // Print mode auto-denies tool use, so a model that decides it needs
            // to run a command produces no answer at all. Say so, because the
            // raw stderr is hundreds of lines of language-server chatter.
            let hint = if stderr.contains("permission") || stderr.contains("tool") {
                "It tried to use a tool; print mode cannot approve tools, so the call was denied."
            } else {
                "See the CLI's stderr for details."
            };
            return Err(RvmError::AiError(format!(
                "AI CLI '{}' produced no output. {hint}",
                self.binary
            )));
        }

        Ok(stdout)
    }
}

fn default_binary() -> &'static str {
    "agy"
}

fn drain(mut pipe: impl Read) -> String {
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).to_string()
}

/// Keep the last `limit` characters, which is where a CLI puts its real error.
fn tail(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= limit {
        return trimmed.to_string();
    }
    chars[chars.len() - limit..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// A stand-in for `agy`, so the client is tested without a model, a
    /// network, or an Antigravity login. `$2` is where the prompt lands.
    fn fake_cli(body: &str) -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake-agy.sh");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        (dir, path)
    }

    fn client_for(path: &PathBuf, timeout: u64) -> CliClient {
        CliClient::new(
            "antigravity_cli",
            path.to_str().unwrap(),
            "gemini-3.8-flash-low",
            timeout,
        )
    }

    #[test]
    fn test_antigravity_argv_shape() {
        let client = CliClient::new("antigravity_cli", "/bin/true", "gemini-3.8-flash-low", 30);
        let argv = client.argv("hello").unwrap();
        assert_eq!(argv[0], "/bin/true");
        assert_eq!(argv[1], "-p");
        assert_eq!(argv[2], "hello");
        assert!(argv.contains(&"--model".to_string()));
        assert!(argv.contains(&"gemini-3.8-flash-low".to_string()));
        assert!(argv.contains(&"--disable-slash-commands".to_string()));
        // The prompt is never re-interpreted as a permission grant.
        assert!(!argv.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn test_unknown_provider_is_rejected() {
        let client = CliClient::new("nope", "/bin/true", "m", 30);
        assert!(client.argv("hello").is_err());
    }

    #[test]
    fn test_missing_binary_is_a_clear_error() {
        let client = CliClient::new(
            "antigravity_cli",
            "/nonexistent/rvm-ai-test-binary",
            "m",
            30,
        );
        let err = client.complete("hi").unwrap_err();
        assert!(format!("{err}").contains("not found"), "{err}");
    }

    #[test]
    fn test_complete_returns_stdout_verbatim() {
        let (_dir, bin) = fake_cli("printf 'the tailored body'");
        let out = client_for(&bin, 30).complete("hi").unwrap();
        assert_eq!(out, "the tailored body");
    }

    #[test]
    fn test_prompt_reaches_the_cli() {
        let (_dir, bin) = fake_cli("printf '%s' \"$2\"");
        let out = client_for(&bin, 30).complete("PROMPT TEXT").unwrap();
        assert_eq!(out, "PROMPT TEXT");
    }

    #[test]
    fn test_nonzero_exit_reports_tail_of_stderr() {
        let (_dir, bin) = fake_cli("echo 'it went wrong' >&2; exit 3");
        let err = client_for(&bin, 30).complete("hi").unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("exited with"), "{text}");
        assert!(text.contains("it went wrong"), "{text}");
    }

    #[test]
    fn test_successful_exit_with_no_stdout_is_an_error() {
        let (_dir, bin) = fake_cli("echo 'tool permission denied' >&2; exit 0");
        let err = client_for(&bin, 30).complete("hi").unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("produced no output"), "{text}");
        assert!(text.contains("tool"), "{text}");
    }

    #[test]
    fn test_timeout_kills_a_hung_child() {
        let (_dir, bin) = fake_cli("sleep 30");
        let started = Instant::now();
        let err = client_for(&bin, 1).complete("hi").unwrap_err();
        assert!(format!("{err}").contains("timeout"), "{err}");
        // Killed on the deadline rather than running the full thirty seconds.
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn test_large_stderr_does_not_deadlock() {
        // More than a pipe buffer's worth of chatter, which would block a naive
        // implementation until the timeout.
        let (_dir, bin) = fake_cli("i=0; while [ $i -lt 4000 ]; do echo 'noisy log line for the language server' >&2; i=$((i+1)); done; printf 'ok'");
        let out = client_for(&bin, 30).complete("hi").unwrap();
        assert_eq!(out, "ok");
    }

    #[test]
    fn test_resolve_binary_prefers_configured_path() {
        assert_eq!(CliClient::resolve_binary(" /opt/agy "), "/opt/agy");
    }

    #[test]
    fn test_tail_keeps_the_end() {
        assert_eq!(tail("abcdef", 3), "def");
        assert_eq!(tail("ab", 3), "ab");
    }
}
