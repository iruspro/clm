//! The half of the event stream that originates inside the app rather than at
//! the terminal: results reported back by the [`Actor`](super::Actor)'s worker.

use application::views::accounts;
use domain::AccountId;

use crate::app::command::RequestId;

/// Events produced by the [`Actor`](crate::Actor)'s workers and folded
/// into [`State`](crate::app::State) by the reducer.
///
/// Write outcomes carry the [`RequestId`] of the [`Command`](crate::app::Command)
/// that produced them, so the reducer can match a result back to the form the
/// user submitted; reads carry no id because nothing is waiting to be restored.
#[derive(Debug)]
pub enum InternalEvent {
    /// An account was created.
    AccountCreated(AccountId, RequestId),
    /// Accounts were loaded.
    AccountsLoaded(Vec<accounts::ResultItem>),
    /// An error occurred during a write operation. The message is already
    /// rendered to a `String`, so the UI layer never touches the error type.
    WriteRequestError(String, RequestId),
    /// An error occurred during a read operation.
    ReadRequestError(String),
}
