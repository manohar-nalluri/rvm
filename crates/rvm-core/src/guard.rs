//! The single choke point for branch protection.
//!
//! Every mutating operation routes through [`ensure_mutable`] so that no code
//! path can write to a protected branch without authentication. Adding a new
//! mutating operation means adding one call here, not re-deriving the rules.

use rvm_types::RvmResult;

use crate::auth::Authenticator;
use crate::config;
use crate::workspace::Workspace;

/// Refuse a mutating operation on `branch` unless it is unprotected or the
/// operator authenticates successfully.
///
/// Unprotected branches short-circuit before the authenticator is consulted, so
/// normal work never prompts and never requires a terminal.
pub fn ensure_mutable(ws: &Workspace, branch: &str, auth: &dyn Authenticator) -> RvmResult<()> {
    if !config::is_protected(ws, branch)? {
        return Ok(());
    }
    auth.authenticate(branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AllowAuthenticator, DenyAuthenticator};
    use tempfile::TempDir;

    fn setup() -> (TempDir, Workspace) {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        (tmp, ws)
    }

    #[test]
    fn test_protected_branch_requires_authentication() {
        let (_tmp, ws) = setup();
        let err = ensure_mutable(&ws, "main", &DenyAuthenticator).unwrap_err();
        assert!(matches!(err, rvm_types::RvmError::AuthenticationUnavailable(_)));
    }

    #[test]
    fn test_protected_branch_allows_authenticated_operator() {
        let (_tmp, ws) = setup();
        assert!(ensure_mutable(&ws, "main", &AllowAuthenticator).is_ok());
    }

    #[test]
    fn test_unprotected_branch_never_asks_for_auth() {
        let (_tmp, ws) = setup();
        // `DenyAuthenticator` proves the authenticator is never consulted.
        assert!(ensure_mutable(&ws, "feature", &DenyAuthenticator).is_ok());
    }
}
