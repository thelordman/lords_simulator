use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Species {
    Spruce,
    Pine,
    Birch,
    Aspen,
    GreyAlder,
    Bilberry,
    Lingonberry,
    Crowberry,
    Heather,
    BogBilberry,
    LabradorTea,
    Cranberry,
    BogRosemary,
    Cloudberry,
    MayLily,
    WoodSorrel,
    ArcticStarflower,
    WoodCranesbill,
    Meadowsweet,
    YellowLoosestrife,
    WavyHairGrass,
    TuftedHairGrass,
    Cottongrass,
    BottleSedge,
    CommonReed,
    OakFern,
    FeatherMoss,
    StairStepMoss,
    PeatMoss,
    ReindeerLichen,
}

impl Species {
    pub const ALL: [Self; 30] = [
        Self::Spruce,
        Self::Pine,
        Self::Birch,
        Self::Aspen,
        Self::GreyAlder,
        Self::Bilberry,
        Self::Lingonberry,
        Self::Crowberry,
        Self::Heather,
        Self::BogBilberry,
        Self::LabradorTea,
        Self::Cranberry,
        Self::BogRosemary,
        Self::Cloudberry,
        Self::MayLily,
        Self::WoodSorrel,
        Self::ArcticStarflower,
        Self::WoodCranesbill,
        Self::Meadowsweet,
        Self::YellowLoosestrife,
        Self::WavyHairGrass,
        Self::TuftedHairGrass,
        Self::Cottongrass,
        Self::BottleSedge,
        Self::CommonReed,
        Self::OakFern,
        Self::FeatherMoss,
        Self::StairStepMoss,
        Self::PeatMoss,
        Self::ReindeerLichen,
    ];

    pub fn form(self) -> Form {
        self.profile().0
    }

    pub fn crop(self) -> Option<Crop> {
        self.profile().1.map(|(part, per_square_meter, bulk)| Crop {
            part,
            per_square_meter,
            bulk,
        })
    }

    pub fn fruiting(self) -> Option<Fruiting> {
        let (ripe_from, peak, over_by, lasting, lasting_until, best_canopy) = match self {
            Self::Cloudberry => (190, (196, 212), 221, 0, 221, 0),
            Self::Bilberry => (195, (205, 226), 262, 0, 262, 40),
            Self::BogBilberry => (212, (221, 247), 273, 0, 273, 0),
            Self::Crowberry => (212, (231, 272), 318, 30, 530, 0),
            Self::Lingonberry => (231, (243, 272), 304, 20, 515, 10),
            Self::Cranberry => (243, (262, 303), 333, 40, 515, 0),
            _ => return None,
        };

        Some(Fruiting {
            ripe_from,
            peak,
            over_by,
            lasting,
            lasting_until,
            best_canopy,
        })
    }

