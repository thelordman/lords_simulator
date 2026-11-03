use std::fmt::{self, Display, Formatter, Write};

/// Combined date and time representation.
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

impl DateTime {
    /// Format with specific date and time ordering and separators.
    pub fn format(
        &self,
        date_order: DateOrder,
        date_sep: DateSeparator,
        time_order: TimeOrder,
        time_sep: TimeSeparator,
    ) -> String {
        format!(
            "{} {}",
            self.date.format(date_order, date_sep),
            self.time.format(time_order, time_sep)
        )
    }

    /// Format using default settings:
    /// DMY with periods, HMS with colons.
    pub fn format_default(&self) -> String {
        self.format(
            DateOrder::Dmy,
            DateSeparator::Period,
            TimeOrder::Hms,
            TimeSeparator::Colon,
        )
    }
}

pub struct Date {
    year: u64,
    month: u8,
    day: u8,
}

impl Default for Date {
    fn default() -> Self {
        Self {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Date {
    pub fn format(&self, order: DateOrder, sep: DateSeparator) -> String {
        let (a, b, c) = match order {
            DateOrder::Dmy => (self.day as u64, self.month as u64, self.year),
            DateOrder::Mdy => (self.month as u64, self.day as u64, self.year),
            DateOrder::Ymd => (self.year, self.month as u64, self.day as u64),
        };
        format!("{a}{sep}{b}{sep}{c}")
    }
}

#[derive(Clone, Copy)]
pub enum DateOrder {
    Dmy,
    Mdy,
    Ymd,
}

#[derive(Clone, Copy)]
pub enum DateSeparator {
    Slash,
    Period,
    Hyphen,
}

impl Display for DateSeparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Self::Slash => '/',
            Self::Period => '.',
            Self::Hyphen => '-',
        })
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

impl Time {
    pub fn format(&self, order: TimeOrder, sep: TimeSeparator) -> String {
        match order {
            TimeOrder::Hms => {
                format!("{:02}{sep}{:02}{sep}{:02}", self.hour, self.minute, self.second)
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum TimeOrder {
    Hms,
}

#[derive(Clone, Copy)]
pub enum TimeSeparator {
    Period,
    Colon,
}

impl Display for TimeSeparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Self::Period => '.',
            Self::Colon => ':',
        })
    }
}
