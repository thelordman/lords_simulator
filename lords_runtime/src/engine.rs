use std::time::{Duration, Instant};

use lords_sim::{Command, Simulation};

use crate::clock::Clock;
use crate::snapshot::Snapshot;

/// A request from the [`Runtime`](crate::Runtime) handle to the simulation thread.
#[derive(Debug)]
pub(crate) enum Request {
    Pause,
    Resume,
    Command(Command),
}

pub(crate) struct Engine {
    sim: Simulation,
    clock: Clock,
}

impl Engine {
    pub(crate) fn new(sim: Simulation, clock: Clock) -> Self {
        Self { sim, clock }
    }

    pub(crate) fn handle(&mut self, request: Request, now: Instant) {
        match request {
            Request::Pause => self.clock.pause(),
            Request::Resume => self.clock.resume(now),
            Request::Command(command) => self.sim.apply(command),
        }
    }

    /// Runs the tick that is due at `now`, if there is one, and returns whether it did.
    pub(crate) fn advance(&mut self, now: Instant) -> bool {
        let due = self.clock.take_due_tick(now);
        if due {
            self.sim.tick();
        }
        due
    }

    /// Returns the moment the next tick is due, or `None` while paused.
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.clock.deadline()
    }

    pub(crate) fn tick_interval(&self) -> Duration {
        self.clock.interval()
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            state: self.sim.state().clone(),
            paused: self.clock.is_paused(),
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Timestamp};

    use super::*;

    const INTERVAL: Duration = Duration::from_millis(100);

    fn engine(start: Instant) -> Engine {
        Engine::new(
            Simulation::new(Name::default(), Sex::Female),
            Clock::starting_at(start, INTERVAL),
        )
    }

    fn after_ticks(ticks: u32) -> Timestamp {
        let mut time = Timestamp::EPOCH;
        for _ in 0..ticks {
            time.advance();
        }
        time
    }

    #[test]
    fn due_ticks_advance_simulation_time() {
        let start = Instant::now();
        let mut engine = engine(start);

        assert!(!engine.advance(start));
        assert!(engine.advance(start + INTERVAL));
        assert!(engine.advance(start + INTERVAL * 2));

        assert_eq!(engine.snapshot().state.time, after_ticks(2));
    }

    #[test]
    fn pausing_stops_time_until_resumed() {
        let start = Instant::now();
        let mut engine = engine(start);

        engine.handle(Request::Pause, start);
        assert!(!engine.advance(start + INTERVAL * 5));
        assert!(engine.snapshot().paused);
        assert_eq!(engine.snapshot().state.time, Timestamp::EPOCH);

        engine.handle(Request::Resume, start + INTERVAL * 5);
        assert!(engine.advance(start + INTERVAL * 6));
        assert!(!engine.snapshot().paused);
        assert_eq!(engine.snapshot().state.time, after_ticks(1));
    }

    #[test]
    fn commands_applied_while_paused() {
        let start = Instant::now();
        let mut engine = engine(start);
        engine.advance(start + INTERVAL);
        engine.handle(Request::Pause, start + INTERVAL);

        engine.handle(Request::Command(Command::Reset), start + INTERVAL);

        assert!(engine.snapshot().paused);
        assert_eq!(engine.snapshot().state.time, Timestamp::EPOCH);
    }

    #[test]
    fn requests_do_not_move_deadline() {
        let start = Instant::now();
        let mut engine = engine(start);

        engine.handle(Request::Command(Command::Reset), start + INTERVAL / 2);
        engine.handle(Request::Resume, start + INTERVAL / 2);

        assert_eq!(engine.deadline(), Some(start + INTERVAL));
    }
}
