use std::sync::mpsc::Sender;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use tui_textarea::TextArea;

use super::{app::TextStyles, events::Event, utils::center};

#[derive(Clone, Debug)]
pub struct Popup {
    pub popup_type: PopupType,

    selected: ListState,
    input: String,

    //input keys for textarea
    //doing this so we dont have
    //to specify a lifetime for TextArea
    keycode: Option<KeyEvent>,
}

#[derive(Clone, Debug)]
pub enum PopupType {
    /// layer, single line?, initial text
    Edit(u8, bool, String),
    Options(u8, Vec<String>),
    Confirm(String),
}

#[derive(Clone, Debug)]
pub enum PopupResult {
    Confirmed,
    Text(u8, String),
    SelectedChoice(u8, usize),
}

impl Popup {
    pub fn new(popup_type: &PopupType) -> Self {
        let mut selected = ListState::default();
        let mut initial_text = String::new();

        match popup_type {
            PopupType::Edit(_, _, initial) => {
                initial_text = initial.to_owned();
            }
            PopupType::Options(_, _) => selected.select_first(),
            _ => {}
        };

        Self {
            popup_type: popup_type.to_owned(),
            selected,
            input: initial_text,
            keycode: None,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        tx: &Sender<Event>,
    ) -> bool {
        match event {
            Event::Key(k) => match &self.popup_type {
                PopupType::Edit(layer, _, _) => {
                    self.handle_editor(tx, *k, *layer)
                }
                PopupType::Options(layer, _) => {
                    let layer = *layer;
                    match k.code {
                        KeyCode::Esc => true,
                        KeyCode::Enter => {
                            if let Some(selected) =
                                self.selected.selected()
                            {
                                tx.send(Event::PopUpReturn(
                                    PopupResult::SelectedChoice(
                                        layer, selected,
                                    ),
                                ))
                                .ok();
                            }
                            true
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.selected.select_next();
                            false
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.selected.select_previous();
                            false
                        }
                        _ => false,
                    }
                }
                PopupType::Confirm(_) => {
                    matches!(k.code, KeyCode::Esc | KeyCode::Enter)
                }
            },
            Event::AppTick => false,
            _ => false,
        }
    }

    fn handle_editor(
        &mut self,
        tx: &Sender<Event>,
        key: KeyEvent,
        layer: u8,
    ) -> bool {
        match key.code {
            KeyCode::Esc => true,
            KeyCode::Enter => {
                tx.send(Event::PopUpReturn(PopupResult::Text(
                    layer,
                    self.input.to_owned(),
                )))
                .ok();
                true
            }
            _ => {
                self.keycode = Some(key);
                false
            }
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
            PopupType::Options(_, options) => {
                render_options(
                    area,
                    buf,
                    options.to_owned(),
                    &mut self.popup.selected,
                    self.text_styles,
                );
            }
            PopupType::Edit(_, single_line, _) => {
                render_edit(
                    area,
                    buf,
                    self.popup.keycode.take(),
                    &mut self.popup.input,
                    self.text_styles,
                    *single_line,
                );
            }
        };
    }
}

fn render_edit(
    area: Rect,
    buf: &mut Buffer,
    key_event: Option<KeyEvent>,
    input: &mut String,
    text_styles: &TextStyles,
    single_line: bool,
) {
    let hint = "Enter Save  Esc Cancel";

    let width = 50.min(area.width.saturating_sub(4));
    let height = if single_line { 5 } else { 10 };

    let centered_area = center(
        area,
        Constraint::Length(width),
        Constraint::Length(height),
    );

    Clear.render(centered_area, buf);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(text_styles.border_style)
        .title("Edit")
        .title_style(text_styles.primary_text_style);

    let inner_area = block.inner(centered_area);

    block.render(centered_area, buf);

    let [text_area, hint_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    let mut text_input = TextArea::default();
    text_input.insert_str(&input);
    text_input.move_cursor(tui_textarea::CursorMove::End);

    if let Some(key) = key_event {
        text_input.input(key);
        *input = text_input.lines().join("\n");
    }

    text_input.render(text_area, buf);

    Paragraph::new(hint)
        .alignment(Alignment::Center)
        .style(text_styles.secondary_text_style)
        .render(hint_area, buf);
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
