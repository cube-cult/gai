use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::stdout;

use super::{
    consts::{PROGRESS_TEMPLATE, PROGRESS_TICK},
    graph::Arena,
};

use crate::{
    configuration::Config,
    git::{log::GaiLog, repo::GaiGit},
    providers::schema::ResponseCommit,
};

// yes lmao
// i realize this wasn't the problem,
// but already made it
// so I'll keep it.
pub struct SpinDeez {
    spinner: ProgressBar,
}

impl SpinDeez {
    pub fn new() -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template(PROGRESS_TEMPLATE)
                .unwrap()
                .tick_strings(PROGRESS_TICK),
        );

        Self { spinner: bar }
    }

    pub fn start(&self, msg: &str) {
        self.spinner.reset();

        self.spinner
            .enable_steady_tick(std::time::Duration::from_millis(80));
        self.spinner.set_message(msg.to_owned());
    }

    pub fn stop(&self, msg: Option<&str>) {
        if let Some(message) = msg {
            self.spinner.finish_with_message(message.to_owned());
        } else {
            self.spinner.finish();
        }
    }
}

// compact status but not as compact code
// might have to rewrite a more generic way of printing different parts
// to avoid repetition but meh
fn compact_status(gai: &GaiGit) -> Result<()> {
    let mut stdout = stdout();
    let branch = &gai.get_branch();
    let status = &gai.status;

    let staged_count = gai.staged_len();
    let unstaged_count = gai.unstaged_len();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("Branch: {}\n", branch)),
        ResetColor
    )?;

    if unstaged_count == 0 && staged_count == 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("No Diffs\n"),
            ResetColor
        )?;
        return Ok(());
    }

    if staged_count > 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print(format!("Staged ({})\n", staged_count)),
            ResetColor
        )?;

        if !status.s_new.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("  A  "),
            )?;
            for (i, file) in status.s_new.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.s_modified.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print("  M  "),
            )?;
            for (i, file) in status.s_modified.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.s_deleted.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("  D  "),
            )?;
            for (i, file) in status.s_deleted.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.s_renamed.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print("  R  "),
            )?;
            for (i, (old, new)) in status.s_renamed.iter().enumerate()
            {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(
                    stdout,
                    Print(format!("{} → {}", old, new))
                )?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }
    }

    if unstaged_count > 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!("Unstaged ({})\n", unstaged_count)),
            ResetColor
        )?;

        if !status.u_new.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("  ?  "),
            )?;
            for (i, file) in status.u_new.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.u_modified.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print("  M  "),
            )?;
            for (i, file) in status.u_modified.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.u_deleted.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("  D  "),
            )?;
            for (i, file) in status.u_deleted.iter().enumerate() {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(stdout, Print(file))?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }

        if !status.u_renamed.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print("  R  "),
            )?;
            for (i, (old, new)) in status.u_renamed.iter().enumerate()
            {
                if i > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(
                    stdout,
                    Print(format!("{} → {}", old, new))
                )?;
            }
            execute!(stdout, Print("\n"), ResetColor)?;
        }
    }

    Ok(())
}

