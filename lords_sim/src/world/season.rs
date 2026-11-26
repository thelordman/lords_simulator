use crate::random::Rng;
use crate::time::Timestamp;
use crate::time::gregorian::Date;
use crate::world::Species;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Season {
    seed: Option<u64>,
    year: u64,
    day: u16,
}

impl Season {
    /// The days after which a crop that ripened is counted as the next year's.
    const YEAR: u16 = 365;

    /// Returns the season at `now` in the world whose years are set by `seed`.
    pub fn at(seed: u64, now: Timestamp) -> Self {
        let date = Date::from_timestamp(now);

        Self {
            seed: Some(seed),
            year: date.year(),
            day: date.day_of_year(),
        }
    }

    /// Returns the season `day` days into a year in which every crop is an average one.
    #[cfg(test)]
    pub(crate) fn usual(day: u16) -> Self {
        Self {
            seed: None,
            year: 1,
            day,
        }
    }

    /// Returns the year of the crop of `species` that is on the plants now and how many
    /// days of that year have passed, which is 0 and no days for a crop that never grows
    /// back.
    pub(crate) fn crop_year(self, species: Species) -> (u64, u16) {
        match species.fruiting() {
            None => (0, 0),
            Some(fruiting) if self.day >= fruiting.ripe_from => (self.year, self.day),
            Some(_) => (self.year.saturating_sub(1), self.day + Self::YEAR),
        }
    }

    /// Returns how much of the crop of `species` that is on the plants now is ripe, in
    /// percent of a usual year's.
    pub(crate) fn ripe(self, species: Species) -> u32 {
        let Some(fruiting) = species.fruiting() else {
            return 100;
        };
        let (year, day) = self.crop_year(species);

        fruiting.ripe(day) * self.crop(species, year) / 100
    }

    /// Returns how good a year `year` is for `species`, in percent of an average year.
    fn crop(self, species: Species, year: u64) -> u32 {
        let Some(seed) = self.seed else {
            return 100;
        };
        let shared = Rng::keyed(&[seed, year]).below(100);
        let own = Rng::keyed(&[seed, year, species as u64]).below(100);

        25 + (shared + own) as u32 * 3 / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::gregorian::SECONDS_PER_DAY;

    /// Returns the season on `day` of the year 2 of a world of seed 7.
    fn on(day: u64) -> Season {
        Season::at(7, Timestamp::from_seconds((365 + day) * SECONDS_PER_DAY))
    }

    #[test]
    fn berries_are_of_the_year_they_ripened_in_until_the_next_ripen() {
        assert_eq!(on(10).crop_year(Species::Lingonberry), (1, 375));
        assert_eq!(on(240).crop_year(Species::Lingonberry), (2, 240));
        assert_eq!(on(240).crop_year(Species::FeatherMoss), (0, 0));
    }

    #[test]
    fn nothing_but_lasting_berries_is_ripe_in_winter() {
        assert_eq!(on(20).ripe(Species::Bilberry), 0);
        assert!(on(20).ripe(Species::Lingonberry) > 0);
        assert!(on(215).ripe(Species::Bilberry) > 0);
        assert_eq!(on(20).ripe(Species::FeatherMoss), 100);
    }

    #[test]
    fn years_are_good_and_poor_around_an_average() {
        let crops: Vec<u32> = (0..2_000)
            .map(|year| on(0).crop(Species::Bilberry, year))
            .collect();
        let average = crops.iter().sum::<u32>() / crops.len() as u32;

        assert!(crops.iter().all(|crop| (25..=175).contains(crop)));
        assert!((95..=105).contains(&average), "{average}");
        assert!(crops.iter().any(|crop| *crop < 40));
        assert!(crops.iter().any(|crop| *crop > 160));
    }
}
