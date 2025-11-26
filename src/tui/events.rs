use std::{sync::mpsc::Receiver, time::Duration};

use anyhow::Result;
use crossterm::event::{
    Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Error,
    AppTick,
    Key(KeyEvent),
    Mouse(MouseEvent),

    ProviderResponse,
    ProviderError,
}

pub fn poll_event(
    rx: &Receiver<Event>,
    timeout: Duration,
) -> Result<Option<Event>> {
    if let Ok(event) = rx.try_recv() {
        return Ok(Some(event));
    }

    if crossterm::event::poll(timeout)? {
        let event = match crossterm::event::read()? {
            CrosstermEvent::Key(key)
                if key.kind == KeyEventKind::Press =>
            {
                Event::Key(key)
            }
            CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
            _ => return Ok(None),
        };

        return Ok(Some(event));
    }

    Ok(None)
}
