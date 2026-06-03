use std::{error::Error, fmt};

pub type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug)]
pub enum RepoError {
    NotFound,
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
