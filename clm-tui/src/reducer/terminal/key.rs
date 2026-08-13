//! Key handling: the split between a modal and the screen behind it.

mod modal;
mod screen;

use ratatui::crossterm::event::KeyEvent;

use crate::app::{Command, State};

/// Routes a key press to whichever handler currently owns the keyboard.
///
/// An open modal takes **every** key — that is what makes it modal — so the
/// screen handlers never have to know a popup exists, and no key can leak
/// through to the list behind it. There is deliberately no shared "global"
/// branch above this split: a modal that wants the quit key is expected to
/// handle it itself.
pub fn reduce(state: &mut State, event: KeyEvent) -> Vec<Command> {
    if state.is_modal() {
        modal::reduce(state, event)
    } else {
        screen::reduce(state, event)
    }
}
