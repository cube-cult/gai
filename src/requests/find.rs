use crate::settings::Settings;

use super::Request;

/// create a find request object
/// takes in git logs and a query
pub fn create_find_request(
    settings: &Settings,
    git_logs: &[String],
    query: &str,
) -> Request {
    let prompt = build_prompt(settings);

    let query = format!("Query:{}", query);

    Request::new(&prompt)
        .insert_content(&query)
        .insert_contents(git_logs)
}

fn build_prompt(_cfg: &Settings) -> String {
    concat!(
        "You are an assistant tasked on finding ",
        "a Git Commit that matches the query. ",
        "Ensure your input your reasoning as well as ",
        "Confidence level to describe your choice.",
    )
    .to_string()
}
