use serde::Deserialize;
use serde_json::Value;

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
    /// reason why you decided to make this
    /// commit. ex. why are they grouped together?
    /// or why decide on this type of change for the
    /// diffs
    pub reasoning: String,

    pub commit_id: u32,
}

/// creates a schema for finding a specific
/// commit based on a query
pub fn create_find_schema(
    schema_settings: SchemaSettings
) -> anyhow::Result<Value> {
    let builder = SchemaBuilder::new()
        .settings(schema_settings.to_owned())
        .insert_str(
            "reasoning",
            Some("reason why you decided to choose this specific commit_id"),
            true,
        )
        .insert_int(
            "commit_id",
            Some("commit index for this commit id"),
            true,
            None,
            None,
        );

    let schema = builder.build();

    Ok(schema)
}
