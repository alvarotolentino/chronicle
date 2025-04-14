use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use git2::{Repository, Sort, Time};

use crate::git_provider::{CommitInfo, GitProvider, Result, TagInfo};

pub struct Git2Provider {
    repo: Repository,
}

impl GitProvider for Git2Provider {
    fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    fn get_commit_ids(&self) -> Result<Vec<String>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push_head()?;

        let commit_ids = revwalk
            .map(|oid_result| oid_result.map(|oid| oid.to_string()))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(commit_ids)
    }

    fn get_commit_info(&self, id: &str) -> Result<CommitInfo> {
        let oid = git2::Oid::from_str(id)?;
        let commit = self.repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").trim().to_string();
        let timestamp = git_time_to_datetime(&commit.time());

        Ok(CommitInfo {
            id: id.to_string(),
            message,
            timestamp,
        })
    }

    fn get_tag_info(&self, version_pattern: &regex::Regex) -> Result<Vec<TagInfo>> {
        let mut tags = Vec::new();

        if let Ok(tag_names) = self.repo.tag_names(None) {
            for tag_name in tag_names.iter().flatten() {
                // Skip tags that don't match the version pattern
                if !version_pattern.is_match(tag_name) {
                    continue;
                }

                let ref_name = format!("refs/tags/{}", tag_name);

                if let Ok(obj) = self.repo.revparse_single(&ref_name) {
                    // Handle annotated tags
                    if let Ok(tag) = obj.peel_to_tag() {
                        let target_id = tag.target_id().to_string();
                        let tag_time = tag
                            .tagger()
                            .map(|tagger| git_time_to_datetime(&tagger.when()));

                        tags.push(TagInfo {
                            name: tag_name.to_string(),
                            target_commit_id: target_id,
                            date: tag_time,
                        });
                    }
                    // Handle lightweight tags
                    else if let Ok(commit) = obj.peel_to_commit() {
                        let commit_id = commit.id().to_string();
                        let commit_time = git_time_to_datetime(&commit.time());

                        tags.push(TagInfo {
                            name: tag_name.to_string(),
                            target_commit_id: commit_id,
                            date: Some(commit_time),
                        });
                    }
                }
            }
        }

        Ok(tags)
    }
}

// Helper function to convert git2::Time to chrono::DateTime<Utc>
fn git_time_to_datetime(time: &Time) -> DateTime<Utc> {
    Utc.timestamp_opt(time.seconds(), 0).unwrap()
}
