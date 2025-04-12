# Chronicle

Chronicle is a changelog generator that creates formatted changelogs from git commit history.

## Features

- Generates structured changelogs from Git commit history
- Groups commits by version (using Git tags)
- Categorizes commits by type (feat, fix, doc, etc.)
- Supports both Markdown and HTML output formats
- Customizable title and output path
- Custom regex patterns for commit parsing and version detection
- Flexible sorting order (newest first or oldest first)

## Installation

To install from source:

```
cargo install --path .
```

## Usage

```
chronicle [OPTIONS]
```

### Options

```
-r, --repository <REPOSITORY>    Path to the git repository [default: .]
-o, --output <OUTPUT>            Output file path for the changelog [default: CHANGELOG.md]
-t, --title <TITLE>              Title for the changelog [default: Changelog]
-f, --format <FORMAT>            Format for the changelog [default: markdown] [possible values: markdown, html]
-s, --sort-order <SORT_ORDER>    Sort order for commits [default: newest-first] [possible values: newest-first, oldest-first]
    --commit-pattern <PATTERN>   Custom regex pattern for parsing commit messages
    --version-pattern <PATTERN>  Custom regex pattern for version tags
-h, --help                       Print help
-V, --version                    Print version
```

## Commit Message Format

Chronicle recognizes the following commit message formats:

| Format | Group in Changelog |
|--------|-------------------|
| `feat(scope): message` | 🚀 Features |
| `fix(scope): message` | 🐛 Bug Fixes |
| `doc(scope): message` | 📚 Documentation |
| `style(scope): message` | 🎨 Styling |
| `refactor(scope): message` | 🚜 Refactor |
| `perf(scope): message` | ⚡ Performance |
| `test(scope): message` | 🧪 Testing |
| `build(scope): message` | 🏗️ Build |
| `ci(scope): message` | 👷 Continuous Integration |
| `chore(scope): message` | 🧹 Chore |

The `scope` is optional and will be displayed in bold in the changelog.

### Sort Order

By default, Chronicle sorts commits by newest first, but you can change this with the `--sort-order` flag:

```
# Display newest commits first (default)
chronicle --sort-order newest

# Display oldest commits first
chronicle --sort-order oldest
```

### Custom Regex Patterns

You can specify custom regex patterns for commit messages and version tags:

```
# Using custom commit pattern
chronicle --commit-pattern "^(\w+):\s(.+)"

# Using custom version pattern
chronicle --version-pattern "^release-(\d+\.\d+\.\d+)$"
```

The commit pattern should include named capture groups for `type`, `scope` (optional), and `message`.

## Example

For a repository with commit messages like:

```
feat: implement changelog processor
fix: split code into separate files for structs, enums, and impl
chore: v0.1.1
```

Chronicle will generate a changelog like:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [v0.1.1] - 2025-04-11

### 🚀 Features

- implement changelog processor

### 🚜 Refactor

- split code into separate files for structs, enums, and impl

### 🧹 Chore

- v0.1.1
```

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.
