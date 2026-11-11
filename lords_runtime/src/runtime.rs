use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvError, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, warn};
use lords_sim::{Command, Simulation};

use crate::clock::{Clock, BASE_INTERVAL};
use crate::engine::{Engine, Request};
use crate::snapshot::{Published, Snapshot};

/// How long before a tick is due the simulation thread stops listening for requests and
/// sleeps instead.
const SLEEP_MARGIN: Duration = Duration::from_millis(20);

/// A handle to a simulation running on its own thread.
pub struct Runtime {
    requests: Sender<Request>,
    published: Arc<Published>,
}

impl Runtime {
    /// Moves `sim` to a new thread and starts ticking it.
    pub fn spawn(sim: Simulation, on_publish: impl Fn() + Send + 'static) -> Self {
        let clock = Clock::starting_at(Instant::now(), BASE_INTERVAL);
        let engine = Engine::new(sim, clock);
        let published = Arc::new(Published::new(engine.snapshot()));
        let (requests, received) = mpsc::channel();

        let thread_published = Arc::clone(&published);
        thread::Builder::new()
            .name("simulation".to_owned())
            .spawn(move || run(engine, &received, &thread_published, on_publish))
            .expect("failed to spawn the simulation thread");

        Self {
            requests,
            published,
        }
    }

    /// Returns the simulation as it was last published.
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.published.get()
    }

    pub fn pause(&self) {
        self.request(Request::Pause);
    }

    pub fn resume(&self) {
        self.request(Request::Resume);
    }

    /// Asks the simulation to apply `command`, which it does before its next tick, or at
    /// once while paused.
    pub fn send(&self, command: Command) {
        self.request(Request::Command(command));
    }

    fn request(&self, request: Request) {
        if self.requests.send(request).is_err() {
            error!("The simulation thread has stopped; a request to it was dropped"); // I love rhymes.
        }
    }
}

fn run(
    mut engine: Engine,
    requests: &Receiver<Request>,
    published: &Published,
    on_publish: impl Fn(),
) {
    while let Ok(request) = wait(requests, engine.deadline()) {
        let now = Instant::now();
        if let Some(request) = request {
            engine.handle(request, now);
        }
        let ticked = engine.advance(now);

        published.set(engine.snapshot());
        on_publish();

        let spent = now.elapsed();
        if ticked && spent > engine.tick_interval() {
            warn!(
                "Tick took {} ms longer than the tick interval",
                (spent - engine.tick_interval()).as_millis()
            );
        }
    }
}

/// Blocks until a request arrives or `deadline` is reached, returning `None` for the latter.
///
/// Fails once the [`Runtime`] has been dropped.
fn wait(
    requests: &Receiver<Request>,
    deadline: Option<Instant>,
) -> Result<Option<Request>, RecvError> {
    let Some(deadline) = deadline else {
        return requests.recv().map(Some);
    };

    // According to testing, channel timeout can be late (about 15 ms on Windows)
    // while `thread::sleep` is around millisecond precision,
    // so we put it to sleep for a little bit in the end.
    let listening = deadline
        .saturating_duration_since(Instant::now())
        .saturating_sub(SLEEP_MARGIN);
    match requests.recv_timeout(listening) {
        Ok(request) => Ok(Some(request)),
        Err(RecvTimeoutError::Disconnected) => Err(RecvError),
        Err(RecvTimeoutError::Timeout) => {
            thread::sleep(deadline.saturating_duration_since(Instant::now()));
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex};

    use super::*;

    const TIMEOUT: Duration = Duration::from_secs(10);

    fn spawn() -> (Runtime, Receiver<()>) {
        let (publishes, published) = mpsc::channel();
        let runtime = Runtime::spawn(Simulation::new(Name::default(), Sex::Female), move || {
            let _ = publishes.send(());
        });
        (runtime, published)
    }

    #[test]
    fn pausing_is_published_without_waiting_for_a_tick() {
        let (runtime, published) = spawn();
        assert!(!runtime.snapshot().paused);

        runtime.pause();

        while !runtime.snapshot().paused {
            published
                .recv_timeout(TIMEOUT)
                .expect("the pause was never published");
        }
    }

    #[test]
    fn dropping_the_handle_stops_the_thread() {
        let (runtime, published) = spawn();

        drop(runtime);

        let stopped = loop {
            match published.recv_timeout(TIMEOUT) {
                Ok(()) => {}
                Err(error) => break error,
            }
        };
        assert_eq!(stopped, RecvTimeoutError::Disconnected);
    }
}
