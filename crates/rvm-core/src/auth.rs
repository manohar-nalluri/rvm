//! Authentication for protected branches.
//!
//! RVM's threat model here is an AI agent driving `rvm` on the operator's
//! behalf. The agent runs as the same OS user, so any secret RVM stores
//! locally (a hash, a token, a key) is readable by the agent. The only secret
//! the agent reliably does *not* hold is the operator's system password, and
//! the only place it can be entered that the agent cannot reach is a terminal
//! the operator is sitting at.
//!
//! Accordingly, protection is enforced by refusing to mutate a protected
//! branch unless an interactive authentication succeeds. There is
//! deliberately no `--force`, no environment-variable bypass, and no
//! non-interactive mode: any such escape hatch would be reachable by the agent
//! and would defeat the feature.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::Command;

use rvm_types::{RvmError, RvmResult};

/// Path to the macOS Directory Services CLI used for password verification.
const DSCL: &str = "/usr/bin/dscl";

/// Number of password attempts before giving up (matches `sudo`).
pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Verifies that the operator may modify a protected branch.
pub trait Authenticator {
    /// Returns `Ok(())` once the operator is authenticated.
    ///
    /// Implementations must **fail closed**: returning `Ok(())` without a
    /// genuine successful verification would silently disable branch
    /// protection.
    fn authenticate(&self, branch: &str) -> RvmResult<()>;
}

/// Denies every request.
///
/// Use this when no interactive authentication is possible, so that callers
/// get an explicit "not permitted" rather than accidentally allowing writes.
#[derive(Debug, Default, Clone, Copy)]
pub struct DenyAuthenticator;

impl Authenticator for DenyAuthenticator {
    fn authenticate(&self, branch: &str) -> RvmResult<()> {
        Err(RvmError::AuthenticationUnavailable(format!(
            "branch '{branch}' is protected and no authentication method is available"
        )))
    }
}

/// Approves every request without verifying anything.
///
/// Intended for tests and for callers that have already authenticated by other
/// means. Never use it on a path an untrusted agent can reach.
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAuthenticator;

impl Authenticator for AllowAuthenticator {
    fn authenticate(&self, _branch: &str) -> RvmResult<()> {
        Ok(())
    }
}

/// Verifies the operator's system (login) password.
///
/// On macOS this delegates to Directory Services:
///
/// ```text
/// dscl . -authonly <user>
/// ```
///
/// The password is deliberately **not** passed to `dscl`. When it is omitted,
/// `dscl` prompts on the controlling terminal with echo disabled, so the
/// password never enters this process's memory, its argv, or its environment.
///
/// Two properties matter here:
///
/// * **No privileged state is created.** `sudo` is never invoked, so no cached
///   sudo timestamp is granted that another process could ride on for the next
///   few minutes.
/// * **No password in argv.** `ps` output is readable by any process running as
///   the same user, which includes the agent.
#[derive(Debug, Clone)]
pub struct SystemAuthenticator {
    /// Login name to authenticate. Resolved lazily so that constructing an
    /// authenticator (done on every mutating command) cannot fail.
    user: Option<String>,
    max_attempts: u32,
    program: PathBuf,
}

impl Default for SystemAuthenticator {
    fn default() -> Self {
        Self {
            user: None,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            program: PathBuf::from(DSCL),
        }
    }
}

impl SystemAuthenticator {
    /// Create an authenticator for the current user. Infallible.
    pub fn new() -> Self {
        Self::default()
    }

    /// Maximum number of password attempts before giving up.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Override the attempt limit.
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts.max(1);
        self
    }

    /// Login name that will be authenticated.
    pub fn user(&self) -> RvmResult<String> {
        match &self.user {
            Some(u) => Ok(u.clone()),
            None => current_user(),
        }
    }

    /// Run one verification attempt, returning whether the password was
    /// accepted. Does not prompt on its own: `dscl` owns the prompt.
    fn verify_once(&self) -> RvmResult<bool> {
        if !cfg!(target_os = "macos") {
            return Err(RvmError::AuthenticationUnavailable(
                "system-password authentication is implemented for macOS only; \
                 on other platforms unprotect the branch from a trusted shell with \
                 `rvm branch --unprotect <branch>`"
                    .to_string(),
            ));
        }

        // stdio is inherited on purpose: `dscl` must be able to prompt on, and
        // read the password from, the operator's terminal.
        let status = Command::new(&self.program)
            .arg(".")
            .arg("-authonly")
            .arg(self.user()?)
            .status()
            .map_err(|e| {
                RvmError::AuthenticationUnavailable(format!(
                    "could not run {}: {e}",
                    self.program.display()
                ))
            })?;

        Ok(status.success())
    }
}

