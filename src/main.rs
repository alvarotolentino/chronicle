mod args;
mod changelog_generator;
mod commit_type;
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
    let args = args::Args::parse();

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
