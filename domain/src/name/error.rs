use std::error::Error;
use std::fmt;

/// An error from a name operation.
#[derive(Debug)]
pub enum NameError {
    /// The name was empty or contained only whitespace.
    Empty,
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => write!(f, "name must not be empty"),
        }
    }
}

impl Error for NameError {}
