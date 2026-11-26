use crate::random::{Noise, Rng};
use crate::space::Location;
use crate::world::{Area, Biome, GroundType, Patch, Species, Terrain, Vegetation, VegetationKind};

use super::EXTENT;

/// How far from the water the beach reaches.
const BEACH: i64 = 5_000;

/// How far from the water the ground is moist and rich enough for herbs and ferns.
const SHORE: i64 = 15_000;

/// About how far apart the wettest and driest stretches of forest are.
const DRYNESS_SCALE: f64 = 50_000.0;

/// About how far along the shore one kind of beach gives way to another.
const BEACH_SCALE: f64 = 20_000.0;

/// How far each species' share of the vegetation strays from its usual, in percent of it.
const SHARE_SPREAD: (i64, i64) = (40, 160);

/// How far the pebbles on a patch stray from what its ground usually has, in percent.
const PEBBLE_SPREAD: (i64, i64) = (50, 150);

/// Returns the ground of the square in `biome`, with the lake's shore as beach.
pub(super) fn generate(rng: &mut Rng, biome: Biome, lake: &Area) -> Terrain {
    let side = Terrain::PATCH_SIDE.as_millimeters();
    let columns = 2 * EXTENT / side;
    let dryness = Noise::new(rng, DRYNESS_SCALE);
    let exposure = Noise::new(rng, BEACH_SCALE);

    let places: Vec<Place> = (0..columns)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .map(|(row, column)| {
            let middle = Location::new(
                -EXTENT + column * side + side / 2,
                -EXTENT + row * side + side / 2,
            );
            Place {
                dryness: dryness.at(middle),
                exposure: exposure.at(middle),
                from_water: from_water(lake, middle),
            }
        })
        .collect();
    let bands = Bands::of(biome, &places);
    let patches: Vec<Patch> = places
        .iter()
        .map(|place| {
            let (ground, vegetation) = bands.site(place);
            patch(rng, ground, vegetation)
        })
        .collect();

    Terrain::new(Location::new(-EXTENT, -EXTENT), columns as u32, patches)
}

/// What decides the ground of a patch.
struct Place {
    dryness: f64,
    exposure: f64,
    /// How far it is from water, in millimeters.
    from_water: i64,
}

impl Place {
    fn is_beach(&self) -> bool {
        self.from_water < BEACH
    }
}

/// Where each band of the forest floor and of the beaches of a biome ends.
struct Bands {
    biome: Biome,
    forest: Vec<f64>,
    beaches: Vec<f64>,
}

impl Bands {
    fn of(biome: Biome, places: &[Place]) -> Self {
        let dryness = places.iter().map(|place| place.dryness).collect();
        let exposure = places
            .iter()
            .filter(|place| place.is_beach())
            .map(|place| place.exposure)
            .collect();
        let forest = biome.forest_floor().iter().map(|(.., share)| *share);
        let beaches = biome.beaches().iter().map(|(.., share)| *share);

        Self {
            biome,
            forest: limits(dryness, forest),
            beaches: limits(exposure, beaches),
        }
    }

    fn site(&self, place: &Place) -> (GroundType, Option<VegetationKind>) {
        if place.is_beach() {
            let beach = band(place.exposure, &self.beaches);
            let (ground, vegetation, _) = self.biome.beaches()[beach];
            return (ground, vegetation);
        }

        let forest = band(place.dryness, &self.forest);
        let (ground, vegetation, _) = self.biome.forest_floor()[forest];
        if place.from_water < SHORE && ground != GroundType::Peat {
            let (ground, vegetation) = self.biome.lakeside();
            (ground, Some(vegetation))
        } else {
            (ground, Some(vegetation))
        }
    }
}

/// Returns how far `at` is from the water of `lake`, in millimeters.
fn from_water(lake: &Area, at: Location) -> i64 {
    if lake.contains(at) {
        0
    } else {
        at.distance_to(lake.nearest_shore(at)).as_millimeters()
    }
}

