use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use log::warn;
use lords_sim::State;
use serde::Serialize;

use crate::error::SaveError;
use crate::format::{self, Codec, Compression, Details, SimulationId, SaveHeader, SaveKind};
use crate::migrate;

pub const AUTOSAVES_KEPT: usize = 3;

const EXTENSION: &str = "lss";
const EXPORT_EXTENSION: &str = "ron";

/// Appended to the name of a file while it is being written.
const TEMP_SUFFIX: &str = ".tmp";

const AUTOSAVE_PREFIX: &str = "auto-";
const MAX_SLUG_LEN: usize = 40;

#[derive(Debug, Clone)]
pub struct SaveStore {
    root: PathBuf,
    codec: Codec,
    compression: Compression,
}

#[derive(Debug, Clone)]
pub struct SaveRequest {
    pub playthrough: SimulationId,
    pub kind: SaveKind,
    pub label: String,
    pub speed: u32,
    pub state: Arc<State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub header: SaveHeader,
}

impl SaveEntry {
    pub fn problem(&self) -> Option<SaveError> {
        migrate::check(self.header.state_version).err()
    }
}

#[derive(Debug)]
pub struct UnreadableSave {
    pub path: PathBuf,
    pub error: SaveError,
}

#[derive(Debug, Default)]
pub struct SaveList {
    /// Newest first.
    pub saves: Vec<SaveEntry>,
    pub unreadable: Vec<UnreadableSave>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSave {
    pub header: SaveHeader,
    pub state: State,
}

impl SaveStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            codec: Codec::MessagePack,
            compression: Compression::Lz4,
        }
    }

    pub fn with_format(mut self, codec: Codec, compression: Compression) -> Self {
        self.codec = codec;
        self.compression = compression;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> SaveList {
        let mut list = SaveList::default();

        for directory in entries(&self.root) {
            if !directory.is_dir() {
                continue;
            }
            for path in entries(&directory) {
                if path.extension() != Some(OsStr::new(EXTENSION)) {
                    continue;
                }
                match read_header_at(&path) {
                    Ok(header) => list.saves.push(SaveEntry { path, header }),
                    Err(error) => list.unreadable.push(UnreadableSave { path, error }),
                }
            }
        }

        list.saves.sort_by(|a, b| {
            b.header
                .saved_at_ms
                .cmp(&a.header.saved_at_ms)
                .then_with(|| a.path.cmp(&b.path))
        });
        list.unreadable.sort_by(|a, b| a.path.cmp(&b.path));
        list
    }

    pub fn load(&self, path: &Path) -> Result<LoadedSave, SaveError> {
        let file = fs::read(path)?;
        let (header, payload) = format::decode(&file)?;
        let state = migrate::decode_state(&header, &payload)?;
        Ok(LoadedSave { header, state })
    }

    pub fn write(&self, request: &SaveRequest) -> Result<SaveEntry, SaveError> {
        self.write_at(request, now_ms())
    }

    fn write_at(&self, request: &SaveRequest, saved_at_ms: u64) -> Result<SaveEntry, SaveError> {
        let label = match request.kind {
            SaveKind::Manual => request.label.as_str(),
            SaveKind::Auto | SaveKind::Exit => "",
        };
        let details = Details {
            playthrough: request.playthrough,
            kind: request.kind,
            label,
            saved_at_ms,
            speed: request.speed,
        };
        let (header, file) = format::encode(details, &request.state, self.codec, self.compression)?;

        let directory = self.root.join(request.playthrough.to_string());
        fs::create_dir_all(&directory)?;
        let path = directory.join(file_name(request.kind, label, saved_at_ms));
        atomic_write(&path, &file)?;

        if request.kind == SaveKind::Auto {
            prune_autosaves(&directory);
        }

        Ok(SaveEntry { path, header })
    }

    pub fn delete(&self, path: &Path) -> Result<(), SaveError> {
        let directory = path.parent().ok_or(SaveError::NotInStore)?;
        if path.extension() != Some(OsStr::new(EXTENSION))
            || directory.parent() != Some(self.root.as_path())
        {
            return Err(SaveError::NotInStore);
        }

        fs::remove_file(path)?;
        let _ = fs::remove_dir(directory);
        Ok(())
    }

    pub fn export_ron(&self, path: &Path) -> Result<PathBuf, SaveError> {
        #[derive(Serialize)]
        struct Export<'a> {
            header: &'a SaveHeader,
            state: &'a State,
        }

        let LoadedSave { header, state } = self.load(path)?;
        let export = Export {
            header: &header,
            state: &state,
        };
        let text = ron::ser::to_string_pretty(&export, ron::ser::PrettyConfig::default())
            .map_err(|error| SaveError::Encode(error.to_string()))?;

        let destination = path.with_extension(EXPORT_EXTENSION);
        atomic_write(&destination, text.as_bytes())?;
        Ok(destination)
    }

    /// Deletes what is left of saves that were interrupted while being written.
    pub(crate) fn remove_stale_temp_files(&self) {
        for directory in entries(&self.root) {
            for path in entries(&directory) {
                let is_temp = path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.ends_with(TEMP_SUFFIX));
                if is_temp && let Err(error) = fs::remove_file(&path) {
                    warn!("Could not delete {}: {error}", path.display());
                }
            }
        }
    }
}

