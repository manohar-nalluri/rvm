use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::commit::CommitHash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetadata {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub parent_branch: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub metadata: BranchMetadata,
    pub head: Option<CommitHash>,
}

impl Branch {
    pub fn new(name: String, parent_branch: Option<String>) -> Self {
        Self {
            metadata: BranchMetadata {
                name,
                created_at: Utc::now(),
                parent_branch,
                archived: false,
            },
            head: None,
        }
    }
}
