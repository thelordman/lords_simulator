const DAYS_IN_REGULAR_YEAR: u64 = 365;
const DAYS_IN_LEAP_YEAR: u64 = 366;

const DAYS_IN_FOUR_YEARS: u64 = DAYS_IN_REGULAR_YEAR * 3 + DAYS_IN_LEAP_YEAR;

const LEAP_YEARS_IN_CENTURY: u64 = 24;
const DAYS_IN_CENTURY: u64 =
    DAYS_IN_LEAP_YEAR * LEAP_YEARS_IN_CENTURY + DAYS_IN_REGULAR_YEAR * (100 - LEAP_YEARS_IN_CENTURY);

/// Number of leap years in a 400-year Gregorian calendar (solar) cycle.
///
/// According to the rules of the Gregorian calendar, a year is a leap year if it is divisible by 4,
/// but not by 100 unless it is also divisible by 400.
/// Because of this, calendar cycles repeat completely every 400 years, where each cycle has the exact same number of days.
///
/// Such a cycle has:
/// - 100 years divisible by 4 (400 / 4 = 100)
/// - Minus 3 years divisible by 100 but not 400 (100, 200, 300)
///
/// This amounts to (400 / 4) - 3 = 97 leap years.
const LEAP_YEARS_IN_CALENDAR_CYCLE: u64 = (400 / 4) - 3;

/// Total number of days in a 400-year Gregorian calendar cycle.
const DAYS_IN_CALENDAR_CYCLE: u64 =
    DAYS_IN_LEAP_YEAR * LEAP_YEARS_IN_CALENDAR_CYCLE + DAYS_IN_REGULAR_YEAR * (400 - LEAP_YEARS_IN_CALENDAR_CYCLE);

const SECONDS_IN_DAY: u64 = 24 * 60 * 60;

/// Converts a given point in time from seconds to full years according to the rules of the Gregorian calendar.
///
/// This function takes into account leap years in the way they are defined by the Gregorian calendar,
/// with the epoch being the instant before the first second (0 seconds) of year 1 (0 years).
///
/// Note that leap seconds are not considered, essentially meaning this function
pub(super) const fn years_from_seconds(seconds: u64) -> u64 {
    let (years, _days) = years_and_days_from_seconds(seconds);

    years
}

pub(super) const fn years_and_days_from_seconds(seconds: u64) -> (u64, u16) {

    let days = seconds / SECONDS_IN_DAY;

    let calendar_cycles = days / DAYS_IN_CALENDAR_CYCLE;
    let mut remaining_days = days % DAYS_IN_CALENDAR_CYCLE;

    let centuries = (remaining_days / DAYS_IN_CENTURY) - (remaining_days / (DAYS_IN_CENTURY * 4));
    remaining_days -= centuries * DAYS_IN_CENTURY;

    let four_year_periods = remaining_days / DAYS_IN_FOUR_YEARS;
    remaining_days %= DAYS_IN_FOUR_YEARS;

    let years = (remaining_days / DAYS_IN_REGULAR_YEAR) - (remaining_days / (DAYS_IN_REGULAR_YEAR * 4));
    remaining_days -= years * DAYS_IN_REGULAR_YEAR;
    assert!(remaining_days <= DAYS_IN_LEAP_YEAR - 1);

    let total_years = calendar_cycles * 400 + centuries * 100 + four_year_periods * 4 + years;

    (total_years, remaining_days as u16) // Safe cast because of the assertion
}

pub(super) const fn is_leap_year(years: u64) -> bool {
    let display = years + 1;
    (display % 4 == 0 && display % 100 != 0) || display % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_leap_year(years: u64) -> bool {
        let display = years + 1;
        (display % 4 == 0 && display % 100 != 0) || display % 400 == 0
    }

    fn days_in_year(years: u64) -> u64 {
        if is_leap_year(years) { 366 } else { 365 }
    }

    fn first_second_of_year(years: u64) -> u64 {
        (0..years).map(days_in_year).sum::<u64>() * SECONDS_IN_DAY
    }

    fn last_second_of_year(years: u64) -> u64 {
        first_second_of_year(years + 1) - 1
    }

    fn assert_years(years: u64) {
        assert_eq!(
            years_from_seconds(first_second_of_year(years)),
            years,
            "first second of year {years}"
        );
        assert_eq!(
            years_from_seconds(last_second_of_year(years)),
            years,
            "last second of year {years}"
        );
    }

    #[test]
    fn test_year_zero() {
        assert_years(0);
    }

    #[test]
    fn test_first_leap_year() {
        assert_years(2);
        assert_years(3);
        assert_years(4);
    }

    #[test]
    fn test_century() {
        assert_years(98);
        assert_years(99);
        assert_years(100);
        assert_years(101);
    }

    #[test]
    fn test_year_400() {
        assert_years(397);
        assert_years(398);
        assert_years(399);
        assert_years(400);
    }

    #[test]
    fn test_second_cycle() {
        assert_years(403);
        assert_years(499);
        assert_years(500);
        assert_years(799);
        assert_years(800);
    }
}