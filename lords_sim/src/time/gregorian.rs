//! The proleptic Gregorian calendar and the 24-hour clock.
//!
//! [`Timestamp::EPOCH`] is midnight at the start of 1 January of year 1. Leap seconds do not
//! exist.

use std::fmt::{self, Display, Formatter};

use crate::time::Timestamp;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

const DAYS_PER_COMMON_YEAR: u64 = 365;

/// Days in four consecutive years of which the last is a leap year.
const DAYS_PER_FOUR_YEARS: u64 = 4 * DAYS_PER_COMMON_YEAR + 1;

/// Days in a century whose last year is not a leap year.
const DAYS_PER_COMMON_CENTURY: u64 = 25 * DAYS_PER_FOUR_YEARS - 1;

/// Days in the 400-year cycle after which the calendar repeats exactly.
///
/// Only the last century of a cycle ends in a leap year.
const DAYS_PER_CYCLE: u64 = 4 * DAYS_PER_COMMON_CENTURY + 1;

/// Days of a common year preceding each month, followed by the length of the year.
const DAYS_BEFORE_MONTH_COMMON: [u64; 13] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

/// Days of a leap year preceding each month, followed by the length of the year.
const DAYS_BEFORE_MONTH_LEAP: [u64; 13] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

/// Returns whether `year` has a 29 February.
///
/// A year is a leap year if it is divisible by 4, unless it is divisible by 100 but not by 400.
pub const fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

const fn at_most(value: u64, limit: u64) -> u64 {
    if value > limit { limit } else { value }
}

/// A calendar date, with the year, month, and day of month all counted from 1.
///
/// Dates order chronologically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u64,
    month: u8,
    day: u8,
}

impl Date {
    /// Returns the date on which `timestamp` falls.
    pub const fn from_timestamp(timestamp: Timestamp) -> Self {
        Self::from_days(timestamp.as_seconds() / SECONDS_PER_DAY)
    }

    /// Returns the date `days` whole days after 1 January of year 1.
    const fn from_days(days: u64) -> Self {
        let cycles = days / DAYS_PER_CYCLE;
        let day_of_cycle = days % DAYS_PER_CYCLE;

        let centuries = at_most(day_of_cycle / DAYS_PER_COMMON_CENTURY, 3);
        let day_of_century = day_of_cycle - centuries * DAYS_PER_COMMON_CENTURY;

        let four_year_periods = day_of_century / DAYS_PER_FOUR_YEARS;
        let day_of_four_years = day_of_century % DAYS_PER_FOUR_YEARS;

        let years = at_most(day_of_four_years / DAYS_PER_COMMON_YEAR, 3);
        let day_of_year = day_of_four_years - years * DAYS_PER_COMMON_YEAR;

        let year = 400 * cycles + 100 * centuries + 4 * four_year_periods + years + 1;

        let days_before_month = if is_leap_year(year) {
            &DAYS_BEFORE_MONTH_LEAP
        } else {
            &DAYS_BEFORE_MONTH_COMMON
        };

        let mut month_index = 0;
        while day_of_year >= days_before_month[month_index + 1] {
            month_index += 1;
        }
        let day_of_month = day_of_year - days_before_month[month_index];

        Self {
            year,
            month: month_index as u8 + 1,
            day: day_of_month as u8 + 1,
        }
    }

    pub const fn year(self) -> u64 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }

    /// Returns the number of full years from `earlier` to this date, the way an age is counted.
    ///
    /// The anniversary of a 29 February falls on 1 March in common years. If `earlier` is
    /// not actually earlier, the result is zero.
    pub const fn years_since(self, earlier: Self) -> u64 {
        let anniversary_reached =
            self.month > earlier.month || (self.month == earlier.month && self.day >= earlier.day);
        let calendar_years = self.year.saturating_sub(earlier.year);

        if anniversary_reached {
            calendar_years
        } else {
            calendar_years.saturating_sub(1)
        }
    }

    /// Returns the number of full months from `earlier` to this date.
    pub const fn months_since(self, earlier: Self) -> u64 {
        if self.year < earlier.year {
            return 0;
        }

        let calendar_months = ((self.year - earlier.year) * 12 + self.month as u64)
            .saturating_sub(earlier.month as u64);

        if self.day >= earlier.day {
            calendar_months
        } else {
            calendar_months.saturating_sub(1)
        }
    }
}

impl From<Timestamp> for Date {
    fn from(timestamp: Timestamp) -> Self {
        Self::from_timestamp(timestamp)
    }
}

/// Formats the date as `YYYY-MM-DD`.
impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A time of day on the 24-hour clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOfDay {
    /// Returns the time of day at `timestamp`.
    pub const fn from_timestamp(timestamp: Timestamp) -> Self {
        let second_of_day = timestamp.as_seconds() % SECONDS_PER_DAY;

        Self {
            hour: (second_of_day / SECONDS_PER_HOUR) as u8,
            minute: (second_of_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE) as u8,
            second: (second_of_day % SECONDS_PER_MINUTE) as u8,
        }
    }

    pub const fn hour(self) -> u8 {
        self.hour
    }

    pub const fn minute(self) -> u8 {
        self.minute
    }

    pub const fn second(self) -> u8 {
        self.second
    }
}

impl From<Timestamp> for TimeOfDay {
    fn from(timestamp: Timestamp) -> Self {
        Self::from_timestamp(timestamp)
    }
}

