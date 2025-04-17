mod args;
mod changelog_generator;
mod commit_type;
mod git2_provider;
mod git_provider;
mod parsed_commit;
mod version;

use changelog_generator::ChangelogGenerator;
use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Markdown,
    Html,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum SortOrder {
    Newest,
    Oldest,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args::Args::parse();

    let path = args.output.clone();
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    if args.format == OutputFormat::Markdown && extension != "md" {
        args.output = path.with_extension("md");
    } else if args.format == OutputFormat::Html && extension != "html" {
        args.output = path.with_extension("html");
    }

    let generator = if args.commit_pattern.is_some() || args.version_pattern.is_some() {
        ChangelogGenerator::with_patterns(
            &args.repository,
            args.version_pattern.as_deref(),
            args.commit_pattern.as_deref(),
            args.sort_order,
        )?
    } else {
        ChangelogGenerator::new(&args.repository, args.sort_order)?
    };

    let versions = generator.generate_changelog()?;

    match args.format {
        OutputFormat::Markdown => {
            generator.write_markdown_changelog(&versions, &args.output, &args.title)?;
        }
        OutputFormat::Html => {
            generator.write_html_changelog(&versions, &args.output, &args.title)?;
        }
    }

    println!("Changelog generated at: {}", args.output.display());

    Ok(())
}

mod mock_git_provider;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit_type::CommitType;
    use crate::git_provider::{CommitInfo, Result, TagInfo};
    use crate::mock_git_provider::MockGitProvider;
    use chrono::{TimeZone, Utc};
    use regex::Regex;

    #[test]
    fn test_parse_commit() {
        let mock_git = MockGitProvider::new();
        let generator = ChangelogGenerator {
            git: mock_git,
            version_regex: Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap(),
            commit_regex: Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?:\s(?P<message>.+)$")
                .unwrap(),
            sort_order: SortOrder::Newest,
        };

        // Test a feature commit with scope
        let commit_info = CommitInfo {
            id: "abc123".to_string(),
            message: "feat(api): add new endpoint".to_string(),
            timestamp: Utc.with_ymd_and_hms(2025, 4, 13, 12, 0, 0).unwrap(),
        };

        let parsed = generator.parse_commit(&commit_info);

        assert_eq!(parsed.id, "abc123");
        assert_eq!(parsed.commit_type, CommitType::Feature);
        assert_eq!(parsed.scope, Some("api".to_string()));
        assert_eq!(parsed.message, "add new endpoint");
    }

    #[test]
    fn test_generate_changelog() -> Result<()> {
        // Create some test commits
        let commits = vec![
            CommitInfo {
                id: "commit1".to_string(),
                message: "feat(core): first feature".to_string(),
                timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            },
            CommitInfo {
                id: "commit2".to_string(),
                message: "fix(ui): fix bug".to_string(),
                timestamp: Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
            },
            CommitInfo {
                id: "commit3".to_string(),
                message: "feat(api): new feature".to_string(),
                timestamp: Utc.with_ymd_and_hms(2025, 1, 3, 0, 0, 0).unwrap(),
            },
        ];

        let tags = vec![TagInfo {
            name: "v1.0.0".to_string(),
            target_commit_id: "commit2".to_string(),
            date: Some(Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()),
        }];

        let mock_git = MockGitProvider::new().with_commits(commits).with_tags(tags);

        let generator = ChangelogGenerator {
            git: mock_git,
            version_regex: Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap(),
            commit_regex: Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?:\s(?P<message>.+)$")
                .unwrap(),
            sort_order: SortOrder::Newest,
        };

        let versions = generator.generate_changelog()?;

        assert_eq!(versions.len(), 2);

        assert_eq!(versions[0].name, "unreleased");
        assert!(
            versions[0]
                .commits_by_type
                .contains_key(&CommitType::Feature)
        );

        // Check v1.0.0 version
        assert_eq!(versions[1].name, "v1.0.0");
        assert!(
            versions[1]
                .commits_by_type
                .contains_key(&CommitType::BugFix)
        );

        Ok(())
    }
}
