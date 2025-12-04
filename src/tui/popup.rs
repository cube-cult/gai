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
        let text_len = self.popup.content.len().min(60) as u16;
        let width = text_len + 4;
        let height = 4;

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

        Paragraph::new(self.popup.content.to_owned())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(self.text_styles.primary_text_style)
            .block(block)
            .render(centered_area, buf);
    }
}