/// Formats the time as `HH:MM:SS`.
impl Display for TimeOfDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn date(year: u64, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }

    const fn time_of_day(hour: u8, minute: u8, second: u8) -> TimeOfDay {
        TimeOfDay {
            hour,
            minute,
            second,
        }
    }

    fn day_after(date: Date) -> Date {
        let days_in_month = match date.month {
            2 if is_leap_year(date.year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };

        if date.day < days_in_month {
            Date {
                day: date.day + 1,
                ..date
            }
        } else if date.month < 12 {
            Date {
                month: date.month + 1,
                day: 1,
                ..date
            }
        } else {
            Date {
                year: date.year + 1,
                month: 1,
                day: 1,
            }
        }
    }

    #[test]
    fn leap_years_follow_gregorian_rule() {
        assert!(!is_leap_year(1));
        assert!(is_leap_year(4));
        assert!(!is_leap_year(100));
        assert!(is_leap_year(400));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }

    #[test]
    fn known_days_convert_to_known_dates() {
        let known_dates = [
            (0, date(1, 1, 1)),
            (58, date(1, 2, 28)),
            (59, date(1, 3, 1)),
            (364, date(1, 12, 31)),
            (365, date(2, 1, 1)),
            (1_154, date(4, 2, 29)),
            (1_155, date(4, 3, 1)),
            (36_217, date(100, 2, 28)),
            (36_218, date(100, 3, 1)),
            (36_523, date(100, 12, 31)),
            (36_524, date(101, 1, 1)),
            (145_790, date(400, 2, 29)),
            (146_096, date(400, 12, 31)),
            (146_097, date(401, 1, 1)),
            (719_162, date(1970, 1, 1)),
            (730_119, date(2000, 1, 1)),
        ];

        for (days, expected) in known_dates {
            assert_eq!(Date::from_days(days), expected, "day {days}");
        }
    }

    #[test]
    fn every_day_follows_the_previous_one() {
        let mut previous = Date::from_days(0);

        for days in 1..=DAYS_PER_CYCLE + DAYS_PER_COMMON_YEAR {
            let current = Date::from_days(days);
            assert_eq!(current, day_after(previous), "day {days}");
            previous = current;
        }
    }

    #[test]
    fn date_changes_at_midnight() {
        let last_second_of_first_day = Timestamp::from_seconds(86_399);
        let first_second_of_second_day = Timestamp::from_seconds(86_400);

        assert_eq!(Date::from(last_second_of_first_day), date(1, 1, 1));
        assert_eq!(Date::from(first_second_of_second_day), date(1, 1, 2));
    }

    #[test]
    fn time_of_day_rolls_over_each_unit() {
        let known_times = [
            (0, time_of_day(0, 0, 0)),
            (59, time_of_day(0, 0, 59)),
            (60, time_of_day(0, 1, 0)),
            (3_599, time_of_day(0, 59, 59)),
            (3_600, time_of_day(1, 0, 0)),
            (86_399, time_of_day(23, 59, 59)),
            (86_400, time_of_day(0, 0, 0)),
        ];

        for (seconds, expected) in known_times {
            let timestamp = Timestamp::from_seconds(seconds);
            assert_eq!(TimeOfDay::from(timestamp), expected, "second {seconds}");
        }
    }

    #[test]
    fn years_since_counts_only_completed_years() {
        let birth = date(10, 6, 15);

        assert_eq!(date(10, 6, 15).years_since(birth), 0);
        assert_eq!(date(11, 6, 14).years_since(birth), 0);
        assert_eq!(date(11, 6, 15).years_since(birth), 1);
        assert_eq!(date(30, 5, 31).years_since(birth), 19);
        assert_eq!(date(30, 7, 1).years_since(birth), 20);
        assert_eq!(date(9, 12, 31).years_since(birth), 0);

        let leap_day_birth = date(4, 2, 29);

        assert_eq!(date(5, 2, 28).years_since(leap_day_birth), 0);
        assert_eq!(date(5, 3, 1).years_since(leap_day_birth), 1);
        assert_eq!(date(8, 2, 29).years_since(leap_day_birth), 4);
    }

    #[test]
    fn months_since_counts_only_completed_months() {
        let birth = date(10, 6, 15);

        assert_eq!(date(10, 6, 15).months_since(birth), 0);
        assert_eq!(date(10, 7, 14).months_since(birth), 0);
        assert_eq!(date(10, 7, 15).months_since(birth), 1);
        assert_eq!(date(11, 1, 15).months_since(birth), 7);
        assert_eq!(date(11, 6, 14).months_since(birth), 11);
        assert_eq!(date(11, 6, 15).months_since(birth), 12);
        assert_eq!(date(10, 5, 31).months_since(birth), 0);
        assert_eq!(date(9, 12, 31).months_since(birth), 0);

        let end_of_month_birth = date(10, 1, 31);

        assert_eq!(date(10, 2, 28).months_since(end_of_month_birth), 0);
        assert_eq!(date(10, 3, 1).months_since(end_of_month_birth), 1);
        assert_eq!(date(10, 3, 31).months_since(end_of_month_birth), 2);
    }

    #[test]
    fn twelve_months_make_year() {
        let birth = date(4, 2, 29);
        let mut day = birth;

        for _ in 0..3 * DAYS_PER_FOUR_YEARS {
            assert_eq!(
                day.months_since(birth) / 12,
                day.years_since(birth),
                "{day}"
            );
            day = day_after(day);
        }
    }
}
