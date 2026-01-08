use console::Style;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::schema::find::FindCommitSchema;

use super::tree::{Tree, TreeItem};

pub fn print(commit: FindCommitSchema) -> anyhow::Result<usize> {
    let mut children = Vec::new();

    let reason_item = TreeItem::new_leaf(
        commit
            .reasoning
            .to_owned(),
        format!("Reason: {}", &commit.reasoning),
    )
    .style(Style::new().cyan());

    children.push(reason_item);

    let confidence_item = TreeItem::new_leaf(
        format!("{}", &commit.confidence),
        format!("Confidence: {}", &commit.confidence),
    )
    .style(Style::new().green());

    children.push(confidence_item);

    let tree = vec![TreeItem::new(
        format!("{}", commit.commit_id),
        format!("{}", commit.commit_id),
        children,
    )?];

    Tree::new(&tree)?.render();

    let options = ["Checkout", "Retry", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option:")
        .items(options)
        .default(0)
        .interact()?;

    Ok(selection)
}
