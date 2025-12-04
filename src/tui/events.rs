use anyhow::Result;
use crossterm::event::{
    Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::{ai::response::ResponseCommit, tui::popup::PopupType};

#[derive(Clone, Debug)]
pub enum Event {
    Error,
    AppTick,
    Key(KeyEvent),
    Mouse(MouseEvent),

    ProviderResponse(Vec<ResponseCommit>),
    ProviderError(String),

    PopUp(String, PopupType),
}

pub fn poll_event(
    rx: &Receiver<Event>,
    timeout: Duration,
) -> Result<Event> {
    if let Ok(event) = rx.try_recv() {
        return Ok(event);
    }

    if crossterm::event::poll(timeout)? {
        let event = match crossterm::event::read()? {
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
