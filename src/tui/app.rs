use ratatui::Frame;

use crate::{
    ai::{
        request::Request,
        response::{Response, get_response},
    },
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::{
        TUIMode,
        tabs::{SelectedTab, TabContent, TabList},
        ui::UI,
    },
};

pub struct App {
    pub running: bool,
    pub cfg: Config,
    pub gai: GaiGit,
    pub ui: UI,

    pub tui_mode: TUIMode,
    pub request: Request,
    pub response: Option<Response>,
    pub is_loading: bool,
    pub applied_commits: bool,
}

/// various ui actions
pub enum Action {
    ScrollUp,
    ScrollDown,

    FocusLeft,
    FocusRight,

    Enter,

    NextTab,
    PreviousTab, // shift+tab(?)

    SendRequest,
    ApplyCommits,
    RemoveCurrentSelected,
    TruncateCurrentSelected,

    Quit,

    DiffTab,
    OpenAITab,
    ClaudeTab,
    GeminiTab,
}

impl App {
    pub fn new(
        request: Request,
        cfg: Config,
        gai: GaiGit,
        tui_mode: TUIMode,
        response: Option<Response>,
    ) -> Self {
        Self {
            running: true,
            cfg,
            gai,
            ui: UI::new(),
            tui_mode,
            request,
            response,
            is_loading: false,
            applied_commits: false,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {}

    pub fn on_tick(&mut self) {
        self.ui.throbber_state.calc_next();
    }

    pub async fn send_request(&mut self) {
        if self.is_loading {
            return;
        }

        let ai = &self.cfg.ai;
        let provider = ai.provider;
        let provider_cfg = ai
            .providers
            .get(&provider)
            .expect("somehow did not find provider config")
            .clone();

        // inexpensive clone?
        self.is_loading = true;

        let mut req = Request::default();
        req.build_prompt(&self.cfg, &self.gai);
        req.build_diffs_string(self.gai.get_file_diffs_as_str());
    }

    pub fn response_received(&mut self) {}

    pub fn apply_commits(&self) {}

    pub fn remove_selected(&mut self) {}

    pub fn truncate_selected(&mut self) {}
}
