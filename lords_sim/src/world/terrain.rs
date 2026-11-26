mod ground_type;
mod patch;
mod vegetation;

use std::ops::Range;

use serde::{Deserialize, Serialize};

pub use self::ground_type::GroundType;
pub use self::patch::{Abundance, Patch};
pub use self::vegetation::{Vegetation, VegetationKind};
use crate::space::{Length, Location};
use crate::world::Store;

/// The ground of a stretch of the world, as a grid of square patches.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terrain {
    /// The south-west corner of the grid.
    corner: Location,
    /// How many patches there are from west to east.
    columns: u32,
    /// Row by row from the south, each from the west.
    patches: Store<Patch>,
}

impl Terrain {
    pub const PATCH_SIDE: Length = Length::from_meters(4);

    /// The ground a patch covers, in square meters.
    pub const PATCH_AREA: u32 = (Self::PATCH_SIDE.as_millimeters() / 1_000).pow(2) as u32;

    /// Returns a grid `columns` patches wide with its south-west corner at `corner`, filled
    /// with `patches` row by row from the south.
    pub fn new(corner: Location, columns: u32, patches: impl IntoIterator<Item = Patch>) -> Self {
        Self {
            corner,
            columns,
            patches: patches.into_iter().collect(),
        }
    }

    pub fn patch_at(&self, at: Location) -> Option<&Patch> {
        self.patches.get(self.index_of(at)?)
    }

    pub(crate) fn patch_at_mut(&mut self, at: Location) -> Option<&mut Patch> {
        let index = self.index_of(at)?;
        self.patches.get_mut(index)
    }

    /// Returns how many patches there are.
    pub(crate) fn len(&self) -> u32 {
        self.patches.len() as u32
    }

    pub(crate) fn patch_mut(&mut self, index: u32) -> Option<&mut Patch> {
        self.patches.get_mut(index)
    }

    /// Returns each patch, with its index, that reaches into the square `reach` each way
    /// from `at`.
    pub(crate) fn around(
        &self,
        at: Location,
        reach: Length,
    ) -> impl Iterator<Item = (u32, &Patch)> {
        let reach = reach.as_millimeters();
        let rows = (self.patches.len() as u32)
            .checked_div(self.columns)
            .unwrap_or(0);
        let columns = span(at.x(), self.corner.x(), reach, self.columns);
        let width = self.columns;

        span(at.y(), self.corner.y(), reach, rows)
            .flat_map(move |row| columns.clone().map(move |column| row * width + column))
            .filter_map(|index| Some((index, self.patches.get(index)?)))
    }

    /// Returns the south-west corner of the patch at `index`.
    pub(crate) fn corner_of(&self, index: u32) -> Location {
        let side = Self::PATCH_SIDE.as_millimeters();
        let columns = self.columns.max(1);
        let east = i64::from(index % columns) * side;
        let north = i64::from(index / columns) * side;

        Location::new(self.corner.x() + east, self.corner.y() + north)
    }

    pub(crate) fn middle_of(&self, index: u32) -> Location {
        let half = Self::PATCH_SIDE.as_millimeters() / 2;
        let corner = self.corner_of(index);

        Location::new(corner.x() + half, corner.y() + half)
    }

    pub(crate) fn index_of(&self, at: Location) -> Option<u32> {
        let side = Self::PATCH_SIDE.as_millimeters();
        let east = at.x().checked_sub(self.corner.x())?;
        let north = at.y().checked_sub(self.corner.y())?;
        if east < 0 || north < 0 {
            return None;
        }

        let column = u32::try_from(east / side).ok()?;
        let row = u32::try_from(north / side).ok()?;
        if column >= self.columns {
            return None;
        }
        let index = row.checked_mul(self.columns)?.checked_add(column)?;
        (index < self.len()).then_some(index)
    }
}

