use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// response object along with any errors
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub result: Result<ResponseSchema, String>,
}

/// response object that a provider will respond with
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, JsonSchema,
)]
pub struct ResponseSchema {
    /// list of commits to create staged changes
    pub commits: Vec<ResponseCommit>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResponseCommit {
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
    Clone, Debug, Serialize, Deserialize, JsonSchema, EnumIter,
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
    Merge,
    Revert,
}

impl ResponseCommit {
    /// only used for UI for now
    /// todo need to refactored out
    pub fn get_commit_prefix(
        &self,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> String {
        let prefix = if capitalize_prefix {
            format!("{:?}", self.prefix).to_uppercase()
        } else {
            format!("{:?}", self.prefix).to_lowercase()
        };

        let breaking = if self.breaking { "!" } else { "" };

        let scope = if include_scope {
            format!("({})", self.scope.to_lowercase())
        } else {
            "".to_owned()
        };

        format!("{}{}{}", prefix, breaking, scope)
    }
}
