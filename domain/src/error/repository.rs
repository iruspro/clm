use std::{error::Error, fmt};

/// Shorthand for a result whose error is a [`RepoError`].
pub type RepoResult<T> = Result<T, RepoError>;

/// An error from a repository operation.
#[derive(Debug)]
pub enum RepoError {
    /// The requested entity does not exist.
    NotFound,
    /// The underlying storage failed.
    Storage(String),
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepoError::NotFound => write!(f, "not found"),
            RepoError::Storage(err) => write!(f, "storage error: {err}"),
        }
    }
}

impl Error for RepoError {}
