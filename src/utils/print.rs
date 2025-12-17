#![allow(dead_code)]
#![allow(unused_variables)]

use std::io::stdout;

use anyhow::Result;
use crossterm::style::Color;
use indicatif::{ProgressBar, ProgressStyle};

use super::consts::{PROGRESS_TEMPLATE, PROGRESS_TICK};

use crate::{
    git::{log::GitLog, repo::GitRepo},
    providers::schema::ResponseCommit,
    settings::Settings,
    utils::graph::Arena,
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

    pub fn start(
        &self,
        msg: &str,
    ) {
        self.spinner.reset();

        self.spinner
            .enable_steady_tick(std::time::Duration::from_millis(80));
        self.spinner.set_message(msg.to_owned());
    }

    pub fn stop(
        &self,
        msg: Option<&str>,
    ) {
        if let Some(message) = msg {
            self.spinner.finish_with_message(message.to_owned());
        } else {
            self.spinner.finish();
        }
    }
}

impl Default for SpinDeez {
    fn default() -> Self {
        Self::new()
    }
}

// compact status but not as compact code
// might have to rewrite a more generic way of printing different parts
// to avoid repetition but meh
fn compact_status(git: &GitRepo) -> Result<()> {
    Ok(())
}

pub fn pretty_print_status(
    git: &GitRepo,
    compact: bool,
) -> Result<()> {
    Ok(())
}

fn compact_print_commits(
    commits: &[ResponseCommit],
    cfg: &Settings,
    git: &GitRepo,
) -> Result<()> {
    Ok(())
}

pub fn pretty_print_commits(
    commits: &[ResponseCommit],
    cfg: &Settings,
    git: &GitRepo,
    compact: bool,
) -> Result<()> {
    if compact {
        return compact_print_commits(commits, cfg, git);
    }

    let mut stdout = stdout();
    let mut arena = Arena::new();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.commit.capitalize_prefix,
            cfg.commit.include_scope,
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

        let files_parent = arena.new_node("Files", Color::Magenta);
        arena.set_count(files_parent, commit.files.len());
        arena.add_child(commit_root, files_parent);

        for file in &commit.files {
            let file_node = arena.new_node(file, Color::White);
            arena.add_child(files_parent, file_node);
        }
    }

    arena.print_tree(&mut stdout)?;

    Ok(())
}

fn compact_print_logs(logs: &[GitLog]) -> Result<()> {
    Ok(())
}

pub fn pretty_print_logs(
    logs: &[GitLog],
    compact: bool,
) -> Result<()> {
    Ok(())
}
