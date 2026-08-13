//! Terminal event source for the TUI.
//!
//! [`spawn`] launches a background thread that polls the terminal (via
//! crossterm) and forwards input to the application as [`TerminalEvent`]s over
//! an [`mpsc`] channel. Polling with a timeout lets the thread periodically
//! re-check the shared [`CancellationToken`] instead of blocking forever on
//! input, so it stops promptly on shutdown.

use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

use crate::app::{CancellationToken, Event};

/// Maximum time a single `poll` blocks before the listener thread loops back to
/// re-check the cancellation token. Bounds the worst-case shutdown latency.
const POLL_TIMEOUT: Duration = Duration::from_millis(100);

/// Terminal events.
pub enum TerminalEvent {
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Spawns a thread that polls the terminal and forwards each key press to `tx`
/// as [`Event::Terminal`], returning its [`JoinHandle`].
///
/// The thread runs until `cancellation_token` is set or `tx`'s receiver is
/// dropped, re-checking the token at least every [`POLL_TIMEOUT`].
pub fn spawn(
    tx: mpsc::Sender<Event>,
    cancellation_token: Arc<CancellationToken>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !cancellation_token.cancelled() {
            if !event::poll(POLL_TIMEOUT).expect("unable to poll for terminal event") {
                continue;
            }

            if match event::read().expect("unable to read terminal event") {
                CrosstermEvent::Key(key) if key.kind == event::KeyEventKind::Press => {
                    tx.send(Event::Terminal(TerminalEvent::Key(key)))
                }
                CrosstermEvent::Resize(cols, rows) => {
                    tx.send(Event::Terminal(TerminalEvent::Resize(cols, rows)))
                }
                _ => Ok(()),
            }
            .is_err()
            {
                // Receiver gone → the app is shutting down, so stop looping.
                break;
            }
        }
    })
}
