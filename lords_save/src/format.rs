//! The layout of a save file.
//!
//! ```text
//! "LSIMSAVE"        8 bytes
//! container version u16, little-endian
//! header length     u32, little-endian
//! header            MessagePack map, see `SaveHeader`
//! payload           everything after the header
//! ```

use std::collections::hash_map::RandomState;
use std::fmt::{self, Display, Formatter};
use std::hash::{BuildHasher, Hasher};
use std::io::{self, Read};

use lords_sim::object::Living;
use lords_sim::{STATE_VERSION, State, Timestamp};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::SaveError;

const MAGIC: [u8; 8] = *b"LSIMSAVE";

/// Version of the framing described in the module documentation.
const CONTAINER_VERSION: u16 = 1;

const MAX_HEADER_LEN: usize = 64 * 1024;

const MAX_PAYLOAD_LEN: u64 = 1 << 30;

const MAX_LZ4_RATIO: u64 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Codec {
    MessagePack,
    Ron,
}

impl Codec {
    fn encode<T: Serialize>(self, value: &T) -> Result<Vec<u8>, SaveError> {
        match self {
            Self::MessagePack => {
                rmp_serde::to_vec_named(value).map_err(|error| SaveError::Encode(error.to_string()))
            }
            Self::Ron => ron::ser::to_string(value)
                .map(String::into_bytes)
                .map_err(|error| SaveError::Encode(error.to_string())),
        }
    }

