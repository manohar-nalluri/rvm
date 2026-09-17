use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RvmConfig {
    #[serde(default)]
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub document: DocumentConfig,
    #[serde(default)]
    pub validation: ValidationConfig,
    #[serde(default)]
    pub protection: ProtectionConfig,
}

/// Branch protection settings.
///
/// A protected branch refuses any mutating operation (commit, merge into it,
/// protect/unprotect, archive, delete) until the operator authenticates with
/// their system password.
///
/// `protected_branches` deliberately defaults to `["main"]` rather than to an
/// empty list. This makes protection *fail closed*: a workspace whose config
/// predates this field, or whose config file has been deleted outright, still
/// treats `main` as protected instead of silently unlocking it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    #[serde(default = "default_protected_branches")]
    pub protected_branches: Vec<String>,
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            protected_branches: default_protected_branches(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default = "default_true")]
    pub auto_compile: bool,
    #[serde(default)]
    pub watch_mode: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            engine: default_engine(),
            auto_compile: true,
            watch_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConfig {
    #[serde(default = "default_page_limit")]
    pub page_limit: usize,
    #[serde(default = "default_max_whitespace")]
    pub max_whitespace_percent: u8,
    #[serde(default = "default_min_font_size")]
    pub min_font_size: u8,
    #[serde(default = "default_required_sections")]
    pub required_sections: Vec<String>,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            page_limit: default_page_limit(),
            max_whitespace_percent: default_max_whitespace(),
            min_font_size: default_min_font_size(),
            required_sections: default_required_sections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default = "default_true")]
    pub strict_mode: bool,
    #[serde(default = "default_true")]
    pub ats_check: bool,
    #[serde(default = "default_true")]
    pub keyword_check: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u8,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            ats_check: true,
            keyword_check: true,
            max_retries: default_max_retries(),
        }
    }
}

fn default_engine() -> String {
    "tectonic".to_string()
}
fn default_true() -> bool {
    true
}
fn default_page_limit() -> usize {
    1
}
fn default_max_whitespace() -> u8 {
    20
}
fn default_min_font_size() -> u8 {
    10
}
fn default_max_retries() -> u8 {
    3
}
fn default_protected_branches() -> Vec<String> {
    vec!["main".to_string()]
}
fn default_required_sections() -> Vec<String> {
    vec![
        "contact".to_string(),
        "experience".to_string(),
        "education".to_string(),
        "skills".to_string(),
    ]
}
