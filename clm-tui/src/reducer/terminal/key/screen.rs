//! Key handling for the active screen, when no modal is open.

use ratatui::crossterm::event::KeyEvent;

use crate::app::state::modal::AccountForm;
use crate::app::state::{ConcreteForm, Modal, Screen};
use crate::app::{Command, State};

/// Handles a key press against the active screen.
///
/// Global keys — quit, help, screen switching, and the create actions — win
/// first and work identically everywhere; anything left over falls through to
/// the handler for the current [`Screen`].
///
/// The match arms are guards (`code if code == keys.quit`) rather than literal
/// patterns because the bindings come from [`KeyBindings`](crate::app::config::KeyBindings)
/// at runtime and so aren't constants the compiler can match on.
pub fn reduce(state: &mut State, event: KeyEvent) -> Vec<Command> {
    let keys = state.keys();

    match event.code {
        // region: Global keys
        code if code == keys.quit => {
            state.open_modal(Modal::Quit);
            vec![]
        }
        code if code == keys.help => {
            state.open_modal(Modal::Help);
            vec![]
        }
        code if code == keys.focus_next => {
            state.next_screen();
            vec![]
        }
        code if code == keys.focus_prev => {
            state.prev_screen();
            vec![]
        }
        code if code == keys.new_account => {
            state.open_modal(Modal::Form(ConcreteForm::Account(AccountForm::new())));
            vec![]
        }
        code if code == keys.new_group => {
            state.open_modal(Modal::Form(ConcreteForm::Group));
            vec![]
        }
        code if code == keys.new_transaction => {
            state.open_modal(Modal::Form(ConcreteForm::Transaction));
            vec![]
        }
        // endregion
        // region: Screen keys
        _ => match state.screen() {
            Screen::Accounts => reduce_accounts(state, event),
            Screen::Categories => reduce_categories(state, event),
            Screen::Transactions => reduce_transactions(state, event),
        },
        // endregion
    }
}

/// Keys for the Accounts screen: moving the selection, and editing the selected
/// account.
///
/// # Panics
/// The edit key is not implemented and hits a `todo!()`.
fn reduce_accounts(state: &mut State, event: KeyEvent) -> Vec<Command> {
    let keys = state.keys();

    match event.code {
        code if code == keys.next_selection => {
            state.accounts_mut().select_next();
            vec![]
        }
        code if code == keys.prev_selection => {
            state.accounts_mut().select_prev();
            vec![]
        }
        code if code == keys.edit => {
            todo!()
        }
        _ => vec![],
    }
}

/// Keys for the Categories screen.
///
/// # Panics
/// Always — the screen has no behaviour yet, so any key not caught by the global
/// arms reaches this `todo!()`.
#[allow(unused)]
fn reduce_categories(state: &mut State, key: KeyEvent) -> Vec<Command> {
    todo!()
}

/// Keys for the Transactions screen.
///
/// # Panics
/// Always — the screen has no behaviour yet, so any key not caught by the global
/// arms reaches this `todo!()`.
#[allow(unused)]
fn reduce_transactions(state: &mut State, key: KeyEvent) -> Vec<Command> {
    todo!()
}
