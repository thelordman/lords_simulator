use serde::{Deserialize, Serialize};

/// What the surface of the earth is made of, under whatever grows on it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GroundType {
    Till,
    Sand,
    Gravel,
    Mud,
    Peat,
}

impl GroundType {
    /// Returns how many pebbles lie loose on a square meter of it, on average.
    pub fn pebbles_per_square_meter(self) -> u32 {
        match self {
            Self::Till => 6,
            Self::Sand => 2,
            Self::Gravel => 250,
            Self::Mud => 1,
            Self::Peat => 0,
        }
    }
}
