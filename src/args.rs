use crate::{OutputFormat, SortOrder};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate a changelog from git commit history"
)]
pub struct Args {
    /// Path to the git repository
    #[arg(short, long, default_value = ".")]
    pub repository: PathBuf,

    /// Output file path for the changelog
    #[arg(short, long, default_value = "CHANGELOG.md")]
    pub output: PathBuf,

    /// Title for the changelog
    #[arg(short, long, default_value = "Changelog")]
    pub title: String,

    /// Format for the changelog
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Sort order for commits
    #[arg(short, long, value_enum, default_value_t = SortOrder::Newest)]
    pub sort_order: SortOrder,

    /// Custom regex pattern for commit messages
    #[arg(long)]
    pub commit_pattern: Option<String>,

    /// Custom regex pattern for version tags
    #[arg(long)]
    pub version_pattern: Option<String>,
}
