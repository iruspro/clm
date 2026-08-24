//! The outermost layer: the terminal, the worker thread, and the loop.

mod terminal;
mod worker;

use std::sync::mpsc;
use std::time::Duration;

use self::terminal::Terminal;
use self::worker::Worker;
use crate::browser::Program;
use crate::core::cmd::{Effect, RuntimeEffect};
use crate::core::events::{Event, EventHandler, InternalEvent};
use crate::core::window::Window;
use crate::core::{Cmd, ElmModel, Result, World};

/// Drive `program` until it quits.
pub fn run<W: World, M: ElmModel<W>>(
    title: impl Into<String>,
    ctx: W::Ctx,
    program: Program<W, M>,
    tick_rate: Duration,
) -> Result<()> {
    let mut runtime = Runtime::new(title.into(), ctx, program, tick_rate)?;

    terminal::enter(&mut runtime.terminal)?;
    // Hold on to the result: the terminal has to be handed back either way.
    let result = runtime.main_loop();
    terminal::exit(&mut runtime.terminal)?;

    result
}

/// Representation of a runtime system.
///
/// It owns the three things that cannot own each other: the terminal, the
/// window, and the model. The window draws its chrome and answers about its
/// own keys; the model does everything else; this joins them up.
struct Runtime<W: World, M: ElmModel<W>> {
    terminal: Terminal,
    events: EventHandler<M::Msg>,
    events_sender: mpsc::Sender<Event<M::Msg>>,
    worker: Worker<W::Ctx>,
    /// The chrome: title, location bar, status line, and the key buffer.
    window: Window<W::Route>,
    model: M,
    program: Program<W, M>,
}

impl<W: World, M: ElmModel<W>> Runtime<W, M> {
    fn new(
        title: String,
        ctx: W::Ctx,
        program: Program<W, M>,
        tick_rate: Duration,
    ) -> Result<Self> {
        let events = EventHandler::new(tick_rate);
        let (model, cmd) = (program.init)(program.start);
        let window = Window::new(title, program.start);

        let mut runtime = Self {
            terminal: terminal::open()?,
            events_sender: events.sender(),
            events,
            worker: Worker::spawn(ctx),
            window,
            model,
            program,
        };
        runtime.dispatch(cmd);

        Ok(runtime)
    }

    // region: Loop
    fn main_loop(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            let event = self.events.next()?;
            let cmd = match event {
                Event::Internal(InternalEvent::Quit) => break,
                Event::Internal(InternalEvent::Msg(msg)) => self.model.update(msg),
                Event::Publisher(publisher) => self.model.subscriptions(publisher),
            };

            self.dispatch(cmd);
        }

        Ok(())
    }
    // endregion

    // region: Cmd dispatcher
    /// Hand a command's effects to whoever can run them.
    fn dispatch(&mut self, cmd: Cmd<W, M::Msg>) {
        for effect in cmd.into_effects() {
            match effect {
                Effect::Msg(msg) => self.send(InternalEvent::Msg(msg)),
                Effect::Task(task) => {
                    let events = mpsc::Sender::clone(&self.events_sender);
                    self.worker.run(move |ctx| {
                        if let Some(msg) = task(ctx) {
                            events
                                .send(InternalEvent::Msg(msg).into())
                                .expect("failed to send message");
                        }
                    });
                }
                Effect::Runtime(RuntimeEffect::Quit) => self.send(InternalEvent::Quit),
                Effect::Runtime(RuntimeEffect::RequestRoute(route)) => {
                    let msg = (self.program.on_route_request)(route);
                    self.send(InternalEvent::Msg(msg));
                }
                Effect::Runtime(RuntimeEffect::PushRoute(route)) => {
                    self.window.push_route(route);
                    self.notify_route_change();
                }
                Effect::Runtime(RuntimeEffect::Back) => {
                    if self.window.back() {
                        self.notify_route_change();
                    }
                }
                Effect::Runtime(RuntimeEffect::Forward) => {
                    if self.window.forward() {
                        self.notify_route_change();
                    }
                }
            }
        }
    }

    /// Tell the model where the window now is. Routing is its business.
    fn notify_route_change(&self) {
        let msg = (self.program.on_route_change)(*self.window.route());
        self.send(InternalEvent::Msg(msg));
    }

    fn send(&self, event: impl Into<Event<M::Msg>>) {
        self.events_sender
            .send(event.into())
            .expect("failed to send event");
    }
    // endregion

    /// Draw the window's chrome, then the model into what is left.
    fn draw(&mut self) -> Result<()> {
        let window = &self.window;
        let model = &self.model;

        self.terminal.draw(|frame| {
            let page_area = window.view(frame);
            model.view(frame, page_area);
        })?;

        Ok(())
    }
}
