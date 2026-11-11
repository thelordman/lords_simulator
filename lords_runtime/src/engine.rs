use std::time::{Duration, Instant};

use lords_sim::{Command, Simulation};

use crate::clock::Clock;
use crate::snapshot::Snapshot;
use crate::speed::Speed;

/// A request from the [`Runtime`](crate::Runtime) handle to the simulation thread.
#[derive(Debug)]
pub(crate) enum Request {
    Pause,
    Resume,
    SetSpeed(Speed),
    Command(Command),
}

pub(crate) struct Engine {
    sim: Simulation,
    clock: Clock,
    speed: Speed,
}

impl Engine {
    pub(crate) fn new(sim: Simulation, speed: Speed, now: Instant) -> Self {
        Self {
            sim,
            clock: Clock::starting_at(now, speed.tick_interval()),
            speed,
        }
    }

    pub(crate) fn handle(&mut self, request: Request, now: Instant) {
        match request {
            Request::Pause => self.clock.pause(),
            Request::Resume => self.clock.resume(now),
            Request::SetSpeed(speed) => {
                self.speed = speed;
                self.clock.set_interval(speed.tick_interval(), now);
            }
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
            speed: self.speed,
        }
    }
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex, Timestamp};

    use super::*;
    use crate::speed::BASE_INTERVAL;

    const INTERVAL: Duration = BASE_INTERVAL;

    fn engine(start: Instant) -> Engine {
        Engine::new(
            Simulation::new(Name::default(), Sex::Female),
            Speed::BASE,
            start,
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

    #[test]
    fn speed_change_is_published_and_shortens_ticks() {
        let start = Instant::now();
        let mut engine = engine(start);
        let fast = Speed::new(10);
        let interval = fast.tick_interval();

        engine.handle(Request::SetSpeed(fast), start);

        assert_eq!(engine.snapshot().speed, fast);
        assert!(engine.advance(start + interval));
        assert!(engine.advance(start + interval * 2));
        assert_eq!(engine.snapshot().state.time, after_ticks(2));
    }
}
