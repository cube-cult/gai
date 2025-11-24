use crate::{
    ai::{request::Request, response::Response},
    config::Config,
    git::repo::GaiGit,
    tui::app::{Action, App},
};
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Flex, Rect},
    prelude::Layout,
    style::{Modifier, Style, palette::tailwind},
    widgets::ListState,
};
use throbber_widgets_tui::{Set, ThrobberState, WhichUse};
use tokio::sync::mpsc;

pub mod app;
pub mod commit;
pub mod events;
pub mod keys;
pub mod tabs;
pub mod ui;

use events::{Event, EventHandler};

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
pub enum TUIMode {
    // Only show diffs
    None,
    Commit,
    All,
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

pub async fn run_tui(
    req: Request,
    cfg: Config,
    gai: GaiGit,
    response: Option<Response>,
) -> Result<()> {
    let mut app = App::new(req, cfg, gai, TUIMode::None, response);

    let (resp_tx, mut resp_rx) = mpsc::channel(1);

    // todo remove this deprecated thingy
    // i doubt users are gonna use the tui
    // for auto requests
    if app.cfg.tui.auto_request {
        app.send_request(resp_tx.clone()).await;
    }

    let mut terminal = ratatui::init();
    let mut event_handler = EventHandler::new(81);
    let mut tui_state = TUIState::default();

    while app.running {
        terminal.draw(|f| app.run(f))?;

        tokio::select! {
            Some(event) = event_handler.next() => {
                handle_event(&mut app, event, resp_tx.clone()).await;
            }

            // i think receiving a response should be an event
            // that's handled by the handle_event
            Some(resp) = resp_rx.recv() => {
                app.response_received(resp);
            }
        }
    }

    event_handler.stop().await?;
    ratatui::restore();

    if app.applied_commits {
        println!("Applied Commits");
    }

    Ok(())
}

async fn handle_event(
    app: &mut App,
    event: Event,
    response_tx: mpsc::Sender<Response>,
) {
    match event {
        Event::Key(key) => {
            if let Some(action) = keys::get_tui_action(key) {
                handle_action(app, action, response_tx).await;
            }
        }
        Event::AppTick => {
            app.on_tick();
        }
        Event::Error => {
            // ignoring for now
            app.running = false;
        }
    }
}

async fn handle_action(
    app: &mut App,
    action: Action,
    response_tx: mpsc::Sender<Response>,
) {
    let ui = &mut app.ui;

    match action {
        Action::Quit => app.running = false,
        Action::ScrollUp => ui.scroll_up(),
        Action::ScrollDown => ui.scroll_down(),
        Action::FocusLeft => ui.focus_left(),
        Action::FocusRight => ui.focus_right(),
        Action::Enter => ui.enter_ui(),
        Action::DiffTab => ui.goto_tab(1),
        Action::OpenAITab => ui.goto_tab(2),
        Action::ClaudeTab => ui.goto_tab(3),
        Action::GeminiTab => ui.goto_tab(4),
        Action::SendRequest => {
            app.send_request(response_tx).await;
        }
        Action::ApplyCommits => {
            app.apply_commits();
            app.applied_commits = true;
            app.running = false;
        }
        Action::RemoveCurrentSelected => {
            app.remove_selected();
        }
        Action::TruncateCurrentSelected => {
            app.truncate_selected();
        }

        _ => {}
    }
}

pub fn center(
    area: Rect,
    horizontal: Constraint,
    vertical: Constraint,
) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] =
        Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
