//! **Effects / database access** (TEA "commands").
//!
//! Owns the [`Actor`]: a background worker thread that holds the SQLite
//! connection and performs every read and write on it. The reducer emits
//! [`Command`]s, the actor runs them serially through the application-layer use
//! cases and views, and reports each outcome back into the loop as an
//! [`InternalEvent`]. All blocking I/O therefore happens off the UI thread.

mod event;

use std::sync::mpsc;
use std::thread;

use application::views::accounts;
use application::{Error, views};
use rusqlite::Connection;

pub use self::event::InternalEvent;
use crate::app::command::RequestId;
use crate::app::{Command, Event, composition};
/// Owns the database side of the app, off the UI thread.
///
/// A single **command worker** thread owns the connection and the use cases and
/// applies commands serially, so it is the only writer and writes never race.
/// [`dispatch`](Self::dispatch) hands work to that thread without blocking the
/// caller, and results come back asynchronously on the [`Event`] channel — the
/// UI thread never waits on the database.
#[derive(Debug)]
pub struct Actor {
    /// Queue feeding the command worker. Dropping it ends the worker's loop.
    commands: mpsc::Sender<Command>,

    /// Join handle kept so [`shutdown`](Self::shutdown) can wait for the worker
    /// to finish rather than leaving it detached.
    command_worker: thread::JoinHandle<()>,
}

impl Actor {
    /// Spawns the command worker against `database_url` and returns a handle to
    /// it. The worker reports its results on `events`.
    ///
    /// The connection is opened on the worker thread, so this call itself does
    /// no I/O; a connection failure panics the worker instead, surfacing at
    /// [`shutdown`](Self::shutdown).
    pub fn new(database_url: &'static str, events: mpsc::Sender<Event>) -> Self {
        let (command_tx, command_rx) = mpsc::channel::<Command>();

        // Command worker: the single writer. Owns the use cases and applies one
        // command at a time.
        let command_worker =
            thread::spawn(move || command_worker(database_url, command_rx, events));

        Self {
            commands: command_tx,
            command_worker,
        }
    }

    /// Hands a command to the workers without blocking the UI thread.
    pub fn dispatch(&self, cmd: Command) {
        let _ = self.commands.send(cmd);
    }

    /// Shuts the worker down cleanly, blocking until its thread has joined.
    ///
    /// The order is forced by the channel graph: the worker's loop only ends
    /// once *every* [`Command`] sender is gone, so this drops the sender before
    /// joining. Joining first would deadlock — the worker would still be parked
    /// in `recv`, waiting for messages that can no longer come.
    ///
    /// Takes `self` by value, so an [`Actor`] cannot be used after shutdown.
    ///
    /// # Panics
    /// Panics if the worker thread panicked.
    pub fn shutdown(self) {
        let Self {
            commands,
            command_worker,
        } = self;

        drop(commands);
        command_worker
            .join()
            .expect("command worker thread panicked");
    }
}

/// The command worker loop: sole owner of the connection and the only writer.
///
/// `for cmd in commands` blocks while idle and ends once every [`Command`]
/// sender has been dropped — i.e. once the [`Actor`] is shut down — giving a
/// clean exit without a poison pill message.
///
/// Send failures are ignored throughout: a closed [`Event`] channel means the
/// app is already tearing down, and there is nowhere left to report to.
///
/// # Panics
/// Panics if the SQLite connection cannot be opened.
fn command_worker(
    database_url: &'static str,
    commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
) {
    let conn = Connection::open(database_url).expect("cannot open a connection to SQLite DB");

    for cmd in commands {
        match cmd {
            // region: Write operations
            Command::CreateAccount(cmd, rid) => {
                match composition::create_account_use_case(&conn).execute(cmd) {
                    Ok(aid) => {
                        let _ =
                            events.send(Event::Internal(InternalEvent::AccountCreated(aid, rid)));
                    }
                    Err(err) => send_write_error(err, rid, &events),
                };
            }
            // endregion
            // region: Read operations
            Command::LoadAccounts => {
                let filters = accounts::Filters {
                    kinds: Some(vec![0, 1, 2]),
                };
                let by = accounts::By::Name;
                let order = views::Order::Asc;
                match accounts::view(&conn, filters, by, order) {
                    Ok(accounts) => {
                        let _ =
                            events.send(Event::Internal(InternalEvent::AccountsLoaded(accounts)));
                    }
                    Err(err) => {
                        let _ = events.send(Event::Internal(InternalEvent::ReadRequestError(
                            err.to_string(),
                        )));
                    }
                }
            } // endregion
        }
    }
}

/// Reports a failed write, tagged with the [`RequestId`] of the request that
/// caused it so the reducer can re-open the form the user submitted.
fn send_write_error(err: Error, rid: RequestId, events: &mpsc::Sender<Event>) {
    let _ = events.send(Event::Internal(InternalEvent::WriteRequestError(
        err.to_string(),
        rid,
    )));
}

/// Reports a failed read. Reads carry no [`RequestId`] — nothing needs to be
/// restored afterwards, so the message alone is enough.
fn send_read_error(err: Error, events: &mpsc::Sender<Event>) {
    let _ = events.send(Event::Internal(InternalEvent::ReadRequestError(
        err.to_string(),
    )));
}
