use thiserror::Error;

/// An error from a name operation.
#[derive(Error, Debug)]
pub enum NameError {
    /// The name was empty or contained only whitespace.
    #[error("name must not be empty")]
    Empty,
}
