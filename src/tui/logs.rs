use ratatui::widgets::Widget;

use crate::{git::commit::GaiCommit, tui::events::Event};

use super::app::TextStyles;

pub struct LogScreen {
    gai_logs: Vec<GaiCommit>,
}

pub struct DiffScreenWidget<'screen> {
    pub screen: &'screen mut LogScreen,
    pub text_styles: &'screen TextStyles,
}

impl LogScreen {
    pub fn new(gai_logs: Vec<GaiCommit>) -> Self {
        Self { gai_logs }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(k) => match k.code {
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
    }
}
