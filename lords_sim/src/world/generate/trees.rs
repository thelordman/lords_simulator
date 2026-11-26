use crate::random::Rng;
use crate::space::{Length, Location};
use crate::world::{Area, Biome, Patch, Plant, Species, Stand, Terrain, Thing};

use super::{EXTENT, is_free, object};

/// The side of the squares of which each has one place for a tree.
const TREE_SPACING: i64 = 3_900;
const TRUNK_DIAMETER: (i64, i64) = (80, 450);
const TREE_SLENDERNESS: (i64, i64) = (55, 75);

/// Returns the trees of the square, which grow by the ground under them.
pub(super) fn generate(
    rng: &mut Rng,
    biome: Biome,
    lake: &Area,
    terrain: &Terrain,
    boulders: &[Thing],
) -> Vec<Thing> {
    let places_per_side = 2 * EXTENT / TREE_SPACING;
    let jitter = (TREE_SPACING / 10, TREE_SPACING * 9 / 10);
    let mut trees = Vec::new();

    for row in 0..places_per_side {
        for column in 0..places_per_side {
            let location = Location::new(
                -EXTENT + column * TREE_SPACING + rng.between(jitter),
                -EXTENT + row * TREE_SPACING + rng.between(jitter),
            );
            let trunk_diameter = rng.mostly_low(TRUNK_DIAMETER);
            let height = trunk_diameter * rng.between(TREE_SLENDERNESS);
            let stand = terrain
                .patch_at(location)
                .and_then(Patch::vegetation)
                .and_then(|vegetation| biome.stand(vegetation.kind()));
            let Some(stand) = stand else {
                continue;
            };
            let species = pick(&stand, rng);

            let radius = trunk_diameter / 2;
            let on_a_boulder = boulders.iter().any(|boulder| {
                let room = boulder.footprint() + Length::from_millimeters(radius);
                location.distance_to(boulder.location()) <= room
            });
            if rng.chance(stand.gaps) || on_a_boulder || !is_free(location, radius, lake) {
                continue;
            }

            trees.push(Thing::Plant(Plant::new(
                object(location),
                species,
                Length::from_millimeters(height),
                Length::from_millimeters(trunk_diameter),
            )));
        }
    }

    trees
}

/// Returns a species of tree from `stand`, each as often as its share there.
fn pick(stand: &Stand, rng: &mut Rng) -> Species {
    let mut roll = rng.below(100);
    for (species, share) in stand.trees {
        if roll < *share {
            return *species;
        }
        roll -= share;
    }
    stand.trees[0].0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Kind, Vegetation, VegetationKind, World};

    #[test]
    fn trees_grow_by_the_ground_under_them() {
        let world = World::generate(1);
        let on = |kind: Option<VegetationKind>, species| {
            world
                .things()
                .filter(|(_, thing)| thing.kind() == Kind::Plant(species))
                .filter(|(_, thing)| {
                    let patch = world.terrain().patch_at(thing.location());
                    patch.and_then(Patch::vegetation).map(Vegetation::kind) == kind
                })
                .count()
        };

        assert_eq!(on(None, Species::Birch), 0);
        assert_eq!(on(Some(VegetationKind::ShoreMeadow), Species::Birch), 0);
        assert_eq!(on(Some(VegetationKind::LichenHeath), Species::Spruce), 0);

        let bilberry = Some(VegetationKind::BilberryHeath);
        let lingonberry = Some(VegetationKind::LingonberryHeath);
        assert!(on(bilberry, Species::Spruce) > 3 * on(bilberry, Species::Pine));
        assert!(on(lingonberry, Species::Pine) > 3 * on(lingonberry, Species::Spruce));
    }
}
