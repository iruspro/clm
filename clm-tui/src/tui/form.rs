//! Form state — the raw, editable input backing the create/edit modals.
//!
//! A form holds *unvalidated* input (text as [`String`], choices as an index
//! into a fixed option list), which field is focused, and any validation errors
//! from the last submit. Turning a form into a domain command (e.g.
//! [`AccountForm::to_command`](crate::app::state::modal::AccountForm::to_command))
//! is the parse-at-the-boundary step: it validates each raw field, collecting a
//! [`FieldError`](validation::FieldError) per failure.

pub mod fields;
pub mod validation;

use self::fields::{FieldKind, FieldView};

/// The form-agnostic view of an input form: navigating and editing its fields,
/// plus a read-only snapshot of them for rendering.
///
/// Every create/edit modal shares this behaviour, so both the reducer and the
/// UI can drive any open form through a `dyn Form` without knowing which one it
/// is — the reducer edits through `&mut dyn Form`, and the renderer draws it
/// from [`title`](Form::title) and [`fields`](Form::fields). Producing a
/// validated domain command stays with each concrete form, since only it knows
/// the command type it parses into.
pub trait Form {
    /// Moves focus to the next field, wrapping around.
    fn focus_next(&mut self);

    /// Moves focus to the previous field, wrapping around.
    fn focus_prev(&mut self);

    /// The kind of the currently focused field, so the reducer can tell whether
    /// a printable key should type text or cycle a choice.
    fn focus_kind(&self) -> FieldKind;

    /// Cycles the focused choice field forward (text fields ignore this).
    fn select_next(&mut self);

    /// Cycles the focused choice field backward (text fields ignore this).
    fn select_prev(&mut self);

    /// Types a character into the focused text field (choice fields ignore it).
    fn input_char(&mut self, c: char);

    /// Deletes the last character of the focused text field.
    fn backspace(&mut self);

    /// Discards the validation errors recorded by the previous submit.
    fn clear_errors(&mut self);

    /// The popup title for this form, e.g. `"New account"`.
    fn title(&self) -> &str;

    /// A read-only view of every field, in display order. This is the read side
    /// that matches the editing methods above, so one renderer can draw any form
    /// without knowing which concrete one it is.
    fn fields(&self) -> Vec<FieldView<'_>>;
}
