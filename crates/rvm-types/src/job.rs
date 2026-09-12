use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    #[default]
    Draft,
    Applied,
    Screening,
    Interviewing,
    Offered,
    Accepted,
    Rejected,
    Ghosted,
    Withdrawn,
}

impl std::fmt::Display for ApplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Draft => "Draft",
            Self::Applied => "Applied",
            Self::Screening => "Screening",
            Self::Interviewing => "Interviewing",
            Self::Offered => "Offered",
            Self::Accepted => "Accepted",
            Self::Rejected => "Rejected",
            Self::Ghosted => "Ghosted",
            Self::Withdrawn => "Withdrawn",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub equity: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEvent {
    pub status: ApplicationStatus,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobMetadata {
    pub company: Option<String>,
    pub role: Option<String>,
    pub jd_text: Option<String>,
    pub jd_url: Option<String>,
    #[serde(default)]
    pub status: ApplicationStatus,
    pub applied_date: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub salary_range: Option<SalaryRange>,
    #[serde(default)]
    pub contacts: Vec<Contact>,
    pub notes: Option<String>,
    #[serde(default)]
    pub events: Vec<StatusEvent>,
}