fn read_header_at(path: &Path) -> Result<SaveHeader, SaveError> {
    let mut reader = BufReader::new(File::open(path)?);
    format::read_header(&mut reader)
}

/// Returns the paths of everything in `directory`, or nothing if it cannot be read.
fn entries(directory: &Path) -> Vec<PathBuf> {
    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect(),
        Err(error) => {
            if !matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) {
                warn!("Could not look in {}: {error}", directory.display());
            }
            Vec::new()
        }
    }
}

fn file_name(kind: SaveKind, label: &str, saved_at_ms: u64) -> String {
    match kind {
        SaveKind::Manual => format!(
            "manual-{}-{:08x}.{EXTENSION}",
            slug(label),
            crc32fast::hash(label.as_bytes())
        ),
        SaveKind::Auto => format!("{AUTOSAVE_PREFIX}{saved_at_ms:014}.{EXTENSION}"),
        SaveKind::Exit => format!("exit.{EXTENSION}"),
    }
}

/// Reduces `label` to something that is safe in a file name on every system.
fn slug(label: &str) -> String {
    let mut slug = String::new();
    for character in label.chars() {
        if slug.len() >= MAX_SLUG_LEN {
            break;
        }
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }

    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "save".to_owned()
    } else {
        slug.to_owned()
    }
}

fn prune_autosaves(directory: &Path) {
    let mut autosaves: Vec<PathBuf> = entries(directory)
        .into_iter()
        .filter(|path| {
            path.extension() == Some(OsStr::new(EXTENSION))
                && path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.starts_with(AUTOSAVE_PREFIX))
        })
        .collect();
    autosaves.sort();

    let excess = autosaves.len().saturating_sub(AUTOSAVES_KEPT);
    for path in &autosaves[..excess] {
        if let Err(error) = fs::remove_file(path) {
            warn!(
                "Could not delete the old autosave {}: {error}",
                path.display()
            );
        }
    }
}

fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(TEMP_SUFFIX);
    let temp = PathBuf::from(temp);

    let moved = write_and_sync(&temp, contents).and_then(|()| fs::rename(&temp, path));
    if moved.is_err() {
        let _ = fs::remove_file(&temp);
    }
    moved
}

