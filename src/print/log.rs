use std::fmt;

use console::{Color, Style, style};
use dialoguer::{FuzzySelect, theme::Theme};

/// theme impl to avoid
/// overriding console-rs styles
pub struct LogTheme;
impl Theme for LogTheme {
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        _cursor_pos: usize,
    ) -> fmt::Result {
        write!(f, "{}: {}", prompt, search_term)
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
            let prefix = style(">").green().bold();
            write!(f, "{} {}", prefix, text)
        } else {
            write!(f, " {}", text)
        }
    }
}

use crate::git::log::GitLog;

use super::tree::{Tree, TreeItem};

pub fn print(
    git_logs: &[GitLog],
    compact: bool,
    interactive: bool,
) -> anyhow::Result<()> {
    let mut items = Vec::new();
    let mut selection_display = Vec::new();

    for git_log in git_logs {
        let mut commit_children = Vec::new();

        // not caring about message bodies
        // though, they will be accounted
        // for in the raw when we implement selection

        // author + date
        let info =
            format!("By {} on {}", &git_log.author, &git_log.date);

        let info_item =
            TreeItem::new_leaf(git_log.commit_hash.to_owned(), &info)
                .style(Style::new().fg(Color::Color256(240)));

        commit_children.push(info_item);

        // short hash
        let short_hash =
            &git_log.commit_hash[..7.min(git_log.commit_hash.len())];
        let hash_display = style(format!("[{}]", short_hash)).dim();

        let message: String = git_log.to_owned().into();

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

        let prefix =
            git_log.prefix.as_ref().map(|s| s.to_lowercase());

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

        let item = TreeItem::new(
            git_log.commit_hash.to_owned(),
            display,
            commit_children,
        )?;

        items.push(item);
    }

    if !interactive {
        Tree::new(&items)?
            .collapsed(compact)
            .style(Style::new().dim())
            .render();
    } else {
        match FuzzySelect::with_theme(&LogTheme)
            .with_prompt("Select a commit")
            .items(&selection_display)
            .interact_opt()?
        {
            Some(s) => println!("{s}"),
            None => println!("None selcted"),
        }
    }

    Ok(())
}
