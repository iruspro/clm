use thiserror::Error;

/// An error from an account operation.
#[derive(Error, Debug)]
pub enum AccountError {
    /// The stored discriminant does not match any [`AccountKind`](crate::account::AccountKind)
    /// variant — the value came from outside the domain (a hand-edited row, an
    /// older schema) and cannot be decoded.
    #[error("unknown account kind discriminant: {0}")]
    UnknownKind(u8),
}
