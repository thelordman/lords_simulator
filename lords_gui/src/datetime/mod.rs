//! Writing simulation time out the way the player prefers.

use std::fmt::{self, Display, Formatter, Write};

use lords_sim::Timestamp;
use lords_sim::time::gregorian::{Date, TimeOfDay};
use serde::{Deserialize, Serialize};

/// How dates and times are written out.
///
/// The default is `DD.MM.YYYY HH:MM:SS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DateTimeFormat {
    pub date_order: DateOrder,
    pub date_separator: DateSeparator,
    pub time_order: TimeOrder,
    pub time_separator: TimeSeparator,
}

impl DateTimeFormat {
    /// Formats the date `timestamp` falls on, with the month and day padded to two digits
    /// and the year to four, so that the result keeps its width from one day to the next.
    pub fn date(&self, timestamp: Timestamp) -> String {
        let date = Date::from(timestamp);
        let (year, month, day) = (date.year(), date.month(), date.day());
        let sep = self.date_separator;

        match self.date_order {
            DateOrder::Dmy => format!("{day:02}{sep}{month:02}{sep}{year:04}"),
            DateOrder::Mdy => format!("{month:02}{sep}{day:02}{sep}{year:04}"),
            DateOrder::Ymd => format!("{year:04}{sep}{month:02}{sep}{day:02}"),
        }
    }

    /// Formats the time of day at `timestamp`.
    pub fn time(&self, timestamp: Timestamp) -> String {
        let time = TimeOfDay::from(timestamp);
        let (hour, minute, second) = (time.hour(), time.minute(), time.second());
        let sep = self.time_separator;

        match self.time_order {
            TimeOrder::Hms => format!("{hour:02}{sep}{minute:02}{sep}{second:02}"),
        }
    }

    /// Formats the date and the time of day at `timestamp`.
    pub fn date_time(&self, timestamp: Timestamp) -> String {
        format!("{} {}", self.date(timestamp), self.time(timestamp))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DateOrder {
    #[default]
    Dmy,
    Mdy,
    Ymd,
}

impl DateOrder {
    pub const ALL: [Self; 3] = [Self::Dmy, Self::Mdy, Self::Ymd];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dmy => "Day, month, year",
            Self::Mdy => "Month, day, year",
            Self::Ymd => "Year, month, day",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DateSeparator {
    Slash,
    #[default]
    Period,
    Hyphen,
}

impl DateSeparator {
    pub const ALL: [Self; 3] = [Self::Period, Self::Slash, Self::Hyphen];

    pub fn label(self) -> &'static str {
        match self {
            Self::Slash => "Slash  /",
            Self::Period => "Period  .",
            Self::Hyphen => "Hyphen  -",
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeOrder {
    #[default]
    Hms,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeSeparator {
    Period,
    #[default]
    Colon,
}

impl TimeSeparator {
    pub const ALL: [Self; 2] = [Self::Colon, Self::Period];

    pub fn label(self) -> &'static str {
        match self {
            Self::Period => "Period  .",
            Self::Colon => "Colon  :",
        }
    }
}

impl Display for TimeSeparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Self::Period => '.',
            Self::Colon => ':',
        })
    }
}

/// A moment whose date reads unambiguously in every [`DateOrder`]: 31 December 2024, one
/// second to midnight.
pub const SAMPLE: Timestamp = Timestamp::from_seconds(739_250 * 86_400 + 86_399);

#[cfg(test)]
mod tests {
    use super::*;

    /// 9 March of year 7, at 04:05:06.
    const EARLY: Timestamp =
        Timestamp::from_seconds((6 * 365 + 1 + 31 + 28 + 8) * 86_400 + 4 * 3_600 + 5 * 60 + 6);

    #[test]
    fn default_format_is_day_first_with_periods_and_colons() {
        let format = DateTimeFormat::default();

        assert_eq!(format.date_time(SAMPLE), "31.12.2024 23:59:59");
        assert_eq!(format.date(SAMPLE), "31.12.2024");
        assert_eq!(format.time(SAMPLE), "23:59:59");
    }

    #[test]
    fn every_part_is_padded_to_a_fixed_width() {
        let format = DateTimeFormat::default();

        assert_eq!(format.date_time(EARLY), "09.03.0007 04:05:06");
        assert_eq!(format.date_time(Timestamp::EPOCH), "01.01.0001 00:00:00");
        assert_eq!(
            format.date_time(EARLY).len(),
            format.date_time(SAMPLE).len()
        );
    }

    #[test]
    fn date_follows_the_chosen_order_and_separator() {
        let format = |date_order, date_separator| DateTimeFormat {
            date_order,
            date_separator,
            ..DateTimeFormat::default()
        };

        assert_eq!(
            format(DateOrder::Dmy, DateSeparator::Slash).date(SAMPLE),
            "31/12/2024"
        );
        assert_eq!(
            format(DateOrder::Mdy, DateSeparator::Slash).date(SAMPLE),
            "12/31/2024"
        );
        assert_eq!(
            format(DateOrder::Ymd, DateSeparator::Hyphen).date(SAMPLE),
            "2024-12-31"
        );
    }

    #[test]
    fn time_follows_the_chosen_separator() {
        let format = DateTimeFormat {
            time_separator: TimeSeparator::Period,
            ..DateTimeFormat::default()
        };

        assert_eq!(format.time(SAMPLE), "23.59.59");
        assert_eq!(format.date_time(SAMPLE), "31.12.2024 23.59.59");
    }
}
