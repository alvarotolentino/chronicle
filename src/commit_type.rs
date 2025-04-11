#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommitType {
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
    pub fn from_prefix(prefix: &str) -> Self {
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

    pub fn to_heading(&self) -> &'static str {
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
