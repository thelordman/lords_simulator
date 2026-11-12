//! Runs a [`lords_sim::Simulation`] in real time.
//!
//! [`Runtime::spawn`] moves a simulation to a thread of its own, which ticks it at the pace
//! of its [`Speed`] and publishes a [`Snapshot`] of it after every change. The returned
//! [`Runtime`] is the only way to reach the simulation from then on: it reads the latest
//! snapshot, and everything it asks of the simulation travels to the thread as a message,
//! to be handled between ticks.
//!
//! The crate knows nothing about what the simulation contains. Changes to the world are
//! [`lords_sim::Command`]s, which it passes along untouched.

mod clock;
mod engine;
mod runtime;
mod snapshot;
mod speed;

pub use runtime::Runtime;
pub use snapshot::Snapshot;
pub use speed::Speed;
