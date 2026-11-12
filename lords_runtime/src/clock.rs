use std::time::{Duration, Instant};

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

    /// Returns a clock that has no tick due until it is resumed.
    pub(crate) fn paused(interval: Duration) -> Self {
        Self {
            interval,
            next_tick: None,
        }
    }

    pub(crate) fn interval(&self) -> Duration {
        self.interval
    }

    /// Changes the interval between ticks.
    pub(crate) fn set_interval(&mut self, interval: Duration, now: Instant) {
        self.interval = interval;
        if let Some(due) = &mut self.next_tick {
            *due = (*due).min(now + interval);
        }
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
    fn clock_created_paused_waits_for_resume() {
        let start = Instant::now();
        let mut clock = Clock::paused(INTERVAL);

        assert!(clock.is_paused());
        assert!(!clock.take_due_tick(start + INTERVAL * 10));

        clock.resume(start);

        assert!(!clock.take_due_tick(start + INTERVAL - MOMENT));
        assert!(clock.take_due_tick(start + INTERVAL));
    }

    #[test]
    fn resuming_running_clock_keeps_deadline() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.resume(start + INTERVAL / 2);

        assert_eq!(clock.deadline(), Some(start + INTERVAL));
    }

    #[test]
    fn shorter_interval_pulls_deadline_in() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.set_interval(INTERVAL / 4, start + MOMENT);

        assert_eq!(clock.deadline(), Some(start + MOMENT + INTERVAL / 4));
    }

    #[test]
    fn longer_interval_applies_after_scheduled_tick() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.set_interval(INTERVAL * 4, start + MOMENT);

        assert_eq!(clock.deadline(), Some(start + INTERVAL));
        assert!(clock.take_due_tick(start + INTERVAL));
        assert_eq!(clock.deadline(), Some(start + INTERVAL * 5));
    }

    #[test]
    fn interval_set_while_paused_applies_on_resume() {
        let start = Instant::now();
        let mut clock = Clock::starting_at(start, INTERVAL);

        clock.pause();
        clock.set_interval(INTERVAL / 4, start);
        assert!(clock.is_paused());

        clock.resume(start);
        assert_eq!(clock.deadline(), Some(start + INTERVAL / 4));
    }
}