    pub(crate) fn decode<T: DeserializeOwned>(self, bytes: &[u8]) -> Result<T, SaveError> {
        match self {
            Self::MessagePack => {
                rmp_serde::from_slice(bytes).map_err(|error| SaveError::Decode(error.to_string()))
            }
            Self::Ron => {
                ron::de::from_bytes(bytes).map_err(|error| SaveError::Decode(error.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compression {
    None,
    Lz4,
}

impl Compression {
    fn compress(self, bytes: Vec<u8>) -> Vec<u8> {
        match self {
            Self::None => bytes,
            Self::Lz4 => lz4_flex::block::compress(&bytes),
        }
    }

    /// Unpacks `stored`, which must come to exactly `len` bytes.
    fn decompress(self, stored: &[u8], len: u64) -> Result<Vec<u8>, SaveError> {
        const WRONG_LENGTH: SaveError =
            SaveError::Corrupt("its contents are not as long as it says");

        if len > MAX_PAYLOAD_LEN {
            return Err(SaveError::Corrupt("it claims to be impossibly large"));
        }

        let unpacked = match self {
            Self::None => stored.to_vec(),
            Self::Lz4 => {
                if len > (stored.len() as u64).saturating_mul(MAX_LZ4_RATIO) {
                    return Err(WRONG_LENGTH);
                }
                lz4_flex::block::decompress(stored, len as usize)
                    .map_err(|_| SaveError::Corrupt("its contents cannot be unpacked"))?
            }
        };

        if unpacked.len() as u64 == len {
            Ok(unpacked)
        } else {
            Err(WRONG_LENGTH)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SaveKind {
    Manual,
    Auto,
    Exit,
}

/// Identifies one life, from character creation onwards, across all of its saves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimulationId(u64);

impl SimulationId {
    pub fn generate() -> Self {
        Self(RandomState::new().build_hasher().finish())
    }
}

impl Display for SimulationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveHeader {
    pub state_version: u32,
    pub game_version: String,
    pub codec: Codec,
    pub compression: Compression,
    pub payload_len: u64,
    pub checksum: u32,
    pub playthrough: SimulationId,
    pub kind: SaveKind,
    pub label: String,
    pub character: String,
    pub game_time: Timestamp,
    /// Real time at which the save was taken, in milliseconds since the Unix epoch.
    pub saved_at_ms: u64,
    pub speed: u32,
}

pub(crate) struct Details<'a> {
    pub(crate) playthrough: SimulationId,
    pub(crate) kind: SaveKind,
    pub(crate) label: &'a str,
    pub(crate) saved_at_ms: u64,
    pub(crate) speed: u32,
}

pub(crate) fn encode(
    details: Details<'_>,
    state: &State,
    codec: Codec,
    compression: Compression,
) -> Result<(SaveHeader, Vec<u8>), SaveError> {
    let encoded = codec.encode(state)?;
    let payload_len = encoded.len() as u64;
    let payload = compression.compress(encoded);

    let header = SaveHeader {
        state_version: STATE_VERSION,
        game_version: env!("CARGO_PKG_VERSION").to_owned(),
        codec,
        compression,
        payload_len,
        checksum: crc32fast::hash(&payload),
        playthrough: details.playthrough,
        kind: details.kind,
        label: details.label.to_owned(),
        character: state.player.name().to_string(),
        game_time: state.time,
        saved_at_ms: details.saved_at_ms,
        speed: details.speed,
    };

    let header_bytes = Codec::MessagePack.encode(&header)?;
    let header_len = u32::try_from(header_bytes.len())
        .ok()
        .filter(|len| *len as usize <= MAX_HEADER_LEN)
        .ok_or_else(|| SaveError::Encode("the save's description is too long".to_owned()))?;

    let mut file = Vec::with_capacity(MAGIC.len() + 6 + header_bytes.len() + payload.len());
    file.extend_from_slice(&MAGIC);
    file.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    file.extend_from_slice(&header_len.to_le_bytes());
    file.extend_from_slice(&header_bytes);
    file.extend_from_slice(&payload);

    Ok((header, file))
}

pub(crate) fn read_header(reader: &mut impl Read) -> Result<SaveHeader, SaveError> {
    let mut magic = [0; MAGIC.len()];
    match reader.read_exact(&mut magic) {
        Ok(()) if magic == MAGIC => {}
        Ok(()) => return Err(SaveError::NotASave),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            return Err(SaveError::NotASave);
        }
        Err(error) => return Err(error.into()),
    }

    let mut version = [0; 2];
    read_exact(reader, &mut version)?;
    let version = u16::from_le_bytes(version);
    if version != CONTAINER_VERSION {
        return Err(SaveError::UnsupportedContainer(version));
    }

    let mut header_len = [0; 4];
    read_exact(reader, &mut header_len)?;
    let header_len = u32::from_le_bytes(header_len) as usize;
    if header_len > MAX_HEADER_LEN {
        return Err(SaveError::Corrupt("its description is impossibly long"));
    }

    let mut header = vec![0; header_len];
    read_exact(reader, &mut header)?;
    Codec::MessagePack.decode(&header)
}

pub(crate) fn decode(file: &[u8]) -> Result<(SaveHeader, Vec<u8>), SaveError> {
    let mut payload = file;
    let header = read_header(&mut payload)?;

    if crc32fast::hash(payload) != header.checksum {
        return Err(SaveError::Corrupt("its contents have been altered"));
    }

    let unpacked = header.compression.decompress(payload, header.payload_len)?;
    Ok((header, unpacked))
}

fn read_exact(reader: &mut impl Read, buffer: &mut [u8]) -> Result<(), SaveError> {
    reader.read_exact(buffer).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            SaveError::Corrupt("it is cut short")
        } else {
            error.into()
        }
    })
}

#[cfg(test)]
mod tests {
    use lords_sim::{Name, Sex};

    use super::*;

    fn state() -> State {
        let mut state = State::new(
            Name::new("Ada".to_owned(), vec!["Mäki".to_owned()], "Lord".to_owned()),
            Sex::Female,
        );
        state.time = Timestamp::from_seconds(123_456_789);
        state
    }

    fn details() -> Details<'static> {
        Details {
            playthrough: SimulationId(7),
            kind: SaveKind::Manual,
            label: "Before the storm",
            saved_at_ms: 1_700_000_000_000,
            speed: 10,
        }
    }

    fn file(codec: Codec, compression: Compression) -> Vec<u8> {
        encode(details(), &state(), codec, compression)
            .expect("the state encodes")
            .1
    }

    fn decode_state(file: &[u8]) -> Result<State, SaveError> {
        let (header, payload) = decode(file)?;
        header.codec.decode(&payload)
    }

    #[test]
    fn state_survives_every_codec_and_compression() {
        for codec in [Codec::MessagePack, Codec::Ron] {
            for compression in [Compression::None, Compression::Lz4] {
                let file = file(codec, compression);

                assert_eq!(
                    decode_state(&file).expect("the file decodes"),
                    state(),
                    "{codec:?} with {compression:?}"
                );
            }
        }
    }

    #[test]
    fn header_describes_the_save() {
        let (written, file) = encode(details(), &state(), Codec::MessagePack, Compression::Lz4)
            .expect("the state encodes");

        let read = read_header(&mut file.as_slice()).expect("the header reads");

        assert_eq!(read, written);
        assert_eq!(read.state_version, STATE_VERSION);
        assert_eq!(read.character, "Ada Mäki Lord");
        assert_eq!(read.game_time, Timestamp::from_seconds(123_456_789));
        assert_eq!(read.label, "Before the storm");
        assert_eq!(read.speed, 10);
    }

    #[test]
    fn header_reads_without_the_payload() {
        let (header, file) = encode(details(), &state(), Codec::MessagePack, Compression::Lz4)
            .expect("the state encodes");
        let payload_start = file.len() - {
            let mut rest = file.as_slice();
            read_header(&mut rest).expect("the header reads");
            rest.len()
        };

        let read = read_header(&mut &file[..payload_start]).expect("the header reads alone");

        assert_eq!(read, header);
    }

    #[test]
    fn other_files_are_not_saves() {
        assert!(matches!(decode(b""), Err(SaveError::NotASave)));
        assert!(matches!(decode(b"LORD"), Err(SaveError::NotASave)));
        assert!(matches!(
            decode(b"definitely not a save file"),
            Err(SaveError::NotASave)
        ));
    }

    #[test]
    fn unknown_container_version_is_refused() {
        let mut file = file(Codec::MessagePack, Compression::Lz4);
        file[MAGIC.len()] = 2;

        assert!(matches!(
            decode(&file),
            Err(SaveError::UnsupportedContainer(2))
        ));
    }

    #[test]
    fn file_cut_short_is_corrupt() {
        let file = file(Codec::MessagePack, Compression::None);

        for len in [MAGIC.len() + 1, MAGIC.len() + 4, MAGIC.len() + 20] {
            assert!(
                matches!(decode(&file[..len]), Err(SaveError::Corrupt(_))),
                "cut to {len} bytes"
            );
        }
        assert!(matches!(
            decode(&file[..file.len() - 1]),
            Err(SaveError::Corrupt(_))
        ));
    }

    #[test]
    fn altered_payload_is_corrupt() {
        for compression in [Compression::None, Compression::Lz4] {
            let mut file = file(Codec::MessagePack, compression);
            let last = file.len() - 1;
            file[last] ^= 0x01;

            assert!(
                matches!(decode(&file), Err(SaveError::Corrupt(_))),
                "{compression:?}"
            );
        }
    }

    #[test]
    fn oversized_header_length_is_corrupt() {
        let mut file = file(Codec::MessagePack, Compression::None);
        let len_at = MAGIC.len() + 2;
        file[len_at..len_at + 4].copy_from_slice(&u32::MAX.to_le_bytes());

        assert!(matches!(decode(&file), Err(SaveError::Corrupt(_))));
    }

    #[test]
    fn wrong_unpacked_length_is_corrupt() {
        let stored = Compression::Lz4.compress(vec![7; 1_000]);

        assert!(Compression::Lz4.decompress(&stored, 1_000).is_ok());
        assert!(matches!(
            Compression::Lz4.decompress(&stored, 999),
            Err(SaveError::Corrupt(_))
        ));
        assert!(matches!(
            Compression::Lz4.decompress(&stored, MAX_PAYLOAD_LEN + 1),
            Err(SaveError::Corrupt(_))
        ));
        assert!(matches!(
            Compression::None.decompress(&[1, 2, 3], 4),
            Err(SaveError::Corrupt(_))
        ));
    }

    #[test]
    fn playthrough_ids_differ_and_name_directories() {
        assert_ne!(SimulationId::generate(), SimulationId::generate());
        assert_eq!(SimulationId(0xAB).to_string(), "00000000000000ab");
    }
}
