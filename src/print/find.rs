use console::{Color, Style, style};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    git::log::{GitLog, get_short_hash},
    schema::find::Confidence,
};

use super::tree::{Tree, TreeItem};

pub fn print(
    commit: &GitLog,
    files: bool,
    reasoning: Option<&str>,
    confidence: Confidence,
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

    if files {
        let logs = commit
            .files
            .join(",");

        let files_item = TreeItem::new_leaf(
            "raw_files".to_string(),
            logs.to_string(),
        )
        .style(Style::new().dim());

        children.push(files_item);
    }

    let (_, max_term_width) = console::Term::stderr().size();
    let avail = (max_term_width as usize).saturating_sub(15);

    let message: String = commit
        .to_owned()
        .into();

    let short_hash = get_short_hash(commit);

    let hash_display = style(format!("[{}]", short_hash)).dim();

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

    let message = style(&truncated).fg(color);

    let display = format!("{} {}", hash_display, message);

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

    let confidence_color = match confidence {
        Confidence::Exact => Color::Green,
        Confidence::Likely => Color::Color256(214),
        Confidence::Ambiguous => Color::Yellow,
    };

    println!(
        "Found a \"{}\" Commit...",
        style(confidence.to_string())
            .fg(confidence_color)
            .bold()
    );

    if let Some(r) = reasoning {
        println!(
            "{}:\n{}",
            style("Reasoning")
                .bold()
                .cyan(),
            style(r).dim(),
        );
    }

    Tree::new(&tree)?.render();

    let options = ["Checkout", "Query Another", "Retry", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option:")
        .items(options)
        .default(0)
        .interact()?;

    Ok(selection)
}
