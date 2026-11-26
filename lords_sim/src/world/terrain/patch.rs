use serde::{Deserialize, Serialize};

use crate::item::Item;
use crate::world::{GroundType, Season, Species, Terrain, Vegetation};

/// Roughly how much of a crop is left to pick.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Abundance {
    Gone,
    Scarce,
    Moderate,
    Plenty,
}

/// A piece of ground, with what grows and lies on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    ground_type: GroundType,
    vegetation: Option<Vegetation>,
    pebbles: u32,
    /// How much of the sky above it tree crowns hide, in percent.
    #[serde(default)]
    canopy: u8,
    /// How well what grows on it bears fruit, in percent of what its kind usually bears.
    #[serde(default = "usual_fruiting")]
    fruiting: u8,
    /// How many of each species' crop have been picked, with the year of the crop, which
    /// is 0 for crops that never grow back.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    picked: Vec<(Species, u64, u32)>,
}

fn usual_fruiting() -> u8 {
    100
}

impl Patch {
    /// Returns a patch under open sky on which plants fruit as their kind usually does.
    pub fn new(ground_type: GroundType, vegetation: Option<Vegetation>, pebbles: u32) -> Self {
        Self {
            ground_type,
            vegetation,
            pebbles,
            canopy: 0,
            fruiting: usual_fruiting(),
            picked: Vec::new(),
        }
    }

    /// Returns the patch with `canopy` percent of its sky hidden by tree crowns, and its
    /// plants bearing `fruiting` percent of the fruit their kind usually bears.
    pub fn growing_under(mut self, canopy: u8, fruiting: u8) -> Self {
        self.grow_under(canopy, fruiting);
        self
    }

    pub(crate) fn grow_under(&mut self, canopy: u8, fruiting: u8) {
        self.canopy = canopy;
        self.fruiting = fruiting;
    }

    pub fn ground_type(&self) -> GroundType {
        self.ground_type
    }

    pub fn vegetation(&self) -> Option<&Vegetation> {
        self.vegetation.as_ref()
    }

    /// Returns how much of the sky above it tree crowns hide, in percent.
    pub fn canopy(&self) -> u8 {
        self.canopy
    }

    /// Returns how well what grows on it bears fruit, in percent of what its kind usually
    /// bears.
    pub fn fruiting(&self) -> u8 {
        self.fruiting
    }

    /// Returns how much of the ground shows through what grows on it, in percent.
    pub fn bare(&self) -> u8 {
        let Some(vegetation) = &self.vegetation else {
            return 100;
        };
        let growing: u32 = vegetation
            .shares()
            .iter()
            .map(|(species, _)| u32::from(self.share(*species)))
            .sum();

        let hidden = u32::from(vegetation.kind().cover()) * growing.min(100) / 100;
        100 - hidden as u8
    }

    /// Returns the share of the vegetation that `species` makes up, in percent.
    pub fn share(&self, species: Species) -> u8 {
        let share = self.usual_share(species);
        let thins = species
            .crop()
            .is_some_and(|crop| !crop.part.leaves_the_plant());
        let grown = self.grown(species, 100);
        if !thins || grown == 0 {
            return share;
        }

        let left = grown.saturating_sub(self.picked(species, 0));
        let thinned = (u32::from(share) * left * 2 + grown) / (grown * 2);
        if left > 0 {
            thinned.max(1) as u8
        } else {
            thinned as u8
        }
    }

    /// Returns how many of `item` are on it still to be gathered in `season`.
    pub fn left(&self, item: Item, season: Season) -> u32 {
        match item {
            Item::Pebble => self.pebbles,
            Item::Picked(species) => {
                let (year, _) = season.crop_year(species);
                let grown = self.grown(species, season.ripe(species));
                grown.saturating_sub(self.picked(species, year))
            }
        }
    }

    /// Takes one of `item` off it in `season`, if any are left.
    pub(crate) fn take(&mut self, item: Item, season: Season) {
        if self.left(item, season) == 0 {
            return;
        }

        match item {
            Item::Pebble => self.pebbles -= 1,
            Item::Picked(species) => {
                let (year, _) = season.crop_year(species);
                self.picked
                    .retain(|(picked, of, _)| *picked != species || *of == year);
                match self
                    .picked
                    .iter_mut()
                    .find(|(picked, _, _)| *picked == species)
                {
                    Some((_, _, count)) => *count += 1,
                    None => self.picked.push((species, year, 1)),
                }
            }
        }
    }

    /// Leaves `count` of `item` on it, of which only pebbles stay to be found again.
    pub(crate) fn leave(&mut self, item: Item, count: u32) {
        match item {
            Item::Pebble => self.pebbles = self.pebbles.saturating_add(count),
            Item::Picked(_) => {}
        }
    }

    fn usual_share(&self, species: Species) -> u8 {
        self.vegetation
            .as_ref()
            .map_or(0, |vegetation| vegetation.share(species))
    }

