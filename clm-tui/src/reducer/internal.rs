//! Transitions for results coming back from the [`Actor`](crate::Actor).

use crate::actor::InternalEvent;
use crate::app::state::Modal;
use crate::app::state::modal::ConcreteError;
use crate::app::{Command, State};

/// Folds one [`InternalEvent`] into the state.
///
/// Two conventions run through this module:
///
/// - **A successful write re-reads rather than patches.** Creating an account
///   emits [`Command::LoadAccounts`] instead of pushing the new row into the
///   list, so what's on screen always came from the database and can't drift
///   from it.
/// - **Errors are queued, not shown.** They arrive while the user is doing
///   something else, so [`open_modal_delayed`](State::open_modal_delayed) puts
///   them behind whatever is already on screen. A write error keeps its
///   [`RequestId`](crate::app::command::RequestId), which is what later lets the
///   rejected form be re-opened with the user's input intact.
pub fn reduce(state: &mut State, event: InternalEvent) -> Vec<Command> {
    match event {
        InternalEvent::AccountCreated(_, rid) => {
            state.close_request(rid);
            vec![Command::LoadAccounts]
        }
        InternalEvent::AccountsLoaded(accounts) => {
            state.accounts_mut().set_accounts(accounts);
            vec![]
        }
        InternalEvent::ReadRequestError(err) => {
            state.open_modal_delayed(Modal::Error(ConcreteError::new(err, None)));
            vec![]
        }
        InternalEvent::WriteRequestError(err, rid) => {
            state.open_modal_delayed(Modal::Error(ConcreteError::new(err, Some(rid))));
            vec![]
        }
    }
}
