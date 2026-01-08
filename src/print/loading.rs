use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// util loading bar
pub struct Loading {
    bar: ProgressBar,
    interval: Duration,
}

impl Loading {
    /// create loading widget
    /// compact is a single line
    /// otherwise, a multiline loading spinner
    pub fn new(
        text: &str,
        compact: bool,
    ) -> anyhow::Result<Self> {
        let bar = ProgressBar::new_spinner();

        let (style, interval) = if compact {
            (
                ProgressStyle::with_template("{spinner:.red} {msg}")?
                    .tick_strings(TICK_COMPACT),
                Duration::from_millis(80),
            )
        } else {
            (
                ProgressStyle::with_template(
                    "{msg}\n{spinner:.red}",
                )?
                .tick_strings(TICK_LONG),
                Duration::from_millis(300),
            )
        };

        bar.set_style(style);
        bar.set_message(text.to_owned());

        Ok(Self { bar, interval })
    }

    pub fn interval(
        mut self,
        interval: Duration,
    ) -> Self {
        self.interval = interval;
        self
    }

    pub fn set_text(
        &mut self,
        text: &str,
    ) {
        self.bar
            .set_message(text.to_owned());
    }

    pub fn start(&self) {
        self.bar
            .enable_steady_tick(self.interval);
    }

    pub fn stop_clear(&self) {
        self.bar
            .finish_and_clear();
        //self.bar.reset();
    }

    pub fn stop_with_message(
        &self,
        text: &str,
    ) {
        self.bar
            .finish_with_message(text.to_owned());
    }

    pub fn stop(&self) {
        self.bar.finish();
    }
}

pub const TICK_COMPACT: &[&str; 9] =
    &["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶", "⣿"];

/// made with ascii-motion.app
const TICK_LONG: &[&str] = &[
    concat!(
        " ||    ||            \n",
        " | O  O |  |         \n",
        " |  vv  |  |         \n",
        " |      |__|         ",
    ),
    concat!(
        " ||    ||            \n",
        " | -  - |  |         \n",
        " |  vv  |  |         \n",
        " |      |__|         ",
    ),
    concat!(
        " ||    ||            \n",
        " | O  o |   /        \n",
        " |  vv  |  /         \n",
        " |      |_/          ",
    ),
    concat!(
        " ||    ||            \n",
        " | ^  ^ |   /        \n",
        " |  vv  |  /         \n",
        " |      |_/  []      ",
    ),
    concat!(
        " ||    ||            \n",
        " | o  O |  |         \n",
        " |  vv  |  |  []     \n",
        " |      |__|  []     ",
    ),
    concat!(
        " ||    ||            \n",
        " | o  o |  |         \n",
        " |  vv  |  | [][]    \n",
        " |      |__| [][]    ",
    ),
    concat!(
        " ||    ||            \n",
        " | O  O |   /  []    \n",
        " |  vv  |  / [][]    \n",
        " |      |_/  [][]    ",
    ),
    concat!(
        " ||    ||            \n",
        " | >  < |  | [][]    \n",
        " |  vv  |  | [][]    \n",
        " |      |__| [][]    ",
    ),
    concat!(
        " ||    ||      []    \n",
        " | O  o |  | [][]    \n",
        " |  ^^  |  | [][]    \n",
        " |      |__| [][]    ",
    ),
    concat!(
        " ||    ||    [][]    \n",
        " | @  @ |   /[][]    \n",
        " |  vv  |  / [][]    \n",
        " |      |_/  [][]    ",
    ),
    concat!(
        " ||    ||    [][]    \n",
        " | o  o |  | [][]    \n",
        " |  vv  |  | [][]    \n",
        " |      |__| [][][]    ",
    ),
    concat!(
        " ||    ||    [][]    \n",
        " | ^  ^ |  | [][]    \n",
        " |  uu  |  | [][][]  \n",
        " |      |__| [][][]  ",
    ),
    concat!(
        " ||    ||    [][]    \n",
        " | O  O |   /[][][]  \n",
        " |  uu  |  / [][][]  \n",
        " |      |_/  [][][]  ",
    ),
    concat!(
        " ||    ||    [][][]  \n",
        " | ^  ^ |  v [][][]  \n",
        " |  UU  |  | [][][]  \n",
        " |      |__| [][][]  ",
    ),
];
