use regex::Regex;
use std::path::Path;

use crate::git_provider::{CommitInfo, GitError, GitProvider, Result, TagInfo};

#[derive(Default)]
pub struct MockGitProvider {
    pub commits: Vec<CommitInfo>,
    pub tags: Vec<TagInfo>,
}

impl MockGitProvider {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_commits(mut self, commits: Vec<CommitInfo>) -> Self {
        self.commits = commits;
        self
    }

    pub fn with_tags(mut self, tags: Vec<TagInfo>) -> Self {
        self.tags = tags;
        self
    }
}

impl GitProvider for MockGitProvider {
    fn open(_path: &Path) -> Result<Self> {
        Ok(Self::new())
    }

    fn get_commit_ids(&self) -> Result<Vec<String>> {
        Ok(self.commits.iter().map(|c| c.id.clone()).collect())
    }

    fn get_commit_info(&self, id: &str) -> Result<CommitInfo> {
        self.commits
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or_else(|| GitError::InvalidCommitMessage(format!("Commit not found: {}", id)))
    }

    fn get_tag_info(&self, version_pattern: &Regex) -> Result<Vec<TagInfo>> {
        Ok(self
            .tags
            .iter()
            .filter(|t| version_pattern.is_match(&t.name))
            .cloned()
            .collect())
    }
}
