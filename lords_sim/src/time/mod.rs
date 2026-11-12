//! Simulation time: the absolute tick counter and the calendars that interpret it.

pub mod gregorian;

use serde::{Deserialize, Serialize};

/// Number of ticks per in-game second.
pub const TICKS_PER_SECOND: u64 = 1;

/// A point in simulation time, counted in ticks since [`Timestamp::EPOCH`].
///
/// A timestamp is absolute and not tied to any planet or calendar. See [`gregorian`] for
/// interpreting it as a date and a time of day.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Timestamp {
    ticks: u64,
}

impl Timestamp {
    /// The start of the simulation.
    pub const EPOCH: Self = Self { ticks: 0 };

    /// Returns the timestamp `seconds` in-game seconds after [`Self::EPOCH`].
    pub const fn from_seconds(seconds: u64) -> Self {
        Self {
            ticks: seconds * TICKS_PER_SECOND,
        }
    }

    /// Returns the number of whole in-game seconds elapsed since [`Self::EPOCH`].
    pub const fn as_seconds(self) -> u64 {
        self.ticks / TICKS_PER_SECOND
    }

    /// Returns the number of whole in-game seconds from `earlier` to this timestamp.
    ///
    /// If `earlier` is not actually earlier, the result is zero.
    pub const fn seconds_since(self, earlier: Self) -> u64 {
        self.ticks.saturating_sub(earlier.ticks) / TICKS_PER_SECOND
    }

    /// Moves this timestamp forward by one tick.
    pub const fn advance(&mut self) {
        self.ticks += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_since_counts_forward_only() {
        let earlier = Timestamp::from_seconds(100);
        let later = Timestamp::from_seconds(250);

        assert_eq!(later.seconds_since(earlier), 150);
        assert_eq!(later.seconds_since(later), 0);
        assert_eq!(earlier.seconds_since(later), 0);
    }
}
