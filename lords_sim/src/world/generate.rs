mod fruiting;
mod terrain;
mod trees;

use std::f64::consts::TAU;

use crate::object::Object;
use crate::random::Rng;
use crate::space::{Length, Location};
use crate::time::Timestamp;
use crate::world::{Area, AreaKind, Biome, Rock, Thing, World};

/// Half the side of the square that things are placed in, around the center of the plane.
const EXTENT: i64 = 200_000;

/// The radius around the center of the plane that is left empty to stand in.
const CLEARING: i64 = 2_000;

const LAKE_VERTICES: u32 = 64;
const LAKE_RADIUS: f64 = 52_000.0;
const LAKE_DISTANCE: (i64, i64) = (100_000, 125_000);

/// How much the lake's radius swells and shrinks around its outline, as pairs of the share
/// of the radius and the number of times it does so on the way around.
const LAKE_WAVES: [(f64, f64); 3] = [(0.12, 2.0), (0.08, 3.0), (0.05, 5.0)];

const BOULDERS: u32 = 320;
const BOULDER_DIAMETER: (i64, i64) = (300, 2_500);
const BOULDER_HEIGHT_PERCENT: (i64, i64) = (40, 80);

impl World {
    /// Returns a stretch of fennoscandian taiga with a lake in it, the same one every time
    /// for the same `seed`.
    ///
    /// The center of the plane is left clear and without water.
    pub fn generate(seed: u64) -> Self {
        let biome = Biome::FennoscandianTaiga;
        let mut rng = Rng(seed);
        let lake = lake(&mut rng);
        let mut terrain = terrain::generate(&mut rng, biome, &lake);
        let boulders = boulders(&mut rng, &lake);
        let trees = trees::generate(&mut rng, biome, &lake, &terrain, &boulders);
        fruiting::generate(&mut rng, &mut terrain, &trees);

        let mut world = Self {
            biome,
            seed,
            ..Self::default()
        };
        world.add_area(lake);
        world.set_terrain(terrain);
        for thing in boulders.into_iter().chain(trees) {
            world.add_thing(thing);
        }
        world
    }
}

fn lake(rng: &mut Rng) -> Area {
    let direction = rng.unit() * TAU;
    let distance = rng.between(LAKE_DISTANCE) as f64;
    let (center_x, center_y) = (distance * direction.cos(), distance * direction.sin());
    let waves = LAKE_WAVES.map(|(share, times)| (share, times, rng.unit() * TAU));

    let outline = (0..LAKE_VERTICES)
        .map(|vertex| {
            let angle = TAU * f64::from(vertex) / f64::from(LAKE_VERTICES);
            let swell: f64 = waves
                .iter()
                .map(|(share, times, shift)| share * (times * angle + shift).cos())
                .sum();
            let radius = LAKE_RADIUS * (1.0 + swell);

            Location::new(
                (center_x + radius * angle.cos()).round() as i64,
                (center_y + radius * angle.sin()).round() as i64,
            )
        })
        .collect();

    Area::new(AreaKind::Lake, outline)
}

fn boulders(rng: &mut Rng, lake: &Area) -> Vec<Thing> {
    let mut boulders = Vec::new();

    for _ in 0..BOULDERS {
        let location = Location::new(
            rng.between((-EXTENT, EXTENT)),
            rng.between((-EXTENT, EXTENT)),
        );
        let diameter = rng.mostly_low(BOULDER_DIAMETER);
        let height = diameter * rng.between(BOULDER_HEIGHT_PERCENT) / 100;

        if is_free(location, diameter / 2, lake) {
            boulders.push(Thing::Rock(Rock::new(
                object(location),
                Length::from_millimeters(diameter),
                Length::from_millimeters(height),
            )));
        }
    }

    boulders
}

/// Returns whether something of `radius` can stand at `location`, which it can on dry
/// ground outside the clearing.
fn is_free(location: Location, radius: i64, lake: &Area) -> bool {
    let from_center = Location::default().distance_to(location).as_millimeters();

    from_center > CLEARING + radius && !lake.contains(location)
}

fn object(location: Location) -> Object {
    Object::new(location, Timestamp::EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Kind, Species};

    #[test]
    fn same_seed_gives_the_same_world() {
        assert_eq!(World::generate(7), World::generate(7));
        assert_ne!(World::generate(7), World::generate(8));
    }

    #[test]
    fn forest_has_a_lake_and_all_kinds_of_things() {
        let world = World::generate(1);

        let count = |kind| {
            world
                .things()
                .filter(|(_, thing)| thing.kind() == kind)
                .count()
        };
        let trees = count(Kind::Plant(Species::Spruce))
            + count(Kind::Plant(Species::Pine))
            + count(Kind::Plant(Species::Birch));

        assert_eq!(world.areas().count(), 1);
        assert!((7_000..10_000).contains(&trees), "{trees} trees");
        assert!(count(Kind::Plant(Species::Spruce)) > 1_000);
        assert!(count(Kind::Plant(Species::Pine)) > 1_000);
        assert!(count(Kind::Plant(Species::Birch)) > 500);
        assert!((250..=320).contains(&count(Kind::Rock)));
    }

    #[test]
    fn nothing_stands_in_the_water_or_at_the_start() {
        for seed in 0..4 {
            let world = World::generate(seed);
            let start = Location::default();

            assert!(!world.is_water(start));
            for (_, thing) in world.things() {
                let location = thing.location();
                let room = start.distance_to(location) - thing.footprint();

                assert!(!world.is_water(location), "{thing:?} of seed {seed}");
                assert!(room.as_millimeters() > CLEARING, "{thing:?} of seed {seed}");
            }
        }
    }

    #[test]
    fn everything_stands_inside_the_square() {
        let world = World::generate(3);

        for (_, thing) in world.things() {
            let location = thing.location();
            assert!(location.x().abs() <= EXTENT && location.y().abs() <= EXTENT);
        }
        for (_, area) in world.areas() {
            for point in area.outline() {
                assert!(point.x().abs() <= EXTENT && point.y().abs() <= EXTENT);
            }
        }
    }
}
