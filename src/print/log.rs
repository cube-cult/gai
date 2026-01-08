use console::{Color, style};
use dialoguer::{FuzzySelect, theme::Theme};
use std::fmt;

use crate::git::log::GitLog;

pub fn print_logs(
    git_logs: &[GitLog]
) -> anyhow::Result<Option<usize>> {
    let mut selection_display = Vec::new();

    for git_log in git_logs {
        // not caring about message bodies
        // though, they will be accounted
        // for in the raw when we implement selection

        // short hash
        let short_hash = &git_log.commit_hash[..7.min(
            git_log
                .commit_hash
                .len(),
        )];
        let hash_display = style(format!("[{}]", short_hash)).dim();

        let message: String = git_log
            .to_owned()
            .into();

        // fixes the bad width when doing fuzzy select
        // though, this may not matter much without interactivity
        // but i think this is better than hardcoding a specific limit
        let (_, max_term_width) = console::Term::stderr().size();
        let avail = (max_term_width as usize).saturating_sub(15);

        let truncated = if message.len() > avail {
            format!("{}...", &message[..avail])
        } else {
            message
        };

        let prefix = git_log
            .prefix
            .as_ref()
            .map(|s| s.to_lowercase());

        let color = match prefix.as_deref() {
            Some("feat") => Color::Green,
            Some("fix") => Color::Red,
            Some("refactor") => Color::Color256(214),
            Some("docs") => Color::Blue,
            _ => Color::White,
        };

        let message = style(&truncated).fg(color);

        let display = format!("{} {}", hash_display, message);

        selection_display.push(display.to_owned());
    }

    let selected = FuzzySelect::with_theme(&LogTheme)
        .with_prompt("Select a commit")
        .items(&selection_display)
        .interact_opt()?;

    Ok(selected)
}

/// theme impl to avoid
/// overriding console-rs styles
struct LogTheme;
impl Theme for LogTheme {
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        _cursor_pos: usize,
    ) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            style(prompt).bold(),
            style(search_term).dim()
        )
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        _highlight_matches: bool,
        _matcher: &fuzzy_matcher::skim::SkimMatcherV2,
        _search_term: &str,
    ) -> fmt::Result {
        if active {
            let prefix = style(">")
                .green()
                .bold();
            write!(f, "{} {}", prefix, text)
        } else {
            write!(f, " {}", text)
        }
    }
}
