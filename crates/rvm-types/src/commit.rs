use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitHash(pub String);

impl CommitHash {
    pub fn compute(content: &str, message: &str, timestamp: &DateTime<Utc>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.update(message.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        Self(hex::encode(hasher.finalize()))
    }

    pub fn short(&self) -> &str {
        &self.0[..7]
    }
}

impl std::fmt::Display for CommitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: CommitHash,
    pub parent: Option<CommitHash>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub snapshot_tex: String,
}

impl Commit {
    pub fn new(
        parent: Option<CommitHash>,
        message: String,
        snapshot_tex: String,
    ) -> Self {
        let timestamp = Utc::now();
        let hash = CommitHash::compute(&snapshot_tex, &message, &timestamp);
        Self {
            hash,
            parent,
            message,
            timestamp,
            snapshot_tex,
        }
    }
}
