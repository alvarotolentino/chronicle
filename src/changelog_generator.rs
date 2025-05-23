use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use regex::Regex;

use crate::{
    SortOrder,
    commit_type::CommitType,
    git_provider::{GitProvider, Result},
    git2_provider::Git2Provider,
    parsed_commit::ParsedCommit,
    version,
};

pub struct ChangelogGenerator<P: GitProvider> {
    pub git: P,
    pub version_regex: Regex,
    pub commit_regex: Regex,
    pub sort_order: SortOrder,
}

impl ChangelogGenerator<Git2Provider> {
    pub fn new(repo_path: &Path, sort_order: SortOrder) -> Result<Self> {
        let git = Git2Provider::open(repo_path)?;
        let version_regex = Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap();
        let commit_regex =
            Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?:\s(?P<message>.+)$").unwrap();

        Ok(Self {
            git,
            version_regex,
            commit_regex,
            sort_order,
        })
    }

    pub fn with_patterns(
        repo_path: &Path,
        version_pattern: Option<&str>,
        commit_pattern: Option<&str>,
        sort_order: SortOrder,
    ) -> Result<Self> {
        let git = Git2Provider::open(repo_path)?;
        let version_regex = version_pattern
            .map(|pattern| Regex::new(pattern).unwrap())
            .unwrap_or_else(|| Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap());
        let commit_regex = commit_pattern
            .map(|pattern| Regex::new(pattern).unwrap())
            .unwrap_or_else(|| {
                Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?:\s(?P<message>.+)$").unwrap()
            });

        Ok(Self {
            git,
            version_regex,
            commit_regex,
            sort_order,
        })
    }
}

impl<P: GitProvider> ChangelogGenerator<P> {
    pub fn parse_commit(&self, commit_info: &crate::git_provider::CommitInfo) -> ParsedCommit {
        let message = commit_info.message.lines().next().unwrap_or("").trim();
        let id = commit_info.id.clone();
        let timestamp = commit_info.timestamp;

        if let Some(captures) = self.commit_regex.captures(message) {
            let commit_type =
                CommitType::from_prefix(captures.name("type").map_or("", |m| m.as_str()));
            let scope = captures.name("scope").map(|m| m.as_str().to_string());
            let message = captures
                .name("message")
                .map_or("", |m| m.as_str())
                .to_string();

            ParsedCommit {
                id,
                commit_type,
                scope,
                message,
                timestamp,
            }
        } else {
            ParsedCommit {
                id,
                commit_type: CommitType::Other,
                scope: None,
                message: message.to_string(),
                timestamp,
            }
        }
    }

    pub fn generate_changelog(&self) -> Result<Vec<version::Version>> {
        let mut versions: Vec<version::Version> = Vec::new();
        let mut current_version = version::Version {
            name: "unreleased".to_string(),
            date: None,
            commits_by_type: HashMap::new(),
        };

        // Get all tag information
        let tags = self.git.get_tag_info(&self.version_regex)?;

        // Create a map of commit ID -> tag info for quick lookup
        let mut tag_map: HashMap<String, (String, Option<DateTime<Utc>>)> = HashMap::new();
        for tag in tags {
            tag_map.insert(tag.target_commit_id, (tag.name, tag.date));
        }

        // Process commits
        let commit_ids = self.git.get_commit_ids()?;

        for commit_id in commit_ids {
            let commit_info = self.git.get_commit_info(&commit_id)?;
            let parsed_commit = self.parse_commit(&commit_info);

            if tag_map.contains_key(&commit_id) {
                // Save current version and start a new one
                if !current_version.commits_by_type.is_empty() {
                    versions.push(current_version);
                }

                let (tag_name, tag_date) = tag_map.get(&commit_id).unwrap().clone();

                current_version = version::Version {
                    name: tag_name,
                    date: tag_date,
                    commits_by_type: HashMap::new(),
                };
            }

            current_version
                .commits_by_type
                .entry(parsed_commit.commit_type.clone())
                .or_default()
                .push(parsed_commit);
        }

        if !current_version.commits_by_type.is_empty() {
            versions.push(current_version);
        }

        match self.sort_order {
            SortOrder::Newest => {}
            SortOrder::Oldest => versions.reverse(),
        }

        Ok(versions)
    }

    // Existing methods for writing changelogs remain unchanged
    pub fn write_markdown_changelog(
        &self,
        versions: &[version::Version],
        path: &Path,
        title: &str,
    ) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(&mut file, "# {}\n", title)?;
        writeln!(
            &mut file,
            "All notable changes to this project will be documented in this file.\n"
        )?;

