use anyhow::Result;
use crossterm::event::{
    Event as CrosstermEvent, KeyEvent, KeyEventKind,
};

// ripped straight from
// https://ratatui.rs/templates/component/tui-rs/#additional-improvements

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Error,
    AppTick,
    Key(KeyEvent),
}

#[derive(Debug)]
pub struct EventHandler {}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        todo!()
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        todo!()
    }
}
