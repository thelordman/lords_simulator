use crate::time::TICKS_PER_SECOND;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Duration {
    ticks: u64
}

impl Duration {
    pub fn new(ticks: u64) -> Self {
        Self { ticks }
    }
    
    pub fn from_seconds(seconds: u64) -> Self {
        Self::new(seconds * TICKS_PER_SECOND)
    }
    
    pub fn from_minutes(minutes: u64) -> Self {
        Self::new(minutes * TICKS_PER_SECOND * 60)
    }
    
    pub fn from_hours(hours: u64) -> Self {
        Self::new(hours * TICKS_PER_SECOND * 60 * 60)
    }
    
    pub fn from_days(days: u64) -> Self {
        Self::new(days * TICKS_PER_SECOND * 60 * 60 * 24)
    }
    
    pub fn from_months(months: u64) -> Self {
        Self::new(months * TICKS_PER_SECOND * 60 * 60 * 24 * 30)
    }
    
    pub fn from_years(years: u64) -> Self {
        Self::new(years * TICKS_PER_SECOND * 60 * 60 * 24 * 365)
    }
    
    pub const fn seconds(self) -> u64 {
        self.ticks / TICKS_PER_SECOND
    }
    
    pub const fn minutes(self) -> u64 {
        self.ticks / (TICKS_PER_SECOND * 60)
    }
    
    pub const fn hours(self) -> u64 {
        self.ticks / (TICKS_PER_SECOND * 60 * 60)
    }
    
    pub const fn days(self) -> u64 {
        self.ticks / (TICKS_PER_SECOND * 60 * 60 * 24)
    }
    
    /// Approximation as 30 days per month.
    pub const fn months(self) -> u64 {
        self.ticks / (TICKS_PER_SECOND * 60 * 60 * 24 * 30)
    }
    
    /// Approximation as 365 days per year, leap years not accounted for.
    pub const fn years(self) -> u64 {
        self.ticks / (TICKS_PER_SECOND * 60 * 60 * 24 * 365)
    }
}