//! [`Side`] — the debit/credit side of a double-entry posting.

/// One side of a double-entry posting.
///
/// Used both as a posting's direction and as an account's normal balance side
/// (see [`AccountKind::normal_balance`](crate::account::AccountKind::normal_balance)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The debit side (left).
    Debit,
    /// The credit side (right).
    Credit,
}
