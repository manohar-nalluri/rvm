use std::path::{Path, PathBuf};

use rvm_types::{Branch, RvmConfig, RvmError, RvmResult};

const RVM_DIR: &str = ".rvm";
const HEAD_FILE: &str = "HEAD";
const CONFIG_FILE: &str = "config";
const BRANCHES_DIR: &str = "branches";
const TEMPLATES_DIR: &str = "templates";
const SKILLS_DIR: &str = "skills";

/// Represents an initialized RVM workspace.
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Initialize a new RVM workspace at the given path.
    pub fn init(root: &Path) -> RvmResult<Self> {
        let rvm_dir = root.join(RVM_DIR);
        if rvm_dir.exists() {
            return Err(RvmError::AlreadyInitialized(
                root.display().to_string(),
            ));
        }

        // Create directory structure
        std::fs::create_dir_all(rvm_dir.join(BRANCHES_DIR).join("main"))?;
        std::fs::create_dir_all(rvm_dir.join(TEMPLATES_DIR))?;
        std::fs::create_dir_all(rvm_dir.join(SKILLS_DIR))?;

        // Write HEAD pointing to main
        std::fs::write(rvm_dir.join(HEAD_FILE), "main")?;

        // Write default config
        let config = RvmConfig::default();
        let config_str = serde_json::to_string_pretty(&config)?;
        std::fs::write(rvm_dir.join(CONFIG_FILE), config_str)?;

        // Create the main branch
        let main_branch = Branch::new("main".to_string(), None);
        let branch_path = rvm_dir.join(BRANCHES_DIR).join("main").join("branch.json");
        std::fs::write(branch_path, serde_json::to_string_pretty(&main_branch)?)?;

        // Create default rvm.toml at workspace root
        let default_toml = toml::to_string_pretty(&RvmConfig::default())
            .map_err(|e| RvmError::ConfigError(e.to_string()))?;
        std::fs::write(root.join("rvm.toml"), default_toml)?;

        tracing::info!("Initialized RVM workspace at {}", root.display());

        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    /// Open an existing RVM workspace.
    pub fn open(root: &Path) -> RvmResult<Self> {
        let rvm_dir = root.join(RVM_DIR);
        if !rvm_dir.exists() {
            return Err(RvmError::NotInitialized);
        }
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    /// Discover the workspace root by walking up from the given path.
    pub fn discover(start: &Path) -> RvmResult<Self> {
        let mut current = start.to_path_buf();
        loop {
            if current.join(RVM_DIR).exists() {
                return Self::open(&current);
            }
            if !current.pop() {
                return Err(RvmError::NotInitialized);
            }
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn rvm_dir(&self) -> PathBuf {
        self.root.join(RVM_DIR)
    }

    pub fn branches_dir(&self) -> PathBuf {
        self.rvm_dir().join(BRANCHES_DIR)
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.rvm_dir().join(TEMPLATES_DIR)
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.rvm_dir().join(SKILLS_DIR)
    }

    /// Find the single .tex file in the workspace root.
    /// Returns an error if zero or more than one .tex file is found.
    pub fn find_tex_file(&self) -> RvmResult<PathBuf> {
        let mut tex_files = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "tex" {
                        tex_files.push(path);
                    }
                }
            }
        }

        match tex_files.len() {
            0 => Err(RvmError::Other(
                "No .tex file found in workspace. Place a .tex file (e.g. resume.tex, yourname.tex) in this directory.".to_string(),
            )),
            1 => Ok(tex_files.remove(0)),
            n => Err(RvmError::Other(format!(
                "Found {} .tex files in workspace. Only one is allowed. Files: {}",
                n,
                tex_files.iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<_>>().join(", ")
            ))),
        }
    }

    /// Get the PDF path corresponding to the workspace's .tex file.
    pub fn find_pdf_path(&self) -> RvmResult<PathBuf> {
        let tex = self.find_tex_file()?;
        Ok(tex.with_extension("pdf"))
    }

    /// Read the current branch name from HEAD.
    pub fn current_branch(&self) -> RvmResult<String> {
        let head = std::fs::read_to_string(self.rvm_dir().join(HEAD_FILE))?;
        Ok(head.trim().to_string())
    }

    /// Set the current branch in HEAD.
    pub fn set_head(&self, branch_name: &str) -> RvmResult<()> {
        std::fs::write(self.rvm_dir().join(HEAD_FILE), branch_name)?;
        Ok(())
    }

    /// Load the workspace config from rvm.toml.
    pub fn load_config(&self) -> RvmResult<RvmConfig> {
        let toml_path = self.toml_config_path();
        if !toml_path.exists() {
            return Ok(RvmConfig::default());
        }
        let content = std::fs::read_to_string(toml_path)?;
        toml::from_str(&content).map_err(|e| RvmError::ConfigError(e.to_string()))
    }

    /// Path to the user-facing workspace config file.
    pub fn toml_config_path(&self) -> PathBuf {
        self.root.join("rvm.toml")
    }

    /// Path to the workspace-local config copy inside `.rvm/`.
    ///
    /// Protection state is mirrored here as well as in `rvm.toml` so that
    /// clearing one file is not enough to silently unlock a protected branch.
    pub fn local_config_path(&self) -> PathBuf {
        self.rvm_dir().join(CONFIG_FILE)
    }

    /// Load the workspace-local config copy from `.rvm/config`.
    pub fn load_local_config(&self) -> RvmResult<RvmConfig> {
        let path = self.local_config_path();
        if !path.exists() {
            return Ok(RvmConfig::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| RvmError::ConfigError(e.to_string()))
    }

    /// Write the workspace config to `rvm.toml`.
    pub fn save_config(&self, config: &RvmConfig) -> RvmResult<()> {
        let content =
            toml::to_string_pretty(config).map_err(|e| RvmError::ConfigError(e.to_string()))?;
        std::fs::write(self.toml_config_path(), content)?;
        Ok(())
    }

    /// Write the workspace-local config copy to `.rvm/config`.
    pub fn save_local_config(&self, config: &RvmConfig) -> RvmResult<()> {
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(self.local_config_path(), content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_workspace() {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        assert!(ws.rvm_dir().exists());
        assert!(ws.branches_dir().join("main").exists());
        assert_eq!(ws.current_branch().unwrap(), "main");
    }

    #[test]
    fn test_init_fails_if_already_exists() {
        let tmp = TempDir::new().unwrap();
        Workspace::init(tmp.path()).unwrap();
        assert!(Workspace::init(tmp.path()).is_err());
    }

    #[test]
    fn test_discover_finds_workspace() {
        let tmp = TempDir::new().unwrap();
        Workspace::init(tmp.path()).unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        let ws = Workspace::discover(&sub).unwrap();
        assert_eq!(ws.root(), tmp.path());
    }

    #[test]
    fn test_find_tex_file_single() {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        std::fs::write(tmp.path().join("manohar.tex"), "content").unwrap();
        let found = ws.find_tex_file().unwrap();
        assert_eq!(found.file_name().unwrap(), "manohar.tex");
    }

    #[test]
    fn test_find_tex_file_none_errors() {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        assert!(ws.find_tex_file().is_err());
    }

    #[test]
    fn test_find_tex_file_multiple_errors() {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        std::fs::write(tmp.path().join("a.tex"), "a").unwrap();
        std::fs::write(tmp.path().join("b.tex"), "b").unwrap();
        assert!(ws.find_tex_file().is_err());
    }

    #[test]
    fn test_find_pdf_path_matches_tex() {
        let tmp = TempDir::new().unwrap();
        let ws = Workspace::init(tmp.path()).unwrap();
        std::fs::write(tmp.path().join("johndoe.tex"), "content").unwrap();
        let pdf = ws.find_pdf_path().unwrap();
        assert_eq!(pdf.file_name().unwrap(), "johndoe.pdf");
    }
}
