//! Validation primitives for the UI's input forms.
//!
//! Forms parse their raw input into domain commands (e.g.
//! [`AccountForm::to_command`](crate::app::state::modal::AccountForm::to_command)),
//! collecting a [`FieldError`] per invalid field so each error can be shown next
//! to the field that produced it.

/// A validation error attached to the input field that produced it.
#[derive(Debug, Clone)]
pub struct FieldError<F> {
    /// Which field failed to validate.
    pub field: F,
    /// Human-readable reason, ready to display.
    pub message: String,
}

impl<F> FieldError<F> {
    /// Creates a field error for `field` with the given `message`.
    pub fn new(field: F, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}