    /// Returns how many of the crop of `species` grow on it when `ripe` percent of a usual
    /// year's crop is ripe, picked or not.
    fn grown(&self, species: Species, ripe: u32) -> u32 {
        let Some(crop) = species.crop() else {
            return 0;
        };
        let mut grown = u64::from(self.usual_share(species))
            * u64::from(Terrain::PATCH_AREA)
            * u64::from(crop.per_square_meter);
        if let Some(fruiting) = species.fruiting() {
            grown = grown
                * u64::from(fruiting.in_light(self.canopy))
                * u64::from(self.fruiting)
                * u64::from(ripe)
                / 1_000_000;
        }

        u32::try_from(grown / 100).unwrap_or(u32::MAX)
    }

    fn picked(&self, species: Species, year: u64) -> u32 {
        self.picked
            .iter()
            .find(|(picked, of, _)| *picked == species && *of == year)
            .map_or(0, |(_, _, count)| *count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Timestamp;
    use crate::time::gregorian::SECONDS_PER_DAY;
    use crate::world::VegetationKind;

    /// The height of summer, when bilberries are at their ripest.
    const SUMMER: u16 = 215;

    fn heath() -> Patch {
        let vegetation = Vegetation::typical(VegetationKind::BilberryHeath);
        Patch::new(GroundType::Till, Some(vegetation), 10).growing_under(40, 100)
    }

    #[test]
    fn vegetation_hides_the_ground_it_covers() {
        assert_eq!(Patch::new(GroundType::Gravel, None, 10).bare(), 100);
        assert_eq!(heath().bare(), 20);
    }

    #[test]
    fn pebbles_taken_are_gone_and_those_left_are_there_to_find_again() {
        let season = Season::usual(SUMMER);
        let mut patch = Patch::new(GroundType::Sand, None, 2);

        patch.take(Item::Pebble, season);
        assert_eq!(patch.left(Item::Pebble, season), 1);
        patch.take(Item::Pebble, season);
        patch.take(Item::Pebble, season);
        assert_eq!(patch.left(Item::Pebble, season), 0);

        patch.leave(Item::Pebble, 5);
        patch.leave(Item::Picked(Species::Bilberry), 5);
        assert_eq!(patch.left(Item::Pebble, season), 5);
        assert_eq!(patch.left(Item::Picked(Species::Bilberry), season), 0);
    }

    #[test]
    fn pulled_up_plants_leave_bare_ground_but_picked_berries_do_not() {
        let season = Season::usual(SUMMER);
        let mut patch = heath();
        let moss = Species::FeatherMoss;
        for _ in 0..50 {
            patch.take(Item::Picked(moss), season);
            patch.take(Item::Picked(Species::Bilberry), season);
        }

        assert_eq!(patch.share(moss), 8);
        assert_eq!(patch.share(Species::Bilberry), 35);
        assert_eq!(patch.bare(), 30);

        for _ in 0..50 {
            patch.take(Item::Picked(moss), season);
        }
        assert_eq!(patch.share(moss), 0);
        assert_eq!(patch.bare(), 36);
    }

    #[test]
    fn plant_with_any_left_keeps_some_share_and_ground_is_never_more_than_hidden() {
        let season = Season::usual(SUMMER);
        let mut patch = heath();
        let moss = Species::FeatherMoss;
        for _ in 0..79 {
            patch.take(Item::Picked(moss), season);
        }
        assert_eq!(patch.share(moss), 1);

        let overgrown = Vegetation::new(
            VegetationKind::PineBog,
            vec![(Species::PeatMoss, 90), (Species::Cloudberry, 90)],
        );
        assert_eq!(Patch::new(GroundType::Peat, Some(overgrown), 0).bare(), 0);
    }

    #[test]
    fn plants_grow_by_their_share_of_the_patch_and_berries_by_the_light_too() {
        let season = Season::usual(SUMMER);
        let bilberries = Item::Picked(Species::Bilberry);

        assert_eq!(heath().left(bilberries, season), 117);
        assert_eq!(heath().growing_under(90, 100).left(bilberries, season), 47);
        assert_eq!(heath().growing_under(40, 50).left(bilberries, season), 58);
        assert_eq!(heath().left(Item::Picked(Species::MayLily), season), 96);
        assert_eq!(heath().left(Item::Picked(Species::OakFern), season), 0);
        assert_eq!(
            Patch::new(GroundType::Gravel, None, 0).left(bilberries, season),
            0
        );
    }

    #[test]
    fn berries_are_there_only_in_their_season() {
        let bilberries = Item::Picked(Species::Bilberry);
        let left = |day| heath().left(bilberries, Season::usual(day));

        assert_eq!(left(100), 0);
        assert!(left(200) > 0 && left(200) < left(SUMMER));
        assert_eq!(left(300), 0);
    }

    #[test]
    fn berries_picked_bare_grow_again_the_next_year() {
        let bilberries = Item::Picked(Species::Bilberry);
        let summer = Season::usual(SUMMER);
        let mut patch = heath();
        while patch.left(bilberries, summer) > 0 {
            patch.take(bilberries, summer);
        }

        let next_year = (365 + u64::from(SUMMER)) * SECONDS_PER_DAY;
        let next_summer = Season::at(0, Timestamp::from_seconds(next_year));

        assert_eq!(patch.picked, [(Species::Bilberry, 1, 117)]);
        assert!(patch.left(bilberries, next_summer) > 0);
    }
}