        for version in versions {
            if version.name == "unreleased" {
                writeln!(&mut file, "## [unreleased]\n")?;
            } else if let Some(date) = version.date {
                writeln!(
                    &mut file,
                    "## [{}] - {}\n",
                    version.name,
                    date.format("%Y-%m-%d")
                )?;
            } else {
                writeln!(&mut file, "## [{}]\n", version.name)?;
            }

            let mut commit_types: Vec<&CommitType> = version.commits_by_type.keys().collect();
            commit_types.sort_by_key(|k| match *k {
                CommitType::Feature => 0,
                CommitType::BugFix => 1,
                CommitType::Documentation => 2,
                CommitType::Style => 3,
                CommitType::Refactor => 4,
                CommitType::Performance => 5,
                CommitType::Testing => 6,
                CommitType::Build => 7,
                CommitType::CI => 8,
                CommitType::Chore => 9,
                CommitType::Other => 10,
            });

            for commit_type in commit_types {
                if let Some(commits) = version.commits_by_type.get(commit_type) {
                    if !commits.is_empty() {
                        writeln!(&mut file, "### {}\n", commit_type.to_heading())?;

                        for commit in commits {
                            if let Some(scope) = &commit.scope {
                                writeln!(&mut file, "- **{}**: {}", scope, commit.message)?;
                            } else {
                                writeln!(&mut file, "- {}", commit.message)?;
                            }
                        }

                        writeln!(&mut file)?;
                    }
                }
            }
        }

        writeln!(&mut file, "<!-- generated by chronicle -->")?;

        Ok(())
    }

    pub fn write_html_changelog(
        &self,
        versions: &[version::Version],
        path: &Path,
        title: &str,
    ) -> std::io::Result<()> {
        // HTML generation code (unchanged)
        let mut file = File::create(path)?;

        // Write HTML header
        write!(
            &mut file,
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; line-height: 1.5; max-width: 800px; margin: 0 auto; padding: 20px; color: #24292e; }}
        h1 {{ border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }}
        h2 {{ margin-top: 24px; margin-bottom: 16px; font-weight: 600; line-height: 1.25; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }}
        h3 {{ margin-top: 24px; margin-bottom: 16px; font-weight: 600; line-height: 1.25; }}
        ul {{ padding-left: 2em; }}
        li {{ margin: 0.25em 0; }}
        .footer {{ margin-top: 30px; color: #6a737d; font-size: 0.9em; text-align: center; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>All notable changes to this project will be documented in this file.</p>
"#,
            title, title
        )?;

        for version in versions {
            if version.name == "unreleased" {
                writeln!(&mut file, "    <h2>[unreleased]</h2>")?;
            } else if let Some(date) = version.date {
                writeln!(
                    &mut file,
                    "    <h2>[{}] - {}</h2>",
                    version.name,
                    date.format("%Y-%m-%d")
                )?;
            } else {
                writeln!(&mut file, "    <h2>[{}]</h2>", version.name)?;
            }

            let mut commit_types: Vec<&CommitType> = version.commits_by_type.keys().collect();
            commit_types.sort_by_key(|k| match *k {
                CommitType::Feature => 0,
                CommitType::BugFix => 1,
                CommitType::Documentation => 2,
                CommitType::Style => 3,
                CommitType::Refactor => 4,
                CommitType::Performance => 5,
                CommitType::Testing => 6,
                CommitType::Build => 7,
                CommitType::CI => 8,
                CommitType::Chore => 9,
                CommitType::Other => 10,
            });

            for commit_type in commit_types {
                if let Some(commits) = version.commits_by_type.get(commit_type) {
                    if !commits.is_empty() {
                        writeln!(&mut file, "    <h3>{}</h3>", commit_type.to_heading())?;
                        writeln!(&mut file, "    <ul>")?;

                        for commit in commits {
                            if let Some(scope) = &commit.scope {
                                writeln!(
                                    &mut file,
                                    "        <li><strong>{}</strong>: {}</li>",
                                    scope, commit.message
                                )?;
                            } else {
                                writeln!(&mut file, "        <li>{}</li>", commit.message)?;
                            }
                        }

                        writeln!(&mut file, "    </ul>")?;
                    }
                }
            }
        }

        write!(
            &mut file,
            r#"    <div class="footer">Generated by chronicle</div>
</body>
</html>
"#
        )?;

        Ok(())
    }
}
