pub mod object;
pub mod time;

use serde::{Deserialize, Serialize};

pub use crate::object::{Human, Name, Sex};
pub use crate::time::Timestamp;

/// Version of [`State`] as it is written to disk.
///
/// Raise this with any change to [`State`], or to anything inside it, that would make a
/// state saved before the change read back wrongly or not at all: removing or renaming a
/// field, changing its type, or changing what its value means. Adding a field marked
/// `#[serde(default)]` does not need it.
pub const STATE_VERSION: u32 = 1;

pub struct Simulation {
    state: State,
}

impl Simulation {
    pub fn new(name: Name, sex: Sex) -> Self {
        Self {
            state: State::new(name, sex),
        }
    }

    pub fn from_state(state: State) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    /// Advances the simulation by one tick.
    pub fn tick(&mut self) {
        self.state.time.advance();
    }

    /// Applies a change requested from outside the simulation.
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_carries_on_from_a_state() {
        let mut original = Simulation::new(Name::default(), Sex::Male);
        original.tick();
        original.tick();

        let mut resumed = Simulation::from_state(original.state().clone());
        assert_eq!(resumed.state(), original.state());

        resumed.tick();
        assert_eq!(resumed.state().time, Timestamp::from_seconds(3));
    }
}
