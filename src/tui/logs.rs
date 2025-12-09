use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    style::{Stylize, palette::tailwind},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Padding,
        Paragraph, StatefulWidget, Widget, Wrap,
    },
};

use super::{app::TextStyles, events::Event};
use crate::{git::log::GaiLog, tui::utils::center};

pub struct LogScreen {
    gai_logs: Vec<GaiLog>,

    selected_log_state: ListState,
}

pub struct LogScreenWidget<'screen> {
    pub screen: &'screen mut LogScreen,
    pub text_styles: &'screen TextStyles,
}

impl LogScreen {
    pub fn new(gai_logs: Vec<GaiLog>) -> Self {
        let mut selected_log_state = ListState::default();
        if !gai_logs.is_empty() {
            selected_log_state.select_first();
        }

        Self {
            gai_logs,
            selected_log_state,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(k) => match k.code {
                KeyCode::Char('k') => {
                    self.selected_log_state.select_previous();
                }
                KeyCode::Char('j') => {
                    self.selected_log_state.select_next();
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }
    }
}

impl<'screen> Widget for LogScreenWidget<'screen> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]);

        let [diff_file_list_area, selected_diffs_area] =
            horizontal.areas(area);

        render_list(
            diff_file_list_area,
            buf,
            &mut self.screen.selected_log_state,
            &self.screen.gai_logs,
            self.text_styles,
        );

        if let Some(selected) =
            self.screen.selected_log_state.selected()
            && selected < self.screen.gai_logs.len()
        {
            let selected_log = &self.screen.gai_logs[selected];

            render_selected_log(
                selected_diffs_area,
                buf,
                selected_log,
                self.text_styles,
            );
        }
    }
}

fn render_selected_log(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    log: &GaiLog,
    text_styles: &TextStyles,
) {
    let mut lines: Vec<Line> = vec![
        Line::from("Commit").fg(tailwind::SLATE.c500).bold(),
        Line::from(log.commit_hash.to_owned())
            .fg(tailwind::AMBER.c400),
        Line::from(""),
        Line::from("Author").fg(tailwind::SLATE.c500).bold(),
        Line::from(log.author.to_owned()).fg(tailwind::CYAN.c300),
        Line::from(""),
        Line::from("Date").fg(tailwind::SLATE.c500).bold(),
        Line::from(log.date.to_owned()).fg(tailwind::SLATE.c300),
        Line::from(""),
    ];

    // since these aren't using the prefixtype enum
    // also merges should be highlighted no?
    if let Some(ref prefix) = log.prefix {
        let prefix_color = match prefix.to_lowercase().as_str() {
            "feat" | "feature" => tailwind::GREEN,
            "fix" => tailwind::RED,
            "refactor" => tailwind::BLUE,
            "style" => tailwind::PURPLE,
            "test" | "tests" => tailwind::YELLOW,
            "docs" | "doc" => tailwind::CYAN,
            "build" => tailwind::ORANGE,
            "ci" => tailwind::INDIGO,
            "ops" => tailwind::PINK,
            "chore" => tailwind::SLATE,
            "merge" => tailwind::VIOLET,
            "revert" => tailwind::ROSE,
            "perf" => tailwind::AMBER,
            _ => tailwind::GRAY,
        };

        let breaking_str = if log.breaking { "!" } else { "" };
        let scope_str = if let Some(ref scope) = log.scope {
            format!("({})", scope)
        } else {
            String::new()
        };

        lines.push(Line::from(vec![
            Span::styled(
                prefix.to_owned(),
                ratatui::style::Style::default()
                    .fg(prefix_color.c200)
                    .bg(prefix_color.c900)
                    .bold(),
            ),
            Span::styled(
                scope_str,
                ratatui::style::Style::default()
                    .fg(tailwind::SLATE.c400)
                    .italic(),
            ),
            Span::styled(
                breaking_str,
                ratatui::style::Style::default()
                    .fg(tailwind::RED.c500)
                    .bold(),
            ),
        ]));
        lines.push(Line::from(""));

        if let Some(ref header) = log.header {
            lines.push(
                Line::from("Header").fg(tailwind::SLATE.c500).bold(),
            );
            lines.push(
                Line::from(header.to_owned())
                    .fg(tailwind::SLATE.c100),
            );
            lines.push(Line::from(""));
        }

        if let Some(ref body) = log.body {
            lines.push(
                Line::from("Body").fg(tailwind::SLATE.c500).bold(),
            );
            for body_line in body.lines() {
                lines.push(
                    Line::from(body_line).fg(tailwind::SLATE.c300),
                );
            }
            lines.push(Line::from(""));
        }
    } else if let Some(ref message) = log.message {
        lines.push(
            Line::from("Message").fg(tailwind::SLATE.c500).bold(),
        );
        for msg_line in message.lines() {
            lines.push(Line::from(msg_line).fg(tailwind::SLATE.c100));
        }
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title("Commit Info")
                .title_style(text_styles.secondary_text_style)
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(text_styles.border_style),
        )
        .wrap(Wrap { trim: false });

    paragraph.render(area, buf);
}

fn render_list(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut ListState,
    logs: &[GaiLog],
    text_styles: &TextStyles,
) {
    let [diff_files_area, help_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let logs: Vec<ListItem> = logs
        .iter()
        .map(|item| {
            //todo use span::styled
            let line = Line::from(vec![
                Span::raw(&item.author),
                Span::raw(" "),
                Span::raw(&item.date),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(logs)
        .block(
            Block::bordered()
                .title_top("Commits")
                .title_style(text_styles.secondary_text_style)
                .border_style(text_styles.border_style),
        )
        .style(text_styles.primary_text_style)
        .highlight_style(text_styles.highlight_text_style);

    StatefulWidget::render(list, diff_files_area, buf, state);

    let text = Line::styled(
        "↑|k Previous  ↓|j Next",
        text_styles.secondary_text_style,
    );

    let text_area = center(
        help_area,
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );

    text.render(text_area, buf);
}
