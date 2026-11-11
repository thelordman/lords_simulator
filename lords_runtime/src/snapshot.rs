use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lords_sim::State;

/// The simulation as it was when the simulation thread last published it.
#[derive(Debug)]
pub struct Snapshot {
    pub state: State,
    pub paused: bool,
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
