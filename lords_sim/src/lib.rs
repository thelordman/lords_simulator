#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

pub mod object;
pub mod time;

pub use crate::object::{Human, Name, Sex};
pub use crate::time::Timestamp;

pub struct Simulation {
    state: State,
}

impl Simulation {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            state: State::new(name, sex),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    /// Advances the simulation by one tick.
    pub fn tick(&mut self) {
        self.state.time.advance();
    }

    /// Applies a change requested from outside the simulation.
    ///
    /// Commands are the only way for anything but [`Self::tick`] to change the state, and
    /// are meant to be applied between ticks.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Reset => self.state.time = Timestamp::EPOCH,
        }
    }
}

/// A change to the simulation requested from outside it, such as by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Rewinds the simulation clock to [`Timestamp::EPOCH`].
    Reset,
}

#[derive(Debug, Clone)]
pub struct State {
    pub player: Human,

    pub time: Timestamp,
}

impl State {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            player: Human::new(Default::default(), Default::default(), name, sex),
            time: Timestamp::EPOCH,
        }
    }
}
