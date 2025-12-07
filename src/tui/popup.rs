use std::sync::mpsc::Sender;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use super::{app::TextStyles, events::Event, utils::center};

#[derive(Clone, Debug)]
pub struct Popup {
    pub popup_type: PopupType,

    selected: ListState,

    input: String,
    char_index: usize,
}

#[derive(Clone, Debug)]
pub enum PopupType {
    Edit(u8, String),
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
        if matches!(popup_type, PopupType::Options(..)) {
            selected.select_first();
        }

        let (input, char_index) = match popup_type {
            PopupType::Edit(_, initial) => {
                let len = initial.chars().count();
                (initial.clone(), len)
            }
            _ => (String::new(), 0),
        };

        Self {
            popup_type: popup_type.to_owned(),
            selected,
            input,
            char_index,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    // ripped from https://ratatui.rs/examples/apps/user_input/
    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete =
                self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete =
                self.input.chars().skip(current_index);

            self.input = before_char_to_delete
                .chain(after_char_to_delete)
                .collect();
            self.move_cursor_left();
        }
    }

    fn delete_char_forward(&mut self) {
        let char_count = self.input.chars().count();
        if self.char_index < char_count {
            let current_index = self.char_index;

            let before_char_to_delete =
                self.input.chars().take(current_index);
            let after_char_to_delete =
                self.input.chars().skip(current_index + 1);

            self.input = before_char_to_delete
                .chain(after_char_to_delete)
                .collect();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_to_start(&mut self) {
        self.char_index = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.char_index = self.input.chars().count();
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        tx: &Sender<Event>,
    ) -> bool {
        match event {
            Event::Key(k) => match &self.popup_type {
                PopupType::Edit(layer, _) => {
                    let layer = *layer;
                    match k.code {
                        KeyCode::Esc => true,
                        KeyCode::Enter => {
                            tx.send(Event::PopUpReturn(
                                PopupResult::Text(
                                    layer,
                                    self.input.to_owned(),
                                ),
                            ))
                            .ok();
                            true
                        }
                        KeyCode::Char(c) => {
                            self.enter_char(c);
                            false
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                            false
                        }
                        KeyCode::Delete => {
                            self.delete_char_forward();
                            false
                        }
                        KeyCode::Left => {
                            self.move_cursor_left();
                            false
                        }
                        KeyCode::Right => {
                            self.move_cursor_right();
                            false
                        }
                        KeyCode::Home => {
                            self.move_cursor_to_start();
                            false
                        }
                        KeyCode::End => {
                            self.move_cursor_to_end();
                            false
                        }
                        _ => false,
                    }
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

    /// Get cursor position for rendering (used by Edit popup)
    pub fn cursor_position(&self) -> usize {
        self.char_index
    }

    /// Get current input text (used by Edit popup)
    pub fn input_text(&self) -> &str {
        &self.input
    }
}

pub struct PopupWidget<'popup> {
    pub popup: &'popup mut Popup,
    pub text_styles: &'popup TextStyles,
}

impl<'popup> PopupWidget<'popup> {
    /// Returns the cursor position if the popup needs cursor rendering
    pub fn cursor_position(&self, area: Rect) -> Option<Position> {
        match &self.popup.popup_type {
            PopupType::Edit(_, _) => {
                let input_len = self.popup.input.len();
                let hint = "Enter Save  Esc Cancel";

                let width =
                    (input_len.max(20).max(hint.len()) + 4) as u16;
                let height = 5u16;

                let centered_area = center(
                    area,
                    Constraint::Length(width),
                    Constraint::Length(height),
                );

                let block = Block::new().borders(Borders::ALL);
                let inner_area = block.inner(centered_area);

                let [input_area, _] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(inner_area);

                Some(Position::new(
                    input_area.x + self.popup.char_index as u16,
                    input_area.y,
                ))
            }
            _ => None,
        }
    }
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
            PopupType::Edit(_, _) => {
                render_edit(
                    area,
                    buf,
                    &self.popup.input,
                    self.text_styles,
                );
            }
        };
    }
}

fn render_edit(
    area: Rect,
    buf: &mut Buffer,
    input: &str,
    text_styles: &TextStyles,
) {
    let hint = "Enter Save  Esc Cancel";

    let input_len = input.len();
    let width = (input_len.max(20).max(hint.len()) + 4) as u16;
    let height = 5u16;

    let centered_area = center(
        area,
        Constraint::Length(width),
        Constraint::Length(height),
    );

    Clear.render(centered_area, buf);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(text_styles.border_style)
        .title(" Edit ");

    let inner_area = block.inner(centered_area);

    block.render(centered_area, buf);

    let [input_area, _spacer, hint_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    Paragraph::new(input)
        .style(text_styles.primary_text_style)
        .render(input_area, buf);

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
