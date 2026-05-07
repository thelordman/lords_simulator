#![warn(clippy::all, rust_2018_idioms)]

mod time;

use time::Time;
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
        let mut name = self.first.clone();
        for middle in &self.middle {
            name.push_str(" ");
            name.push_str(middle);
        }
        name.push_str(" ");
        name.push_str(&self.last);
        name
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