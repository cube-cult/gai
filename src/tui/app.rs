use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Margin},
    style::{Modifier, Style, Styled, palette::tailwind},
    text::Line,
    widgets::{Block, Borders, ListState, Widget},
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
};
use crate::{
    config::Config,
    git::repo::GaiGit,
    tui::{
        logs::{LogScreen, LogScreenWidget},
        popup::{Popup, PopupWidget},
    },
};

const PRIMARY_TEXT: Style = Style::new().fg(tailwind::WHITE);
const SECONDARY_TEXT: Style = Style::new().fg(tailwind::CYAN.c400);
const TERTIARY_TEXT: Style = Style::new().fg(tailwind::AMBER.c400);
const HIGHLIGHT_STYLE: Style = Style::new()
    .fg(tailwind::WHITE)
    .bg(tailwind::CYAN.c800)
    .add_modifier(Modifier::BOLD);
const BORDER_STYLE: Style = Style::new().fg(tailwind::SLATE.c800);
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
    pub tertiary_text_style: Style,
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
    pub log_screen: LogScreen,

    pub popup: Option<Popup>,

    pub throbber_styles: ThrobberStyles,
    pub text_styles: TextStyles,

    pub event_tx: mpsc::Sender<Event>,
}

impl Default for TextStyles {
    fn default() -> Self {
        Self {
            primary_text_style: PRIMARY_TEXT,
            secondary_text_style: SECONDARY_TEXT,
            tertiary_text_style: TERTIARY_TEXT,
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

        if let Some(ref mut popup) = app.popup {
            if popup.handle_event(&event, &app.event_tx) {
                app.popup = None;
            }
            continue;
        }

        match app.current_screen {
            CurrentScreen::Commits => app.commit_screen.handle_event(
                &event,
                &app.event_tx,
                &app.cfg,
                &app.gai,
            ),
            CurrentScreen::Diffs => {
                app.diff_screen.handle_event(&event, &mut app.gai)
            }
            CurrentScreen::Logs => {
                app.log_screen.handle_event(&event);
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
        let commit_screen =
            CommitScreen::new(&cfg.ai, &cfg.gai.commit_config);
        let log_screen =
            LogScreen::new(gai.get_logs(None, false).unwrap());

        Self {
            running: true,
            cfg,
            gai,
            current_screen,
            commit_screen,
            diff_screen,
            log_screen,
            popup: None,
            tui_state,
            throbber_styles: ThrobberStyles::default(),
            text_styles: TextStyles::default(),
            event_tx,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
        ]);

        let [screen_list_area, screen_area] =
            vertical.areas(frame.area());

        let b = Block::default()
            .border_style(self.text_styles.border_style)
            .borders(Borders::ALL);

        frame.render_widget(&b, screen_list_area);

        let screen_list_area =
            screen_list_area.inner(Margin::new(1, 1));

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
            CurrentScreen::Logs => {
                LogScreenWidget {
                    screen: &mut self.log_screen,
                    text_styles: &self.text_styles,
                }
                .render(screen_area, frame.buffer_mut());
            }
        }

        // todo use popup content
        // only render popup
        // to avoid clone
        if let Some(ref mut popup) = self.popup {
            PopupWidget {
                popup,
                text_styles: &self.text_styles,
            }
            .render(screen_area, frame.buffer_mut());
        }
    }

    fn handle_main_events(&mut self, event: &Event) -> bool {
        match event {
            Event::Mouse(_) => {}
            Event::Key(k) => {
                if self.popup.is_none() {
                    match k.code {
                        KeyCode::Esc => return true,
                        KeyCode::Left | KeyCode::BackTab => {
                            self.go_back()
                        }
                        KeyCode::Right | KeyCode::Tab => {
                            self.go_next()
                        }
                        KeyCode::Char('z') => {
                            self.popup = Some(Popup::new(
                                &super::popup::PopupType::Confirm(
                                    "LULW".to_owned(),
                                ),
                            ));
                        }
                        KeyCode::Char('1') => self.go_tab(1),
                        KeyCode::Char('2') => self.go_tab(2),
                        KeyCode::Char('3') => self.go_tab(3),
                        KeyCode::Char('4') => self.go_tab(4),
                        KeyCode::Char('5') => self.go_tab(5),
                        _ => {}
                    }
                }
            }
            Event::PopUp(popup_type) => {
                self.popup = Some(Popup::new(popup_type));
            }
            _ => {}
        }

        false
    }

    fn go_tab(&mut self, num: usize) {
        if num > CurrentScreen::iter().len() {
            return;
        }

        self.tui_state.selected_screen.select(Some(num - 1));

        self.set_current_screen(
            self.tui_state.selected_screen.selected(),
        );
    }

    fn go_back(&mut self) {
        let screens = CurrentScreen::iter().len();
        if let Some(selected) =
            self.tui_state.selected_screen.selected()
        {
            let new_selected = if selected == 0 {
                screens - 1
            } else {
                selected - 1
            };
            self.tui_state.selected_screen.select(Some(new_selected));
            self.set_current_screen(Some(new_selected));
        }
    }

    fn go_next(&mut self) {
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

        let constraints: Vec<Constraint> = screens
            .iter()
            .map(|_| Constraint::Ratio(1, screens.len() as u32))
            .collect();

        let layout = Layout::horizontal(constraints).split(area);

        for (i, screen) in screens.iter().enumerate() {
            let item_area = layout[i];
            let is_selected = Some(i) == selected_idx;
            let screen = format!(" [{}]{} ", i + 1, screen);

            let line =
                if is_selected {
                    Line::from(screen.set_style(
                        self.text_styles.highlight_text_style,
                    ))
                } else {
                    Line::from(screen)
                        .style(self.text_styles.primary_text_style)
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
