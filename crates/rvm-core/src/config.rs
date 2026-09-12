use std::path::Path;

use rvm_types::{RvmConfig, RvmError, RvmResult};

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
