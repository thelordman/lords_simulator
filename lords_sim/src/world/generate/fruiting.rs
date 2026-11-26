use crate::random::{Noise, Rng};
use crate::space::Location;
use crate::world::{Terrain, Thing};

/// How far a tree's crown reaches from its trunk, in diameters of the trunk.
const CROWN_REACH: i64 = 9;

/// How many points along each side of a patch are looked at to tell how much of it is
/// shaded.
const SHADE_POINTS: i64 = 4;

/// About how far apart the stretches where plants fruit well and poorly are.
const FRUITING_SCALE: f64 = 25_000.0;

/// How far each patch's fruiting strays from that of its stretch, in percent.
const FRUITING_SPREAD: (i64, i64) = (50, 150);

/// Sets how much of the sky above each patch of `terrain` the crowns of `trees` hide, and
/// how well the plants on it fruit.
pub(super) fn generate(rng: &mut Rng, terrain: &mut Terrain, trees: &[Thing]) {
    let shade = shade(terrain, trees);
    let stretches = Noise::new(rng, FRUITING_SCALE);
    let points = (SHADE_POINTS * SHADE_POINTS) as u32;

    for (index, shaded) in (0..terrain.len()).zip(shade) {
        let stretch = ((stretches.at(terrain.middle_of(index)) - 0.25) * 300.0).max(0.0);
        let fruiting = stretch as i64 * rng.between(FRUITING_SPREAD) / 100;
        let canopy = shaded.count_ones() * 100 / points;
        if let Some(patch) = terrain.patch_mut(index) {
            patch.grow_under(canopy as u8, fruiting.min(255) as u8);
        }
    }
}

/// Returns for each patch of `terrain` which of the points looked at on it are under the
/// crown of one of `trees`, each as a bit.
fn shade(terrain: &Terrain, trees: &[Thing]) -> Vec<u16> {
    let mut shade = vec![0_u16; terrain.len() as usize];
    let spacing = Terrain::PATCH_SIDE.as_millimeters() / SHADE_POINTS;
    let origin = terrain.corner_of(0);
    let point = |column: i64, row: i64| {
        Location::new(
            origin.x() + column * spacing + spacing / 2,
            origin.y() + row * spacing + spacing / 2,
        )
    };

    for tree in trees {
        let trunk = tree.location();
        let reach = tree.footprint().as_millimeters() * 2 * CROWN_REACH;
        let columns = (trunk.x() - reach - origin.x()).div_euclid(spacing)
            ..=(trunk.x() + reach - origin.x()).div_euclid(spacing);
        let rows = (trunk.y() - reach - origin.y()).div_euclid(spacing)
            ..=(trunk.y() + reach - origin.y()).div_euclid(spacing);

        for row in rows {
            for column in columns.clone() {
                let at = point(column, row);
                if trunk.distance_to(at).as_millimeters() > reach {
                    continue;
                }
                let Some(index) = terrain.index_of(at) else {
                    continue;
                };
                let bit =
                    column.rem_euclid(SHADE_POINTS) + SHADE_POINTS * row.rem_euclid(SHADE_POINTS);
                shade[index as usize] |= 1 << bit;
            }
        }
    }

    shade
}

#[cfg(test)]
mod tests {
    use crate::world::{Patch, World};

    #[test]
    fn forest_is_shaded_and_fruits_unevenly() {
        let world = World::generate(1);
        let terrain = world.terrain();
        let patches: Vec<_> = (0..terrain.len())
            .filter_map(|index| terrain.patch_at(terrain.middle_of(index)))
            .collect();
        let average = |of: &dyn Fn(&&Patch) -> u32| {
            patches.iter().map(of).sum::<u32>() / patches.len() as u32
        };

        let canopy = average(&|patch| u32::from(patch.canopy()));
        assert!((30..=70).contains(&canopy), "{canopy}% shaded on average");
        assert!(patches.iter().any(|patch| patch.canopy() == 0));
        assert!(patches.iter().any(|patch| patch.canopy() == 100));

        let fruiting = average(&|patch| u32::from(patch.fruiting()));
        assert!(
            (50..=110).contains(&fruiting),
            "{fruiting}% fruiting on average"
        );
        assert!(patches.iter().any(|patch| patch.fruiting() == 0));
        assert!(patches.iter().any(|patch| patch.fruiting() > 150));
    }
}