impl Authenticator for SystemAuthenticator {
    fn authenticate(&self, branch: &str) -> RvmResult<()> {
        // Hard requirement: without a terminal the prompt cannot be answered by
        // a human, so refuse rather than hang or read a piped guess.
        if !std::io::stdin().is_terminal() {
            return Err(RvmError::AuthenticationUnavailable(format!(
                "branch '{branch}' is protected and needs your system password, but no \
                 interactive terminal is available (stdin is not a TTY)"
            )));
        }

        let user = self.user()?;

        // Written to stderr directly rather than via `tracing`: a password
        // prompt must never be suppressible by a log filter.
        eprintln!("Branch '{branch}' is protected.");
        eprintln!("Authenticating as '{user}' using your system password.");

        attempt_loop(branch, self.max_attempts, || self.verify_once())?;
        eprintln!("Authenticated.");
        Ok(())
    }
}

/// Drive `verify` up to `attempts` times, stopping at the first acceptance.
///
/// Extracted from [`SystemAuthenticator::authenticate`] so the retry policy can
/// be tested without performing real (and possibly rate-limited) login attempts
/// against the operator's account.
fn attempt_loop<F>(branch: &str, attempts: u32, mut verify: F) -> RvmResult<()>
where
    F: FnMut() -> RvmResult<bool>,
{
    for attempt in 1..=attempts {
        if verify()? {
            return Ok(());
        }
        if attempt < attempts {
            let remaining = attempts - attempt;
            eprintln!(
                "Incorrect password ({} attempt{} remaining).",
                remaining,
                if remaining == 1 { "" } else { "s" }
            );
        }
    }
    Err(RvmError::AuthenticationFailed(branch.to_string()))
}

/// Resolve the current login name.
fn current_user() -> RvmResult<String> {
    if let Ok(user) = std::env::var("USER") {
        if !user.trim().is_empty() {
            return Ok(user);
        }
    }

    let output = Command::new("/usr/bin/id")
        .arg("-un")
        .output()
        .map_err(|e| RvmError::AuthenticationUnavailable(format!("could not determine user: {e}")))?;

    let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if user.is_empty() {
        return Err(RvmError::AuthenticationUnavailable(
            "could not determine the current user; set the USER environment variable"
                .to_string(),
        ));
    }
    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deny_authenticator_always_fails() {
        assert!(DenyAuthenticator.authenticate("main").is_err());
    }

    #[test]
    fn test_allow_authenticator_always_succeeds() {
        assert!(AllowAuthenticator.authenticate("main").is_ok());
    }

    #[test]
    fn test_verify_once_maps_exit_status_to_acceptance() {
        let accepting = SystemAuthenticator {
            user: Some("tester".to_string()),
            max_attempts: 1,
            program: PathBuf::from("/usr/bin/true"),
        };
        assert!(accepting.verify_once().unwrap());

        let rejecting = SystemAuthenticator {
            user: Some("tester".to_string()),
            max_attempts: 1,
            program: PathBuf::from("/usr/bin/false"),
        };
        assert!(!rejecting.verify_once().unwrap());
    }

    #[test]
    fn test_verify_once_reports_missing_program() {
        let missing = SystemAuthenticator {
            user: Some("tester".to_string()),
            max_attempts: 1,
            program: PathBuf::from("/nonexistent/dscl"),
        };
        let err = missing.verify_once().unwrap_err();
        assert!(matches!(err, RvmError::AuthenticationUnavailable(_)));
    }

    #[test]
    fn test_user_prefers_explicit_value() {
        let auth = SystemAuthenticator {
            user: Some("explicit-user".to_string()),
            ..SystemAuthenticator::default()
        };
        assert_eq!(auth.user().unwrap(), "explicit-user");
    }

    #[test]
    fn test_max_attempts_is_at_least_one() {
        assert_eq!(SystemAuthenticator::new().with_max_attempts(0).max_attempts(), 1);
    }

    #[test]
    fn test_current_user_is_resolved() {
        // Either the USER env var or `id -un` must yield something on a normal box.
        assert!(!current_user().unwrap().trim().is_empty());
    }

    #[test]
    fn test_attempt_loop_stops_at_first_success() {
        let mut calls = 0;
        attempt_loop("main", 3, || {
            calls += 1;
            Ok(true)
        })
        .unwrap();
        assert_eq!(calls, 1, "a correct password must not be retried");
    }

    #[test]
    fn test_attempt_loop_retries_then_succeeds() {
        let mut calls = 0;
        attempt_loop("main", 3, || {
            calls += 1;
            Ok(calls == 3)
        })
        .unwrap();
        assert_eq!(calls, 3);
    }

    #[test]
    fn test_attempt_loop_exhausts_attempts_and_fails_closed() {
        let mut calls = 0;
        let err = attempt_loop("main", 3, || {
            calls += 1;
            Ok(false)
        })
        .unwrap_err();
        assert!(matches!(err, RvmError::AuthenticationFailed(_)));
        assert_eq!(calls, 3, "must try exactly max_attempts times, no more");
    }

    #[test]
    fn test_attempt_loop_propagates_verifier_errors() {
        let err = attempt_loop("main", 3, || {
            Err(RvmError::AuthenticationUnavailable("no dscl".to_string()))
        })
        .unwrap_err();
        assert!(matches!(err, RvmError::AuthenticationUnavailable(_)));
    }

    #[test]
    fn test_default_attempt_limit_matches_sudo() {
        assert_eq!(SystemAuthenticator::new().max_attempts(), 3);
    }
}
