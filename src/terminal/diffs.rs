use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    style::{Stylize, palette::tailwind},
    text::Line,
    widgets::{
        Block, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};

use crate::git::repo::{DiffType, GaiFile, GaiGit};

use super::{app::TextStyles, events::Event, utils::center};

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

    pub fn handle_event(&mut self, event: &Event, gai: &mut GaiGit) {
        match event {
            Event::Key(k) => match k.code {
                KeyCode::Char('k') => {
                    self.selected_file_state.select_previous();
                }
                KeyCode::Char('j') => {
                    self.selected_file_state.select_next();
                }
                KeyCode::Char('d') => {
                    self.remove_selected(gai);
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }
    }

    fn remove_selected(&mut self, gai: &mut GaiGit) {
        if let Some(selected) = self.selected_file_state.selected()
            && selected < gai.files.len()
        {
            gai.files.remove(selected);
            self.files.remove(selected);
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
            Constraint::Percentage(30),
            Constraint::Percentage(70),
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
                .title_top("Changes")
                .title_style(text_styles.secondary_text_style)
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
    let [diff_files_area, help_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let diff_files: Vec<ListItem> = files
        .iter()
        .map(|item| ListItem::new(item.path.to_owned()))
        .collect();

    let list = List::new(diff_files)
        .block(
            Block::bordered()
                .title_top("Files")
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
