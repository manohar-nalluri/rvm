use std::collections::BTreeSet;
use std::path::Path;

use rvm_types::{RvmConfig, RvmError, RvmResult};

use crate::workspace::Workspace;

/// Load config from an rvm.toml file.
pub fn load(path: &Path) -> RvmResult<RvmConfig> {
    if !path.exists() {
        return Ok(RvmConfig::default());
    }
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| RvmError::ConfigError(e.to_string()))
}

/// Save config to an rvm.toml file.
pub fn save(path: &Path, config: &RvmConfig) -> RvmResult<()> {
    let content =
        toml::to_string_pretty(config).map_err(|e| RvmError::ConfigError(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Branch protection
// ---------------------------------------------------------------------------

/// Effective set of protected branches.
///
/// This is the **union** of the lists in `rvm.toml` and `.rvm/config`. Each
/// file independently falls back to `["main"]` when it is missing, unreadable,
/// or predates the `protection` field, which makes protection fail closed.
///
/// Union semantics exist so that clearing one file is not enough to unlock a
/// branch: a hand-edit of `rvm.toml` alone leaves the `.rvm/config` copy in
/// force.
pub fn list_protected(ws: &Workspace) -> RvmResult<Vec<String>> {
    let mut set = BTreeSet::new();
    for config in [ws.load_config()?, ws.load_local_config()?] {
        for branch in config.protection.protected_branches {
            set.insert(branch);
        }
    }
    Ok(set.into_iter().collect())
}

/// Whether `branch` is protected.
pub fn is_protected(ws: &Workspace, branch: &str) -> RvmResult<bool> {
    Ok(list_protected(ws)?.iter().any(|b| b == branch))
}

/// Add or remove a branch from the protected set.
///
/// The resulting list is materialised into **both** config files. Writing only
/// one would leave the other's `["main"]` default in place, so a branch could
/// never actually be unprotected.
pub fn set_protected(ws: &Workspace, branch: &str, protected: bool) -> RvmResult<()> {
    let mut effective = list_protected(ws)?;

    if protected {
        if !effective.iter().any(|b| b == branch) {
            effective.push(branch.to_string());
        }
    } else {
        effective.retain(|b| b != branch);
    }
    effective.sort();
    effective.dedup();

    let mut toml_config = ws.load_config()?;
    toml_config.protection.protected_branches = effective.clone();
    ws.save_config(&toml_config)?;

    let mut local_config = ws.load_local_config()?;
    local_config.protection.protected_branches = effective;
    ws.save_local_config(&local_config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Workspace) {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        (tmp, ws)
    }

    #[test]
    fn test_main_is_protected_by_default() {
        let (_tmp, ws) = setup();
        assert!(is_protected(&ws, "main").unwrap());
        assert!(!is_protected(&ws, "feature").unwrap());
    }

    #[test]
    fn test_protection_fails_closed_when_config_files_are_deleted() {
        let (tmp, ws) = setup();
        std::fs::remove_file(tmp.path().join("rvm.toml")).unwrap();
        std::fs::remove_file(tmp.path().join(".rvm/config")).unwrap();
        assert!(
            is_protected(&ws, "main").unwrap(),
            "deleting the config must not unlock main"
        );
    }

    #[test]
    fn test_legacy_config_without_protection_field_still_protects_main() {
        let (tmp, ws) = setup();
        // Exactly what a workspace created before this feature looks like:
        // no `[protection]` table at all.
        std::fs::write(
            tmp.path().join("rvm.toml"),
            "[compiler]\nengine = \"tectonic\"\nauto_compile = true\nwatch_mode = false\n",
        )
        .unwrap();
        assert!(
            is_protected(&ws, "main").unwrap(),
            "pre-existing workspaces must protect main without running init again"
        );
    }

    #[test]
    fn test_legacy_local_config_without_protection_field_still_protects_main() {
        let (tmp, ws) = setup();
        std::fs::write(tmp.path().join(".rvm/config"), "{}").unwrap();
        assert!(is_protected(&ws, "main").unwrap());
    }

    #[test]
    fn test_unprotect_rewrites_both_config_files() {
        let (tmp, ws) = setup();
        set_protected(&ws, "main", false).unwrap();
        assert!(!is_protected(&ws, "main").unwrap());

        // Both files must carry an explicit empty list. If either still held the
        // built-in default, its fallback would silently re-protect main.
        let toml = std::fs::read_to_string(tmp.path().join("rvm.toml")).unwrap();
        let json = std::fs::read_to_string(tmp.path().join(".rvm/config")).unwrap();
        assert!(toml.contains("protected_branches = []"), "toml: {toml}");
        assert!(json.contains("\"protected_branches\": []"), "json: {json}");
    }

    #[test]
    fn test_deleting_a_config_file_reprotects_main_rather_than_unlocking_it() {
        let (tmp, ws) = setup();
        set_protected(&ws, "main", false).unwrap();
        assert!(!is_protected(&ws, "main").unwrap());

        // Deleting a config file falls back to the built-in default, which
        // protects main. This is the safe direction: an agent that deletes
        // config can lock itself out but can never unlock the operator's branch.
        std::fs::remove_file(tmp.path().join("rvm.toml")).unwrap();
        assert!(is_protected(&ws, "main").unwrap());
    }

    #[test]
    fn test_hand_editing_both_config_files_can_unprotect_main() {
        // Documents a known limitation of command-level gating. An agent that
        // rewrites *both* config files directly does bypass the gate. Closing
        // this requires sealing protected-branch data with a keyed HMAC derived
        // from a secret the agent cannot read, which is out of scope here.
        //
        // If this test ever starts failing, the limitation has been closed and
        // this comment should be deleted.
        let (tmp, ws) = setup();
        std::fs::write(
            tmp.path().join("rvm.toml"),
            "[protection]\nprotected_branches = []\n",
        )
        .unwrap();
        std::fs::write(
            tmp.path().join(".rvm/config"),
            r#"{"protection":{"protected_branches":[]}}"#,
        )
        .unwrap();
        assert!(!is_protected(&ws, "main").unwrap());
    }

    #[test]
    fn test_protect_then_unprotect_roundtrip() {
        let (_tmp, ws) = setup();

        set_protected(&ws, "feature", true).unwrap();
        assert!(is_protected(&ws, "feature").unwrap());

        set_protected(&ws, "feature", false).unwrap();
        assert!(!is_protected(&ws, "feature").unwrap());

        // Removing a branch that was never protected is a no-op.
        set_protected(&ws, "other", false).unwrap();
        assert!(!is_protected(&ws, "other").unwrap());
    }

    #[test]
    fn test_protect_is_idempotent() {
        let (_tmp, ws) = setup();
        set_protected(&ws, "main", true).unwrap();
        set_protected(&ws, "main", true).unwrap();
        assert_eq!(
            list_protected(&ws).unwrap().iter().filter(|b| *b == "main").count(),
            1
        );
    }

    #[test]
    fn test_local_config_write_alone_cannot_unlock_main() {
        let (tmp, ws) = setup();
        // Simulate an agent hand-editing only the workspace-local copy.
        std::fs::write(
            tmp.path().join(".rvm/config"),
            r#"{"protection":{"protected_branches":[]}}"#,
        )
        .unwrap();
        assert!(
            is_protected(&ws, "main").unwrap(),
            "clearing one file must not unlock main"
        );
    }
}
