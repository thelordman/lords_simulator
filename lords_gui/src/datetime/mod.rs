use std::fmt::Display;

#[derive(Default)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl From<lords_sim::Time> for DateTime {
    fn from(time: lords_sim::Time) -> Self {
        Self {
            date: Date::from(time),
            time: Time::from(time),
        }
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.date, self.time)
    }
}

pub struct Date {
    year: u64,
    month: u8,
    day: u8,
}

impl Default for Date {
    fn default() -> Self {
        Date {
            year: 1,
            month: 1,
            day: 1,
        }
    }
}

impl From<lords_sim::Time> for Date {
    fn from(time: lords_sim::Time) -> Self {
        Self {
            year: time.year(),
            month: time.month(),
            day: time.day(),
        }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Default)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

impl From<lords_sim::Time> for Time {
    fn from(time: lords_sim::Time) -> Self {
        Self {
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{:02}:{:02}:{:02}Z", self.hour, self.minute, self.second)
    }
}