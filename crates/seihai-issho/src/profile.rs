//! Which version Issho will run.
//!
//! v6 selects this crate. v3–v5 do not. A v7 label is rejected.

use seihai_core::protocol::ClientVersion;
use std::fmt;

/// The engine a profile names.
///
/// Only [`EngineProfile::V6`] is ever returned. [`EngineProfile::V7`] is
/// reserved so a later Unity engine has a place to land; nothing constructs
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineProfile {
    /// v6. The quattro_lib client (`quattro_lib`), not Torque.
    V6,
    /// Reserved. Unity is the expected engine. No v7 profile is supported.
    ///
    /// [`select_version`] and [`select_label`] never return this variant.
    /// [`Unbuilt`] has no constructor, and this crate never builds the variant.
    V7(Unbuilt),
}

/// Blocks [`EngineProfile::V7`]. No public fields, so other crates cannot
/// build it, and this crate has no constructor either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // v7 is not constructed
pub struct Unbuilt {
    _private: (),
}

/// Why a label or version is not Issho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectError {
    /// v3, v4 or v5. Those stay on the Torque line. Issho does not call it.
    TorqueEra(ClientVersion),
    /// A real profile whose major version is not 6 (1.x and 2.x).
    OtherClient(ClientVersion),
    /// Not a profile row. Packed 6.037 and 6.113 land here: they are not
    /// the engine, and they are not a v6 row.
    Unknown,
    /// A 7.x label. Unity is the expected engine. Not selectable.
    UnsupportedV7,
}

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TorqueEra(v) => write!(f, "{} is v3–v5; Issho does not select it", v.label()),
            Self::OtherClient(v) => write!(f, "{} is not a v6 profile", v.label()),
            Self::Unknown => write!(f, "unknown version"),
            Self::UnsupportedV7 => write!(
                f,
                "v7 is not a supported profile; Unity is the expected engine"
            ),
        }
    }
}

/// Leading major version of a profile label (`6` from `6.03r7 Linux`).
pub fn major(version: ClientVersion) -> Option<u8> {
    let digits: String = version
        .label()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// True when `version` is a 6.x profile row.
pub fn is_v6(version: ClientVersion) -> bool {
    major(version) == Some(6)
}

/// True for a label that means v7 (`7`, `7.0`, `v7`, `V7.1`).
///
/// This is not a version matrix. It only stops those labels from falling
/// through as an unknown client.
pub fn is_v7_label(label: &str) -> bool {
    let lower = label.trim().to_ascii_lowercase();
    let rest = lower.strip_prefix('v').unwrap_or(&lower);
    rest == "7" || rest.starts_with("7.")
}

/// The profile Issho would run for this version.
///
/// `Ok` is only [`EngineProfile::V6`]. v3–v5 are [`SelectError::TorqueEra`].
pub fn select_version(version: ClientVersion) -> Result<EngineProfile, SelectError> {
    match major(version) {
        Some(6) => Ok(EngineProfile::V6),
        Some(3) | Some(4) | Some(5) => Err(SelectError::TorqueEra(version)),
        _ => Err(SelectError::OtherClient(version)),
    }
}

/// The profile for a launcher label. v7 is rejected before the profile table,
/// so it cannot become an unknown version that another line would start.
pub fn select_label(label: &str) -> Result<EngineProfile, SelectError> {
    if is_v7_label(label) {
        return Err(SelectError::UnsupportedV7);
    }
    match ClientVersion::from_label(label) {
        Some(version) => select_version(version),
        None => Err(SelectError::Unknown),
    }
}

/// No v7 profile is supported. Always [`None`]: [`EngineProfile::V7`] is
/// not constructed.
pub fn try_v7() -> Option<EngineProfile> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v6_selects_issho_and_v3_to_v5_do_not() {
        for version in ClientVersion::all_versions() {
            let selected = select_version(*version);
            assert!(
                !matches!(selected, Ok(EngineProfile::V7(_))),
                "{}",
                version.label()
            );
            match major(*version) {
                Some(6) => assert_eq!(selected, Ok(EngineProfile::V6), "{}", version.label()),
                Some(3) | Some(4) | Some(5) => {
                    assert!(
                        matches!(selected, Err(SelectError::TorqueEra(_))),
                        "{}",
                        version.label()
                    );
                }
                _ => assert!(
                    matches!(selected, Err(SelectError::OtherClient(_))),
                    "{}",
                    version.label()
                ),
            }
        }
    }

    #[test]
    fn labels_follow_the_same_rule() {
        for label in ["6.00r2", "6.015", "6.034", "6.03r7", "6.03r7 Linux"] {
            assert_eq!(select_label(label), Ok(EngineProfile::V6), "{label}");
        }
        for label in [
            "3.04", "3.153", "4.04r2", "4.20r6", "4.20r8", "5.00r7", "5.12", "5.24r4", "5.30r3",
            "5.31r4",
        ] {
            assert!(
                matches!(select_label(label), Err(SelectError::TorqueEra(_))),
                "{label}"
            );
        }
        assert!(matches!(
            select_label("2.22"),
            Err(SelectError::OtherClient(_))
        ));
        assert!(matches!(
            select_label("1.39"),
            Err(SelectError::OtherClient(_))
        ));
    }

    #[test]
    fn v7_cannot_be_selected() {
        assert!(try_v7().is_none());
        for label in ["7", "7.0", "7.1.3", "v7", "V7", "V7.2", " 7.0 "] {
            assert!(is_v7_label(label), "{label}");
            assert_eq!(
                select_label(label),
                Err(SelectError::UnsupportedV7),
                "{label}"
            );
        }
        assert!(!is_v7_label("6.03r7"));
        assert!(!is_v7_label("70"));
        assert!(!is_v7_label("v70"));
    }

    #[test]
    fn packed_stubs_are_not_the_engine() {
        assert_eq!(select_label("6.037"), Err(SelectError::Unknown));
        assert_eq!(select_label("6.113"), Err(SelectError::Unknown));
    }

    #[test]
    fn the_600r4_row_selects_issho() {
        assert_eq!(select_label("6.00r4"), Ok(EngineProfile::V6));
    }
}
