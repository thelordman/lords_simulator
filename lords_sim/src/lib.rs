#![warn(clippy::all, rust_2018_idioms)]

mod time;

pub use time::Time;
use std::fmt::{Display, Formatter};

pub struct Simulation {
    pub state: State,
}

impl Simulation {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self { state: State::new(name, sex) }
    }

    pub fn tick(&mut self) {
        self.state.time.advance()
    }
}

#[derive(Clone)]
pub struct State {
    pub name: Name,
    pub sex: Sex,
    pub time: Time,
}

impl State {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self { name, sex, time: Time::default() }
    }
}

#[derive(Default, Clone)]
pub struct Name {
    pub first: String,
    pub middle: Vec<String>,
    pub last: String,
}

impl Name {
    pub fn new(first: String, middle: Vec<String>, last: String) -> Self {
        Self { first, middle, last }
    }

    pub fn full_name(&self) -> String {
        std::iter::once(self.first.as_str())
            .chain(self.middle.iter().map(|s| s.as_str()))
            .chain(std::iter::once(self.last.as_str()))
            .collect::<Vec<_>>()
            .join(" ")
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