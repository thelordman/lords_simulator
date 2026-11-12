use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lords_sim::State;

use crate::speed::Speed;

/// The simulation as it was when the simulation thread last published it.
#[derive(Debug)]
pub struct Snapshot {
    /// Shared, so that it can be handed on, for instance to be saved, without copying it.
    pub state: Arc<State>,
    pub paused: bool,
    pub speed: Speed,
}

/// The latest [`Snapshot`], written by the simulation thread and read by its handle.
pub(crate) struct Published(Mutex<Arc<Snapshot>>);

impl Published {
    pub(crate) fn new(snapshot: Snapshot) -> Self {
        Self(Mutex::new(Arc::new(snapshot)))
    }

    pub(crate) fn get(&self) -> Arc<Snapshot> {
        Arc::clone(&self.lock())
    }

    pub(crate) fn set(&self, snapshot: Snapshot) {
        *self.lock() = Arc::new(snapshot);
    }

    fn lock(&self) -> MutexGuard<'_, Arc<Snapshot>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
