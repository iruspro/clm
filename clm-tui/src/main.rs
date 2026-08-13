//! `clm-tui` — the terminal front end for the double-entry finance app.
//!
//! The binary is structured as **The Elm Architecture (TEA)**: a single
//! unidirectional loop where UI and I/O only ever *produce events* and *consume
//! state*, and all change funnels through one pure reducer. [`main`] just wires
//! the pieces together; [`App::run`](app::App::run) owns the loop itself.
//!
//! ```text
//!            Terminal input ─┐                                    ┌─ worker (Actor)
//!                            │─ Event ─ reduce(State, Event)      │
//!            Actor results ──┘                     └─ (State, Vec<Command>)
//!                                                         │
//!                                                    Tui::draw(State)
//! ```
//!
//! One [`reduce`](reducer::reduce) call takes the current [`State`](app::State)
//! plus an incoming [`Event`](app::Event) and returns the next `State` and any
//! [`Command`](app::Command)s to run. State is rendered; commands are dispatched
//! to the actor, whose results come back as new events. The reducer is pure — no
//! I/O, no threads — which keeps the state transitions easy to reason about and
//! test.
//!
//! # Module map
//!
//! | Module | TEA role | Responsibility |
//! | --- | --- | --- |
//! | [`app`] | model | composition root, the [`State`](app::State) tree, [`Config`] |
//! | [`reducer`] | update | the pure `(State, Event) -> (State, Vec<Command>)` step |
//! | [`tui`] | view | terminal lifecycle, the input listener, and the widgets |
//! | [`actor`] | commands | background worker that runs [`Command`](app::Command)s against the database |
//!
//! The schema itself lives outside the binary, in the [`db`] crate: [`main`]
//! calls [`db::reset`] once, before the loop starts and before the
//! [`Actor`] opens its own connection.
//!
//! Each module's own header documents it in detail. Note that the prose lives
//! there rather than on the `mod` declarations below: rustdoc resolves the links
//! in a merged doc comment against the *declaring* module, so an outer `///` here
//! plus an inner `//!` there would silently break every relative link.

mod actor;
mod app;
mod reducer;
mod tui;

use color_eyre::eyre::Result;
use rusqlite::Connection;

pub use crate::actor::Actor;
use crate::app::{App, Config};

/// Installs the terminal-restoring error hooks, prepares the database, then
/// hands control to [`App::run`].
///
/// The connection opened here is only used to set the schema up and is dropped
/// straight after; the [`Actor`] opens its own on the worker thread.
fn main() -> Result<()> {
    tui::install_hooks()?;

    let config = Config::parse();

    // Destructive: development only. Swap for `db::migrate` to keep the data.
    let conn = Connection::open(config.database_url)?;
    db::reset(&conn)?;
    drop(conn);

    let app = App::new(config)?;
    app.run()?;

    Ok(())
}
