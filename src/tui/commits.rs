use std::{sync::mpsc::Sender, thread};

use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    style::{Stylize, palette::tailwind},
    text::{Line, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Padding,
        Paragraph, StatefulWidget, Widget, Wrap,
    },
};
use strum::IntoEnumIterator;
use throbber_widgets_tui::{Throbber, ThrobberState};

use super::{
    app::{TextStyles, ThrobberStyles},
    events::Event,
    popup::{PopupResult, PopupType},
    utils::center,
};
use crate::{
    git::{commit::GaiCommit, repo::GaiGit},
    providers::{
        provider::{ProviderKind::Gai, extract_from_provider},
        request::Request,
        schema::{PrefixType, ResponseCommit},
    },
    settings::{CommitSettings, Settings},
};

pub struct CommitScreen {
    pub provider: String,
    pub model: String,

    pub capitalize_prefix: bool,
    pub include_scope: bool,

    pub commits: Vec<ResponseCommit>,
    pub error: Option<String>,

    pub request_sent: bool,
    pub is_waiting: bool,
    pub is_error: bool,

    pub selected_commit_state: ListState,
    pub throbber_state: ThrobberState,
}

#[repr(u8)]
pub enum CommitLayers {
    Initial = 0,
    Prefixes = 1,
    Header = 2,
    Body = 3,
    Scope = 4,
    Breaking = 5,
}

pub struct CommitScreenWidget<'screen> {
    pub screen: &'screen mut CommitScreen,
    pub throbber_styles: &'screen ThrobberStyles,
    pub text_styles: &'screen TextStyles,
}

