use serde::{Deserialize, Serialize};

use crate::world::Species;

/// A kind of plant community covering the ground under any trees.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VegetationKind {
    BilberryHeath,
    LingonberryHeath,
    LichenHeath,
    HerbRichHeath,
    PineBog,
    ShoreMeadow,
}

impl VegetationKind {
    pub const ALL: [Self; 6] = [
        Self::BilberryHeath,
        Self::LingonberryHeath,
        Self::LichenHeath,
        Self::HerbRichHeath,
        Self::PineBog,
        Self::ShoreMeadow,
    ];

    /// Returns how much of the ground it hides, in percent.
    pub fn cover(self) -> u8 {
        match self {
            Self::BilberryHeath => 80,
            Self::LingonberryHeath => 75,
            Self::LichenHeath => 60,
            Self::HerbRichHeath => 70,
            Self::PineBog => 100,
            Self::ShoreMeadow => 50,
        }
    }

    /// Returns the species it is usually made up of, each with its share in percent.
    pub fn composition(self) -> &'static [(Species, u8)] {
        use Species::*;

        match self {
            Self::BilberryHeath => &[
                (Bilberry, 35),
                (FeatherMoss, 20),
                (StairStepMoss, 20),
                (Lingonberry, 10),
                (WavyHairGrass, 5),
                (MayLily, 5),
                (ArcticStarflower, 5),
            ],
            Self::LingonberryHeath => &[
                (Lingonberry, 30),
                (FeatherMoss, 30),
                (Bilberry, 10),
                (Crowberry, 10),
                (Heather, 10),
                (ReindeerLichen, 10),
            ],
            Self::LichenHeath => &[
                (ReindeerLichen, 50),
                (Heather, 20),
                (Lingonberry, 15),
                (Crowberry, 10),
                (FeatherMoss, 5),
            ],
            Self::HerbRichHeath => &[
                (Bilberry, 20),
                (StairStepMoss, 20),
                (WoodSorrel, 15),
                (MayLily, 15),
                (OakFern, 10),
                (ArcticStarflower, 10),
                (WoodCranesbill, 10),
            ],
            Self::PineBog => &[
                (PeatMoss, 45),
                (BogBilberry, 10),
                (LabradorTea, 10),
                (Cottongrass, 8),
                (Crowberry, 7),
                (Cranberry, 7),
                (Cloudberry, 5),
                (BogRosemary, 5),
                (Heather, 3),
            ],
            Self::ShoreMeadow => &[
                (BottleSedge, 40),
                (TuftedHairGrass, 20),
                (Meadowsweet, 20),
                (YellowLoosestrife, 10),
                (CommonReed, 10),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vegetation {
    kind: VegetationKind,
    /// Each species with its share in percent, the largest first.
    shares: Vec<(Species, u8)>,
}

impl Vegetation {
    /// Returns vegetation of `kind` made up of `shares`, leaving out species without a share.
    pub fn new(kind: VegetationKind, mut shares: Vec<(Species, u8)>) -> Self {
        shares.retain(|(_, percent)| *percent > 0);
        shares.sort_by_key(|(species, percent)| (u8::MAX - percent, *species));

        Self { kind, shares }
    }

    /// Returns vegetation of `kind` with the shares it usually has.
    pub fn typical(kind: VegetationKind) -> Self {
        Self::new(kind, kind.composition().to_vec())
    }

    pub fn kind(&self) -> VegetationKind {
        self.kind
    }

    pub fn shares(&self) -> &[(Species, u8)] {
        &self.shares
    }

    /// Returns the share of `species` in percent.
    pub fn share(&self, species: Species) -> u8 {
        self.shares
            .iter()
            .find(|(other, _)| *other == species)
            .map_or(0, |(_, percent)| *percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_of_vegetation_is_a_whole_of_different_species() {
        for kind in VegetationKind::ALL {
            let composition = kind.composition();
            let total: u32 = composition.iter().map(|(_, share)| u32::from(*share)).sum();
            let mut species: Vec<Species> = composition.iter().map(|(id, _)| *id).collect();
            species.sort_unstable();
            species.dedup();

            assert_eq!(total, 100, "{kind:?}");
            assert_eq!(species.len(), composition.len(), "{kind:?}");
            assert!(kind.cover() <= 100);
        }
    }

    #[test]
    fn shares_are_kept_largest_first_and_only_if_there_are_any() {
        let vegetation = Vegetation::new(
            VegetationKind::ShoreMeadow,
            vec![
                (Species::CommonReed, 10),
                (Species::Meadowsweet, 0),
                (Species::BottleSedge, 60),
                (Species::TuftedHairGrass, 30),
            ],
        );

        assert_eq!(
            vegetation.shares(),
            [
                (Species::BottleSedge, 60),
                (Species::TuftedHairGrass, 30),
                (Species::CommonReed, 10),
            ]
        );
        assert_eq!(vegetation.share(Species::TuftedHairGrass), 30);
        assert_eq!(vegetation.share(Species::Meadowsweet), 0);
    }
}
