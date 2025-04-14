use std::path::Path;

use chrono::{DateTime, Utc};
use git2::Error as Git2Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git2Error(#[from] Git2Error),

    #[error("Invalid commit message: {0}")]
    InvalidCommitMessage(String),

    #[error("Invalid tag: {0}")]
    InvalidTag(String),
}

pub type Result<T> = std::result::Result<T, GitError>;

/// Commit details from the repository
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Tag details from the repository
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub target_commit_id: String,
    pub date: Option<DateTime<Utc>>,
}

/// A trait that abstracts Git operations needed for changelog generation
pub trait GitProvider {
    /// Open a Git repository at the given path
    fn open(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Get a list of all commit IDs in chronological order
    fn get_commit_ids(&self) -> Result<Vec<String>>;

    /// Get details for a specific commit by ID
    fn get_commit_info(&self, id: &str) -> Result<CommitInfo>;

    /// Get all tags that match a specific pattern with their target commit IDs
    fn get_tag_info(&self, version_pattern: &regex::Regex) -> Result<Vec<TagInfo>>;
}
