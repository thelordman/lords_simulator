use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lords_save::{LoadedSave, SaveKind, SaveList, SaveRequest, SaveStore, SaveWorker};

/// How long the player is told that the game was saved.
const SAVED_NOTICE: Duration = Duration::from_secs(2);

pub struct Saves {
    /// `None` if there is nowhere to keep saves on this system.
    disk: Option<(SaveStore, SaveWorker)>,
    generation: u64,
    saved_at: Option<Instant>,
    error: Option<String>,
}

impl Saves {
    pub fn new(directory: Option<PathBuf>, ctx: &egui::Context) -> Self {
        let disk = directory.map(|directory| {
            let store = SaveStore::new(directory);
            let ctx = ctx.clone();
            let worker = SaveWorker::spawn(store.clone(), move || ctx.request_repaint());
            (store, worker)
        });
        if disk.is_none() {
            log::warn!("There is nowhere to keep saves on this system; saving is turned off");
        }

        Self {
            disk,
            generation: 0,
            saved_at: None,
            error: None,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn poll(&mut self) {
        let Some((_, worker)) = &self.disk else {
            return;
        };

        while let Some(outcome) = worker.poll() {
            self.generation += 1;
            match outcome.result {
                Ok(_) if outcome.kind == SaveKind::Manual => self.saved_at = Some(Instant::now()),
                Ok(_) => {}
                Err(error) => {
                    log::error!("Saving failed: {error}");
                    self.error = Some(format!("The game could not be saved: {error}."));
                }
            }
        }
    }

    pub fn list(&self) -> SaveList {
        self.disk
            .as_ref()
            .map(|(store, _)| store.list())
            .unwrap_or_default()
    }

    pub fn save(&mut self, request: SaveRequest) {
        match &self.disk {
            Some((_, worker)) => worker.save(request),
            None if request.kind == SaveKind::Manual => {
                self.error = Some(
                    "The game could not be saved: there is nowhere to keep saves on this system."
                        .to_owned(),
                );
            }
            None => {}
        }
    }

    pub fn load(&mut self, path: &Path) -> Option<LoadedSave> {
        let (store, _) = self.disk.as_ref()?;

        match store.load(path) {
            Ok(save) => Some(save),
            Err(error) => {
                self.error = Some(format!("The save could not be loaded: {error}."));
                None
            }
        }
    }

    pub fn delete(&mut self, path: &Path) {
        let Some((store, _)) = &self.disk else {
            return;
        };

        if let Err(error) = store.delete(path) {
            self.error = Some(format!("The save could not be deleted: {error}."));
        }
        self.generation += 1;
    }

    pub fn export(&mut self, path: &Path) -> Option<PathBuf> {
        let (store, _) = self.disk.as_ref()?;

        match store.export_ron(path) {
            Ok(exported) => Some(exported),
            Err(error) => {
                self.error = Some(format!("The save could not be exported: {error}."));
                None
            }
        }
    }

    pub fn saved_notice(&self) -> Option<Duration> {
        SAVED_NOTICE.checked_sub(self.saved_at?.elapsed())
    }

    /// Returns what went wrong last, until it is [dismissed](Self::dismiss_error).
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn dismiss_error(&mut self) {
        self.error = None;
    }

    pub fn finish(&mut self) {
        if let Some((_, worker)) = &mut self.disk {
            worker.finish();
        }
        self.poll();
    }
}
