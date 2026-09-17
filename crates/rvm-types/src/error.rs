use thiserror::Error;

pub type RvmResult<T> = Result<T, RvmError>;

#[derive(Error, Debug)]
pub enum RvmError {
    #[error("Workspace not initialized. Run `rvm init` first.")]
    NotInitialized,

    #[error("Workspace already initialized at {0}")]
    AlreadyInitialized(String),

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("Branch '{0}' already exists")]
    BranchAlreadyExists(String),

    #[error("No commits on current branch")]
    NoCommits,

    #[error("Merge conflict in {0} region(s). Resolve conflicts and commit.")]
    MergeConflict(usize),

    #[error("LaTeX compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Page limit exceeded: {actual} pages (limit: {limit})")]
    PageLimitExceeded { actual: usize, limit: usize },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication failed. Branch '{0}' was not modified.")]
    AuthenticationFailed(String),

    #[error("{0}")]
    AuthenticationUnavailable(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    #[error("AI operation failed: {0}")]
    AiError(String),

    #[error("{0}")]
    Other(String),
}
