use serde::{Deserialize, Serialize};

use crate::world::{GroundType, Species, VegetationKind};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    #[default]
    FennoscandianTaiga,
}

impl Biome {
    /// Ordered from the wettest to the driest.
    pub fn forest_floor(self) -> &'static [(GroundType, VegetationKind, u32)] {
        match self {
            Self::FennoscandianTaiga => &[
                (GroundType::Peat, VegetationKind::PineBog, 8),
                (GroundType::Till, VegetationKind::HerbRichHeath, 21),
                (GroundType::Till, VegetationKind::BilberryHeath, 43),
                (GroundType::Till, VegetationKind::LingonberryHeath, 24),
                (GroundType::Sand, VegetationKind::LichenHeath, 4),
            ],
        }
    }

    pub fn beaches(self) -> &'static [(GroundType, Option<VegetationKind>, u32)] {
        match self {
            Self::FennoscandianTaiga => &[
                (GroundType::Gravel, None, 55),
                (GroundType::Mud, Some(VegetationKind::ShoreMeadow), 35),
                (GroundType::Sand, None, 10),
            ],
        }
    }

    pub fn lakeside(self) -> (GroundType, VegetationKind) {
        match self {
            Self::FennoscandianTaiga => (GroundType::Till, VegetationKind::HerbRichHeath),
        }
    }

    /// Returns the trees that grow on `vegetation`, if any.
    pub fn stand(self, vegetation: VegetationKind) -> Option<Stand> {
        use Species::{Aspen, Birch, GreyAlder, Pine, Spruce};

        let (gaps, trees): (u64, &'static [(Species, u64)]) = match (self, vegetation) {
            (Self::FennoscandianTaiga, VegetationKind::BilberryHeath) => {
                (10, &[(Spruce, 70), (Pine, 12), (Birch, 13), (Aspen, 5)])
            }
            (Self::FennoscandianTaiga, VegetationKind::LingonberryHeath) => {
                (10, &[(Pine, 75), (Spruce, 12), (Birch, 13)])
            }
            (Self::FennoscandianTaiga, VegetationKind::LichenHeath) => (35, &[(Pine, 95), (Birch, 5)]),
            (Self::FennoscandianTaiga, VegetationKind::HerbRichHeath) => (
                10,
                &[
                    (Spruce, 45),
                    (Birch, 30),
                    (Aspen, 10),
                    (GreyAlder, 10),
                    (Pine, 5),
                ],
            ),
            (Self::FennoscandianTaiga, VegetationKind::PineBog) => {
                (60, &[(Pine, 80), (Birch, 18), (Spruce, 2)])
            }
            (Self::FennoscandianTaiga, VegetationKind::ShoreMeadow) => return None,
        };

        Some(Stand { gaps, trees })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stand {
    /// Percentage of open places for a tree.
    pub gaps: u64,
    /// Each species with their respective percentages.
    pub trees: &'static [(Species, u64)],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Form;

    fn shares<T>(table: &[T], share: impl Fn(&T) -> u64) -> u64 {
        table.iter().map(share).sum()
    }

    #[test]
    fn forest_and_shore_are_shared_out_in_full() {
        let biome = Biome::FennoscandianTaiga;

        assert_eq!(
            shares(biome.forest_floor(), |(.., share)| u64::from(*share)),
            100
        );
        assert_eq!(
            shares(biome.beaches(), |(.., share)| u64::from(*share)),
            100
        );
    }

    #[test]
    fn stands_are_trees_shared_out_in_full() {
        let biome = Biome::FennoscandianTaiga;

        for vegetation in VegetationKind::ALL {
            let Some(stand) = biome.stand(vegetation) else {
                continue;
            };

            assert_eq!(
                shares(stand.trees, |(_, share)| *share),
                100,
                "{vegetation:?}"
            );
            assert!(stand.gaps < 100, "{vegetation:?}");
            for (species, _) in stand.trees {
                assert_eq!(species.form(), Form::Tree, "{species:?} on {vegetation:?}");
            }
        }
    }
}
