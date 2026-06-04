//! [`Name`] — a non-empty, trimmed text label shared by domain entities
//! (e.g. [`Account`](crate::account::Account) and
//! [`AccountGroup`](crate::account_group::AccountGroup)).

use std::{borrow::Borrow, fmt, ops::Deref};

pub mod error;
pub use error::{NameError, NameResult};

/// A non-empty, whitespace-trimmed name used to label domain entities.
///
/// Construct via [`Name::new`] (or `TryFrom<&str>`), which rejects empty or
/// whitespace-only input — so holding a `Name` guarantees it is non-empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// Creates a name, trimming surrounding whitespace.
    ///
    /// Returns [`NameError::Empty`] if the input is empty or only whitespace.
    pub fn new(raw_name: &str) -> NameResult<Self> {
        let trimmed = raw_name.trim().to_string();
        if trimmed.is_empty() {
            return Err(NameError::Empty);
        }

        Ok(Name(trimmed))
    }

    /// Consumes the name and returns the inner `String`.
    pub fn into_string(self) -> String {
        self.0
    }

    /// Wraps a string as a `Name` without trimming or validating it.
    ///
    /// For trusted input only (e.g. a value already validated and then loaded
    /// from storage). Passing empty/untrimmed text yields a `Name` that breaks
    /// the non-empty invariant — but that is a logical concern, not a
    /// memory-safety one, so this function is safe to call.
    pub fn new_unchecked(raw_name: String) -> Self {
        Name(raw_name)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Fallible conversion, equivalent to [`Name::new`].
impl TryFrom<&str> for Name {
    type Error = NameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Name::new(value)
    }
}

/// Borrows the name as a string slice.
impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Lets a `Name` be used directly with `str` methods (e.g. `len`, `chars`).
impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Enables looking up a `Name` key in a map/set by `&str`.
impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        &self.0
    }
}
