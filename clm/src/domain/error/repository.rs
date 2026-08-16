use thiserror::Error;

/// An error from a repository operation.
#[derive(Error, Debug)]
pub enum RepoError {
    /// The requested entity does not exist.
    #[error("not found")]
    NotFound,
    /// The underlying storage failed.
    #[error("storage error: {0}")]
    Storage(String),
}
