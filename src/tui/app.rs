use anyhow::Result;
use ratatui::Frame;
use ratatui::widgets::StatefulWidget;
use ratatui::{
    style::{Modifier, Style, palette::tailwind},
    widgets::ListState,
};
use std::{sync::mpsc, time::Duration};
use throbber_widgets_tui::{Set, ThrobberState, WhichUse};

use super::events::{Event, poll_event};
use crate::tui::commit::CommitScreen;
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
    pub selected: usize,
    pub selected_state: ListState,

    pub primary_text_style: Style,
    pub secondary_text_style: Style,
    pub highlight_text_style: Style,

    pub throbber_state: ThrobberState,
    pub throbber_style: Style,
    pub throbber_set: Set,
    pub throbber_type: WhichUse,
}

#[derive(Debug)]
pub enum CurrentScreen {
    Diffs,
    Commit,
}

pub struct App {
    pub running: bool,
    pub cfg: Config,
    pub gai: GaiGit,

    pub tui_state: TUIState,

    pub current_screen: CurrentScreen,
}

impl Default for TUIState {
    fn default() -> Self {
        let mut selected_state = ListState::default();
        selected_state.select_first();

        Self {
            selected: 0,
            selected_state,
            primary_text_style: PRIMARY_TEXT,
            secondary_text_style: SECONDARY_TEXT,
            highlight_text_style: HIGHLIGHT_STYLE,

            throbber_state: ThrobberState::default(),
            throbber_style: THROBBER_STYLE,
            throbber_set: THROBBER_SET,
            throbber_type: THROBBER_TYPE,
        }
    }
}

pub fn run_tui(cfg: Config, gai: GaiGit) -> Result<()> {
    let mut terminal = ratatui::init();
    let timeout = Duration::from_millis(50);
    let mut tui_state = TUIState::default();

    let (tx, rx) = mpsc::channel::<Event>();

    let mut app = App::new(cfg, gai, None);

    while app.running {
        terminal.draw(|f| app.run(f))?;

        if let Some(event) = poll_event(&rx, timeout)? {
            match app.current_screen {
                CurrentScreen::Commit => todo!(),
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
            None => CurrentScreen::Commit,
        };

        Self {
            running: true,
            cfg,
            gai,
            tui_state: TUIState::default(),
            current_screen,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        match self.current_screen {
            CurrentScreen::Diffs => {}
            CurrentScreen::Commit => {
                let commit_screen = CommitScreen {
                    provider: self.cfg.ai.provider.to_string(),
                    model: "todo: implement provider".to_owned(),
                    capitalize_prefix: self
                        .cfg
                        .gai
                        .commit_config
                        .capitalize_prefix,
                    include_scope: self
                        .cfg
                        .gai
                        .commit_config
                        .include_scope,
                    commits: Vec::new(),
                    request_sent: false,
                    is_waiting: false,
                };

                commit_screen.render(
                    frame.area(),
                    frame.buffer_mut(),
                    &mut self.tui_state,
                );
            }
        }
    }

    pub async fn send_request(&mut self) -> Result<()> {
        let ai = &self.cfg.ai;
        let provider = ai.provider;
        let provider_cfg =
            ai.providers.get(&provider).ok_or(anyhow::anyhow!(
                "Somehow did not find a valid provider config.",
            ))?;

        let mut req = Request::default();

        req.build_prompt(&self.cfg, &self.gai);
        req.build_diffs_string(self.gai.get_file_diffs_as_str());

        Ok(())
    }
}
