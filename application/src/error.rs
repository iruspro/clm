//! [`AppError`] — the aggregate error returned by application use cases.
//!
//! A use case orchestrates domain value-object parsing (which can fail with
//! e.g. [`NameError`]) and repository calls (which can fail with [`RepoError`]),
//! so it needs one error type that all of those convert into. The [`From`] impls
//! let `?` collapse each underlying error into an `AppError` automatically —
//! `?` desugars to `From::from`.

use std::{error::Error, fmt};

use domain::{
    NameError, RepoError,
    journal::{BalancedPostingsError, MagnitudeError},
    money::MoneyError,
};

/// Shorthand for a result whose error is an [`AppError`].
pub type AppResult<T> = Result<T, AppError>;

/// Anything that can go wrong while running a use case.
///
/// Wraps the domain validation errors and the repository port's [`RepoError`].
/// Each variant keeps the underlying error as its [`source`](Error::source), so
/// the full chain stays walkable while each level's `Display` adds a short prefix.
#[derive(Debug)]
pub enum AppError {
    /// A [`Name`](domain::name::Name) failed validation.
    Name(NameError),
    /// A monetary operation failed (currency mismatch or overflow).
    Money(MoneyError),
    /// A posting magnitude was not strictly positive.
    Magnitude(MagnitudeError),
    /// A set of postings was too small or did not balance.
    BalancedPostings(BalancedPostingsError),
    /// A repository operation failed (not found or storage error).
    Repo(RepoError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Name(err) => write!(f, "name error: {err}"),
            AppError::Money(err) => write!(f, "money error: {err}"),
            AppError::Magnitude(err) => write!(f, "magnitude error: {err}"),
            AppError::BalancedPostings(err) => write!(f, "balanced postings error: {err}"),
            AppError::Repo(err) => write!(f, "repo error: {err}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Name(err) => Some(err),
            AppError::Money(err) => Some(err),
            AppError::Magnitude(err) => Some(err),
            AppError::BalancedPostings(err) => Some(err),
            AppError::Repo(err) => Some(err),
        }
    }
}

impl From<NameError> for AppError {
    fn from(value: NameError) -> Self {
        AppError::Name(value)
    }
}

impl From<MoneyError> for AppError {
    fn from(value: MoneyError) -> Self {
        AppError::Money(value)
    }
}

impl From<MagnitudeError> for AppError {
    fn from(value: MagnitudeError) -> Self {
        AppError::Magnitude(value)
    }
}

impl From<BalancedPostingsError> for AppError {
    fn from(value: BalancedPostingsError) -> Self {
        AppError::BalancedPostings(value)
    }
}

impl From<RepoError> for AppError {
    fn from(value: RepoError) -> Self {
        AppError::Repo(value)
    }
}
