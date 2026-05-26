/// Represents a location in a 2-dimensional plane, counted in millimeters.
///
/// A location is represented by the x and y coordinates relative to the center of the plane.
/// The coordinates are stored as integers, which means floating point errors are not a consideration.
/// This location implementation should only be used for simplistic representations of 2-dimensional flat worlds
/// and never in more complex, planetary contexts.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Location {
    x: i64,
    y: i64,
}

impl Location {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}