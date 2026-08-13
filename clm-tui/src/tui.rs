//! Terminal lifecycle and rendering for the TUI.
//!
//! [`Tui`] owns the terminal handle and the background input thread:
//! [`enter`](Tui::enter) switches into raw mode and the alternate screen and
//! starts the event listener, [`draw`](Tui::draw) renders one frame from the
//! application [`State`], and [`exit`](Tui::exit) stops the listener. The
//! terminal is always restored when a [`Tui`] is dropped, so a panic or an
//! early `?` return can never leave it in raw mode.
//!
//! [`install_hooks`] wires that same restore into color_eyre, so panic and
//! error reports print to a restored terminal instead of over the UI.

pub mod event;
pub mod form;
pub mod ui;

use std::io::{Stdout, stdout};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

use color_eyre::eyre::Result;
use crossterm::cursor;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend as Backend;

pub use self::event::TerminalEvent;
use crate::app::{CancellationToken, Event, State};

/// Owns the terminal and the background input thread for the lifetime of the UI.
///
/// Build it with [`new`](Self::new), then call [`enter`](Self::enter) once to
/// take over the terminal, [`draw`](Self::draw) per frame, and
/// [`exit`](Self::exit) to shut the listener down. The terminal is restored in
/// [`Drop`].
#[derive(Debug)]
pub struct Tui {
    /// Cloned into the listener so it can forward terminal input as events.
    tx: mpsc::Sender<Event>,
    /// Shared shutdown flag; the listener thread stops once it is set.
    cancellation_token: Arc<CancellationToken>,

    /// Handle to the terminal backend.
    terminal: ratatui::Terminal<Backend<Stdout>>,
    /// Join handle for the input thread; `None` until [`enter`](Self::enter).
    listener: Option<JoinHandle<()>>,
}

impl Tui {
    /// Creates a [`Tui`] over the current stdout without changing the terminal
    /// mode yet; call [`enter`](Self::enter) to take it over.
    pub fn new(
        tx: mpsc::Sender<Event>,
        cancellation_token: Arc<CancellationToken>,
    ) -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(stdout()))?;

        Ok(Tui {
            tx,
            cancellation_token,
            terminal,
            listener: None,
        })
    }

    /// Takes over the terminal — enables raw mode, enters the alternate screen,
    /// hides the cursor — and spawns the input listener thread.
    ///
    /// # Panics
    /// Panics if the app has already been cancelled; a [`Tui`] is not meant to
    /// be entered during shutdown.
    pub fn enter(&mut self) -> Result<()> {
        debug_assert!(
            !self.cancellation_token.cancelled(),
            "enter called after cancellation"
        );
        debug_assert!(self.listener.is_none(), "enter called twice");

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;

        self.listener = Some(event::spawn(
            self.tx.clone(),
            Arc::clone(&self.cancellation_token),
        ));

        Ok(())
    }

    /// Renders a single frame from `state`.
    ///
    /// # Panics
    /// Panics if called after the app has been cancelled.
    pub fn draw(&mut self, state: &State) -> Result<()> {
        debug_assert!(
            !self.cancellation_token.cancelled(),
            "draw called after cancellation"
        );

        self.terminal.draw(|f| ui::render(f, state))?;

        Ok(())
    }

    /// Stops and joins the input listener; the terminal itself is restored
    /// afterwards by [`Drop`].
    ///
    /// # Panics
    /// Panics if called before the app has been cancelled, or if the listener
    /// thread panicked.
    pub fn exit(mut self) {
        debug_assert!(
            self.cancellation_token.cancelled(),
            "exit called before cancellation"
        );

        if let Some(listener) = self.listener.take() {
            listener.join().expect("event listener thread panicked");
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Best-effort restore during teardown/unwinding; there's nothing useful
        // to do with an error at this point.
        let _ = restore_terminal();
    }
}

/// Leaves the alternate screen, shows the cursor and disables raw mode — but
/// only when raw mode is actually on, so it is safe to call more than once.
fn restore_terminal() -> Result<()> {
    if crossterm::terminal::is_raw_mode_enabled()? {
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;
    }

    Ok(())
}

/// Installs color_eyre's panic and error hooks, wrapped so the terminal is
/// restored *before* the report is printed. Call once, at startup.
pub fn install_hooks() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();

    // Panics: restore the terminal, then let color_eyre print its report.
    let panic_hook = panic_hook.into_panic_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic_hook(info);
    }));

    // Errors bubbled out of `main` as `Err(...)`: same idea.
    let eyre_hook = eyre_hook.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |error| {
        let _ = restore_terminal();
        eyre_hook(error)
    }))?;

    Ok(())
}
