use std::time::Duration;

use lords_sim::time::TICKS_PER_SECOND;

/// The tick interval at which simulation time passes as fast as real time.
pub(crate) const BASE_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND);

/// How fast simulation time passes, as a multiple of real time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Speed(u32);

impl Speed {
    pub const BASE: Self = Self(1);

    pub const MAX: Self = Self(100);

    /// Returns the speed `multiplier` times the base time, clamped to between
    /// [`Self::BASE`] and [`Self::MAX`].
    pub fn new(multiplier: u32) -> Self {
        Self(multiplier.clamp(Self::BASE.0, Self::MAX.0))
    }

    pub fn multiplier(self) -> u32 {
        self.0
    }

    pub(crate) fn tick_interval(self) -> Duration {
        BASE_INTERVAL / self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplier_is_clamped_to_supported_range() {
        assert_eq!(Speed::new(0), Speed::BASE);
        assert_eq!(Speed::new(u32::MAX), Speed::MAX);
        assert_eq!(Speed::new(10).multiplier(), 10);
    }

    #[test]
    fn faster_speed_has_shorter_tick_interval() {
        assert_eq!(Speed::BASE.tick_interval(), BASE_INTERVAL);
        assert_eq!(Speed::new(10).tick_interval(), BASE_INTERVAL / 10);
    }
}
