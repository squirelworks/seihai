//! Explicit receive-table selection, separate from the baseline wire profile.
//!
//! These static build observations do not select login, send, GS1 execution
//! or complete gameplay semantics. Shared tokens never select a build here.

use crate::protocol::graal2002::{Graal2002Build, Graal210ReceiveBuild};
use crate::protocol::{ClientVersion, SessionState};

/// Resolve only the core descriptor's exact labels; no token or version aliases.
pub fn parse(label: &str) -> Result<Graal210ReceiveBuild, String> {
    std::iter::once(Graal210ReceiveBuild::Public20011113)
        .chain(
            Graal2002Build::ALL
                .into_iter()
                .map(Graal210ReceiveBuild::World),
        )
        .find(|build| build.label() == label)
        .ok_or_else(|| format!("unknown receive build {label}"))
}

/// Construct every new connection's state with the same explicit selection.
pub fn session(
    version: ClientVersion,
    build: Option<Graal210ReceiveBuild>,
) -> Result<SessionState, &'static str> {
    if build.is_some() && version != ClientVersion::V210 {
        return Err("--receive-build requires an explicitly selected --version 2.1 baseline");
    }
    let mut state = SessionState::for_version(version);
    if let Some(build) = build {
        state.set_graal210_receive_build(build);
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receive_build_requires_explicit_identity_and_compatible_baseline() {
        for alias in ["GNW31101", "GNW03081", "2.1", "2.1.0.4", "unknown"] {
            assert!(parse(alias).is_err(), "{alias} must not select a build");
        }
        let build = parse("v22").unwrap();
        assert!(session(ClientVersion::V205, Some(build)).is_err());
        assert!(session(ClientVersion::V603R7, Some(build)).is_err());
    }

    #[test]
    fn selection_reaches_each_new_sessions_receive_gate() {
        use crate::protocol::Message;

        // These builds share the generic baseline's login token, but an
        // explicit label must reach different receive behavior.
        for (label, restored, mode) in [
            ("ver10", false, false),
            ("public-2001-11-13", true, false),
            ("v22", true, true),
        ] {
            let build = parse(label).unwrap();
            // Initial login and subsequent connections use the same constructor.
            for _ in 0..2 {
                let mut state = session(ClientVersion::V210, Some(build)).unwrap();
                assert!(state.chests.is_empty() && state.server_mode.is_none());
                state.apply(&Message {
                    id: 4,
                    payload: b"abc".to_vec(),
                });
                state.apply(&Message {
                    id: 176,
                    payload: Vec::new(),
                });
                assert_eq!(!state.chests.is_empty(), restored, "{label}: chest receive");
                assert_eq!(state.server_mode.is_some(), mode, "{label}: mode receive");
            }
        }
        let generic = session(ClientVersion::V210, None).unwrap();
        let baseline = SessionState::for_version(ClientVersion::V210);
        for id in 0..=255 {
            assert_eq!(generic.handles(id), baseline.handles(id));
        }
    }

    #[test]
    fn public_layout_is_opted_into_by_the_consumer_selector_only() {
        use crate::protocol::Message;

        let packet = Message {
            id: 171,
            payload: b"fixture.txt,fixture.png".to_vec(),
        };
        for label in ["public-2001-11-13", "ver10", "v22"] {
            let build = parse(label).unwrap();
            let mut state = session(ClientVersion::V210, Some(build)).unwrap();
            state.apply(&packet);
            assert_eq!(
                state.bigmap.as_ref().map(|map| (map.x, map.y)),
                (label == "public-2001-11-13").then_some((0.0, 0.0)),
                "{label}: binding alone must not select the public reader"
            );
        }
        let mut generic = session(ClientVersion::V210, None).unwrap();
        generic.apply(&packet);
        assert!(generic.bigmap.is_none());
    }
}
