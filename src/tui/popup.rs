use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Stylize, palette::tailwind},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::tui::events::Event;

use super::{app::TextStyles, utils::center};

#[derive(Clone, Debug)]
pub struct Popup {
    pub content: String,
    pub popup_type: PopupType,
}

#[derive(Clone, Debug)]
pub enum PopupType {
    Edit(String),
    Confirm,
}

impl Popup {
    pub fn new(content: &str, popup_type: &PopupType) -> Self {
        Self {
            content: content.to_owned(),
            popup_type: popup_type.to_owned(),
        }
    }
    pub fn handle_event(&self, event: &Event) -> bool {
        match event {
            Event::Key(k) => {
                matches!(k.code, KeyCode::Enter | KeyCode::Esc)
            }
            Event::AppTick => false,
            _ => false,
        }
    }
}

pub struct PopupWidget<'popup> {
    pub popup: &'popup Popup,
    pub text_styles: &'popup TextStyles,
}

impl<'popup> Widget for PopupWidget<'popup> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hint = match self.popup.popup_type {
            PopupType::Confirm => "Press Enter/Esc to Continue",
            _ => "",
        };

        let text_len = self.popup.content.len().min(60) as u16;
        let hint_len = hint.len() as u16;
        let width = text_len.max(hint_len) + 4;
        let height = if hint.is_empty() { 4 } else { 5 };

        let centered_area = center(
            area,
            Constraint::Length(width),
            Constraint::Length(height),
        );

        Clear.render(centered_area, buf);

        let block = Block::new()
            .borders(Borders::ALL)
            .bg(tailwind::BLACK)
            .fg(tailwind::WHITE)
            .border_style(self.text_styles.border_style);

        let text = if hint.is_empty() {
            self.popup.content.clone()
        } else {
            format!("{}\n\n{}", self.popup.content, hint)
        };

        Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(self.text_styles.primary_text_style)
            .block(block)
            .render(centered_area, buf);
    }
}
