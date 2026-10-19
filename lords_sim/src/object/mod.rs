use std::fmt;
use std::fmt::{Display, Formatter};
use crate::Time;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Object {
    location: Location,
    creation: Time
}

impl Object {
    pub fn new(location: Location, creation: Time) -> Self {
        Self { location, creation }
    }
}

/// Represents a location on a 2-dimensional plane, with millimeter accuracy.
///
/// A location is represented by the x and y coordinates relative to the center of the plane.
/// The coordinates are stored as integers, and floating point errors are not a consideration.
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animal {
    object: Object,
    name: Name,
    sex: Sex,
}

impl Animal {
    pub fn new(location: Location, time: Time, name: Name, sex: Sex) -> Self {
        Self { object: Object::new(location, time), name, sex }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Name {
    first: String,
    middle: Vec<String>,
    last: String,
}

impl Name {
    pub fn new(first: String, middle: Vec<String>, last: String) -> Self {
        Self { first, middle, last }
    }

    pub fn first(&self) -> &str { &self.first }
    pub fn middle(&self) -> &[String] { &self.middle }
    pub fn last(&self) -> &str { &self.last }

    pub fn full(&self) -> String {
        self.to_string()
    }

    pub fn parts(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.first.as_str())
            .chain(self.middle.iter().map(String::as_str))
            .chain(std::iter::once(self.last.as_str()))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.first)?;
        for m in &self.middle {
            write!(f, " {m}")?;
        }
        write!(f, " {}", self.last)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

impl Display for Sex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Human {
    animal: Animal,
}

impl Human {
    pub fn new(location: Location, time: Time, name: Name, sex: Sex) -> Self {
        Self { animal: Animal::new(location, time, name, sex) }
    }
}

pub trait Living {
    fn animal(&self) -> &Animal;
    fn name(&self) -> &Name { &self.animal().name }
    fn sex(&self) -> Sex { self.animal().sex }
}

impl Living for Human {
    fn animal(&self) -> &Animal { &self.animal }
}