    /// Returns how it grows and what is picked of it, as the part, how many of the part grow
    /// on a square meter that the plant covers, and the room one takes up in cubic
    /// millimeters.
    ///
    /// Berries are counted for an average year, on plants that fruit as well as their kind
    /// usually does.
    fn profile(self) -> (Form, Option<(Part, u32, u32)>) {
        use Form::*;
        use Part::*;

        match self {
            Self::Spruce => (Tree, None),
            Self::Pine => (Tree, None),
            Self::Birch => (Tree, None),
            Self::Aspen => (Tree, None),
            Self::GreyAlder => (Tree, None),
            Self::Bilberry => (DwarfShrub, Some((Berry, 15, 700))),
            Self::Lingonberry => (DwarfShrub, Some((Berry, 45, 600))),
            Self::Crowberry => (DwarfShrub, Some((Berry, 50, 400))),
            Self::Heather => (DwarfShrub, Some((Sprig, 40, 20_000))),
            Self::BogBilberry => (DwarfShrub, Some((Berry, 15, 900))),
            Self::LabradorTea => (DwarfShrub, Some((Sprig, 20, 20_000))),
            Self::Cranberry => (DwarfShrub, Some((Berry, 60, 700))),
            Self::BogRosemary => (DwarfShrub, Some((Sprig, 20, 15_000))),
            Self::Cloudberry => (Herb, Some((Berry, 12, 3_000))),
            Self::MayLily => (Herb, Some((Stem, 120, 8_000))),
            Self::WoodSorrel => (Herb, Some((Leaf, 300, 1_000))),
            Self::ArcticStarflower => (Herb, Some((Stem, 60, 5_000))),
            Self::WoodCranesbill => (Herb, Some((Stem, 15, 40_000))),
            Self::Meadowsweet => (Herb, Some((Stem, 20, 60_000))),
            Self::YellowLoosestrife => (Herb, Some((Stem, 20, 50_000))),
            Self::WavyHairGrass => (Graminoid, Some((Tuft, 40, 20_000))),
            Self::TuftedHairGrass => (Graminoid, Some((Tuft, 20, 40_000))),
            Self::Cottongrass => (Graminoid, Some((Tuft, 30, 15_000))),
            Self::BottleSedge => (Graminoid, Some((Tuft, 40, 40_000))),
            Self::CommonReed => (Graminoid, Some((Stalk, 80, 150_000))),
            Self::OakFern => (Fern, Some((Frond, 30, 30_000))),
            Self::FeatherMoss => (Moss, Some((Clump, 25, 100_000))),
            Self::StairStepMoss => (Moss, Some((Clump, 25, 100_000))),
            Self::PeatMoss => (Moss, Some((Clump, 25, 150_000))),
            Self::ReindeerLichen => (Lichen, Some((Tuft, 25, 50_000))),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Form {
    Tree,
    /// A low woody plant, such as a berry bush or heather.
    DwarfShrub,
    /// A flowering plant with soft stems.
    Herb,
    /// A grass, sedge, or reed.
    Graminoid,
    Fern,
    Moss,
    Lichen,
}

impl Form {
    pub fn covers_the_ground(self) -> bool {
        match self {
            Self::Tree => false,
            Self::DwarfShrub
            | Self::Herb
            | Self::Graminoid
            | Self::Fern
            | Self::Moss
            | Self::Lichen => true,
        }
    }
}

/// What is picked of a plant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Crop {
    pub part: Part,
    pub per_square_meter: u32,
    /// The room one takes up when carried, in cubic millimeters.
    pub bulk: u32,
}

/// A part of a plant that is picked from it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Berry,
    Sprig,
    Leaf,
    Stem,
    Tuft,
    Stalk,
    Frond,
    Clump,
}

impl Part {
    /// Returns whether the plant goes on covering the ground once it is picked.
    pub fn leaves_the_plant(self) -> bool {
        match self {
            Self::Berry => true,
            Self::Sprig
            | Self::Leaf
            | Self::Stem
            | Self::Tuft
            | Self::Stalk
            | Self::Frond
            | Self::Clump => false,
        }
    }
}

/// When a plant's berries are on it and how much light it fruits best in.
///
/// Days are counted from 0 on 1 January of the year the berries ripen, and on past the end
/// of that year for berries that last the winter.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fruiting {
    /// The day the first berries are ripe.
    pub ripe_from: u16,
    /// The first and the last day on which the whole crop is ripe.
    pub peak: (u16, u16),
    /// The day by which the berries have dropped or been eaten, but for those that last.
    pub over_by: u16,
    /// The share of the crop that lasts on the plants after that, in percent.
    pub lasting: u8,
    /// The day by which the berries that last are gone too.
    pub lasting_until: u16,
    /// The share of the sky hidden by tree crowns that it fruits best under, in percent.
    pub best_canopy: u8,
}

impl Fruiting {
    /// Returns how much of the crop is ripe on the plants `day` days into its year, in
    /// percent.
    pub fn ripe(self, day: u16) -> u32 {
        let (peak_from, peak_to) = self.peak;
        let lasting = u32::from(self.lasting);
        let between = |from: u16, to: u16, start: u32, end: u32| {
            let along = u32::from(day - from);
            let length = u32::from(to - from).max(1);
            (start * (length - along) + end * along) / length
        };

        if day < self.ripe_from {
            0
        } else if day < peak_from {
            between(self.ripe_from, peak_from, 0, 100)
        } else if day <= peak_to {
            100
        } else if day < self.over_by {
            between(peak_to, self.over_by, 100, lasting)
        } else if day < self.lasting_until {
            lasting
        } else {
            0
        }
    }

