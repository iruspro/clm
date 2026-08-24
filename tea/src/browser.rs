//! The window a program runs in, and the [`Program`] that fills it.

pub mod navigation;

use crate::core::cmd::Cmd;
use crate::core::{ElmModel, World};

/// Builds the model a route opens at, and the work that fills it.
pub type Init<W, M> = fn(<W as World>::Route) -> (M, Cmd<W, <M as ElmModel<W>>::Msg>);

/// Turns a route the window has reached into a message for the model.
pub type OnRoute<W, M> = fn(<W as World>::Route) -> <M as ElmModel<W>>::Msg;

/// Everything the runtime needs to know about a program, as a value.
pub struct Program<W: World, M: ElmModel<W>> {
    /// The route the window opens at, and the one `init` builds a model for.
    pub(crate) start: W::Route,
    pub(crate) init: Init<W, M>,
    pub(crate) on_route_request: OnRoute<W, M>,
    pub(crate) on_route_change: OnRoute<W, M>,
}

impl<W: World, M: ElmModel<W>> Program<W, M> {
    /// Collects the four things the runtime needs to start a program.
    pub fn new(
        start: W::Route,
        init: Init<W, M>,
        on_route_request: OnRoute<W, M>,
        on_route_change: OnRoute<W, M>,
    ) -> Self {
        Self {
            start,
            init,
            on_route_request,
            on_route_change,
        }
    }
}
