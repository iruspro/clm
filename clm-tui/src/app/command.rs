//! The [`Command`] type: the reducer's way of asking for work it cannot do
//! itself.
//!
//! The reducer is pure, so instead of touching the database it returns commands.
//! [`App`](crate::app::App) forwards each one to the
//! [`Actor`](crate::Actor), which runs it and reports back as an
//! [`InternalEvent`](crate::actor::InternalEvent).

use application::CreateAccountCommand;
use uuid::Uuid;

/// A side effect for the [`Actor`](crate::Actor) to run off the UI
/// thread — typically executing a use case or a query.
#[derive(Debug)]
pub enum Command {
    /// Create an account from already-validated input. The [`RequestId`] ties
    /// the eventual result back to the form that produced it.
    CreateAccount(CreateAccountCommand, RequestId),
    /// Re-read every account summary. Emitted after any write that could have
    /// changed a balance, so the list is refreshed from the database rather than
    /// patched in memory.
    LoadAccounts,
}

// region: RequestId
/// Correlates a write [`Command`] with the event it eventually produces.
///
/// The reducer parks the submitted form under this id
/// ([`State::open_request`](crate::app::State::open_request)) and closes the
/// modal, so the UI stays responsive while the write is in flight. When the
/// result arrives, the id says which parked form it belongs to — the form is
/// dropped on success, or re-opened with the error on failure.
///
/// Backed by a UUIDv7, so ids are unique and sort by creation time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(Uuid);

impl RequestId {
    /// Generates a fresh, time-ordered id.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
// endregion
