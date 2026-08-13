//! **Composition root & shared state.**
//!
//! [`App`] constructs the channels, the [`Actor`] and the [`Tui`], then runs the
//! central event loop that ties them together. This module also hosts the pieces
//! that loop operates on: the parsed [`Config`], the [`State`] tree (accounts,
//! categories, transactions, modals), the [`Event`] type both input sources feed
//! into, the [`Command`]s the reducer emits, the [`composition`] module that
//! builds use cases over a connection, and the [`CancellationToken`] used for a
//! clean shutdown.

pub mod command;
pub mod composition;
pub mod config;
mod event;
pub mod state;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

use color_eyre::eyre::Result;

pub use self::command::Command;
pub use self::config::Config;
pub use self::event::Event;
pub use self::state::{AccountsState, CategoriesState, State, TransactionsState};
use crate::actor::Actor;
use crate::reducer;
use crate::tui::Tui;

// region: App
/// The composition root: owns every long-lived piece and drives the event loop.
///
/// Both event sources — the [`Tui`]'s input listener and the [`Actor`]'s worker
/// — hold a clone of one [`mpsc::Sender<Event>`](mpsc::Sender), and `App` holds
/// the single receiver. That makes the loop a simple `recv` over one merged
/// stream regardless of how many producers there are.
#[derive(Debug)]
pub struct App {
    /// Receiving end of the merged event stream; every producer holds a clone
    /// of the matching sender.
    rx: mpsc::Receiver<Event>,
    /// Shared shutdown flag, observed by the input listener.
    cancellation_token: Arc<CancellationToken>,

    config: Config,
    actor: Actor,
    tui: Tui,
}

impl App {
    /// Wires up the event channel, the [`Actor`] and the [`Tui`] from `config`.
    ///
    /// The terminal is *not* taken over yet — that happens in [`run`](Self::run)
    /// — so a failure here still leaves the terminal usable.
    ///
    /// # Errors
    /// Returns an error if the terminal backend cannot be created.
    pub fn new(config: Config) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<Event>();
        let cancellation_token = Arc::new(CancellationToken::new());

        let actor = Actor::new(config.database_url, tx.clone());
        let tui = Tui::new(tx.clone(), Arc::clone(&cancellation_token))?;

        Ok(App {
            rx,
            cancellation_token,
            config,
            actor,
            tui,
        })
    }

    /// Takes over the terminal, draws the first frame and runs the event loop
    /// until the state asks to quit or the channel closes.
    ///
    /// Consumes `self`: teardown runs on **every** exit path, including an early
    /// error out of the loop, so the terminal is always restored and both
    /// threads are always joined.
    ///
    /// # Errors
    /// Propagates the first terminal error from entering the alternate screen or
    /// drawing a frame; teardown still happens first.
    pub fn run(mut self) -> Result<()> {
        let state = self.init_state();
        self.tui.enter()?;
        self.tui.draw(&state)?;

        let result = self.event_loop(state);

        // Tear down on every exit path — including an early error from
        // `event_loop` — so the listener is always cancelled and joined.
        self.quit();

        result
    }

    /// The initial [`State`], seeded with the configured title and key bindings.
    fn init_state(&self) -> State {
        State::new(self.config.title, self.config.keys)
    }

    /// The TEA loop itself: block on the next [`Event`], fold it into the state
    /// with [`reducer::reduce`], dispatch whatever [`Command`]s came back, and
    /// redraw.
    ///
    /// Redrawing happens *after* the quit check, so no frame is painted on the
    /// way out. `recv` returning `Err` means every sender is gone, which also
    /// ends the loop.
    fn event_loop(&mut self, mut state: State) -> Result<()> {
        while let Ok(event) = self.rx.recv() {
            let commands;
            (state, commands) = reducer::reduce(state, event);

            for cmd in commands {
                self.actor.dispatch(cmd);
            }

            if state.should_quit() {
                break;
            }

            self.tui.draw(&state)?;
        }

        Ok(())
    }

    /// Signals cancellation and joins both threads.
    ///
    /// # Panics
    /// Panics if either the worker or the input listener panicked.
    fn quit(self) {
        self.cancellation_token.cancel();

        // Join the workers, then the input listener. Ordering
        // between the two is free of deadlock — the Event channel is unbounded,
        // so neither side blocks the other on the way out.
        self.actor.shutdown();
        self.tui.exit();
    }
}
// endregion

// region: Cancellation token
/// A one-way "please stop" flag shared with the background threads.
///
/// Wrapped in an [`Arc`] and handed to the [`Tui`]'s input listener, which
/// cannot simply block on a channel — it is parked in a terminal `poll` — and so
/// checks this flag between polls instead.
///
/// [`Ordering::Relaxed`] is enough: the flag carries no data with it, and the
/// only guarantee needed is that the store eventually becomes visible. The
/// actual happens-before edge for shutdown comes from joining the threads.
#[derive(Debug, Default)]
pub struct CancellationToken {
    cancelled: AtomicBool,
}

impl CancellationToken {
    /// A fresh, un-cancelled token.
    fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
        }
    }

    /// Whether [`cancel`](Self::cancel) has been called.
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Sets the flag. Idempotent, and never unset — cancellation is one-way.
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
// endregion
