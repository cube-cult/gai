use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{EnumIter, IntoEnumIterator};

use crate::git::StagingStrategy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitSchema {
    /// reason why you decided to make this
    /// commit. ex. why are they grouped together?
    /// or why decide on this type of change for the
    /// diffs
    pub reasoning: String,

    /// paths to apply commit to
    /// ex. main.rs doubloon.rs
    pub files: Vec<String>,

    // populated/used when stage_hunks
    // is enabled
    /// hunk "ids" per file
    /// using format file:index
    /// ex: src/main.rs:0
    pub hunk_ids: Vec<String>,

    // commit message components
    /// commit type
    pub prefix: PrefixType,

    /// scope of the change
    pub scope: String,

    /// is a breaking change?
    pub breaking: bool,

    /// short commit description
    /// used as a initial view
    pub header: String,

    /// extended description
    pub body: String,
}

/// conventional commit type prefix
#[derive(
    Clone, Debug, Serialize, Deserialize, EnumIter, strum::Display,
)]
#[serde(rename_all = "lowercase")]
pub enum PrefixType {
    Feat,
    Fix,
    Refactor,
    Style,
    Test,
    Docs,
    Build,
    CI,
    Ops,
    Chore,
    // for newbranch
    // the ai may hallucinate
    // and use these
    // on non-new branch creations
    // should we even have these clankers
    // create branches?
    //Merge,
    //Revert,
}

impl PrefixType {
    /// get all enum variants as a Vec<String>
    pub fn variants() -> Vec<String> {
        Self::iter().map(|p| p.to_string()).collect()
    }
}

/// creates a schema for commits
/// staging strategy
/// determines overall structure
/// which includes, whether or
/// not multiple commits are needed
pub fn create_commit_response_schema(
    staging_strategy: StagingStrategy
) -> Value {
    match staging_strategy {
        StagingStrategy::Hunks => todo!(),
        StagingStrategy::OneFilePerCommit => todo!(),
        StagingStrategy::AtomicCommits => todo!(),
        StagingStrategy::AllFilesOneCommit => todo!(),
    }
}