/// Returns a patch of `ground_type` under `vegetation`, with its pebbles and species strayed
/// from the usual.
fn patch(rng: &mut Rng, ground_type: GroundType, vegetation: Option<VegetationKind>) -> Patch {
    let usual = i64::from(ground_type.pebbles_per_square_meter() * Terrain::PATCH_AREA);
    let pebbles = usual * rng.between(PEBBLE_SPREAD) / 100;
    let vegetation = vegetation.map(|kind| Vegetation::new(kind, shares(rng, kind)));

    Patch::new(ground_type, vegetation, pebbles as u32)
}

/// Returns the values below which `values` fall in the given shares of cases, in percent,
/// each share adding to the ones before it.
fn limits(mut values: Vec<f64>, shares: impl IntoIterator<Item = u32>) -> Vec<f64> {
    values.sort_by(f64::total_cmp);
    let mut below = 0;

    shares
        .into_iter()
        .map(|share| {
            below += share as usize;
            values
                .get(values.len() * below / 100)
                .copied()
                .unwrap_or(f64::INFINITY)
        })
        .collect()
}

/// Returns which of the bands that end at `limits` holds `value`.
fn band(value: f64, limits: &[f64]) -> usize {
    limits
        .iter()
        .position(|limit| value < *limit)
        .unwrap_or(limits.len() - 1)
}

