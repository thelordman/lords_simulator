use std::fmt::{self, Display, Formatter, Write};

use lords_sim::Timestamp;
use lords_sim::time::gregorian::{Date, TimeOfDay};

/// Combined date and time representation.
pub struct DateTime {
    date: Date,
    time: TimeOfDay,
}

impl From<Timestamp> for DateTime {
    fn from(timestamp: Timestamp) -> Self {
        Self {
            date: Date::from(timestamp),
            time: TimeOfDay::from(timestamp),
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
            format_date(self.date, date_order, date_sep),
            format_time(self.time, time_order, time_sep)
        )
    }

    /// Format using default settings:
    /// `DD.MM.YY HH:MM:SS`
    pub fn format_default(&self) -> String {
        self.format(
            DateOrder::Dmy,
            DateSeparator::Period,
            TimeOrder::Hms,
            TimeSeparator::Colon,
        )
    }
}

fn format_date(date: Date, order: DateOrder, sep: DateSeparator) -> String {
    let (year, month, day) = (date.year(), date.month(), date.day());

    match order {
        DateOrder::Dmy => format!("{day}{sep}{month}{sep}{year}"),
        DateOrder::Mdy => format!("{month}{sep}{day}{sep}{year}"),
        DateOrder::Ymd => format!("{year}{sep}{month}{sep}{day}"),
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

fn format_time(time: TimeOfDay, order: TimeOrder, sep: TimeSeparator) -> String {
    let (hour, minute, second) = (time.hour(), time.minute(), time.second());

    match order {
        TimeOrder::Hms => format!("{hour:02}{sep}{minute:02}{sep}{second:02}"),
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
