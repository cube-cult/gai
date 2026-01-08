use serde::Deserialize;
use serde_json::Value;
use strum::VariantNames;

use crate::schema::{SchemaBuilder, SchemaSettings};

/// wrapper struct to house
#[derive(Debug, Deserialize)]
pub struct FindResponse {
    #[serde(default)]
    pub commits: Vec<FindCommitSchema>,
}

/// raw find schema struct, used when we
/// deserialize the response Value object
#[derive(Clone, Debug, Deserialize)]
pub struct FindCommitSchema {
    /// reason why you decided to chose this
    /// commit. ex. why are they grouped together?
    /// or why decide on this type of change for the
    /// diffs
    pub reasoning: String,

    /// commit id, instead of using
    /// an entire hash or short hash
    /// makes it easier for the LLM
    /// to respond with
    pub commit_id: u32,

    /// assigned by the LLM response
    /// having mixed thoughts on adding this
    /// and wether or not its a good indicator
    /// some online forums, say its kind waste of
    /// output tokens, but using a specific enum
    /// instead of an arbitrary number might help
    pub confidence: Confidence,
}

#[derive(
    Clone, Debug, Deserialize, strum::Display, strum::VariantNames,
)]
pub enum Confidence {
    // this is absolutely the commit
    Exact,
    // pretty sure but needs checking
    Likely,
}

/// creates a schema for finding a specific
/// commit based on a query
pub fn create_find_schema(
    schema_settings: SchemaSettings,
    max: u32,
) -> anyhow::Result<Value> {
    let builder = SchemaBuilder::new()
        .settings(schema_settings.to_owned())
        .insert_str(
            "reasoning",
            Some("reason why you decided to chose this specific commit"),
            true,
        )
        .insert_int(
            "commit_id",
            Some("commit index for the chosen commit"),
            true,
            Some(0),
            Some(max),
        )
        .insert_enum(
            "confidence", 
            Some("choose your confidence on whether or not this matches the query"), 
            true,
            Confidence::VARIANTS
        );

    let schema = builder.build();

    Ok(schema)
}
