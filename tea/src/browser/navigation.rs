//! Moving the window about, and closing it.

use crate::core::{Cmd, World};

/// Go to `route`, adding it to the history.
pub fn push_route<W: World, Msg>(route: W::Route) -> Cmd<W, Msg> {
    Cmd::push_route(route)
}

/// Step back through the history.
pub fn back<W: World, Msg>() -> Cmd<W, Msg> {
    Cmd::back()
}

/// Undo a step [`back`].
pub fn forward<W: World, Msg>() -> Cmd<W, Msg> {
    Cmd::forward()
}
