//! Key handling for the modal on screen.

mod form;

use ratatui::crossterm::event::KeyEvent;

use crate::app::state::Modal;
use crate::app::{Command, State};

/// Handles a key press against the visible modal.
///
/// Every modal swallows keys it doesn't recognise rather than passing them on,
/// so an unanswered popup can't be navigated around.
///
/// Dismissing an [`Error`](Modal::Error) is the one branch that does more than
/// close: if the error carries a request id, the form parked under it is taken
/// back out and re-opened, so a rejected write returns the user to their input
/// instead of an empty screen.
///
/// # Panics
/// Panics if no modal is open. Only reachable via
/// [`key::reduce`](super::reduce), which checks
/// [`State::is_modal`](crate::app::State::is_modal) first.
pub fn reduce(state: &mut State, event: KeyEvent) -> Vec<Command> {
    let keys = state.keys();
    let modal = state.modal().expect("a modal is open");

    match modal {
        Modal::Quit => {
            match event.code {
                code if code == keys.yes => {
                    state.quit();
                }
                code if code == keys.no => {
                    state.close_modal();
                }
                _ => {}
            };

            vec![]
        }
        Modal::Help => {
            state.close_modal();
            vec![]
        }
        Modal::Error(err) => {
            if event.code == keys.confirm {
                let rid = err.request_id();

                state.close_modal();
                if let Some(rid) = rid
                    && let Some(form) = state.close_request(rid)
                {
                    state.open_modal(Modal::Form(form));
                }
            }
            vec![]
        }
        Modal::Form(_) => form::reduce(state, event),
    }
}