pub fn pretty_print_status(
    gai: &GaiGit,
    compact: bool,
) -> Result<()> {
    if compact {
        return compact_status(gai);
    }

    let mut stdout = stdout();
    let mut arena = Arena::new();

    let branch = &gai.get_branch();
    let status = &gai.status;

    let staged_count = gai.staged_len();
    let unstaged_count = gai.unstaged_len();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("On Branch: {}\n", branch).bold()),
        ResetColor
    )?;

    if unstaged_count == 0 && staged_count == 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("No Diffs".bold()),
            ResetColor
        )?;

        return Ok(());
    }

    if staged_count > 0 {
        let staged_root = arena.new_node("✓ Staged", Color::Green);
        arena.set_count(staged_root, staged_count);

        if !status.s_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.s_new.len());
            arena.add_child(staged_root, new_node);

            for file in &status.s_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "A");
                arena.add_child(new_node, file_node);
            }
        }

        // mod
        if !status.s_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.s_modified.len());
            arena.add_child(staged_root, modified_node);

            for file in &status.s_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        // del
        if !status.s_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.s_deleted.len());
            arena.add_child(staged_root, deleted_node);

            for file in &status.s_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        // ren
        if !status.s_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.s_renamed.len());
            arena.add_child(staged_root, renamed_node);

            for (old, new) in &status.s_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    if unstaged_count > 0 {
        let unstaged_root =
            arena.new_node("⚠ Unstaged", Color::Yellow);
        arena.set_count(unstaged_root, unstaged_count);

        if !status.u_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.u_new.len());
            arena.add_child(unstaged_root, new_node);

            for file in &status.u_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "?");
                arena.add_child(new_node, file_node);
            }
        }

        if !status.u_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.u_modified.len());
            arena.add_child(unstaged_root, modified_node);

            for file in &status.u_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        if !status.u_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.u_deleted.len());
            arena.add_child(unstaged_root, deleted_node);

            for file in &status.u_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        if !status.u_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.u_renamed.len());
            arena.add_child(unstaged_root, renamed_node);

            for (old, new) in &status.u_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    arena.print_tree(&mut stdout)?;

    Ok(())
}

fn compact_print_commits(
    commits: &[ResponseCommit],
    cfg: &Config,
    gai: &GaiGit,
) -> Result<()> {
    let mut stdout = stdout();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.gai.commit_config.capitalize_prefix,
            cfg.gai.commit_config.include_scope,
        );

        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("Commit {}: ", i + 1)),
            ResetColor,
            SetForegroundColor(Color::Green),
            Print(format!("{} ", prefix)),
            ResetColor,
            SetForegroundColor(Color::White),
            Print(format!("{}\n", commit.header)),
            ResetColor
        )?;

        if !commit.body.is_empty() {
            let body_preview = if commit.body.len() > 60 {
                format!("{}...", &commit.body[..60])
            } else {
                commit.body.clone()
            };
            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(format!("  {}\n", body_preview)),
                ResetColor
            )?;
        }

        if gai.stage_hunks {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print(format!("  Hunks: {:?}\n", commit.hunk_ids)),
                ResetColor
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print(format!("  Files ({}): ", commit.files.len())),
                ResetColor
            )?;

            for (j, file) in commit.files.iter().enumerate() {
                if j > 0 {
                    execute!(stdout, Print(", "))?;
                }
                execute!(
                    stdout,
                    SetForegroundColor(Color::White),
                    Print(file),
                    ResetColor
                )?;
            }
            execute!(stdout, Print("\n"))?;
        }

        if i < commits.len() - 1 {
            execute!(stdout, Print("\n"))?;
        }
    }

    Ok(())
}

pub fn pretty_print_commits(
    commits: &[ResponseCommit],
    cfg: &Config,
    gai: &GaiGit,
    compact: bool,
) -> Result<()> {
    if compact {
        return compact_print_commits(commits, cfg, gai);
    }

    let mut stdout = stdout();
    let mut arena = Arena::new();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.gai.commit_config.capitalize_prefix,
            cfg.gai.commit_config.include_scope,
        );

        let commit_root = arena
            .new_node(format!("Commit {}", i + 1), Color::DarkGrey);

        let prefix_node = arena.new_node(prefix, Color::Green);
        arena.add_child(commit_root, prefix_node);

        let header_node = arena.new_node(
            format!("Header: {}", commit.header),
            Color::White,
        );
        arena.add_child(commit_root, header_node);

        if !commit.body.is_empty() {
            let body_text = arena.truncate(&commit.body, 45);
            let body_node = arena.new_node(
                format!("Body: {}", body_text),
                Color::Blue,
            );
            arena.add_child(commit_root, body_node);
        }

        if gai.stage_hunks {
            let hunks_node = arena.new_node(
                format!("Hunks: {:?}", commit.hunk_ids),
                Color::Magenta,
            );
            arena.add_child(commit_root, hunks_node);
        } else {
            let files_parent =
                arena.new_node("Files", Color::Magenta);
            arena.set_count(files_parent, commit.files.len());
            arena.add_child(commit_root, files_parent);

            for file in &commit.files {
                let file_node = arena.new_node(file, Color::White);
                arena.add_child(files_parent, file_node);
            }
        }
    }

    arena.print_tree(&mut stdout)?;

    Ok(())
}

