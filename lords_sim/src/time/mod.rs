mod year;
mod month;
mod day;

/// Number of ticks per in-game second.
pub const TICKS_PER_SECOND: u64 = 1;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Time {
    tick: u64
}

impl Time {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    pub fn zero(&mut self) {
        self.tick = 0;
    }

    pub const fn seconds(self) -> u64 {
        self.tick / TICKS_PER_SECOND
    }

    pub const fn minutes(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60)
    }

    pub const fn hours(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60)
    }

    pub const fn days(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24)
    }

    pub const fn day(self) -> u8 {
        day::days_from_seconds(self.tick / TICKS_PER_SECOND) + 1
    }

    pub const fn month(self) -> u8 {
        month::months_from_seconds(self.tick / TICKS_PER_SECOND) + 1
    }

    pub const fn years(self) -> u64 {
        year::years_from_seconds(self.tick / TICKS_PER_SECOND)
    }

    pub const fn year(self) -> u64 {
        self.years() + 1
    }
}