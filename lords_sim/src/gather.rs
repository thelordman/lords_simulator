use crate::item::Item;
use crate::object::{Activity, Human, Living};
use crate::random::Rng;
use crate::space::Location;
use crate::time::{TICKS_PER_SECOND, Timestamp};
use crate::world::{Abundance, Part, Patch, Season, Species, Terrain};

/// How long gathering goes on with nothing turning up before it is given up, in seconds.
pub(crate) const PATIENCE: u64 = 180;

const MILLION: u64 = 1_000_000;

/// Returns the activity of gathering `item` from `now` on, if `gatherer` can gather it where
/// they stand in `season`.
pub(crate) fn begin(
    gatherer: &Human,
    terrain: &Terrain,
    item: Item,
    now: Timestamp,
    season: Season,
) -> Option<Activity> {
    can_gather(gatherer, terrain, item, season).then_some(Activity::Gathering {
        item,
        gathered: 0,
        fruitless_since: now,
    })
}

/// Carries on any gathering of `gatherer` through the tick that ends at `now`, in `season`.
pub(crate) fn tick(gatherer: &mut Human, terrain: &mut Terrain, now: Timestamp, season: Season) {
    let Some(Activity::Gathering {
        item,
        gathered,
        fruitless_since,
    }) = gatherer.activity()
    else {
        return;
    };
    if !can_gather(gatherer, terrain, item, season) {
        gatherer.animal_mut().set_activity(None);
        return;
    }

    let at = gatherer.location();
    let found = terrain
        .patch_at(at)
        .is_some_and(|patch| finds(patch, item, at, now, season));
    if found {
        if let Some(patch) = terrain.patch_at_mut(at) {
            patch.take(item, season);
        }
        gatherer.inventory_mut().add(item);
        gatherer
            .animal_mut()
            .set_activity(Some(Activity::Gathering {
                item,
                gathered: gathered + 1,
                fruitless_since: now,
            }));
    } else if now.seconds_since(fruitless_since) >= PATIENCE {
        gatherer.animal_mut().set_activity(None);
    }
}

pub(crate) fn abundance(patch: &Patch, species: Species, season: Season) -> Abundance {
    let item = Item::Picked(species);
    let chance = u128::from(chance(patch, item, season));
    let (_, surest) = pace(item);

    if patch.left(item, season) == 0 {
        Abundance::Gone
    } else if chance * 2 >= surest {
        Abundance::Plenty
    } else if chance * 8 >= surest {
        Abundance::Moderate
    } else {
        Abundance::Scarce
    }
}

fn can_gather(gatherer: &Human, terrain: &Terrain, item: Item, season: Season) -> bool {
    let patch = terrain.patch_at(gatherer.location());

    patch.is_some_and(|patch| !seen_to_be_gone(patch, item, season)) && gatherer.has_room_for(item)
}

/// Returns whether gathering `item` from `patch` at `at` in `season` turns one up in the
/// tick that ends at `now`.
fn finds(patch: &Patch, item: Item, at: Location, now: Timestamp, season: Season) -> bool {
    let roll = Rng::keyed(&[now.ticks(), at.x() as u64, at.y() as u64]).below(MILLION);

    roll < chance(patch, item, season) / TICKS_PER_SECOND
}

/// Returns whether it can be seen that there are none of `item` on `patch` in `season`.
fn seen_to_be_gone(patch: &Patch, item: Item, season: Season) -> bool {
    match item {
        Item::Pebble => false,
        Item::Picked(_) => patch.left(item, season) == 0,
    }
}

/// Returns the chance in a million that a second of gathering `item` from `patch` in
/// `season` turns one up.
fn chance(patch: &Patch, item: Item, season: Season) -> u64 {
    let (looked_over, surest) = pace(item);
    let left = u128::from(patch.left(item, season));
    let in_sight = match item {
        Item::Pebble => u128::from(patch.bare()),
        Item::Picked(_) => 100,
    };
    let side = Terrain::PATCH_SIDE.as_millimeters() as u128;
    let chance = left * looked_over * u128::from(MILLION) * in_sight / (side * side * 100);

    chance.min(surest) as u64
}