/// Returns the share of each species in vegetation of `kind`, strayed from the usual and in
/// whole percent that add up to a hundred.
fn shares(rng: &mut Rng, kind: VegetationKind) -> Vec<(Species, u8)> {
    let weights: Vec<(Species, i64)> = kind
        .composition()
        .iter()
        .map(|(species, percent)| (*species, i64::from(*percent) * rng.between(SHARE_SPREAD)))
        .collect();
    let total: i64 = weights.iter().map(|(_, weight)| weight).sum();

    let mut shares: Vec<(Species, i64, i64)> = weights
        .into_iter()
        .map(|(species, weight)| (species, weight * 100 / total, weight * 100 % total))
        .collect();
    let left_over = 100 - shares.iter().map(|(_, percent, _)| percent).sum::<i64>();
    shares.sort_by_key(|(_, _, remainder)| -remainder);
    for (_, percent, _) in shares.iter_mut().take(left_over as usize) {
        *percent += 1;
    }

    shares
        .into_iter()
        .map(|(species, percent, _)| (species, percent as u8))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    use crate::space::Length;
    use crate::world::{Season, World};

    /// Returns the patch of `world` that `at` is on, which every place in the square has.
    fn patch_at(world: &World, at: Location) -> &Patch {
        world
            .terrain()
            .patch_at(at)
            .unwrap_or_else(|| panic!("{at:?} has no ground"))
    }

    fn vegetation(patch: &Patch) -> Option<VegetationKind> {
        patch.vegetation().map(Vegetation::kind)
    }

    #[test]
    fn whole_square_has_ground() {
        let world = World::generate(2);

        for x in [-EXTENT, 0, EXTENT - 1] {
            for y in [-EXTENT, 0, EXTENT - 1] {
                patch_at(&world, Location::new(x, y));
            }
        }
        assert!(world.terrain().patch_at(Location::new(EXTENT, 0)).is_none());
    }

    #[test]
    fn forest_has_every_kind_of_ground_and_vegetation() {
        let world = World::generate(1);
        let side = Terrain::PATCH_SIDE.as_millimeters();
        let mut found: Vec<(GroundType, Option<VegetationKind>)> = (0..2 * EXTENT / side)
            .flat_map(|row| {
                (0..2 * EXTENT / side)
                    .map(move |column| Location::new(-EXTENT + column * side, -EXTENT + row * side))
            })
            .map(|at| patch_at(&world, at))
            .map(|patch| (patch.ground_type(), vegetation(patch)))
            .collect();
        found.sort_unstable();
        found.dedup();

        let biome = Biome::FennoscandianTaiga;
        let beaches = biome
            .beaches()
            .iter()
            .map(|(ground, vegetation, _)| (*ground, *vegetation));
        let forest = biome
            .forest_floor()
            .iter()
            .map(|(ground, vegetation, _)| (*ground, Some(*vegetation)));
        let (ground, vegetation) = biome.lakeside();
        let mut expected: Vec<_> = beaches
            .chain(forest)
            .chain([(ground, Some(vegetation))])
            .collect();
        expected.sort_unstable();
        expected.dedup();

        assert_eq!(found, expected);
    }

    #[test]
    fn shore_is_beach_and_the_forest_by_it_is_rich() {
        for seed in 0..4 {
            let world = World::generate(seed);
            let (_, lake) = world.areas().next().expect("there is a lake");
            let outline = lake.outline();
            let middle = Location::new(
                outline.iter().map(|point| point.x()).sum::<i64>() / outline.len() as i64,
                outline.iter().map(|point| point.y()).sum::<i64>() / outline.len() as i64,
            );

            for point in outline {
                let on_the_beach = point.away_from(middle, Length::from_millimeters(500));
                let inland = point.away_from(middle, Length::from_meters(10));
                let beach = patch_at(&world, on_the_beach);
                let forest = patch_at(&world, inland);

                assert!(!world.is_water(on_the_beach));
                assert!(
                    [GroundType::Gravel, GroundType::Sand, GroundType::Mud]
                        .contains(&beach.ground_type()),
                    "{beach:?} at {on_the_beach:?} of seed {seed}"
                );
                assert!(
                    [VegetationKind::HerbRichHeath, VegetationKind::PineBog]
                        .map(Some)
                        .contains(&vegetation(forest)),
                    "{forest:?} at {inland:?} of seed {seed}"
                );
            }
        }
    }

    #[test]
    fn vegetation_is_shared_out_in_whole_percent_among_its_usual_species() {
        let world = World::generate(1);

        for x in (-EXTENT..EXTENT).step_by(9_000) {
            for y in (-EXTENT..EXTENT).step_by(9_000) {
                let Some(vegetation) = patch_at(&world, Location::new(x, y)).vegetation() else {
                    continue;
                };
                let usual = vegetation.kind().composition();
                let total: u32 = vegetation
                    .shares()
                    .iter()
                    .map(|(_, percent)| u32::from(*percent))
                    .sum();

                assert_eq!(total, 100, "{vegetation:?}");
                for (species, _) in vegetation.shares() {
                    assert!(usual.iter().any(|(usual, _)| usual == species));
                }
            }
        }
    }

    #[test]
    fn pebbles_lie_where_the_ground_has_them() {
        let world = World::generate(1);
        let side = Terrain::PATCH_SIDE.as_millimeters();
        let mut most: Vec<(GroundType, u32)> = Vec::new();

        for x in (-EXTENT..EXTENT).step_by(side as usize) {
            for y in (-EXTENT..EXTENT).step_by(side as usize) {
                let patch = patch_at(&world, Location::new(x, y));
                let pebbles = patch.left(Item::Pebble, Season::usual(0));
                match most
                    .iter_mut()
                    .find(|(ground, _)| *ground == patch.ground_type())
                {
                    Some((_, most)) => *most = (*most).max(pebbles),
                    None => most.push((patch.ground_type(), pebbles)),
                }
            }
        }
        let most = |ground| {
            most.iter()
                .find(|(of, _)| *of == ground)
                .map_or(0, |(_, pebbles)| *pebbles)
        };

        assert_eq!(most(GroundType::Peat), 0);
        assert!(most(GroundType::Till) > 0);
        assert!(most(GroundType::Gravel) > 10 * most(GroundType::Till));
    }

    #[test]
    fn values_are_banded_by_their_share_of_all() {
        let values: Vec<f64> = (0..200).map(f64::from).rev().collect();
        let limits = limits(values, [10, 40, 50]);

        assert_eq!(limits, [20.0, 100.0, f64::INFINITY]);
        assert_eq!(band(0.0, &limits), 0);
        assert_eq!(band(20.0, &limits), 1);
        assert_eq!(band(150.0, &limits), 2);
        assert_eq!(band(f64::INFINITY, &limits), 2);
    }
}
