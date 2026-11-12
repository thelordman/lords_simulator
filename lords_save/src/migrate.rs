//! Reading states written by other versions of the game.

use std::cmp::Ordering;

use lords_sim::{STATE_VERSION, State};

use crate::error::SaveError;
use crate::format::SaveHeader;

/// The older versions that [`upgrade`] converts.
const UPGRADABLE_VERSIONS: &[u32] = &[];

/// Returns whether a state written with `state_version` can be read at all.
pub(crate) fn check(state_version: u32) -> Result<(), SaveError> {
    match state_version.cmp(&STATE_VERSION) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(SaveError::NewerVersion {
            found: state_version,
            supported: STATE_VERSION,
        }),
        Ordering::Less if UPGRADABLE_VERSIONS.contains(&state_version) => Ok(()),
        Ordering::Less => Err(SaveError::IncompatibleVersion {
            found: state_version,
        }),
    }
}

pub(crate) fn decode_state(header: &SaveHeader, payload: &[u8]) -> Result<State, SaveError> {
    check(header.state_version)?;

    if header.state_version == STATE_VERSION {
        header.codec.decode(payload)
    } else {
        upgrade(header, payload)
    }
}

fn upgrade(header: &SaveHeader, _payload: &[u8]) -> Result<State, SaveError> {
    Err(SaveError::IncompatibleVersion {
        found: header.state_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_versions_can_be_read() {
        assert!(check(STATE_VERSION).is_ok());
        assert!(matches!(
            check(STATE_VERSION + 1),
            Err(SaveError::NewerVersion { found, supported })
                if found == STATE_VERSION + 1 && supported == STATE_VERSION
        ));
        assert!(matches!(
            check(0),
            Err(SaveError::IncompatibleVersion { found: 0 })
        ));
    }
}
