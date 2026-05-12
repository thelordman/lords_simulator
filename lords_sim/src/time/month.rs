use crate::time::year::{is_leap_year, years_and_days_from_seconds};

const REGULAR_YEAR_MONTHS: [u16; 12] = [31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const LEAP_YEAR_MONTHS: [u16; 12] = [31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

pub(super) const fn months_from_seconds(seconds: u64) -> u8 {
    let (months, _days) = months_and_days_from_seconds(seconds);

    months
}

pub(super) const fn months_and_days_from_seconds(seconds: u64) -> (u8, u8) {
    let (years, days) = years_and_days_from_seconds(seconds);

    let year_months = if is_leap_year(years) { LEAP_YEAR_MONTHS } else { REGULAR_YEAR_MONTHS };

    let mut months = 0;

    // TODO: Binary search for O(log n) time complexity (low priority)
    while months < 12 && days >= year_months[months] {
        months += 1;
    }

    let days = if months == 0 { days } else { days - year_months[months - 1] };
    assert!(days <= 31);

    (months as u8, days as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECONDS_IN_DAY: u64 = 24 * 60 * 60;

    fn is_leap_year(years: u64) -> bool {
        let display = years + 1;
        (display % 4 == 0 && display % 100 != 0) || display % 400 == 0
    }

    fn days_in_month(years: u64, months: u8) -> u64 {
        match months {
            0 => 31,
            1 => if is_leap_year(years) { 29 } else { 28 },
            2 => 31,
            3 => 30,
            4 => 31,
            5 => 30,
            6 => 31,
            7 => 31,
            8 => 30,
            9 => 31,
            10 => 30,
            11 => 31,
            _ => panic!("invalid month {months}"),
        }
    }

    fn first_second_of_month(years: u64, months: u8) -> u64 {
        let days_in_prior_years: u64 = (0..years)
            .map(|y| if is_leap_year(y) { 366u64 } else { 365u64 })
            .sum();
        let days_in_prior_months: u64 = (0..months)
            .map(|m| days_in_month(years, m))
            .sum();
        (days_in_prior_years + days_in_prior_months) * SECONDS_IN_DAY
    }

    fn last_second_of_month(years: u64, months: u8) -> u64 {
        first_second_of_month(years, months) + days_in_month(years, months) * SECONDS_IN_DAY - 1
    }

    fn assert_month(years: u64, months: u8) {
        assert_eq!(
            months_from_seconds(first_second_of_month(years, months)),
            months,
            "first second of month {months} in year {years}"
        );
        assert_eq!(
            months_from_seconds(last_second_of_month(years, months)),
            months,
            "last second of month {months} in year {years}"
        );
    }

    #[test]
    fn test_all_months_regular_year() {
        for month in 0..12 {
            assert_month(0, month);
        }
    }

    #[test]
    fn test_all_months_leap_year() {
        for month in 0..12 {
            assert_month(3, month);
        }
    }

    #[test]
    fn test_february_boundary() {
        assert_month(0, 1);
        assert_month(0, 2);
        assert_month(3, 1);
        assert_month(3, 2);
    }

    #[test]
    fn test_century_year() {
        for month in 0..12 {
            assert_month(99, month);
        }
    }

    #[test]
    fn test_leap_century_year() {
        for month in 0..12 {
            assert_month(399, month);
        }
    }
}