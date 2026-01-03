use console::{Color, Style, style};

use crate::git::log::GitLog;

use super::tree::{Tree, TreeItem};

pub fn print(
    git_logs: &[GitLog],
    compact: bool,
) -> anyhow::Result<()> {
    let mut items = Vec::new();

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

        // set max to 100
        // i think we can make this configurable?
        let truncated = if message.len() > 100 {
            format!("{}...", &message[..100])
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

        let message = style(&truncated).fg(color).bold();

        let display = format!("{} {}", hash_display, message);

        let item = TreeItem::new(
            git_log.commit_hash.to_owned(),
            display,
            commit_children,
        )?;

        items.push(item);
    }

    if !items.is_empty() {
        Tree::new(&items)?
            .collapsed(compact)
            .style(Style::new().dim())
            .render();
    }

    Ok(())
}
