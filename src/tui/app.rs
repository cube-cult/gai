use anyhow::Result;
use ratatui::Frame;
use ratatui::widgets::Widget;
use ratatui::{
    style::{Modifier, Style, palette::tailwind},
    widgets::ListState,
};
use std::{sync::mpsc, time::Duration};
use throbber_widgets_tui::{Set, ThrobberState, WhichUse};

use super::{
    commit::CommitScreen,
    events::{Event, poll_event},
};
use crate::tui::commit::CommitScreenWidget;
use crate::{
    ai::request::Request, config::Config, git::repo::GaiGit,
};

const PRIMARY_TEXT: Style = Style::new().fg(tailwind::WHITE);
const SECONDARY_TEXT: Style = Style::new().fg(tailwind::CYAN.c400);
const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(tailwind::CYAN.c800)
    .add_modifier(Modifier::BOLD);
const THROBBER_STYLE: Style = Style::new()
    .fg(ratatui::style::Color::Cyan)
    .add_modifier(Modifier::BOLD);
const THROBBER_SET: Set = throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE;
const THROBBER_TYPE: WhichUse = throbber_widgets_tui::WhichUse::Spin;

pub struct TUIState {
    pub selected_screen: ListState,
}

pub struct ThrobberStyles {
    pub throbber_style: Style,
    pub throbber_set: Set,
    pub throbber_type: WhichUse,
}

pub struct TextStyles {
    pub primary_text_style: Style,
    pub secondary_text_style: Style,
    pub highlight_text_style: Style,
}

#[derive(Debug)]
pub enum CurrentScreen {
    Diffs,
    Commits,
}

pub struct App {
    pub running: bool,
    pub cfg: Config,
    pub gai: GaiGit,

    pub tui_state: TUIState,
    pub current_screen: CurrentScreen,

    pub commit_screen: CommitScreen,

    pub throbber_styles: ThrobberStyles,
    pub text_styles: TextStyles,
}

impl Default for TUIState {
    fn default() -> Self {
        let mut selected_state = ListState::default();
        selected_state.select_first();

        Self {
            selected_screen: selected_state,
        }
    }
}

impl Default for TextStyles {
    fn default() -> Self {
        Self {
            primary_text_style: PRIMARY_TEXT,
            secondary_text_style: SECONDARY_TEXT,
            highlight_text_style: HIGHLIGHT_STYLE,
        }
    }
}

impl Default for ThrobberStyles {
    fn default() -> Self {
        Self {
            throbber_style: THROBBER_STYLE,
            throbber_set: THROBBER_SET,
            throbber_type: THROBBER_TYPE,
        }
    }
}

pub fn run_tui(cfg: Config, gai: GaiGit) -> Result<()> {
    let mut terminal = ratatui::init();
    let timeout = Duration::from_millis(50);

    let (tx, rx) = mpsc::channel::<Event>();

    let mut app = App::new(cfg, gai, None);

    while app.running {
        terminal.draw(|f| app.run(f))?;

        if let Some(event) = poll_event(&rx, timeout)? {
            match app.current_screen {
                CurrentScreen::Commits => todo!(),
                CurrentScreen::Diffs => todo!(),
            }
        }
    }

    ratatui::restore();

    Ok(())
}

impl App {
    pub fn new(
        cfg: Config,
        gai: GaiGit,
        curr_screen: Option<CurrentScreen>,
    ) -> Self {
        let current_screen = match curr_screen {
            Some(c) => c,
            None => CurrentScreen::Commits,
        };

        let commit_screen =
            CommitScreen::new(&cfg.ai, &cfg.gai.commit_config);

        Self {
            running: true,
            cfg,
            gai,
            current_screen,
            commit_screen,
            tui_state: TUIState::default(),
            throbber_styles: ThrobberStyles::default(),
            text_styles: TextStyles::default(),
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        match self.current_screen {
            CurrentScreen::Diffs => {}
            CurrentScreen::Commits => {
                CommitScreenWidget {
                    screen: &self.commit_screen,
                    throbber_styles: &self.throbber_styles,
                    text_styles: &self.text_styles,
                }
                .render(frame.area(), frame.buffer_mut());
            }
        }
    }
}
