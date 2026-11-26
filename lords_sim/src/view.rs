use std::sync::Arc;

use crate::map::Map;
use crate::perception::Surroundings;
use crate::space::{Length, Location};
use crate::world::{Abundance, GroundType, Kind, Species, VegetationKind};
use crate::{Human, Timestamp};

/// What the player is shown of the simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub time: Timestamp,
    pub player: Human,
    pub surroundings: Arc<Surroundings>,
    pub map: Arc<Map>,
    pub walk: Option<Walk>,
    pub ground: Option<Ground>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    pub toward: Kind,
    pub destination: Location,
    pub via: Vec<Location>,
    pub remaining: Length,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ground {
    pub ground_type: GroundType,
    pub vegetation: Option<VegetationKind>,
    pub plants: Vec<Growth>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Growth {
    pub species: Species,
    /// The percentage of the total vegetation.
    pub share: u8,
    pub crop: Abundance,
}
