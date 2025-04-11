use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use clap::{Parser, ValueEnum};
use git2::{Commit, ObjectType, Repository, Sort, Time};
use regex::Regex;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate a changelog from git commit history"
)]
pub struct Args {
    /// Path to the git repository
    #[arg(short, long, default_value = ".")]
    repository: PathBuf,

    /// Output file path for the changelog
    #[arg(short, long, default_value = "CHANGELOG.md")]
    output: PathBuf,

    /// Title for the changelog
    #[arg(short, long, default_value = "Changelog")]
    title: String,

    /// Format for the changelog
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Markdown,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CommitType {
    Feature,
    BugFix,
    Documentation,
    Style,
    Refactor,
    Performance,
    Testing,
    Build,
    CI,
    Chore,
    Other,
}

impl CommitType {
    fn from_prefix(prefix: &str) -> Self {
        match prefix {
            "feat" => CommitType::Feature,
            "fix" => CommitType::BugFix,
            "doc" => CommitType::Documentation,
            "style" => CommitType::Style,
            "refactor" => CommitType::Refactor,
            "perf" => CommitType::Performance,
            "test" => CommitType::Testing,
            "build" => CommitType::Build,
            "ci" => CommitType::CI,
            "chore" => CommitType::Chore,
            _ => CommitType::Other,
        }
    }

    fn to_heading(&self) -> &'static str {
        match self {
            CommitType::Feature => "🚀 Features",
            CommitType::BugFix => "🐛 Bug Fixes",
            CommitType::Documentation => "📚 Documentation",
            CommitType::Style => "🎨 Styling",
            CommitType::Refactor => "🚜 Refactor",
            CommitType::Performance => "⚡ Performance",
            CommitType::Testing => "🧪 Testing",
            CommitType::Build => "🏗️ Build",
            CommitType::CI => "👷 Continuous Integration",
            CommitType::Chore => "🧹 Chore",
            CommitType::Other => "Miscellaneous Tasks",
        }
    }
}

#[derive(Debug)]
struct ParsedCommit {
    id: String,
    commit_type: CommitType,
    scope: Option<String>,
    message: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Version {
    name: String,
    date: Option<DateTime<Utc>>,
    commits_by_type: HashMap<CommitType, Vec<ParsedCommit>>,
}

pub struct ChangelogGenerator {
    repo: Repository,
    version_regex: Regex,
    commit_regex: Regex,
}

impl ChangelogGenerator {
    fn new(repo_path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        let version_regex = Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap();
        let commit_regex =
            Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>.+)\))?:\s(?P<message>.+)$").unwrap();

        Ok(Self {
            repo,
            version_regex,
            commit_regex,
        })
    }

    fn git_time_to_datetime(time: &Time) -> DateTime<Utc> {
        Utc.timestamp_opt(time.seconds(), 0).unwrap()
    }

    fn parse_commit(&self, commit: &Commit) -> ParsedCommit {
        let message = commit.message().unwrap_or("").trim();
        let id = commit.id().to_string();
        let timestamp = Self::git_time_to_datetime(&commit.time());

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
            // Fallback for commits not matching the pattern
            ParsedCommit {
                id,
                commit_type: CommitType::Other,
                scope: None,
                message: message.to_string(),
                timestamp,
            }
        }
    }

    fn get_tag_info(
        &self,
    ) -> Result<HashMap<String, (String, Option<DateTime<Utc>>)>, git2::Error> {
        let mut tag_info = HashMap::new();

        self.repo.tag_names(None)?.iter().for_each(|tag_name| {
            if let Some(name) = tag_name {
                if let Ok(obj) = self.repo.revparse_single(&format!("refs/tags/{}", name)) {
                    if let Ok(tag) = obj.peel_to_tag() {
                        let tag_id = tag.target_id().to_string();
                        let tag_time = tag
                            .tagger()
                            .map(|tagger| Self::git_time_to_datetime(&tagger.when()));

                        if self.version_regex.is_match(name) {
                            tag_info.insert(tag_id, (name.to_string(), tag_time));
                        }
                    } else if let Ok(commit) = obj.peel_to_commit() {
                        let commit_id = commit.id().to_string();
                        let commit_time = Self::git_time_to_datetime(&commit.time());

                        if self.version_regex.is_match(name) {
                            tag_info.insert(commit_id, (name.to_string(), Some(commit_time)));
                        }
                    }
                }
            }
        });

        Ok(tag_info)
    }

    pub fn generate_changelog(&self) -> Result<Vec<Version>, git2::Error> {
        let mut versions: Vec<Version> = Vec::new();
        let mut current_version = Version {
            name: "unreleased".to_string(),
            date: None,
            commits_by_type: HashMap::new(),
        };

        let tag_info = self.get_tag_info()?;
        let mut revwalk = self.repo.revwalk()?;

        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push_head()?;

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let commit_id = oid.to_string();

            let parsed_commit = self.parse_commit(&commit);

            if tag_info.contains_key(&commit_id) {
                // Save current version and start a new one
                if !current_version.commits_by_type.is_empty() {
                    versions.push(current_version);
                }

                let (tag_name, tag_date) = tag_info.get(&commit_id).unwrap().clone();

                current_version = Version {
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

        // Add the last version if it has commits
        if !current_version.commits_by_type.is_empty() {
            versions.push(current_version);
        }

        // Reverse to get chronological order (oldest first)
        versions.reverse();

        Ok(versions)
    }

    fn write_markdown_changelog(
        &self,
        versions: &[Version],
        path: &Path,
        title: &str,
    ) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        // Write header
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

            // Sort commit types for consistent order
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

    fn write_html_changelog(
        &self,
        versions: &[Version],
        path: &Path,
        title: &str,
    ) -> std::io::Result<()> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let generator = ChangelogGenerator::new(&args.repository)?;
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
