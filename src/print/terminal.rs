use ratatui::{Terminal, TerminalOptions, prelude::Backend};

// i was thinking of geting rid of
// ratatui completely and sticking
// with dialoguer, console-rs
// but i was thinking of how to
// handle certain ui interactions and
// views, specifically when dealing with
// content that has to fill the screen
// we could create our own scrolling view
// but maybe using something existing
// would work well enough

/// initialize ratatui
pub fn start() -> anyhow::Result<Terminal<impl Backend>> {
    // inline view
    // this is will essentially fullscreen
    // if the content exceeds the term height
    // we should handle this per view
    // likely turning it into a scrolling view
    let (_width, term_height) = ratatui::crossterm::terminal::size()?;
    let height = term_height.min(20);

    let terminal = ratatui::init_with_options(TerminalOptions {
        viewport: ratatui::Viewport::Inline(height),
    });

    Ok(terminal)
}

/// restores terminal
pub fn stop() -> anyhow::Result<()> {
    ratatui::restore();

    // this is needed,
    // if we print anythign after
    // leaving ratatui
    // it gets tacked on
    println!();

    Ok(())
}
