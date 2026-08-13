//! The building blocks a form is made of.
//!
//! Two input primitives — [`TextInput`] and [`Choice`] — plus the two types that
//! let one renderer and one key handler cope with any form without knowing which
//! it is: [`FieldView`] (what a field looks like) and [`FieldKind`] (what a key
//! press should do to it).

use std::borrow::Cow;

/// A read-only snapshot of one field, in the shape the UI needs to draw it.
///
/// This is the *rendering* counterpart to the editing methods on
/// [`Form`](super::Form): a form describes each of its fields as a `FieldView`
/// (via [`Form::fields`](super::Form::fields)) and one generic renderer draws
/// any form from that list — no renderer knows which concrete form it is.
///
/// [`value`](FieldView::value) is a [`Cow`] because a text field can lend its
/// stored string directly (`Borrowed`), while a choice field has to format its
/// selected option into a fresh `String` (`Owned`). [`new`](FieldView::new)
/// takes anything convertible into a `Cow<str>`, so `&str` and `String` callers
/// pass their value the same way.
pub struct FieldView<'a> {
    /// The field's label, e.g. `"Name"`.
    pub label: &'a str,
    /// The value to display: borrowed from the field, or freshly formatted.
    pub value: Cow<'a, str>,
    /// Whether this field currently has focus.
    pub focused: bool,
    /// The validation error to show beneath the field, if the last submit
    /// rejected it.
    pub error: Option<&'a str>,
}

impl<'a> FieldView<'a> {
    /// Builds a view of one field. `value` accepts a `&str` (borrowed) or a
    /// `String` (owned); both convert into the stored [`Cow`].
    pub fn new(
        label: &'a str,
        value: impl Into<Cow<'a, str>>,
        focused: bool,
        error: Option<&'a str>,
    ) -> Self {
        Self {
            label,
            value: value.into(),
            focused,
            error,
        }
    }
}

/// What kind of input a field is, so key handling can route a keypress: a
/// printable key types into a [`Text`](FieldKind::Text) field but cycles a
/// [`Choice`](FieldKind::Choice) field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A free-text field, like [`TextInput`], that accepts typed characters.
    Text,
    /// A fixed-option field, like [`Choice`], cycled with the selection keys.
    Choice,
}

// region: TextInput
/// A single-line text field.
///
/// Editing appends to / trims from the end only — there is no mid-string cursor
/// yet (TODO: cursor movement).
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    value: String,
}

impl TextInput {
    /// The current text.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Appends a typed character.
    pub fn push(&mut self, c: char) {
        self.value.push(c);
    }

    /// Removes the last character, if any.
    pub fn pop(&mut self) {
        self.value.pop();
    }
}
// endregion

// region: Choice
/// A selection among a fixed, non-empty list of options.
///
/// Every option is already a valid domain value, so a `Choice` can never be
/// "invalid" — which is why choice fields never produce a
/// [`FieldError`](super::validation::FieldError).
#[derive(Debug, Clone)]
pub struct Choice<T> {
    options: Vec<T>,
    selected: usize,
}

impl<T: Copy> Choice<T> {
    /// Wraps a non-empty list of options, selecting the first.
    pub fn new(options: Vec<T>) -> Self {
        debug_assert!(!options.is_empty(), "a Choice needs at least one option");
        Self {
            options,
            selected: 0,
        }
    }

    /// The currently selected option.
    pub fn selected(&self) -> T {
        self.options[self.selected]
    }

    /// Moves the selection to the next option, wrapping around.
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    /// Moves the selection to the previous option, wrapping around.
    pub fn prev(&mut self) {
        self.selected = (self.selected + self.options.len() - 1) % self.options.len();
    }
}
// endregion