impl CommitScreen {
    pub fn new(
        provider: &str,
        model: &str,
        commit_settings: &CommitSettings,
    ) -> Self {
        let selected_commit_state = ListState::default();

        Self {
            provider: provider.to_owned(),
            model: model.to_owned(),
            capitalize_prefix: commit_settings.capitalize_prefix,
            include_scope: commit_settings.include_scope,
            commits: Vec::new(),
            is_error: false,
            error: None,
            request_sent: false,
            is_waiting: false,
            selected_commit_state,
            throbber_state: ThrobberState::default(),
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        tx: &Sender<Event>,
        cfg: &Settings,
        gai: &GaiGit,
    ) {
        match event {
            Event::AppTick => {
                if self.is_waiting {
                    self.throbber_state.calc_next();
                }
            }
            Event::ProviderResponse(commits) => {
                self.is_waiting = false;
                self.is_error = false;
                if !commits.is_empty() {
                    self.selected_commit_state.select_first();
                    self.commits = commits.to_owned();
                }
            }
            Event::ProviderError(error) => {
                self.is_waiting = false;
                self.is_error = true;
                self.error = Some(error.to_owned());
            }
            Event::PopUpReturn(val) => match val {
                PopupResult::SelectedChoice(layer, choice) => {
                    let layer = *layer;
                    let choice = *choice;

                    // asuming ones already selected
                    // this wouldnt be proc'd otherwise
                    let selected_commit = self
                        .selected_commit_state
                        .selected()
                        .expect("somehow no commit selected");

                    if layer == CommitLayers::Initial as u8 {
                        if choice == 0 {
                            let prefix_opts: Vec<String> =
                                PrefixType::iter()
                                    .map(|prefix| {
                                        format!("{:?}", prefix)
                                    })
                                    .collect();
                            let event =
                                Event::PopUp(PopupType::Options(
                                    CommitLayers::Prefixes as u8,
                                    prefix_opts,
                                ));

                            tx.send(event).ok();
                        } else if choice == 1 {
                            //header
                            let text = self.commits[selected_commit]
                                .header
                                .to_owned();

                            let event =
                                Event::PopUp(PopupType::Edit(
                                    CommitLayers::Header as u8,
                                    true,
                                    text,
                                ));

                            tx.send(event).ok();
                        } else if choice == 2 {
                            //body
                            let text = self.commits[selected_commit]
                                .body
                                .to_owned();

                            let event =
                                Event::PopUp(PopupType::Edit(
                                    CommitLayers::Body as u8,
                                    false,
                                    text,
                                ));

                            tx.send(event).ok();
                        } else if choice == 3 {
                            //scope
                            let text = self.commits[selected_commit]
                                .scope
                                .to_owned();
                            let event =
                                Event::PopUp(PopupType::Edit(
                                    CommitLayers::Scope as u8,
                                    true,
                                    text,
                                ));
                            tx.send(event).ok();
                        } else if choice == 4 {
                            //breaking
                            let event =
                                Event::PopUp(PopupType::Options(
                                    CommitLayers::Breaking as u8,
                                    vec![
                                        "Breaking Change".to_owned(),
                                        "Not".to_owned(),
                                    ],
                                ));
                            tx.send(event).ok();
                        }
                    } else if layer == CommitLayers::Prefixes as u8 {
                        let new_prefix =
                            PrefixType::iter().nth(choice).expect(
                                "somehow couldn't find prefixtype",
                            );

                        self.commits[selected_commit].prefix =
                            new_prefix;
                    } else if layer == CommitLayers::Breaking as u8 {
                        self.commits[selected_commit].breaking =
                            choice == 0;
                    }
                }
                PopupResult::Text(layer, result) => {
                    let selected_commit = self
                        .selected_commit_state
                        .selected()
                        .expect("somehow no commit selected");

                    match *layer {
                        val if val == CommitLayers::Header as u8 => {
                            self.commits[selected_commit].header =
                                result.to_owned();
                        }
                        val if val == CommitLayers::Body as u8 => {
                            self.commits[selected_commit].body =
                                result.to_owned();
                        }
                        val if val == CommitLayers::Scope as u8 => {
                            self.commits[selected_commit].scope =
                                result.to_owned();
                        }
                        _ => {}
                    }
                }
                PopupResult::Confirmed => {}
            },
            Event::Key(k) => match k.code {
                KeyCode::Enter => {
                    if self.selected_commit_state.selected().is_some()
                    {
                        self.edit_commit(tx);
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if !self.is_waiting {
                        self.send_request(tx.clone(), cfg, gai);
                    }
                }
                KeyCode::Char('k') => {
                    self.selected_commit_state.select_previous();
                }
                KeyCode::Char('j') => {
                    self.selected_commit_state.select_next();
                }
                KeyCode::Char('x') => {
                    self.apply_commits(tx, cfg, gai);
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }
    }

    fn send_request(
        &mut self,
        tx: Sender<Event>,
        cfg: &Settings,
        gai: &GaiGit,
    ) {
        self.request_sent = true;
        self.is_waiting = true;

        let mut request = Request::default();
        request.build_prompt(cfg, gai);
        request.build_diffs_string(gai.get_file_diffs_as_str());

        thread::spawn(move || {
            let response = extract_from_provider(
                &Gai,
                &request.prompt,
                &request.diffs,
            );

            match response {
                Ok(res) => {
                    tx.send(Event::ProviderResponse(res.commits))
                        .ok();
                }
                Err(e) => {
                    tx.send(Event::ProviderError(e.to_string())).ok();
                }
            }
        });
    }

    fn apply_commits(
        &self,
        tx: &Sender<Event>,
        cfg: &Settings,
        gai: &GaiGit,
    ) {
        if self.commits.is_empty() {
            return;
        }

        let commits: Vec<GaiCommit> = self
            .commits
            .iter()
            .map(|r_c| {
                GaiCommit::from_response(
                    r_c,
                    cfg.commit.capitalize_prefix,
                    cfg.commit.include_scope,
                )
            })
            .collect();

        match gai.apply_commits(&commits) {
            Ok(_) => tx
                .send(Event::PopUp(PopupType::Confirm(
                    "Successfully Applied Commits".to_owned(),
                )))
                .ok(),
            Err(e) => tx
                .send(Event::PopUp(PopupType::Confirm(e.to_string())))
                .ok(),
        };
    }

    fn edit_commit(&self, tx: &Sender<Event>) {
        if self.commits.is_empty() {
            return;
        }

        if let Some(selected) = self.selected_commit_state.selected()
            && selected < self.commits.len()
        {
            tx.send(Event::PopUp(PopupType::Options(
                CommitLayers::Initial as u8,
                vec![
                    "Prefix".to_owned(),
                    "Header".to_owned(),
                    "Body".to_owned(),
                    "Scope".to_owned(),
                    "Breaking".to_owned(),
                ],
            )))
            .ok();
        }
    }
}

impl<'screen> Widget for CommitScreenWidget<'screen> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        if !self.screen.request_sent {
            render_send_prompt(area, buf, self.text_styles);
        }

        if self.screen.is_waiting {
            render_loading(
                area,
                buf,
                &self.screen.provider,
                &self.screen.model,
                self.text_styles,
                self.throbber_styles,
                &mut self.screen.throbber_state,
            );
        }

        if let Some(error) = &self.screen.error
            && !self.screen.is_waiting
            && self.screen.request_sent
            && self.screen.is_error
        {
            render_error(area, buf, error, self.text_styles);
        }

        if !self.screen.commits.is_empty()
            && !self.screen.is_waiting
            && self.screen.request_sent
        {
            render_commits(
                area,
                buf,
                &mut self.screen.selected_commit_state,
                self.screen.capitalize_prefix,
                self.screen.include_scope,
                &self.screen.commits,
                self.text_styles,
            );
        }
    }
}

fn render_error(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    error: &str,
    text_styles: &TextStyles,
) {
    let text = Text::styled(error, text_styles.primary_text_style);

    let text_area = center(
        area,
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );

    text.render(text_area, buf);
}

fn render_commits(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut ListState,
    capitalize_prefix: bool,
    include_scope: bool,
    commits: &[ResponseCommit],
    text_styles: &TextStyles,
) {
    let horizontal = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(75),
    ]);

