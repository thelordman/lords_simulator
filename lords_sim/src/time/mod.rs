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

    pub fn seconds(self) -> u64 {
        self.tick / TICKS_PER_SECOND
    }

    pub fn minutes(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60)
    }

    pub fn hours(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60)
    }

    pub fn days(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24)
    }

    pub fn weeks(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 7)
    }

    // TODO: Proper month system
    pub fn months(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 30)
    }

    // TODO: Leap years
    pub fn years(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 365)
    }
}