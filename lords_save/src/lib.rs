//! Saves a [`lords_sim::State`] to disk and reads it back.

mod error;
mod format;
mod migrate;
mod store;
mod worker;

pub use error::SaveError;
pub use format::{Codec, Compression, SaveHeader, SaveKind, SimulationId};
pub use store::{
    AUTOSAVES_KEPT, LoadedSave, SaveEntry, SaveList, SaveRequest, SaveStore, UnreadableSave,
};
pub use worker::{SaveOutcome, SaveWorker};