    /// Returns how well it fruits with `canopy` percent of the sky hidden by tree crowns, in
    /// percent of what it usually bears.
    pub fn in_light(self, canopy: u8) -> u32 {
        let away = u32::from(canopy.abs_diff(self.best_canopy));
        140_u32.saturating_sub(away * 5 / 3).max(15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::VegetationKind;

    #[test]
    fn every_species_is_listed_once() {
        let mut listed = Species::ALL.to_vec();
        listed.sort_unstable();
        listed.dedup();

        assert_eq!(listed.len(), Species::ALL.len());
    }

    #[test]
    fn every_crop_grows_and_is_small_enough_to_carry() {
        for species in Species::ALL {
            let Some(crop) = species.crop() else {
                continue;
            };

            assert!(crop.per_square_meter > 0, "{species:?}");
            assert!((1..=200_000).contains(&crop.bulk), "{species:?}");
        }
        assert_eq!(Species::Spruce.crop(), None);
        assert_eq!(
            Species::Bilberry.crop().map(|crop| crop.part),
            Some(Part::Berry)
        );
    }

    #[test]
    fn what_covers_the_ground_grows_in_some_vegetation_and_nothing_else_does() {
        for species in Species::ALL {
            let grows = VegetationKind::ALL.iter().any(|kind| {
                kind.composition()
                    .iter()
                    .any(|(grown, _)| *grown == species)
            });

            assert_eq!(grows, species.form().covers_the_ground(), "{species:?}");
        }
    }

    #[test]
    fn berries_and_only_berries_ripen_in_season() {
        for species in Species::ALL {
            let berry = species.crop().is_some_and(|crop| crop.part == Part::Berry);
            let Some(fruiting) = species.fruiting() else {
                assert!(!berry, "{species:?} bears berries that never ripen");
                continue;
            };
            let (peak_from, peak_to) = fruiting.peak;

            assert!(berry, "{species:?} ripens without bearing berries");
            assert!(fruiting.ripe_from < peak_from, "{species:?}");
            assert!(
                peak_from <= peak_to && peak_to < fruiting.over_by,
                "{species:?}"
            );
            assert!(fruiting.over_by <= fruiting.lasting_until, "{species:?}");
            assert!(
                fruiting.lasting_until < fruiting.ripe_from + 365,
                "{species:?}"
            );
        }
    }

    #[test]
    fn bilberries_ripen_in_july_and_are_gone_by_autumn() {
        let fruiting = Species::Bilberry.fruiting().expect("bilberries ripen");

        assert_eq!(fruiting.ripe(150), 0);
        assert_eq!(fruiting.ripe(200), 50);
        assert_eq!(fruiting.ripe(215), 100);
        assert!((1..100).contains(&fruiting.ripe(240)));
        assert_eq!(fruiting.ripe(300), 0);
    }

    #[test]
    fn some_lingonberries_last_the_winter() {
        let fruiting = Species::Lingonberry
            .fruiting()
            .expect("lingonberries ripen");

        assert_eq!(fruiting.ripe(365 + 60), 20);
        assert_eq!(fruiting.ripe(365 + 160), 0);
    }

    #[test]
    fn berries_fruit_best_in_the_light_they_like() {
        let bilberry = Species::Bilberry.fruiting().expect("bilberries ripen");
        let lingonberry = Species::Lingonberry
            .fruiting()
            .expect("lingonberries ripen");

        assert_eq!(bilberry.in_light(40), 140);
        assert!(bilberry.in_light(40) > bilberry.in_light(0));
        assert!(bilberry.in_light(0) > bilberry.in_light(100));
        assert!(lingonberry.in_light(10) > lingonberry.in_light(60));
        assert_eq!(lingonberry.in_light(100), 15);
    }
}
