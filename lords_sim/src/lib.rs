use std::fmt::{Display, Formatter};

pub const TICKS_PER_SECOND: u64 = 1;

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
        Self { name, sex, time: Time::new() }
    }
}

#[derive(Clone)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Time {
    tick: u64
}


impl Time {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }
    
    pub fn zero(&mut self) {
        self.tick = 0;
    }

    pub fn seconds(self) -> u64 {
        self.tick / TICKS_PER_SECOND
    }

    pub fn minutes(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60)
    }

    pub fn hours(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60)
    }

    pub fn days(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24)
    }

    pub fn weeks(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 7)
    }

    // TODO: Proper month system
    pub fn months(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 30)
    }

    // TODO: Leap years
    pub fn years(self) -> u64 {
        self.tick / (TICKS_PER_SECOND * 60 * 60 * 24 * 365)
    }
}