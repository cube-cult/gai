use console::{Color, Style, style};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::git::log::GitLog;

use super::tree::{Tree, TreeItem};

pub fn print(
    commit: &GitLog,
    reasoning: &str,
    confidence: &str,
) -> anyhow::Result<usize> {
    let mut children = Vec::new();

    let date_item = TreeItem::new_leaf(
        commit
            .date
            .to_owned(),
        format!("Date: {}", commit.date),
    )
    .style(Style::new().dim());

    children.push(date_item);

    let author_item = TreeItem::new_leaf(
        commit
            .author
            .to_owned(),
        format!("Author: {}", commit.author),
    )
    .style(Style::new().dim());

    children.push(author_item);

    let hash_item = TreeItem::new_leaf(
        commit
            .commit_hash
            .to_owned(),
        format!("[{}]", commit.commit_hash),
    )
    .style(Style::new().dim());

    children.push(hash_item);

    let (_, max_term_width) = console::Term::stderr().size();
    let avail = (max_term_width as usize).saturating_sub(15);

    let message: String = commit
        .to_owned()
        .into();

    let truncated = if message.len() > avail {
        format!("{}...", &message[..avail])
    } else {
        message
    };

    let prefix = commit
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

    let display = style(&truncated).fg(color);

    let tree = vec![
        TreeItem::new(
            commit
                .raw
                .to_string(),
            display.to_string(),
            children,
        )?
        .style(Style::new()),
    ];

    Tree::new(&tree)?.render();

    let options = ["Checkout", "Retry", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option:")
        .items(options)
        .default(0)
        .interact()?;

    Ok(selection)
}
