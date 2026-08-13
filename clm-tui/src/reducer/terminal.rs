//! Transitions for input coming from the terminal.

mod key;

use crate::app::{Command, State};
use crate::tui::TerminalEvent;

/// Folds one [`TerminalEvent`] into the state.
///
/// Resizes need no transition: the layout is recomputed from the frame area on
/// every draw, so the event only has to wake the loop up for a redraw.
pub fn reduce(state: &mut State, event: TerminalEvent) -> Vec<Command> {
    match event {
        TerminalEvent::Key(event) => key::reduce(state, event),
        TerminalEvent::Resize(_, _) => vec![],
    }
}
