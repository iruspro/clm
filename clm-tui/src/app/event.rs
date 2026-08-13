//! The single event type the loop consumes.

use crate::actor::InternalEvent;
use crate::tui::TerminalEvent;

/// One thing that happened, from either source that can produce work.
///
/// The two producers — the terminal input listener and the
/// [`Actor`](crate::Actor)'s worker — share one channel, so
/// [`App`](crate::app::App) selects over a single stream and
/// [`reduce`](crate::reducer::reduce) has exactly one entry point.
pub enum Event {
    /// The user pressed a key or resized the window.
    Terminal(TerminalEvent),
    /// A dispatched [`Command`](crate::app::Command) finished, for better or
    /// worse.
    Internal(InternalEvent),
}
