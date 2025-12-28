use ratatui::Frame;

use crate::git::status::GitStatus;

use super::terminal;

pub fn print(
    status: &GitStatus,
    _compact: bool,
) -> anyhow::Result<()> {
    let height = (status.statuses.len() + 3).min(20) as u16;
    let mut terminal = terminal::start(height)?;

    let app = App::new(status);

    terminal.draw(|f| app.draw(f))?;

    terminal::stop()?;

    Ok(())
}

struct App {}

impl App {
    fn new(status: &GitStatus) -> Self {
        Self {}
    }

    fn draw(
        &self,
        frame: &mut Frame,
    ) {
    }
}
