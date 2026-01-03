use console::{Color, Style, style};

use crate::git::status::{FileStatus, StatusItemType};

use super::tree::{Tree, TreeItem};

pub fn print(
    branch: &str,
    staged_statuses: &[FileStatus],
    working_dir_statuses: &[FileStatus],
    compact: bool,
) -> anyhow::Result<()> {
    let mut modified = Vec::new();
    let mut new = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();

    for status in working_dir_statuses {
        match status.status {
            StatusItemType::New => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(Style::new().fg(Color::Red));
                new.push(item);
            }
            StatusItemType::Modified => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(Style::new().fg(Color::Yellow));
                modified.push(item);
            }
            StatusItemType::Deleted => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(Style::new().fg(Color::Red).dim());
                deleted.push(item);
            }
            StatusItemType::Renamed => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(Style::new().fg(Color::Cyan));
                renamed.push(item);
            }
            _ => {}
        }
    }

    let mut unstaged = Vec::new();

    if !modified.is_empty() {
        let count = modified.len();
        unstaged.push(
            TreeItem::new(
                "modified".to_owned(),
                format!("Modified ({})", count),
                modified,
            )?
            .style(Style::new().fg(Color::Yellow).bold()),
        );
    }

    if !deleted.is_empty() {
        let count = deleted.len();
        unstaged.push(
            TreeItem::new(
                "deleted".to_owned(),
                format!("Deleted ({})", count),
                deleted,
            )?
            .style(Style::new().fg(Color::Red).dim().bold()),
        );
    }

    if !renamed.is_empty() {
        let count = renamed.len();
        unstaged.push(
            TreeItem::new(
                "renamed".to_owned(),
                format!("Renamed ({})", count),
                renamed,
            )?
            .style(Style::new().fg(Color::Cyan).bold()),
        );
    }

    if !new.is_empty() {
        let count = new.len();
        unstaged.push(
            TreeItem::new(
                "untracked".to_owned(),
                format!("Untracked ({})", count),
                new,
            )?
            .style(Style::new().fg(Color::Red).bold()),
        );
    }

    let mut staged_items = Vec::new();

    for status in staged_statuses {
        let item =
            TreeItem::new_leaf(status.path.clone(), &status.path)
                .style(Style::new().fg(Color::Green));
        staged_items.push(item);
    }

    let mut root_items = Vec::new();

    if !staged_items.is_empty() {
        let count = staged_items.len();
        root_items.push(
            TreeItem::new(
                "staged".to_owned(),
                format!("Staged Changes ({})", count),
                staged_items,
            )?
            .style(Style::new().fg(Color::Green).bold()),
        );
    }

    if !unstaged.is_empty() {
        let count: usize =
            unstaged.iter().map(|c| c.children().len()).sum();
        root_items.push(
            TreeItem::new(
                "unstaged".to_owned(),
                format!("Unstaged Changes ({})", count),
                unstaged,
            )?
            .style(Style::new().fg(Color::Yellow).bold()),
        );
    }

    println!("On Branch: {}", style(branch).cyan());

    if !root_items.is_empty() {
        Tree::new(&root_items)?
            .collapsed(compact)
            .style(Style::new().dim())
            .render();
    }

    Ok(())
}