/// Returns which of `count` rows or columns starting at `low` reach into the stretch
/// `reach` either way of `from`.
fn span(from: i64, low: i64, reach: i64, count: u32) -> Range<u32> {
    let side = Terrain::PATCH_SIDE.as_millimeters();
    let count = i64::from(count);
    let first = from
        .saturating_sub(reach)
        .saturating_sub(low)
        .div_euclid(side);
    let last = from
        .saturating_add(reach)
        .saturating_sub(low)
        .div_euclid(side);
    let first = first.clamp(0, count);
    let end = (last + 1).clamp(first, count);

    first as u32..end as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A grid two patches wide and three tall, from 8 m west and 4 m south of the center.
    fn terrain() -> Terrain {
        let grounds = [
            GroundType::Till,
            GroundType::Sand,
            GroundType::Gravel,
            GroundType::Mud,
            GroundType::Peat,
            GroundType::Till,
        ];

        Terrain::new(
            Location::new(-8_000, -4_000),
            2,
            grounds.map(|ground| Patch::new(ground, None, 0)),
        )
    }

    #[test]
    fn location_is_on_the_patch_whose_square_holds_it() {
        let terrain = terrain();
        let ground = |x, y| {
            terrain
                .patch_at(Location::new(x, y))
                .map(Patch::ground_type)
        };

        assert_eq!(ground(-8_000, -4_000), Some(GroundType::Till));
        assert_eq!(ground(-4_001, -1), Some(GroundType::Till));
        assert_eq!(ground(-4_000, -4_000), Some(GroundType::Sand));
        assert_eq!(ground(-8_000, 0), Some(GroundType::Gravel));
        assert_eq!(ground(-1, 3_999), Some(GroundType::Mud));
        assert_eq!(ground(-5_000, 4_000), Some(GroundType::Peat));
        assert_eq!(ground(-1, 7_999), Some(GroundType::Till));
    }

    #[test]
    fn nothing_is_off_the_grid() {
        let terrain = terrain();
        let off = |x, y| terrain.patch_at(Location::new(x, y)).is_none();

        assert!(off(-8_001, 0));
        assert!(off(-5_000, -4_001));
        assert!(off(0, 0));
        assert!(off(-5_000, 8_000));
        assert!(off(i64::MAX, i64::MAX));
        assert!(off(i64::MIN, 0));
        assert!(Terrain::default().patch_at(Location::default()).is_none());
    }

    #[test]
    fn patches_around_a_place_are_those_some_part_of_which_is_in_reach() {
        let terrain = terrain();
        let around = |x, y, reach| {
            terrain
                .around(Location::new(x, y), Length::from_millimeters(reach))
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        };

        assert_eq!(around(-6_000, -2_000, 0), [0]);
        assert_eq!(around(-4_000, 0, 1), [0, 1, 2, 3]);
        assert_eq!(around(-4_000, 0, 100_000), [0, 1, 2, 3, 4, 5]);
        assert_eq!(around(-1_000, 7_000, 500), [5]);
        assert_eq!(around(10_000, 0, 1_000), []);
        assert_eq!(around(-6_000, -20_000, 1_000), []);
        assert_eq!(around(0, 0, i64::MAX), [0, 1, 2, 3, 4, 5]);
        assert_eq!(around(i64::MIN, i64::MIN, i64::MAX), [0, 1]);
        assert_eq!(around(i64::MAX, i64::MAX, i64::MAX), []);
        let empty = Terrain::default();
        assert_eq!(
            empty
                .around(Location::default(), Length::from_meters(10))
                .count(),
            0
        );
    }

    #[test]
    fn patch_is_found_again_at_its_corner_and_middle() {
        let terrain = terrain();

        assert_eq!(terrain.corner_of(0), Location::new(-8_000, -4_000));
        assert_eq!(terrain.corner_of(1), Location::new(-4_000, -4_000));
        assert_eq!(terrain.middle_of(4), Location::new(-6_000, 6_000));
        for (index, patch) in terrain.around(Location::default(), Length::from_meters(20)) {
            assert_eq!(terrain.patch_at(terrain.corner_of(index)), Some(patch));
            assert_eq!(terrain.patch_at(terrain.middle_of(index)), Some(patch));
        }
    }

    #[test]
    fn patch_changed_through_the_grid_stays_changed() {
        let mut terrain = terrain();
        let at = Location::new(-1, -1);
        *terrain.patch_at_mut(at).expect("the patch is there") =
            Patch::new(GroundType::Peat, None, 3);

        assert_eq!(
            terrain.patch_at(at),
            Some(&Patch::new(GroundType::Peat, None, 3))
        );
        assert!(terrain.patch_at_mut(Location::new(1, 1)).is_none());
    }

    #[test]
    fn patch_covers_the_square_of_its_side() {
        let side = Terrain::PATCH_SIDE.as_millimeters();

        assert_eq!(i64::from(Terrain::PATCH_AREA) * 1_000_000, side * side);
    }
}
