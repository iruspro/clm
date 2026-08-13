//! Modal overlays: the popups drawn over the active screen.
//!
//! While any modal is open it takes every key press — the key handler routes to
//! the modal branch before the screen branch — so a modal is the app's only
//! modal-in-the-literal-sense state.

mod form;

pub use self::form::AccountForm;
use crate::app::command::RequestId;
use crate::tui::form::Form;

/// A popup drawn over the active screen.
///
/// Each variant is answered differently: [`Quit`](Modal::Quit) waits for
/// yes/no, [`Help`](Modal::Help) closes on any key, [`Error`](Modal::Error)
/// waits for the confirm key, and [`Form`](Modal::Form) hands its keys to the
/// form.
#[derive(Debug)]
pub enum Modal {
    /// "Exit the app?" confirmation.
    Quit,
    /// The key-binding cheatsheet.
    Help,
    /// A failure to report, optionally tied to the request that caused it.
    Error(ConcreteError),
    /// A create/edit form.
    Form(ConcreteForm),
}

// region: Error
/// An error message to show, plus the request it came from.
///
/// `rid` is what makes a failed write recoverable: it identifies the form parked
/// in [`State::request_forms`](crate::app::State), so dismissing the error can
/// put the user's input back on screen rather than making them retype it. A
/// `None` id means there is nothing to restore — a failed read, for instance.
#[derive(Debug)]
pub struct ConcreteError {
    msg: String,
    rid: Option<RequestId>,
}

impl ConcreteError {
    /// Wraps a message, tagged with the request that produced it if there is one.
    pub fn new(msg: impl Into<String>, request_id: Option<RequestId>) -> Self {
        Self {
            msg: msg.into(),
            rid: request_id,
        }
    }

    /// The message to display.
    pub fn msg(&self) -> &str {
        &self.msg
    }

    /// The request to restore on dismiss, if any.
    pub fn request_id(&self) -> Option<RequestId> {
        self.rid
    }
}
// endregion

// region: Form
/// Which form a [`Modal::Form`] is holding.
///
/// Forms are an enum rather than a `Box<dyn Form>` because submitting one needs
/// its concrete type — only [`AccountForm`] knows it parses into a
/// `CreateAccountCommand`. Everything else (navigating, typing, rendering) goes
/// through the [`Form`] trait via [`as_form`](Self::as_form) and
/// [`as_form_mut`](Self::as_form_mut).
#[derive(Debug)]
pub enum ConcreteForm {
    /// The "new account group" form. Not implemented yet.
    Group,
    /// The "new account" form.
    Account(AccountForm),
    /// The "new transaction" form. Not implemented yet.
    Transaction,
}

impl ConcreteForm {
    /// The form-agnostic view of this modal's form for rendering, or `None` for
    /// a variant that isn't implemented yet.
    ///
    /// The renderer treats `None` as "draw the coming-soon placeholder".
    pub fn as_form(&self) -> Option<&dyn Form> {
        match self {
            ConcreteForm::Account(form) => Some(form),
            ConcreteForm::Group | ConcreteForm::Transaction => None,
        }
    }

    /// The form-agnostic editing view of this modal's form.
    ///
    /// This is what lets the reducer drive field editing without matching on
    /// each concrete form; submission still needs the concrete type.
    ///
    /// # Panics
    /// Panics ([`todo!`]) for a variant that isn't implemented yet. Unlike
    /// [`as_form`](Self::as_form) this has no `None` to hand back, so callers
    /// must check the variant is real before editing through it.
    ///
    /// <div class="warning">
    ///
    /// The key handler does **not** currently check: opening the Group or
    /// Transaction form and pressing any editing key reaches this `todo!()`.
    ///
    /// </div>
    pub fn as_form_mut(&mut self) -> &mut dyn Form {
        match self {
            ConcreteForm::Account(form) => form,
            ConcreteForm::Group | ConcreteForm::Transaction => todo!(),
        }
    }
}
// endregion
