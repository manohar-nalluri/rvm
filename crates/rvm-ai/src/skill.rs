use std::path::PathBuf;

use rvm_types::{RvmError, RvmResult};

const SKILL_FILENAME: &str = "SKILL.md";

/// Manages the SKILL.md file that teaches Claude Code about RVM.
pub struct SkillManager {
    skills_dir: PathBuf,
}

impl SkillManager {
    pub fn new(skills_dir: impl Into<PathBuf>) -> Self {
        Self {
            skills_dir: skills_dir.into(),
        }
    }

    /// Initialize default skill files in the skills directory.
    pub fn init_default_skills(&self) -> RvmResult<()> {
        let content = Self::generate_default_skill();
        self.write_skill(&content)?;
        Ok(())
    }

    /// Check if the SKILL.md file exists.
    pub fn skill_exists(&self) -> bool {
        self.skill_path().exists()
    }

    /// Get the path to the SKILL.md file.
    pub fn skill_path(&self) -> PathBuf {
        self.skills_dir.join(SKILL_FILENAME)
    }

    /// Read the current SKILL.md content.
    pub fn read_skill(&self) -> RvmResult<String> {
        let path = self.skill_path();
        if !path.exists() {
            return Err(RvmError::Other("SKILL.md not found. Run `rvm ai init` to generate it.".to_string()));
        }
        Ok(std::fs::read_to_string(path)?)
    }

    /// Write or update the SKILL.md file.
    pub fn write_skill(&self, content: &str) -> RvmResult<()> {
        std::fs::create_dir_all(&self.skills_dir)?;
        std::fs::write(self.skill_path(), content)?;
        Ok(())
    }

    /// Generate the default SKILL.md content based on the current RVM configuration.
    pub fn generate_default_skill() -> String {
        include_str!("templates/default_skill.md").to_string()
    }
}
