use std::fmt::{Display, Formatter, Write};

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

impl DateTime {
    pub fn format(&self, date_order: DateOrder, date_separator: DateSeparator, time_order: TimeOrder, time_separator: TimeSeparator) -> String {
        format!("{} {}", self.date.format(date_order, date_separator), self.time.format(time_order, time_separator))
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

pub enum DateOrder {
    DMY,
    MDY,
    YMD,
}

pub enum DateSeparator {
    Slash,
    Period,
    Hyphen,
}

impl Display for DateSeparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateSeparator::Slash => f.write_char('/'),
            DateSeparator::Period => f.write_char('.'),
            DateSeparator::Hyphen => f.write_char('-'),
        }
    }
}

impl Date {
    pub fn format(&self, order: DateOrder, separator: DateSeparator) -> String {
        match order {
            DateOrder::DMY => format!("{}{separator}{}{separator}{}", self.day, self.month, self.year),
            DateOrder::MDY => format!("{}{separator}{}{separator}{}", self.month, self.day, self.year),
            DateOrder::YMD => format!("{}{separator}{}{separator}{}", self.year, self.month, self.day),
        }
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

pub enum TimeOrder {
    HMS,
}

pub enum TimeSeparator {
    Period,
    Colon,
}

impl Display for TimeSeparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSeparator::Period => f.write_char('.'),
            TimeSeparator::Colon => f.write_char(':'),
        }
    }
}

impl Time {
    pub fn format(&self, order: TimeOrder, separator: TimeSeparator) -> String {
        match order {
            TimeOrder::HMS => format!("{:02}{separator}{:02}{separator}{:02}", self.hour, self.minute, self.second)
        }
    }
}