use crate::commit_type::CommitType;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ParsedCommit {
    pub id: String,
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}
