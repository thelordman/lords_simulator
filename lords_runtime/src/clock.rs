use std::time::{Duration, Instant};

use lords_sim::time::TICKS_PER_SECOND;

/// The tick interval at which simulation time passes as fast as real time.
pub(crate) const BASE_INTERVAL: Duration =
    Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND);

/// Decides when the next tick is due.
#[derive(Debug)]
pub(crate) struct Clock {
    interval: Duration,
    /// `None` while paused.
    next_tick: Option<Instant>,
}

impl Clock {
    /// Returns a running clock whose first tick is due one `interval` after `now`.
    pub(crate) fn starting_at(now: Instant, interval: Duration) -> Self {
        Self {
            interval,
            next_tick: Some(now + interval),
        }
    }

    pub(crate) fn interval(&self) -> Duration {
        self.interval
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.next_tick.is_none()
    }

    /// Returns the moment the next tick is due, or `None` while paused.
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.next_tick
    }

    pub(crate) fn pause(&mut self) {
        self.next_tick = None;
    }

    /// Resumes a paused clock, making the next tick due one interval after `now`.
    ///
    /// Does nothing if the clock is already running.
    pub(crate) fn resume(&mut self, now: Instant) {
        self.next_tick.get_or_insert(now + self.interval);
    }

    /// Returns whether a tick is due at `now`, and if so starts the wait for the next one.
    pub(crate) fn take_due_tick(&mut self, now: Instant) -> bool {
        match self.next_tick {
            Some(due) if due <= now => {
                self.next_tick = Some(now + self.interval);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTERVAL: Duration = Duration::from_millis(100);
    const MOMENT: Duration = Duration::from_millis(1);

    #[test]
    fn tick_is_due_once_interval_has_passed() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        assert!(!clock.take_due_tick(start + INTERVAL - MOMENT));
        assert!(clock.take_due_tick(start + INTERVAL));
    }

    #[test]
    fn tick_is_taken_only_once() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        assert!(clock.take_due_tick(start + INTERVAL));
        assert!(!clock.take_due_tick(start + INTERVAL));
    }

    #[test]
    fn late_tick_is_followed_by_full_interval() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);
        let late = start + INTERVAL * 5 / 2;

        assert!(clock.take_due_tick(late));

        assert_eq!(clock.deadline(), Some(late + INTERVAL));
        assert!(!clock.take_due_tick(late + INTERVAL - MOMENT));
        assert!(clock.take_due_tick(late + INTERVAL));
    }

    #[test]
    fn paused_clock_has_no_tick_due() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.pause();

        assert!(clock.is_paused());
        assert_eq!(clock.deadline(), None);
        assert!(!clock.take_due_tick(start + INTERVAL * 10));
    }

    #[test]
    fn resuming_waits_full_interval() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);
        let resumed = start + INTERVAL * 10;

        clock.pause();
        clock.resume(resumed);

        assert!(!clock.is_paused());
        assert!(!clock.take_due_tick(resumed + INTERVAL - MOMENT));
        assert!(clock.take_due_tick(resumed + INTERVAL));
    }

    #[test]
    fn resuming_running_clock_keeps_deadline() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.resume(start + INTERVAL / 2);

        assert_eq!(clock.deadline(), Some(start + INTERVAL));
    }
}
