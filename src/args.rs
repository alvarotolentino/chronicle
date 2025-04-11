use crate::OutputFormat;
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
}
