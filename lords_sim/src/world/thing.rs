use serde::{Deserialize, Serialize};

use crate::object::Object;
use crate::space::{Length, Location};
use crate::world::{Kind, Species};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Thing {
    Plant(Plant),
    Rock(Rock),
}

impl Thing {
    pub fn object(&self) -> &Object {
        match self {
            Self::Plant(plant) => &plant.object,
            Self::Rock(rock) => &rock.object,
        }
    }

    pub fn location(&self) -> Location {
        self.object().location()
    }

    pub fn kind(&self) -> Kind {
        match self {
            Self::Plant(plant) => Kind::Plant(plant.species),
            Self::Rock(_) => Kind::Rock,
        }
    }

    /// Returns the radius of the ground this thing stands on.
    pub fn footprint(&self) -> Length {
        let diameter = match self {
            Self::Plant(plant) => plant.trunk_diameter,
            Self::Rock(rock) => rock.diameter,
        };
        diameter / 2
    }

    pub fn height(&self) -> Length {
        match self {
            Self::Plant(plant) => plant.height,
            Self::Rock(rock) => rock.height,
        }
    }
}

/// A plant that stands as a thing of its own, such as a tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plant {
    object: Object,
    species: Species,
    height: Length,
    trunk_diameter: Length,
}

impl Plant {
    pub fn new(object: Object, species: Species, height: Length, trunk_diameter: Length) -> Self {
        Self {
            object,
            species,
            height,
            trunk_diameter,
        }
    }

    pub fn species(&self) -> Species {
        self.species
    }

    pub fn height(&self) -> Length {
        self.height
    }

    pub fn trunk_diameter(&self) -> Length {
        self.trunk_diameter
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rock {
    object: Object,
    diameter: Length,
    height: Length,
}

impl Rock {
    pub fn new(object: Object, diameter: Length, height: Length) -> Self {
        Self {
            object,
            diameter,
            height,
        }
    }

    pub fn diameter(&self) -> Length {
        self.diameter
    }

    pub fn height(&self) -> Length {
        self.height
    }
}
