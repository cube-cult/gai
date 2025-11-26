use ratatui::{
    layout::{Constraint, Layout},
    style::{Stylize, palette::tailwind},
    text::{Line, Text},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use throbber_widgets_tui::Throbber;

use super::{app::TUIState, utils::center};

use crate::ai::response::{PrefixType, ResponseCommit};

pub struct CommitScreen {
    pub provider: String,
    pub model: String,

    pub capitalize_prefix: bool,
    pub include_scope: bool,

    pub commits: Vec<ResponseCommit>,

    pub request_sent: bool,
    pub is_waiting: bool,
}

impl StatefulWidget for CommitScreen {
    type State = TUIState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if !self.request_sent {
            render_send_prompt(area, buf, state);
        }

        if self.is_waiting {
            render_still_loading(
                area,
                buf,
                state,
                &self.provider,
                &self.model,
            );
        }

        if !self.commits.is_empty()
            && !self.is_waiting
            && self.request_sent
        {
            render_commits(
                area,
                buf,
                state,
                self.capitalize_prefix,
                self.include_scope,
                &self.commits,
            );
        }
    }
}

fn render_commits(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut TUIState,
    capitalize_prefix: bool,
    include_scope: bool,
    commits: &[ResponseCommit],
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
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(state.highlight_text_style);

    StatefulWidget::render(
        list,
        commit_list_area,
        buf,
        &mut state.selected_state,
    );

    if let Some(selected) = state.selected_state.selected()
        && selected < commits.len()
    {
        let commit = commits[selected].to_owned();
        render_commit_message(commit_message_area, buf, &commit);
    }
}

fn render_commit_message(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    commit: &ResponseCommit,
) {
    let mut lines: Vec<Line> = Vec::new();

    let prefix_color = match commit.message.prefix {
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

    let prefix_str =
        format!("{:?}", commit.message.prefix).to_lowercase();
    let breaking_str = if commit.message.breaking { "!" } else { "" };
    let scope_str = if !commit.message.scope.is_empty() {
        format!("({})", commit.message.scope)
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
        Line::from(commit.message.header.clone())
            .fg(tailwind::SLATE.c100),
    );
    lines.push(Line::from(""));

    if !commit.message.body.is_empty() {
        lines
            .push(Line::from("Body").fg(tailwind::SLATE.c500).bold());
        for body_line in commit.message.body.lines() {
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
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: false });

    paragraph.render(area, buf);
}

fn render_send_prompt(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut TUIState,
) {
    let text = Text::styled(
        "Press 'P' to Generate Commits",
        state.primary_text_style,
    );

    let text_area = center(
        area,
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );

    text.render(text_area, buf);
}

fn render_still_loading(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut TUIState,
    provider: &str,
    model: &str,
) {
    let message = format!(
        "Awaiting Response from {} using {}...",
        provider, model
    );

    let throbber = Throbber::default()
        .label(message)
        .style(state.secondary_text_style)
        .throbber_style(state.throbber_style)
        .throbber_set(state.throbber_set.to_owned())
        .use_type(state.throbber_type.to_owned());

    StatefulWidget::render(
        throbber,
        area,
        buf,
        &mut state.throbber_state,
    );
}
