use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use log::error;

use crate::error::SaveError;
use crate::format::SaveKind;
use crate::store::{SaveEntry, SaveRequest, SaveStore};

#[derive(Debug)]
pub struct SaveOutcome {
    pub kind: SaveKind,
    pub result: Result<SaveEntry, SaveError>,
}

pub struct SaveWorker {
    /// `None` once finished.
    requests: Option<Sender<SaveRequest>>,
    outcomes: Receiver<SaveOutcome>,
    thread: Option<JoinHandle<()>>,
}

impl SaveWorker {
    /// Starts a thread that writes to `store`, calling `on_done` from it after every save.
    pub fn spawn(store: SaveStore, on_done: impl Fn() + Send + 'static) -> Self {
        let (requests, received) = mpsc::channel::<SaveRequest>();
        let (finished, outcomes) = mpsc::channel();

        let thread = thread::Builder::new()
            .name("saving".to_owned())
            .spawn(move || {
                store.remove_stale_temp_files();

                for request in received {
                    let outcome = SaveOutcome {
                        kind: request.kind,
                        result: store.write(&request),
                    };
                    let _ = finished.send(outcome);
                    on_done();
                }
            })
            .expect("failed to spawn the saving thread");

        Self {
            requests: Some(requests),
            outcomes,
            thread: Some(thread),
        }
    }

    /// Asks for a save to be written, without waiting for it.
    pub fn save(&self, request: SaveRequest) {
        let sent = self
            .requests
            .as_ref()
            .is_some_and(|requests| requests.send(request).is_ok());
        if !sent {
            error!("The saving thread has stopped; the game was not saved");
        }
    }

    pub fn poll(&self) -> Option<SaveOutcome> {
        self.outcomes.try_recv().ok()
    }

    pub fn finish(&mut self) {
        self.requests = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            error!("The saving thread panicked");
        }
    }
}

impl Drop for SaveWorker {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use lords_sim::{Name, Sex, State};

    use super::*;
    use crate::format::SimulationId;

    const TIMEOUT: Duration = Duration::from_secs(10);

    fn request(playthrough: SimulationId, label: &str) -> SaveRequest {
        SaveRequest {
            playthrough,
            kind: SaveKind::Manual,
            label: label.to_owned(),
            speed: 1,
            state: Arc::new(State::new(Name::default(), Sex::Male)),
        }
    }

    #[test]
    fn save_is_written_in_the_background_and_reported() {
        let directory = tempfile::tempdir().expect("a temporary directory can be made");
        let store = SaveStore::new(directory.path());
        let (done, notified) = mpsc::channel();
        let worker = SaveWorker::spawn(store.clone(), move || {
            let _ = done.send(());
        });
        assert!(worker.poll().is_none());

        worker.save(request(SimulationId::generate(), "Background"));
        notified
            .recv_timeout(TIMEOUT)
            .expect("the save never finished");

        let outcome = worker.poll().expect("the save was not reported");
        let entry = outcome.result.expect("the save failed");
        assert_eq!(outcome.kind, SaveKind::Manual);
        assert_eq!(entry.header.label, "Background");
        assert_eq!(store.list().saves, [entry]);
        assert!(worker.poll().is_none());
    }

    #[test]
    fn finishing_writes_everything_asked_for() {
        let directory = tempfile::tempdir().expect("a temporary directory can be made");
        let store = SaveStore::new(directory.path());
        let mut worker = SaveWorker::spawn(store.clone(), || {});
        let playthrough = SimulationId::generate();

        for label in ["a", "b", "c", "d", "e"] {
            worker.save(request(playthrough, label));
        }
        worker.finish();

        assert_eq!(store.list().saves.len(), 5);
        let mut reported = 0;
        while let Some(outcome) = worker.poll() {
            assert!(outcome.result.is_ok());
            reported += 1;
        }
        assert_eq!(reported, 5);

        worker.save(request(playthrough, "too late"));
        assert_eq!(store.list().saves.len(), 5);
    }

    #[test]
    fn failed_save_is_reported() {
        let directory = tempfile::tempdir().expect("a temporary directory can be made");
        let blocked = directory.path().join("saves");
        std::fs::write(&blocked, b"a file where the store should be").expect("the file is written");
        let mut worker = SaveWorker::spawn(SaveStore::new(&blocked), || {});

        worker.save(request(SimulationId::generate(), "Doomed"));
        worker.finish();

        let outcome = worker.poll().expect("the failure was not reported");
        assert!(matches!(outcome.result, Err(SaveError::Io(_))));
    }
}
