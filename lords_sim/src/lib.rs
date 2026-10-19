#![warn(clippy::all, rust_2018_idioms)]

mod time;
pub mod object;

pub use time::Time;
pub use crate::object::{Human, Name, Sex};

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
    pub player: Human,

    pub time: Time,
}

impl State {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self { player: Human::new(Default::default(), Default::default(), name, sex), time: Time::default() }
    }
}