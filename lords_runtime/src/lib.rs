//! Runs a [`lords_sim::Simulation`] in real time.
//!
//! [`Runtime::spawn`] moves a simulation to a thread of its own, which ticks it at a steady
//! pace and publishes a [`Snapshot`] of it after every change. The returned [`Runtime`] is
//! the only way to reach the simulation from then on: it reads the latest snapshot, and
//! everything it asks of the simulation travels to the thread as a message, to be handled
//! between ticks.
//!
//! The crate knows nothing about what the simulation contains. Changes to the world are
//! [`lords_sim::Command`]s, which it passes along untouched.

#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod clock;
mod engine;
mod runtime;
mod snapshot;

pub use runtime::Runtime;
pub use snapshot::Snapshot;
