use anyhow::Result;
use ratatui::crossterm::event::{
    Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::{
    ai::schema::ResponseCommit,
    tui::popup::{PopupResult, PopupType},
};

#[derive(Clone, Debug)]
pub enum Event {
    Error,
    AppTick,
    Key(KeyEvent),
    Mouse(MouseEvent),

    ProviderResponse(Vec<ResponseCommit>),
    ProviderError(String),

    PopUp(PopupType),
    PopUpReturn(PopupResult),
}

pub fn poll_event(
    rx: &Receiver<Event>,
    timeout: Duration,
) -> Result<Event> {
    if let Ok(event) = rx.try_recv() {
        return Ok(event);
    }

    if ratatui::crossterm::event::poll(timeout)? {
        let event = match ratatui::crossterm::event::read()? {
            CrosstermEvent::Key(key)
                if key.kind == KeyEventKind::Press =>
            {
                Event::Key(key)
            }
            CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
            _ => return Ok(Event::AppTick),
        };

        return Ok(event);
    }

    Ok(Event::AppTick)
}
