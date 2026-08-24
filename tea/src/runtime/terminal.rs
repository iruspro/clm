//! Taking the terminal over, and handing it back.

use std::io::{self, Stdout};
use std::panic;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::{execute, terminal};

use crate::core::Result;

/// The terminal the runtime draws through.
pub(crate) type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

/// Opens the terminal, without yet claiming the screen.
pub(crate) fn open() -> Result<Terminal> {
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

/// Initializes the terminal interface.
///
/// It enables the raw mode and sets terminal properties.
pub(crate) fn enter(terminal: &mut Terminal) -> Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Define a custom panic hook to reset the terminal properties.
    // This way, we won't have our terminal messed up if an unexpected error happens.
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        reset().expect("failed to reset the terminal");
        panic_hook(panic);
    }));

    terminal.hide_cursor()?;
    Ok(())
}

/// Exits the terminal interface.
///
/// It disables the raw mode and reverts back the terminal properties.
pub(crate) fn exit(terminal: &mut Terminal) -> Result<()> {
    reset()?;
    terminal.show_cursor()?;

    Ok(())
}

/// Resets the terminal interface.
///
/// This function is also used for the panic hook to revert the terminal
/// properties if unexpected errors occur.
fn reset() -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
