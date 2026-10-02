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
    #[serde(default)]
    pub ai: AiConfig,
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

/// Settings for the AI tailoring backend.
///
/// RVM drives an external agent CLI rather than an HTTP API: the CLI already
/// owns the operator's credentials, so no API key passes through RVM's argv,
/// environment or memory. That is the same reasoning that puts branch
/// authentication in `dscl` instead of collecting a password here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Provider dialect. Currently only `antigravity_cli` (Google's `agy`).
    #[serde(default = "default_ai_provider")]
    pub provider: String,

    /// Model id handed to the provider.
    #[serde(default = "default_ai_model")]
    pub model: String,

    /// Binary to run. Empty means: `RVM_AI_CLI` if set, else `agy` on `PATH`.
    #[serde(default)]
    pub cli_path: String,

    /// Wall-clock budget for one model call. Tailoring a full resume takes tens
    /// of seconds, so this is far larger than an interactive timeout would be.
    #[serde(default = "default_ai_timeout")]
    pub timeout_seconds: u64,

    /// Extra model attempts after an answer fails to produce usable LaTeX.
    #[serde(default = "default_ai_retries")]
    pub max_retries: u32,

    /// Terms that must survive tailoring. The tailorer is otherwise forbidden
    /// from introducing technology that was not already in the resume; this is
    /// the only sanctioned exception, so it stays empty unless deliberately set.
    #[serde(default)]
    pub required_terms: Vec<String>,

    /// Free-form house rules appended to the tailoring prompt, for workspace
    /// conventions that do not belong in the binary.
    #[serde(default)]
    pub extra_instructions: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            model: default_ai_model(),
            cli_path: String::new(),
            timeout_seconds: default_ai_timeout(),
            max_retries: default_ai_retries(),
            required_terms: Vec::new(),
            extra_instructions: String::new(),
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

    /// Also keep a `.docx` beside the PDF, rebuilt from the same `.tex` on
    /// every commit and checkout.
    ///
    /// Defaults to true so existing workspaces pick it up without an edit.
    /// Fails soft: a machine without `pandoc` prints one line of reason and
    /// still commits, because the PDF is the artifact that gates the work.
    /// Set `docx = false` for a workspace that must not shell out to pandoc.
    #[serde(default = "default_true")]
    pub docx: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            engine: default_engine(),
            auto_compile: true,
            watch_mode: false,
            docx: true,
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

fn default_ai_provider() -> String {
    "antigravity_cli".to_string()
}
fn default_ai_model() -> String {
    "gemini-3.8-flash-low".to_string()
}
fn default_ai_timeout() -> u64 {
    300
}
fn default_ai_retries() -> u32 {
    2
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