fn compact_print_logs(logs: &[GaiLog]) -> Result<()> {
    let mut stdout = stdout();

    for log in logs {
        let short_hash =
            &log.commit_hash[..7.min(log.commit_hash.len())];

        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!("{} ", short_hash)),
        )?;

        if let Some(ref message) = log.message {
            let first = message.lines().next().unwrap_or("");
            let msg = if first.len() > 60 {
                format!("{}...", &first[..60])
            } else {
                first.to_string()
            };
            execute!(
                stdout,
                SetForegroundColor(Color::White),
                Print(msg),
            )?;
        } else {
            if let Some(ref prefix) = log.prefix {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print(prefix),
                )?;
            }

            if let Some(ref scope) = log.scope {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print(format!("({})", scope)),
                )?;
            }

            if log.breaking {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    Print("!"),
                )?;
            }

            if log.prefix.is_some() || log.scope.is_some() {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print(": "),
                )?;
            }

            if let Some(ref header) = log.header {
                execute!(
                    stdout,
                    SetForegroundColor(Color::White),
                    Print(header),
                )?;
            }
        }

        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" - {} ({})", log.author, log.date)),
            ResetColor,
            Print("\n"),
        )?;
    }

    Ok(())
}

pub fn pretty_print_logs(
    logs: &[GaiLog],
    compact: bool,
) -> Result<()> {
    if compact {
        return compact_print_logs(logs);
    }

    let mut stdout = stdout();
    let mut arena = Arena::new();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("Commit History({}):\n", logs.len()).bold()),
        ResetColor
    )?;

    for log in logs {
        let short_hash =
            &log.commit_hash[..7.min(log.commit_hash.len())];
        let log_root = arena.new_node(short_hash, Color::Yellow);

        if let Some(ref message) = log.message {
            // avoid breaking into another tree
            // when message has a
            // \n
            let first = message.lines().next().unwrap_or("");
            let msg_node = arena
                .new_node(arena.truncate(first, 60), Color::White);
            arena.add_child(log_root, msg_node);
        } else {
            let mut title_parts = Vec::new();

            if let Some(ref prefix) = log.prefix {
                title_parts.push(prefix.to_owned());
            }

            if let Some(ref scope) = log.scope {
                title_parts.push(format!("({})", scope));
            }

            if log.breaking {
                title_parts.push("!".to_string());
            }

            if !title_parts.is_empty() {
                let prefix_str = title_parts.join("");
                let prefix_node =
                    arena.new_node(prefix_str, Color::Green);
                arena.add_child(log_root, prefix_node);
            }

            if let Some(ref header) = log.header {
                let header_node =
                    arena.new_node(header, Color::White);
                arena.add_child(log_root, header_node);
            }

            if let Some(ref body) = log.body {
                let body_text = arena.truncate(body, 50);
                let body_node =
                    arena.new_node(body_text, Color::DarkGrey);
                arena.add_child(log_root, body_node);
            }
        }

        let author_node = arena.new_node(&log.author, Color::Blue);
        arena.add_child(log_root, author_node);

        let date_node = arena.new_node(&log.date, Color::DarkGrey);
        arena.add_child(log_root, date_node);
    }

    arena.print_tree(&mut stdout)?;

    Ok(())
}
