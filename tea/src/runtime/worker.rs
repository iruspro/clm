//! The one thread that runs the work a [`Cmd`](crate::Cmd) asks for.

use std::sync::mpsc;
use std::thread;

/// A job queued on the [`Worker`].
type Job<Ctx> = Box<dyn FnOnce(&mut Ctx) + Send + 'static>;

/// The one thread that runs [`Cmd`](crate::Cmd) tasks.
pub(crate) struct Worker<Ctx> {
    jobs: mpsc::Sender<Job<Ctx>>,
}

impl<Ctx: Send + 'static> Worker<Ctx> {
    /// Takes ownership of `ctx` and starts running jobs against it.
    pub(crate) fn spawn(mut ctx: Ctx) -> Self {
        let (jobs, receiver) = mpsc::channel::<Job<Ctx>>();

        thread::spawn(move || {
            for job in receiver {
                job(&mut ctx);
            }
        });

        Self { jobs }
    }

    pub(crate) fn run(&self, job: impl FnOnce(&mut Ctx) + Send + 'static) {
        self.jobs.send(Box::new(job)).expect("failed to send job");
    }
}
