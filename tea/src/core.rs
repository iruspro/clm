//! Single-page terminal applications in the Elm architecture.

pub mod cmd;
pub(crate) mod events;
pub(crate) mod window;

use std::fmt::Display;

use ratatui::Frame;
use ratatui::layout::Rect;

pub use crate::core::cmd::Cmd;
pub use crate::core::events::{Publisher, TerminalEvent};

/// The types a program is built from.
pub trait World: Sized + 'static {
    type Route: Display + Clone + Copy + PartialEq;

    /// What a [`Cmd`]'s tasks may reach — a database handle, an http client,
    /// a configuration.
    type Ctx: Send + 'static;
}

/// A model: some state, the messages that change it, and the three functions
/// that fold, draw and listen.
pub trait ElmModel<W: World>: Sized {
    /// The messages this model produces and consumes.
    type Msg: Send + 'static;

    /// Fold a message into the model, returning work for the runtime to do.
    fn update(&mut self, msg: Self::Msg) -> Cmd<W, Self::Msg>;

    fn view(&self, frame: &mut Frame, area: Rect);

    /// Say which cmd an event means here, or `None` to ignore it.
    fn subscriptions(&self, publisher: Publisher) -> Cmd<W, Self::Msg>;
}

/// Something went wrong in the runtime itself.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The terminal could not be set up, drawn to, or handed back.
    #[error("terminal: {0}")]
    Terminal(#[from] std::io::Error),

    /// Every sender is gone, so no further event can ever arrive.
    #[error("the event loop has nothing left to listen to")]
    Disconnected(#[from] std::sync::mpsc::RecvError),
}

/// A [`Result`](std::result::Result) whose error defaults to this crate's
/// [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
