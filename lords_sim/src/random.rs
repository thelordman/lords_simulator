use crate::space::Location;

pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn keyed(parts: &[u64]) -> Self {
        let mut rng = Self(0);
        for part in parts {
            rng.0 ^= part;
            rng.0 = rng.next();
        }
        rng
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// Returns a number from zero up to but not including `bound`.
    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        ((u128::from(self.next()) * u128::from(bound)) >> 64) as u64
    }

    /// Returns a number from `low` to `high` with both included.
    pub(crate) fn between(&mut self, (low, high): (i64, i64)) -> i64 {
        debug_assert!(low <= high);
        low + self.below((high - low + 1) as u64) as i64
    }

    /// Returns a number from `low` to `high` that is more often near `low`.
    pub(crate) fn mostly_low(&mut self, range: (i64, i64)) -> i64 {
        self.between(range).min(self.between(range))
    }

    pub(crate) fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    /// Returns a number from zero up to but not including one.
    pub(crate) fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}

/// Values from zero to one that drift smoothly from low to high over about `scale`
/// millimeters.
pub(crate) struct Noise {
    seed: u64,
    scale: f64,
}

impl Noise {
    pub(crate) fn new(rng: &mut Rng, scale: f64) -> Self {
        Self {
            seed: rng.next(),
            scale,
        }
    }

    pub(crate) fn at(&self, at: Location) -> f64 {
        let (x, y) = (at.x() as f64 / self.scale, at.y() as f64 / self.scale);
        let (column, row) = (x.floor(), y.floor());
        let ease = |share: f64| share * share * (3.0 - 2.0 * share);
        let (east, north) = (ease(x - column), ease(y - row));
        let corner = |east: i64, north: i64| {
            let parts = [
                self.seed,
                (column as i64 + east) as u64,
                (row as i64 + north) as u64,
            ];
            Rng::keyed(&parts).unit()
        };
        let between = |from: f64, to: f64, share: f64| from + (to - from) * share;

        let south = between(corner(0, 0), corner(1, 0), east);
        let north_side = between(corner(0, 1), corner(1, 1), east);
        between(south, north_side, north)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_stay_within_their_bounds() {
        let mut rng = Rng(42);

        for _ in 0..10_000 {
            assert!(rng.below(7) < 7);
            assert!((-3..=3).contains(&rng.between((-3, 3))));
            assert!((0.0..1.0).contains(&rng.unit()));
        }
    }

    #[test]
    fn keyed_numbers_change_with_every_part_of_the_key() {
        let first = |parts: &[u64]| Rng::keyed(parts).next();

        assert_eq!(first(&[1, 2, 3]), first(&[1, 2, 3]));
        assert_ne!(first(&[1, 2, 3]), first(&[1, 2, 4]));
        assert_ne!(first(&[1, 2, 3]), first(&[3, 2, 1]));
        assert_ne!(first(&[0]), first(&[0, 0]));
    }

    #[test]
    fn noise_drifts_smoothly_between_zero_and_one() {
        let noise = Noise::new(&mut Rng(5), 10_000.0);
        let values: Vec<f64> = (0..1_000)
            .map(|step| noise.at(Location::new(step * 100, -step * 37)))
            .collect();

        assert!(values.iter().all(|value| (0.0..1.0).contains(value)));
        assert!(
            values
                .windows(2)
                .all(|pair| (pair[0] - pair[1]).abs() < 0.05)
        );
        assert!(values.iter().any(|value| *value < 0.3));
        assert!(values.iter().any(|value| *value > 0.7));
    }
}
