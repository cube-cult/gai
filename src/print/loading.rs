use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// util loading bar
pub struct Loading {
    bar: ProgressBar,
}

impl Loading {
    /// compact spinner
    pub fn new_compact(text: &str) -> anyhow::Result<Self> {
        let bar = ProgressBar::new_spinner();

        let style =
            ProgressStyle::with_template("{spinner:.cyan} {msg}")?
                .tick_strings(TICK_COMPACT);

        bar.set_style(style);
        bar.set_message(text.to_owned());

        Ok(Self { bar })
    }

    /// large loading message
    /// looks like an email being sent two to pcs
    pub fn new(text: &str) -> anyhow::Result<Self> {
        let bar = ProgressBar::new_spinner();

        let style =
            ProgressStyle::with_template("{msg}\n{spinner}\n")?
                .tick_strings(TICK_LONG);

        bar.set_style(style);
        bar.set_message(text.to_owned());

        Ok(Self { bar })
    }

    pub fn set_text(
        &mut self,
        text: &str,
    ) {
        self.bar.set_message(text.to_owned());
    }

    pub fn start(&self) {
        let interval = Duration::from_millis(80);

        self.bar.enable_steady_tick(interval);
    }

    pub fn stop(&self) {
        self.bar.finish_and_clear();

        self.bar.reset();
    }
}

pub const TICK_COMPACT: &[&str; 9] =
    &["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶", "⣿"];

/// made with ascii-motion.app
const TICK_LONG: &[&str] = &[
    concat!(
        " __          __\n",
        "|▪▪|@-------|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|-@------|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|--@-----|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|---@----|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|----@---|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|-----@--|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|------@-|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|-------@|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|------@-|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|-----@--|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|----@---|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|---@----|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|--@-----|▪▪|\n",
        "|__|        |__|",
    ),
    concat!(
        " __          __\n",
        "|▪▪|-@------|▪▪|\n",
        "|__|        |__|",
    ),
];
