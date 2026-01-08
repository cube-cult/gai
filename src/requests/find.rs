use crate::{git::log::GitLog, settings::Settings};

use super::Request;

/// create a find request object
/// takes in git logs and a query
pub fn create_find_request(
    settings: &Settings,
    logs: &[GitLog],
    query: &str,
) -> Request {
    let prompt = build_prompt(settings);

    let logs: Vec<String> = logs
        .iter()
        .map(|l| l.raw.to_string())
        .collect();

    let query = format!("Query: {}", query);

    Request::new(&prompt)
        .insert_content(&query)
        .insert_contents(&logs)
}

fn build_prompt(_cfg: &Settings) -> String {
    String::new()
}
