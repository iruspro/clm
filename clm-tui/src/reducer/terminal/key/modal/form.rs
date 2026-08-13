//! Key handling for an open form: editing fields, cancelling, and submitting.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::command::RequestId;
use crate::app::config::KeyBindings;
use crate::app::state::{ConcreteForm, Modal};
use crate::app::{Command, State};
use crate::tui::form::Form;
use crate::tui::form::fields::FieldKind;

/// Handles a key press against the open form.
///
/// Cancel and confirm are checked first and never reach the form; everything
/// else is field editing. Cancelling discards the input outright — there is no
/// "are you sure?" step.
pub fn reduce(state: &mut State, event: KeyEvent) -> Vec<Command> {
    let keys = state.keys();

    if event.code == keys.cancel {
        state.close_modal();
        return Vec::new();
    }

    if event.code == keys.confirm {
        return submit(state);
    }

    if let Some(form) = state.form_mut() {
        edit(form.as_form_mut(), keys, event);
    }

    Vec::new()
}

/// Applies a field-editing key to whichever form is focused: moving between
/// fields, cycling choices, and typing into text fields. Editing clears any
/// stale validation errors so they don't linger over changed input.
fn edit(form: &mut dyn Form, keys: KeyBindings, event: KeyEvent) {
    match event.code {
        code if code == keys.focus_next => form.focus_next(),
        code if code == keys.focus_prev => form.focus_prev(),
        // The selection keys cycle a focused choice field; on a text field they
        // aren't special and fall through to type like any other character.
        code if code == keys.prev_selection && form.focus_kind() == FieldKind::Choice => {
            form.select_prev();
        }
        code if code == keys.next_selection && form.focus_kind() == FieldKind::Choice => {
            form.select_next();
        }
        KeyCode::Backspace => {
            form.clear_errors();
            form.backspace();
        }
        KeyCode::Char(c) => {
            form.clear_errors();
            form.input_char(c);
        }
        _ => {}
    }
}

/// Validates the open form. On success it emits the resulting command; on
/// failure it records the per-field errors so they render beside their fields.
fn submit(state: &mut State) -> Vec<Command> {
    let form = state
        .form_mut()
        .expect("submit runs only while a form modal is open");

    match form {
        ConcreteForm::Account(account) => match account.to_command() {
            Ok(command) => {
                let rid = file_request(state);
                vec![Command::CreateAccount(command, rid)]
            }
            Err(errors) => {
                account.set_errors(errors);
                Vec::new()
            }
        },
        // TODO: the Group and Transaction forms.
        ConcreteForm::Group | ConcreteForm::Transaction => Vec::new(),
    }
}

/// Takes the open form out of the modal stack and files it under a fresh
/// request id, so the [`Modal::Error`] handler can re-open it if the command it
/// produced later fails.
fn file_request(state: &mut State) -> RequestId {
    match state.close_modal() {
        Some(Modal::Form(form)) => state.open_request(form),
        _ => unreachable!("submit runs only while a form modal is open"),
    }
}
