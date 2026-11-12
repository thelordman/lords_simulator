use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

/// Why a save could not be written or read.
#[derive(Debug)]
pub enum SaveError {
    /// The file could not be read or written.
    Io(io::Error),
    /// The file does not start the way a save does.
    NotASave,
    /// The file is laid out in a way this version of the game does not know.
    UnsupportedContainer(u16),
    /// The save was written by a newer version of the game.
    NewerVersion { found: u32, supported: u32 },
    /// The save was written by an older version of the game, and nothing converts it.
    IncompatibleVersion { found: u32 },
    /// The file is damaged, cut short, altered, or at odds with its own header.
    Corrupt(&'static str),
    /// The contents could not be turned back into a game.
    Decode(String),
    /// The game could not be turned into bytes.
    Encode(String),
    /// The path does not name a save inside the store.
    NotInStore,
}

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "the file could not be accessed: {error}"),
            Self::NotASave => f.write_str("this is not a save file"),
            Self::UnsupportedContainer(version) => {
                write!(f, "unknown save file layout (version {version})")
            }
            Self::NewerVersion { found, supported } => write!(
                f,
                "saved by a newer version of the game (save version {found}, this game reads \
                 up to {supported})"
            ),
            Self::IncompatibleVersion { found } => write!(
                f,
                "saved by an older version of the game that is no longer supported (save \
                 version {found})"
            ),
            Self::Corrupt(what) => write!(f, "the file is corrupted: {what}"),
            Self::Decode(error) => write!(f, "the save could not be read: {error}"),
            Self::Encode(error) => write!(f, "the game could not be saved: {error}"),
            Self::NotInStore => f.write_str("this file is not one of the game's saves"),
        }
    }
}

impl Error for SaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for SaveError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
