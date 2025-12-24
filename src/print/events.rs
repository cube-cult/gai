use ratatui::crossterm::event::{
    self, Event as CrosstermEvent, KeyEvent, KeyEventKind,
};
use std::time::Duration;

/// simplified version of the original
/// event handlign
/// this time its blocking,
/// just polls for key events
/// no channels here
pub fn poll_key() -> Option<KeyEvent> {
    let timeout = Duration::from_millis(200);

    if event::poll(timeout).ok()?
        && let Ok(CrosstermEvent::Key(key)) = event::read()
        && key.kind == KeyEventKind::Press
    {
        return Some(key);
    }
    None
}
