use std::sync::mpsc::Sender;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use crate::tui::events::Event;

use super::{app::TextStyles, utils::center};

#[derive(Clone, Debug)]
pub struct Popup {
    pub popup_type: PopupType,

    selected: ListState,
}

#[derive(Clone, Debug)]
pub enum PopupType {
    Edit(String),
    Options(Vec<String>),
    Confirm(String),
}

#[derive(Clone, Debug)]
pub enum PopupResult {
    Confirmed,
    Text(String),
    SelectedChoice(usize),
}

impl Popup {
    pub fn new(popup_type: &PopupType) -> Self {
        let mut selected = ListState::default();
        if matches!(popup_type, PopupType::Options(_)) {
            selected.select_first();
        }

        Self {
            popup_type: popup_type.to_owned(),
            selected,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        tx: &Sender<Event>,
    ) -> bool {
        match event {
            Event::Key(k) => match k.code {
                KeyCode::Esc => true,
                KeyCode::Enter => match self.popup_type {
                    PopupType::Confirm(_) => true,
                    PopupType::Options(_) => {
                        if let Some(selected) =
                            self.selected.selected()
                        {
                            tx.send(Event::PopUpReturn(
                                PopupResult::SelectedChoice(selected),
                            ))
                            .ok();
                        }
                        true
                    }
                    _ => true,
                },
                KeyCode::Char('j') | KeyCode::Down => {
                    if matches!(
                        self.popup_type,
                        PopupType::Options(_)
                    ) {
                        self.selected.select_next();
                    }
                    false
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if matches!(
                        self.popup_type,
                        PopupType::Options(_)
                    ) {
                        self.selected.select_previous();
                    }
                    false
                }

                _ => false,
            },
            Event::AppTick => false,
            _ => false,
        }
    }
}

pub struct PopupWidget<'popup> {
    pub popup: &'popup mut Popup,
    pub text_styles: &'popup TextStyles,
}

impl<'popup> Widget for PopupWidget<'popup> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.popup.popup_type {
            PopupType::Confirm(message) => {
                render_confirm(area, buf, message, self.text_styles);
            }
            PopupType::Options(options) => {
                render_options(
                    area,
                    buf,
                    options.to_owned(),
                    &mut self.popup.selected,
                    self.text_styles,
                );
            }
            _ => {}
        };
    }
}

fn render_options(
    area: Rect,
    buf: &mut Buffer,
    options: Vec<String>,
    state: &mut ListState,
    text_styles: &TextStyles,
) {
    let longest_option =
        options.iter().map(|s| s.len()).max().unwrap_or(0);

    let hint = "↑/k Up  ↓/j Down  Enter Select  Esc Cancel";

    let width = (longest_option.max(hint.len()) + 4) as u16;
    let height = (options.len() + 4) as u16;

    let centered_area = center(
        area,
        Constraint::Length(width),
        Constraint::Length(height),
    );

    Clear.render(centered_area, buf);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(text_styles.border_style);

    let inner_area = block.inner(centered_area);

    block.render(centered_area, buf);

    let [list_area, hint_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    let items: Vec<ListItem> = options
        .iter()
        .map(|opt| ListItem::new(opt.as_str()))
        .collect();

    let list = List::new(items)
        .style(text_styles.primary_text_style)
        .highlight_style(text_styles.highlight_text_style);

    StatefulWidget::render(list, list_area, buf, state);

    Paragraph::new(hint)
        .alignment(Alignment::Center)
        .style(text_styles.secondary_text_style)
        .render(hint_area, buf);
}

fn render_confirm(
    area: Rect,
    buf: &mut Buffer,
    message: &str,
    text_styles: &TextStyles,
) {
    let hint = "Press Enter/Esc to Continue";

    let hint_len = hint.len() as u16;
    let text_len = message.len().min(60) as u16;

    let width = text_len.max(hint_len) + 4;
    let height = 5;

    let centered_area = center(
        area,
        Constraint::Length(width),
        Constraint::Length(height),
    );

    Clear.render(centered_area, buf);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(text_styles.border_style);

    let inner_area = block.inner(centered_area);

    block.render(centered_area, buf);

    let [text_area, hint_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    Paragraph::new(message)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(text_styles.primary_text_style)
        .render(text_area, buf);

    Paragraph::new(hint)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(text_styles.secondary_text_style)
        .render(hint_area, buf);
}
