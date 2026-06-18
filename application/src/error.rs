//! [`AppError`] — the aggregate error returned by application use cases.
//!
//! A use case orchestrates domain value-object parsing (which can fail with
//! e.g. [`NameError`]) and repository calls (which can fail with [`RepoError`]),
//! so it needs one error type that all of those convert into. The [`From`] impls
//! let `?` collapse each underlying error into an `AppError` automatically —
//! `?` desugars to `From::from`.

use domain::journal::{BalancedPostingsError, MagnitudeError};
use domain::money::MoneyError;
use domain::{NameError, RepoError};
use thiserror::Error;

/// Anything that can go wrong while running a use case.
///
/// Wraps the domain validation errors and the repository port's [`RepoError`].
/// Each variant keeps the underlying error as its [`source`](std::error::Error::source), so
/// the full chain stays walkable while each level's `Display` adds a short prefix.
#[derive(Error, Debug)]
pub enum AppError {
    /// A [`Name`](domain::Name) failed validation.
    #[error("name error: {0}")]
    Name(#[from] NameError),
    /// A monetary operation failed (currency mismatch or overflow).
    #[error("money error: {0}")]
    Money(#[from] MoneyError),
    /// A posting magnitude was not strictly positive.
    #[error("magnitude error: {0}")]
    Magnitude(#[from] MagnitudeError),
    /// A set of postings was too small or did not balance.
    #[error("balanced postings error: {0}")]
    BalancedPostings(#[from] BalancedPostingsError),
    /// A repository operation failed (not found or storage error).
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}
