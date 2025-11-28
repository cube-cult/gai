use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Stylize, palette::tailwind},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, ListState, Padding,
        Paragraph, StatefulWidget, Widget, Wrap,
    },
};

use crate::{
    git::repo::{DiffType, GaiFile},
    tui::{events::Event, utils::center},
};

use super::app::TextStyles;

pub struct DiffScreen {
    pub files: Vec<GaiFile>,
    pub selected_file_state: ListState,
}

pub struct DiffScreenWidget<'screen> {
    pub screen: &'screen mut DiffScreen,
    pub text_styles: &'screen TextStyles,
}

impl DiffScreen {
    pub fn new(files: &[GaiFile]) -> Self {
        let mut selected_file_state = ListState::default();
        selected_file_state.select_first();

        Self {
            files: files.to_vec(),
            selected_file_state,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(k) => match k.code {
                KeyCode::Char('k') => {
                    self.selected_file_state.select_previous();
                }
                KeyCode::Char('j') => {
                    self.selected_file_state.select_next();
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }
    }
}

impl<'screen> Widget for DiffScreenWidget<'screen> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]);

        let [diff_file_list_area, selected_diffs_area] =
            horizontal.areas(area);

        render_list(
            diff_file_list_area,
            buf,
            &mut self.screen.selected_file_state,
            &self.screen.files,
            self.text_styles,
        );

        if let Some(selected) =
            self.screen.selected_file_state.selected()
            && selected < self.screen.files.len()
        {
            let selected_diff = &self.screen.files[selected];

            render_selected_diff(
                selected_diffs_area,
                buf,
                selected_diff,
                self.text_styles,
            );
        }
    }
}

fn render_selected_diff(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    selected_diff: &GaiFile,
    text_styles: &TextStyles,
) {
    let mut lines: Vec<Line> = Vec::new();

    for hunk in &selected_diff.hunks {
        lines.push(
            Line::from(hunk.header.clone()).bg(tailwind::BLUE.c900),
        );

        for line_diff in &hunk.line_diffs {
            let styled_line = match line_diff.diff_type {
                DiffType::Additions => {
                    Line::from(format!("+{}", line_diff.content))
                        .bg(tailwind::GREEN.c950)
                }
                DiffType::Deletions => {
                    Line::from(format!("-{}", line_diff.content))
                        .bg(tailwind::RED.c950)
                }
                DiffType::Unchanged => {
                    Line::from(format!(" {}", line_diff.content))
                }
            };
            lines.push(styled_line);
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::bordered()
                .padding(Padding::horizontal(1))
                .borders(Borders::LEFT)
                .border_style(text_styles.border_style),
        )
        .wrap(Wrap { trim: false });

    paragraph.render(area, buf);
}

fn render_list(
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
    state: &mut ListState,
    files: &[GaiFile],
    text_styles: &TextStyles,
) {
    let diff_files: Vec<ListItem> = files
        .iter()
        .map(|item| ListItem::new(item.path.to_owned()))
        .collect();

    let list = List::new(diff_files)
        .highlight_style(text_styles.highlight_text_style);

    let total_height = list.len() as u16;

    let centered_diff_list_area = center(
        area,
        Constraint::Length(area.width),
        Constraint::Length(total_height),
    );

    StatefulWidget::render(list, centered_diff_list_area, buf, state);
}
