use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use super::consts::{PROGRESS_TEMPLATE, PROGRESS_TICK};

use crate::{
    git::{log::GaiLog, repo::GaiGit},
    providers::schema::ResponseCommit,
    settings::Settings,
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

impl Default for SpinDeez {
    fn default() -> Self {
        Self::new()
    }
}

// compact status but not as compact code
// might have to rewrite a more generic way of printing different parts
// to avoid repetition but meh
fn compact_status(gai: &GaiGit) -> Result<()> {
    Ok(())
}

pub fn pretty_print_status(
    gai: &GaiGit,
    compact: bool,
) -> Result<()> {
    Ok(())
}

fn compact_print_commits(
    commits: &[ResponseCommit],
    cfg: &Settings,
    gai: &GaiGit,
) -> Result<()> {
    Ok(())
}

pub fn pretty_print_commits(
    commits: &[ResponseCommit],
    cfg: &Settings,
    gai: &GaiGit,
    compact: bool,
) -> Result<()> {
    Ok(())
}

fn compact_print_logs(logs: &[GaiLog]) -> Result<()> {
    Ok(())
}

pub fn pretty_print_logs(
    logs: &[GaiLog],
    compact: bool,
) -> Result<()> {
    Ok(())
}