    let [commit_list_area, commit_message_area] =
        horizontal.areas(area);

    let mut commit_list = Vec::new();

    for commit in commits {
        let prefix = commit
            .get_commit_prefix(capitalize_prefix, include_scope);
        commit_list.push(prefix);
    }

    let items: Vec<ListItem> = commit_list
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title("Commits")
                .title_style(text_styles.secondary_text_style)
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .style(text_styles.border_style),
        )
        .style(text_styles.primary_text_style)
        .highlight_style(text_styles.highlight_text_style);

    StatefulWidget::render(list, commit_list_area, buf, state);

    if let Some(selected) = state.selected()
        && selected < commits.len()
    {
        let commit = commits[selected].to_owned();
        render_commit_message(
            commit_message_area,
            buf,
            &commit,
            text_styles,
        );
    }
}

fn render_commit_message(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    commit: &ResponseCommit,
    text_styles: &TextStyles,
) {
    let mut lines: Vec<Line> = Vec::new();

    let prefix_color = match commit.prefix {
        PrefixType::Feat => tailwind::GREEN,
        PrefixType::Fix => tailwind::RED,
        PrefixType::Refactor => tailwind::BLUE,
        PrefixType::Style => tailwind::PURPLE,
        PrefixType::Test => tailwind::YELLOW,
        PrefixType::Docs => tailwind::CYAN,
        PrefixType::Build => tailwind::ORANGE,
        PrefixType::CI => tailwind::INDIGO,
        PrefixType::Ops => tailwind::PINK,
        PrefixType::Chore => tailwind::SLATE,
        PrefixType::Merge => tailwind::VIOLET,
        PrefixType::Revert => tailwind::ROSE,
    };

    let prefix_str = format!("{:?}", commit.prefix).to_lowercase();
    let breaking_str = if commit.breaking { "!" } else { "" };
    let scope_str = if !commit.scope.is_empty() {
        format!("({})", commit.scope)
    } else {
        String::new()
    };

    lines.push(Line::from(vec![
        prefix_str
            .fg(prefix_color.c200)
            .bg(prefix_color.c900)
            .bold(),
        scope_str.fg(tailwind::SLATE.c400).italic(),
        breaking_str.fg(tailwind::RED.c500).bold(),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from("Header").fg(tailwind::SLATE.c500).bold());
    lines.push(
        Line::from(commit.header.clone()).fg(tailwind::SLATE.c100),
    );
    lines.push(Line::from(""));

    if !commit.body.is_empty() {
        lines
            .push(Line::from("Body").fg(tailwind::SLATE.c500).bold());
        for body_line in commit.body.lines() {
            lines
                .push(Line::from(body_line).fg(tailwind::SLATE.c300));
        }
        lines.push(Line::from(""));
    }

    if !commit.files.is_empty() {
        lines.push(
            Line::from("Files").fg(tailwind::SLATE.c500).bold(),
        );
        for file in &commit.files {
            lines.push(
                Line::from(format!("  • {}", file))
                    .fg(tailwind::CYAN.c400),
            );
        }
        lines.push(Line::from(""));
    }

    if !commit.hunk_ids.is_empty() {
        lines.push(
            Line::from("Hunks").fg(tailwind::SLATE.c500).bold(),
        );
        for hunk_id in &commit.hunk_ids {
            lines.push(
                Line::from(format!("  • {}", hunk_id))
                    .fg(tailwind::AMBER.c400),
            );
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title("Commit Info")
                .title_style(text_styles.secondary_text_style)
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        )
        .style(text_styles.border_style)
        .wrap(Wrap { trim: false });

    paragraph.render(area, buf);
}

fn render_send_prompt(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    text_styles: &TextStyles,
) {
    let text = Text::styled(
        "Press 'P' to Generate Commits",
        text_styles.primary_text_style,
    );

    let text_area = center(
        area,
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );

    text.render(text_area, buf);
}

fn render_loading(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    provider: &str,
    model: &str,
    text_styles: &TextStyles,
    throbber_styles: &ThrobberStyles,
    throbber_state: &mut ThrobberState,
) {
    let message = format!(
        "Awaiting Response from {} using {}...",
        provider, model
    );

    let centered_load_area = center(
        area,
        Constraint::Length(message.len() as u16 + 2),
        Constraint::Length(1),
    );

    let throbber = Throbber::default()
        .label(message)
        .style(text_styles.secondary_text_style)
        .throbber_style(throbber_styles.throbber_style)
        .throbber_set(throbber_styles.throbber_set.to_owned())
        .use_type(throbber_styles.throbber_type.to_owned());

    StatefulWidget::render(
        throbber,
        centered_load_area,
        buf,
        throbber_state,
    );
}
