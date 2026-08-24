use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

use crate::core::Result;

// region: Events
/// Everything the main loop can wake up for.
#[derive(Debug, Clone, PartialEq)]
pub enum Event<Msg> {
    Internal(InternalEvent<Msg>),
    Publisher(Publisher),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InternalEvent<Msg> {
    /// Leave the event loop.
    Quit,
    /// A message on its way to `update`.
    Msg(Msg),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Publisher {
    /// Input from the terminal.
    Terminal(TerminalEvent),
}

/// Terminal events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalEvent {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

impl<Msg> From<InternalEvent<Msg>> for Event<Msg> {
    fn from(value: InternalEvent<Msg>) -> Self {
        Self::Internal(value)
    }
}

impl<Msg> From<Publisher> for Event<Msg> {
    fn from(value: Publisher) -> Self {
        Self::Publisher(value)
    }
}

impl<Msg> From<TerminalEvent> for Event<Msg> {
    fn from(value: TerminalEvent) -> Self {
        Publisher::Terminal(value).into()
    }
}
// endregion

// region: Events handler
/// Event handler.
pub(crate) struct EventHandler<Msg> {
    /// Event sender channel, cloned for anything that reports back.
    sender: mpsc::Sender<Event<Msg>>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event<Msg>>,
    /// Event handler threads.
    #[expect(
        dead_code,
        reason = "the loop never joins these; it dies with the process"
    )]
    handler: JoinHandle<()>,
}

impl<Msg: Send + 'static> EventHandler<Msg> {
    pub(crate) fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let terminal_event_sender = mpsc::Sender::clone(&sender);

        let handler =
            thread::spawn(move || terminal_event_handler(terminal_event_sender, tick_rate));

        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// A handle the runtime posts messages back through.
    pub(crate) fn sender(&self) -> mpsc::Sender<Event<Msg>> {
        mpsc::Sender::clone(&self.sender)
    }

    /// Receive the next event from the handler threads.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent.
    pub(crate) fn next(&self) -> Result<Event<Msg>> {
        Ok(self.receiver.recv()?)
    }
}

fn terminal_event_handler<Msg>(sender: mpsc::Sender<Event<Msg>>, tick_rate: Duration) {
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(tick_rate);

        if event::poll(timeout).expect("unable to poll for event") {
            match event::read().expect("unable to read event") {
                CrosstermEvent::Key(e) => {
                    if e.kind == event::KeyEventKind::Press {
                        sender.send(TerminalEvent::Key(e).into())
                    } else {
                        Ok(()) // ignore KeyEventKind::Release on windows
                    }
                }
                CrosstermEvent::Resize(w, h) => sender.send(TerminalEvent::Resize(w, h).into()),
                _ => Ok(()),
            }
            .expect("failed to send terminal event");
        }

        if last_tick.elapsed() >= tick_rate {
            sender
                .send(TerminalEvent::Tick.into())
                .expect("failed to send tick event");
            last_tick = Instant::now();
        }
    }
}
// endregion
