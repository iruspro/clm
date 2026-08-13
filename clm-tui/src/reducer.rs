//! **Pure state transitions** (TEA "update").
//!
//! One function, [`reduce`], is the only thing in the app that changes
//! [`State`]. It performs no I/O, spawns no threads and touches no globals: work
//! that needs the outside world is *returned* as [`Command`]s for the
//! [`Actor`](crate::Actor) to run, so a transition is a plain value-in,
//! value-out step that can be exercised without a terminal or a database.
//!
//! The dispatch mirrors the two event sources, then narrows:
//!
//! ```text
//! reduce
//! ├─ internal          actor results — loaded data, write outcomes, errors
//! └─ terminal
//!    └─ key
//!       ├─ modal       whenever a modal is open: quit / help / error / form
//!       │  └─ form     field editing and submission
//!       └─ screen      otherwise: global keys, then per-screen keys
//! ```
//!
//! Each level returns the [`Command`]s it wants run, so the vectors bubble back
//! up unchanged.

mod internal;
mod terminal;

use crate::app::{Command, Event, State};

/// Folds one [`Event`] into the state, returning the next state and any side
/// effects to dispatch.
///
/// Takes `State` by value and hands it back so the loop can't accidentally keep
/// the previous version around. Returning an empty `Vec` is the normal case —
/// most key presses only move the cursor.
pub fn reduce(mut state: State, event: Event) -> (State, Vec<Command>) {
    let commands = match event {
        Event::Internal(event) => internal::reduce(&mut state, event),
        Event::Terminal(event) => terminal::reduce(&mut state, event),
    };

    (state, commands)
}
