use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{commit_type::CommitType, parsed_commit::ParsedCommit};

#[derive(Debug)]
pub struct Version {
    pub name: String,
    pub date: Option<DateTime<Utc>>,
    pub commits_by_type: HashMap<CommitType, Vec<ParsedCommit>>,
}