/// Returns the ground looked over in a second of gathering `item`, in square millimeters,
/// and the best chance in a million that a second turns one up.
fn pace(item: Item) -> (u128, u128) {
    match item {
        Item::Pebble => (20_000, 500_000),
        Item::Picked(species) => match species.crop().map(|crop| crop.part) {
            Some(Part::Berry) => (30_000, 800_000),
            Some(
                Part::Sprig
                | Part::Leaf
                | Part::Stem
                | Part::Tuft
                | Part::Stalk
                | Part::Frond
                | Part::Clump,
            ) => (1_000_000, 250_000),
            None => (0, 0),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Name, Sex};
    use crate::world::{GroundType, Vegetation, VegetationKind};

    /// The height of summer, when bilberries are at their ripest.
    const SUMMER: u16 = 215;

    fn heath(pebbles: u32) -> Patch {
        let vegetation = Vegetation::typical(VegetationKind::BilberryHeath);
        Patch::new(GroundType::Till, Some(vegetation), pebbles)
    }

    /// Returns ground of only `patch`, around the center.
    fn terrain(patch: Patch) -> Terrain {
        Terrain::new(Location::new(-2_000, -2_000), 1, [patch])
    }

    fn gatherer() -> Human {
        Human::new(
            Location::default(),
            Timestamp::EPOCH,
            Name::default(),
            Sex::Female,
        )
    }

    #[test]
    fn gathering_begins_only_where_and_when_there_is_something_to_gather() {
        let heath = terrain(heath(0));
        let bilberries = Item::Picked(Species::Bilberry);
        let begin = |terrain: &Terrain, item, day| {
            begin(
                &gatherer(),
                terrain,
                item,
                Timestamp::EPOCH,
                Season::usual(day),
            )
        };

        assert_eq!(
            begin(&heath, bilberries, SUMMER),
            Some(Activity::Gathering {
                item: bilberries,
                gathered: 0,
                fruitless_since: Timestamp::EPOCH,
            })
        );
        assert_eq!(begin(&heath, bilberries, 30), None);
        assert!(begin(&heath, Item::Pebble, 30).is_some());
        assert_eq!(begin(&heath, Item::Picked(Species::OakFern), SUMMER), None);
        assert_eq!(begin(&Terrain::default(), Item::Pebble, SUMMER), None);
    }

    #[test]
    fn chance_grows_with_the_pebbles_left_and_how_much_is_in_sight() {
        let pebbles = |patch: &Patch| chance(patch, Item::Pebble, Season::usual(SUMMER));

        assert_eq!(pebbles(&heath(0)), 0);
        assert_eq!(pebbles(&heath(100)), 25_000);
        assert_eq!(pebbles(&heath(50)), 12_500);
        assert_eq!(pebbles(&Patch::new(GroundType::Sand, None, 50)), 62_500);
    }

    #[test]
    fn even_a_beach_full_of_pebbles_takes_time_to_pick_from() {
        let beach = Patch::new(GroundType::Gravel, None, u32::MAX);

        assert_eq!(chance(&beach, Item::Pebble, Season::usual(SUMMER)), 500_000);
    }

    #[test]
    fn berries_are_picked_as_fast_as_they_are_found_and_plants_as_fast_as_pulled() {
        let bog = Patch::new(
            GroundType::Peat,
            Some(Vegetation::typical(VegetationKind::PineBog)),
            0,
        );
        let picked =
            |patch: &Patch, species, day| chance(patch, Item::Picked(species), Season::usual(day));

        assert_eq!(picked(&heath(0), Species::Bilberry, SUMMER), 116_250);
        assert_eq!(picked(&bog, Species::Cloudberry, 200), 24_375);
        assert_eq!(picked(&heath(0), Species::FeatherMoss, SUMMER), 250_000);
        assert_eq!(picked(&heath(0), Species::OakFern, SUMMER), 0);
    }

    #[test]
    fn crop_is_told_by_how_soon_one_turns_up() {
        let bilberry = |patch: &Patch, day| abundance(patch, Species::Bilberry, Season::usual(day));
        let rich = heath(0).growing_under(40, 200);

        assert_eq!(bilberry(&rich, SUMMER), Abundance::Plenty);
        assert_eq!(bilberry(&heath(0), SUMMER), Abundance::Moderate);
        assert_eq!(
            bilberry(&heath(0).growing_under(95, 20), SUMMER),
            Abundance::Scarce
        );
        assert_eq!(bilberry(&rich, 30), Abundance::Gone);
    }

    #[test]
    fn only_plants_are_seen_to_be_picked_bare() {
        let bare = Patch::new(GroundType::Gravel, None, 0);
        let summer = Season::usual(SUMMER);

        assert!(seen_to_be_gone(
            &bare,
            Item::Picked(Species::Bilberry),
            summer
        ));
        assert!(!seen_to_be_gone(
            &heath(0),
            Item::Picked(Species::Bilberry),
            summer
        ));
        assert!(!seen_to_be_gone(&bare, Item::Pebble, summer));
    }

    #[test]
    fn finds_come_about_as_often_as_their_chance() {
        let patch = Patch::new(GroundType::Sand, None, 200);
        let at = Location::new(1_234, -5_678);
        let summer = Season::usual(SUMMER);
        let finds = |patch: &Patch, second| {
            finds(
                patch,
                Item::Pebble,
                at,
                Timestamp::from_seconds(second),
                summer,
            )
        };

        let found = (0..10_000).filter(|second| finds(&patch, *second)).count();

        assert!((2_300..2_700).contains(&found), "{found} found");
        assert!(!finds(&heath(0), 0));
    }
}
