use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style, Styled, Stylize, palette::tailwind},
    text::Line,
    widgets::{ListState, Widget},
};
use std::{
    sync::mpsc::{self, Sender},
    time::Duration,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use throbber_widgets_tui::{Set, WhichUse};

use super::{
    commits::{CommitScreen, CommitScreenWidget},
    diffs::{DiffScreen, DiffScreenWidget},
    events::{Event, poll_event},
    utils::center,
};
use crate::{
    ai::provider::Provider::Gai, config::Config, git::repo::GaiGit,
};

const PRIMARY_TEXT: Style = Style::new().fg(tailwind::WHITE);
const SECONDARY_TEXT: Style = Style::new().fg(tailwind::CYAN.c400);
const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(tailwind::CYAN.c800)
    .add_modifier(Modifier::BOLD);
const BORDER_STYLE: Style = Style::new().bg(tailwind::SLATE.c800);
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
    pub border_style: Style,
}

#[derive(Debug, Display, EnumIter, FromRepr, PartialEq)]
pub enum CurrentScreen {
    Diffs,
    Commits,
    Logs,
}

pub struct App {
    pub running: bool,
    pub cfg: Config,
    pub gai: GaiGit,

    pub tui_state: TUIState,
    pub current_screen: CurrentScreen,

    pub commit_screen: CommitScreen,
    pub diff_screen: DiffScreen,

    pub throbber_styles: ThrobberStyles,
    pub text_styles: TextStyles,

    pub event_tx: mpsc::Sender<Event>,
}

impl Default for TextStyles {
    fn default() -> Self {
        Self {
            primary_text_style: PRIMARY_TEXT,
            secondary_text_style: SECONDARY_TEXT,
            highlight_text_style: HIGHLIGHT_STYLE,
            border_style: BORDER_STYLE,
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

    let mut app = App::new(cfg, gai, None, tx);

    while app.running {
        terminal.draw(|f| app.run(f))?;

        let event = poll_event(&rx, timeout)?;

        if app.handle_main_events(&event) {
            break;
        }

        match app.current_screen {
            CurrentScreen::Commits => app.commit_screen.handle_event(
                &event,
                &app.event_tx,
                &app.cfg,
                &app.gai,
            ),
            CurrentScreen::Diffs => {
                app.diff_screen.handle_event(&event)
            }
            _ => {}
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
        event_tx: Sender<Event>,
    ) -> Self {
        let current_screen = match curr_screen {
            Some(c) => c,
            None => CurrentScreen::Diffs,
        };

        let mut selected_screen = ListState::default();
        selected_screen.select(Some(0));

        let tui_state = TUIState { selected_screen };

        let diff_screen = DiffScreen::new(&gai.files);
        let commit_screen = CommitScreen::new(
            &cfg.ai,
            &cfg.gai.commit_config,
            cfg.ai.providers.get(&Gai).unwrap(),
        );

        Self {
            running: true,
            cfg,
            gai,
            current_screen,
            commit_screen,
            diff_screen,
            tui_state,
            throbber_styles: ThrobberStyles::default(),
            text_styles: TextStyles::default(),
            event_tx,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(90),
        ]);

        let [screen_list_area, screen_area] =
            horizontal.areas(frame.area());

        self.render_screen_list(screen_list_area, frame.buffer_mut());

        match self.current_screen {
            CurrentScreen::Diffs => {
                DiffScreenWidget {
                    screen: &mut self.diff_screen,
                    text_styles: &self.text_styles,
                }
                .render(screen_area, frame.buffer_mut());
            }
            CurrentScreen::Commits => {
                CommitScreenWidget {
                    screen: &mut self.commit_screen,
                    throbber_styles: &self.throbber_styles,
                    text_styles: &self.text_styles,
                }
                .render(screen_area, frame.buffer_mut());
            }
            _ => {}
        }
    }

    fn handle_main_events(&mut self, event: &Event) -> bool {
        match event {
            Event::Mouse(_) => {}
            Event::Key(k) => match k.code {
                KeyCode::Esc => return true,
                KeyCode::Up | KeyCode::BackTab => self.go_up(),
                KeyCode::Down | KeyCode::Tab => self.go_down(),
                _ => {}
            },
            _ => {}
        }

        false
    }

    fn go_up(&mut self) {
        if let Some(selected) =
            self.tui_state.selected_screen.selected()
        {
            if selected == 0 {
                self.tui_state.selected_screen.select_last();
            } else {
                self.tui_state.selected_screen.select_previous();
            }

            self.set_current_screen(
                self.tui_state.selected_screen.selected(),
            );
        }
    }

    fn go_down(&mut self) {
        if let Some(selected) =
            self.tui_state.selected_screen.selected()
        {
            if selected + 1 >= CurrentScreen::iter().len() {
                self.tui_state.selected_screen.select_first();
            } else {
                self.tui_state.selected_screen.select_next();
            }

            self.set_current_screen(
                self.tui_state.selected_screen.selected(),
            );
        }
    }

    fn set_current_screen(
        &mut self,
        tui_state_selected: Option<usize>,
    ) {
        if let Some(selected) = tui_state_selected
            && let Some(screen) = CurrentScreen::from_repr(selected)
        {
            self.current_screen = screen;
        }
    }

    fn render_screen_list(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let screens: Vec<CurrentScreen> =
            CurrentScreen::iter().collect();
        let selected_idx = self.tui_state.selected_screen.selected();

        let total_height = screens.len() as u16;

        let centered_area = center(
            area,
            Constraint::Length(area.width),
            Constraint::Length(total_height),
        );

        let constraints: Vec<Constraint> =
            screens.iter().map(|_| Constraint::Length(1)).collect();

        let layout =
            Layout::vertical(constraints).split(centered_area);

        for (i, screen) in screens.iter().enumerate() {
            let item_area = layout[i];
            let is_selected = Some(i) == selected_idx;

            let line = if is_selected {
                Line::from(vec![
                    " ↪ ".set_style(
                        self.text_styles.secondary_text_style,
                    ),
                    screen.to_string().set_style(
                        self.text_styles.highlight_text_style,
                    ),
                ])
            } else {
                Line::from(vec![
                    "   ".into(),
                    screen.to_string().fg(tailwind::SLATE.c600),
                ])
            };

            buf.set_line(
                item_area.x,
                item_area.y,
                &line,
                item_area.width,
            );
        }
    }
}