fn write_and_sync(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| {
            u64::try_from(since_epoch.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, STATE_VERSION, Sex, Timestamp};
    use tempfile::TempDir;

    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    fn store() -> (TempDir, SaveStore) {
        let directory = tempfile::tempdir().expect("a temporary directory can be made");
        let store = SaveStore::new(directory.path().join("saves"));
        (directory, store)
    }

    fn state(seconds: u64) -> Arc<State> {
        let mut state = State::new(
            Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned()),
            Sex::Female,
        );
        state.time = Timestamp::from_seconds(seconds);
        Arc::new(state)
    }

    fn request(playthrough: &SimulationId, kind: SaveKind, label: &str) -> SaveRequest {
        SaveRequest {
            playthrough: *playthrough,
            kind,
            label: label.to_owned(),
            speed: 5,
            state: state(60),
        }
    }

    fn file_names(directory: &Path) -> Vec<String> {
        let mut names: Vec<String> = entries(directory)
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_owned))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn written_save_is_listed_and_loads() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();

        let written = store
            .write(&request(&playthrough, SaveKind::Manual, "First steps"))
            .expect("the save is written");

        let list = store.list();
        assert_eq!(list.saves, std::slice::from_ref(&written));
        assert!(list.unreadable.is_empty());
        assert!(written.problem().is_none());

        let loaded = store.load(&written.path).expect("the save loads");
        assert_eq!(loaded.header, written.header);
        assert_eq!(loaded.state, *state(60));
        assert_eq!(loaded.header.playthrough, playthrough);
        assert_eq!(loaded.header.kind, SaveKind::Manual);
        assert_eq!(loaded.header.label, "First steps");
        assert_eq!(loaded.header.speed, 5);
    }

    #[test]
    fn store_never_saved_to_is_empty() {
        let (_directory, store) = store();

        let list = store.list();

        assert!(list.saves.is_empty());
        assert!(list.unreadable.is_empty());
    }

    #[test]
    fn saves_are_listed_newest_first_across_playthroughs() {
        let (_directory, store) = store();
        let first = SimulationId::generate();
        let second = SimulationId::generate();

        for (playthrough, label, saved_at_ms) in [
            (&first, "a", 2_000),
            (&second, "b", 3_000),
            (&first, "c", 1_000),
        ] {
            store
                .write_at(&request(playthrough, SaveKind::Manual, label), saved_at_ms)
                .expect("the save is written");
        }

        let labels: Vec<String> = store
            .list()
            .saves
            .into_iter()
            .map(|save| save.header.label)
            .collect();
        assert_eq!(labels, ["b", "a", "c"]);
    }

    #[test]
    fn saving_under_the_same_name_replaces_the_save() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();

        let mut later = request(&playthrough, SaveKind::Manual, "Checkpoint");
        store.write(&later).expect("the save is written");
        later.state = state(120);
        store.write(&later).expect("the save is replaced");

        let list = store.list();
        assert_eq!(list.saves.len(), 1);
        assert_eq!(list.saves[0].header.game_time, Timestamp::from_seconds(120));
    }

    #[test]
    fn names_that_look_alike_as_files_stay_apart() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();

        for label in ["Äiti", "Öiti", "äiti", "???", "!!!"] {
            store
                .write(&request(&playthrough, SaveKind::Manual, label))
                .expect("the save is written");
        }

        assert_eq!(store.list().saves.len(), 5);
    }

    #[test]
    fn slugs_are_safe_file_names() {
        assert_eq!(slug("Before the storm!"), "before-the-storm");
        assert_eq!(slug("  01.01.0001  "), "01-01-0001");
        assert_eq!(slug("../../etc/passwd"), "etc-passwd");
        assert_eq!(slug("CON"), "con");
        assert_eq!(slug("日本"), "save");
        assert_eq!(slug(""), "save");
        assert!(slug(&"long name ".repeat(20)).len() <= MAX_SLUG_LEN);
    }

    #[test]
    fn only_the_newest_autosaves_are_kept() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();
        store
            .write(&request(&playthrough, SaveKind::Manual, "Kept"))
            .expect("the save is written");

        for saved_at_ms in 1..=5 {
            store
                .write_at(&request(&playthrough, SaveKind::Auto, ""), saved_at_ms)
                .expect("the autosave is written");
        }

        let list = store.list();
        let autosaves: Vec<u64> = list
            .saves
            .iter()
            .filter(|save| save.header.kind == SaveKind::Auto)
            .map(|save| save.header.saved_at_ms)
            .collect();
        assert_eq!(autosaves.len(), AUTOSAVES_KEPT);
        assert_eq!(autosaves, [5, 4, 3]);
        assert_eq!(list.saves.len(), AUTOSAVES_KEPT + 1);
    }

    #[test]
    fn each_playthrough_has_one_exit_save() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();

        for saved_at_ms in [10, 20] {
            store
                .write_at(
                    &request(&playthrough, SaveKind::Exit, "ignored"),
                    saved_at_ms,
                )
                .expect("the exit save is written");
        }

        let list = store.list();
        assert_eq!(list.saves.len(), 1);
        assert_eq!(list.saves[0].header.saved_at_ms, 20);
        assert_eq!(list.saves[0].header.label, "");
    }

    #[test]
    fn nothing_is_left_behind_by_writing() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();

        store
            .write(&request(&playthrough, SaveKind::Exit, ""))
            .expect("the save is written");

        assert_eq!(
            file_names(&store.root().join(playthrough.to_string())),
            ["exit.lsave"]
        );
    }

    #[test]
    fn damaged_saves_are_listed_as_unreadable() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();
        let written = store
            .write(&request(&playthrough, SaveKind::Manual, "Good"))
            .expect("the save is written");
        let directory = written.path.parent().expect("the save is in a directory");

        fs::write(directory.join("garbage.lsave"), b"not a save").expect("the file is written");
        fs::write(directory.join("notes.txt"), b"ignored").expect("the file is written");
        let full = fs::read(&written.path).expect("the save reads");
        fs::write(directory.join("cut.lsave"), &full[..12]).expect("the file is written");

        let list = store.list();
        assert_eq!(list.saves, [written]);
        assert_eq!(list.unreadable.len(), 2);
        assert!(matches!(list.unreadable[0].error, SaveError::Corrupt(_)));
        assert!(matches!(list.unreadable[1].error, SaveError::NotASave));
        assert!(matches!(
            store.load(&list.unreadable[1].path),
            Err(SaveError::NotASave)
        ));
    }

    #[test]
    fn save_with_altered_contents_does_not_load() {
        let (_directory, store) = store();
        let written = store
            .write(&request(&SimulationId::generate(), SaveKind::Manual, "x"))
            .expect("the save is written");

        let mut file = fs::read(&written.path).expect("the save reads");
        let last = file.len() - 1;
        file[last] ^= 0xFF;
        fs::write(&written.path, file).expect("the save is overwritten");

        // The header is intact, so it still lists; only loading finds the damage.
        assert_eq!(store.list().saves.len(), 1);
        assert!(matches!(
            store.load(&written.path),
            Err(SaveError::Corrupt(_))
        ));
    }

    #[test]
    fn save_from_a_newer_game_is_listed_but_does_not_load() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();
        let mut entry = store
            .write(&request(&playthrough, SaveKind::Manual, "x"))
            .expect("the save is written");

        entry.header.state_version = STATE_VERSION + 1;
        assert!(matches!(
            entry.problem(),
            Some(SaveError::NewerVersion { .. })
        ));

        let payload = b"whatever a newer game writes";
        assert!(matches!(
            migrate::decode_state(&entry.header, payload),
            Err(SaveError::NewerVersion { .. })
        ));
    }

    #[test]
    fn deleting_removes_the_save_and_an_emptied_playthrough() {
        let (_directory, store) = store();
        let playthrough = SimulationId::generate();
        let first = store
            .write(&request(&playthrough, SaveKind::Manual, "one"))
            .expect("the save is written");
        let second = store
            .write(&request(&playthrough, SaveKind::Manual, "two"))
            .expect("the save is written");
        let directory = store.root().join(playthrough.to_string());

        store.delete(&first.path).expect("the save is deleted");
        assert_eq!(store.list().saves, std::slice::from_ref(&second));
        assert!(directory.exists());

        store.delete(&second.path).expect("the save is deleted");
        assert!(store.list().saves.is_empty());
        assert!(!directory.exists());
    }

    #[test]
    fn only_saves_in_the_store_can_be_deleted() {
        let (directory, store) = store();
        let outside = directory.path().join("outside.lsave");
        fs::write(&outside, b"keep me").expect("the file is written");
        let written = store
            .write(&request(&SimulationId::generate(), SaveKind::Manual, "x"))
            .expect("the save is written");
        let not_a_save = written.path.with_extension("txt");
        fs::write(&not_a_save, b"keep me").expect("the file is written");

        assert!(matches!(store.delete(&outside), Err(SaveError::NotInStore)));
        assert!(matches!(
            store.delete(&not_a_save),
            Err(SaveError::NotInStore)
        ));
        assert!(matches!(
            store.delete(store.root()),
            Err(SaveError::NotInStore)
        ));
        assert!(outside.exists());
        assert!(not_a_save.exists());
    }

    #[test]
    fn export_is_readable_text_and_not_a_save() {
        let (_directory, store) = store();
        let written = store
            .write(&request(&SimulationId::generate(), SaveKind::Manual, "x"))
            .expect("the save is written");

        let exported = store.export_ron(&written.path).expect("the save exports");

        let text = fs::read_to_string(&exported).expect("the export is text");
        assert!(text.contains("Ada"), "{text}");
        assert!(text.contains("state_version"), "{text}");
        assert_eq!(store.list().saves, [written]);
        assert!(store.list().unreadable.is_empty());
    }

    #[test]
    fn readable_saves_can_be_written_and_load_like_any_other() {
        let (_directory, store) = store();
        let store = store.with_format(Codec::Ron, Compression::None);

        let written = store
            .write(&request(&SimulationId::generate(), SaveKind::Manual, "x"))
            .expect("the save is written");

        assert_eq!(written.header.codec, Codec::Ron);
        let loaded = SaveStore::new(store.root())
            .load(&written.path)
            .expect("the save loads");
        assert_eq!(loaded.state, *state(60));
    }

    #[test]
    fn interrupted_writes_are_cleaned_up() {
        let (_directory, store) = store();
        let written = store
            .write(&request(&SimulationId::generate(), SaveKind::Manual, "x"))
            .expect("the save is written");
        let directory = written.path.parent().expect("the save is in a directory");
        fs::write(directory.join("exit.lsave.tmp"), b"half a save").expect("the file is written");

        assert_eq!(store.list().saves.len(), 1);
        assert!(store.list().unreadable.is_empty());

        store.remove_stale_temp_files();

        assert_eq!(
            file_names(directory),
            [written
                .path
                .file_name()
                .and_then(OsStr::to_str)
                .expect("the save has a name")]
        );
    }

    fn fixture_state() -> State {
        let mut state = State::new(
            Name::new(
                "Ada".to_owned(),
                vec!["Maria".to_owned(), "Mäki".to_owned()],
                "Lord".to_owned(),
            ),
            Sex::Female,
        );
        state.time = Timestamp::from_seconds(63_072_000_000);
        state
    }

    fn fixture_path(state_version: u32) -> PathBuf {
        Path::new(FIXTURES).join(format!("v{state_version}.{EXTENSION}"))
    }

    /// Guards against changing [`State`] in a way that breaks existing saves without
    /// raising [`STATE_VERSION`].
    #[test]
    fn save_written_by_this_version_still_loads() {
        let path = fixture_path(STATE_VERSION);

        let loaded = SaveStore::new(FIXTURES)
            .load(&path)
            .unwrap_or_else(|error| {
                panic!(
                    "{} does not load: {error}. If `State` changed on purpose, raise \
                 `STATE_VERSION` and run `cargo test -p lords_save -- --ignored write_fixture`",
                    path.display()
                )
            });

        assert_eq!(loaded.state, fixture_state());
        assert_eq!(loaded.header.state_version, STATE_VERSION);
        assert_eq!(loaded.header.character, "Ada Maria Mäki Lord");
        assert_eq!(loaded.header.label, "Fixture");
    }

    /// Every fixture ever committed, from whichever version, either loads or is turned
    /// down for being too old. None may be mistaken for damaged, or for something newer.
    #[test]
    fn saves_written_by_older_versions_load_or_are_refused() {
        let fixtures: Vec<PathBuf> = entries(Path::new(FIXTURES))
            .into_iter()
            .filter(|path| path.extension() == Some(OsStr::new(EXTENSION)))
            .collect();
        assert!(fixtures.contains(&fixture_path(STATE_VERSION)));

        for path in fixtures {
            match SaveStore::new(FIXTURES).load(&path) {
                Ok(_) => {}
                Err(SaveError::IncompatibleVersion { found }) if found < STATE_VERSION => {}
                Err(error) => panic!("{}: {error}", path.display()),
            }
        }
    }

    #[test]
    #[ignore = "writes the fixture for the current STATE_VERSION; run after raising it"]
    fn write_fixture() {
        let details = Details {
            playthrough: SimulationId::generate(),
            kind: SaveKind::Manual,
            label: "Fixture",
            saved_at_ms: 1_800_000_000_000,
            speed: 1,
        };
        let (_, file) = format::encode(
            details,
            &fixture_state(),
            Codec::MessagePack,
            Compression::Lz4,
        )
        .expect("the fixture encodes");

        fs::create_dir_all(FIXTURES).expect("the fixture directory can be made");
        fs::write(fixture_path(STATE_VERSION), file).expect("the fixture is written");
    }
}
