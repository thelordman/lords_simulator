use crate::time::month::months_and_days_from_seconds;

pub(super) const fn days_from_seconds(seconds: u64) -> u8 {
    let (_months, days) = months_and_days_from_seconds(seconds);

    days
}