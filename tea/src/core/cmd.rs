//! [`Cmd`]: how a program asks for work it cannot do itself.

use crate::core::World;

// region: Cmd
#[must_use = "a Cmd does nothing unless it is handed back to the runtime"]
pub struct Cmd<W: World, Msg> {
    effects: Vec<Effect<W, Msg>>,
}

impl<W: World, Msg> Cmd<W, Msg> {
    /// Nothing to do.
    pub fn none() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Every command in `cmds`, in no particular order.
    pub fn batch(cmds: impl IntoIterator<Item = Self>) -> Self {
        Self {
            effects: cmds.into_iter().flat_map(Self::into_effects).collect(),
        }
    }

    /// A message with no work behind it: hand it straight back to `update`.
    pub fn msg(msg: Msg) -> Self {
        Self {
            effects: vec![Effect::Msg(msg)],
        }
    }

    /// Run `f` on the worker thread; what it returns becomes a message.
    pub fn task<F>(f: F) -> Self
    where
        F: FnOnce(&mut W::Ctx) -> Msg + Send + 'static,
        Msg: 'static,
    {
        Self {
            effects: vec![Effect::Task(Box::new(move |ctx| Some(f(ctx))))],
        }
    }

    /// Like [`task`](Self::task), for work that may have nothing to report.
    pub fn attempt<F>(f: F) -> Self
    where
        F: FnOnce(&mut W::Ctx) -> Option<Msg> + Send + 'static,
        Msg: 'static,
    {
        Self {
            effects: vec![Effect::Task(Box::new(f))],
        }
    }

    pub fn quit() -> Self {
        Self::runtime(RuntimeEffect::Quit)
    }

    pub fn request_route(route: W::Route) -> Self {
        Self::runtime(RuntimeEffect::RequestRoute(route))
    }

    pub(crate) fn push_route(route: W::Route) -> Self {
        Self::runtime(RuntimeEffect::PushRoute(route))
    }

    pub(crate) fn back() -> Self {
        Self::runtime(RuntimeEffect::Back)
    }

    pub(crate) fn forward() -> Self {
        Self::runtime(RuntimeEffect::Forward)
    }

    /// Unwrap for the runtime, the only thing allowed to run these.
    pub(crate) fn into_effects(self) -> Vec<Effect<W, Msg>> {
        self.effects
    }

    /// Wrap a runtime effect on its own.
    fn runtime(effect: RuntimeEffect<W::Route>) -> Self {
        Self {
            effects: vec![Effect::Runtime(effect)],
        }
    }
}

impl<W: World, X: 'static> Cmd<W, X> {
    /// Relabel a child's messages as its parent's — Elm's `Cmd.map`.
    pub fn map<Y: 'static>(self, f: fn(X) -> Y) -> Cmd<W, Y> {
        let effects = self
            .effects
            .into_iter()
            .map(|effect| match effect {
                Effect::Msg(msg) => Effect::Msg(f(msg)),
                Effect::Task(task) => Effect::Task(Box::new(move |ctx| task(ctx).map(f))),
                // Carries no message, so there is nothing to relabel.
                Effect::Runtime(effect) => Effect::Runtime(effect),
            })
            .collect();

        Cmd { effects }
    }
}
// endregion

// region: Effects
/// A blocking job, owned so that it can cross to the worker thread.
pub(crate) type Job<W, Msg> =
    Box<dyn FnOnce(&mut <W as World>::Ctx) -> Option<Msg> + Send + 'static>;

/// One unit of work inside a [`Cmd`].
pub(crate) enum Effect<W: World, Msg> {
    /// Feed a message straight back into `update` on the next turn.
    Msg(Msg),
    /// Run a blocking job away from the UI thread.
    Task(Job<W, Msg>),
    /// Ask the runtime itself to act.
    Runtime(RuntimeEffect<W::Route>),
}

/// Work only the runtime can do, because it concerns the window rather than
/// the model.
pub(crate) enum RuntimeEffect<P> {
    Quit,
    RequestRoute(P),
    PushRoute(P),
    Back,
    Forward,
}
// endregion
