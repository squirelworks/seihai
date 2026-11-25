//! Consumer mode: the start window picks the version for each server.
//!
//! A game server says which versions it takes only when it turns a login
//! away (`Your client version is not allowed on this server.` and an
//! `Allowed:` list). So a join tries a version: the one remembered for that
//! server, or core's [`version_pick::first_try`]. If the client comes back
//! to the start window with a refusal, the window reads the list, asks
//! [`version_pick::pick`] and joins again with the new version, or asks the
//! player when the server takes two eras with a gap between them.
//!
//! The state lives in `launcher.conf`, because Mae comes back to the start
//! window by starting the program again: `picked.<host:port>` (the version
//! to join as), `allowed.<host:port>` (the server's last list), and the join
//! in progress (`pending`). The design is seihai-core
//! `documentation/auto-version.md`.

use seihai_core::protocol::version_pick::{self, AllowedList, Era, Need, Pick};
use seihai_core::protocol::ClientVersion;

/// How many versions one join tries before it stops and says why.
pub const MAX_TRIES: usize = 3;

/// The start window's mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// The window picks the version for each server.
    #[default]
    Consumer,
    /// The version box: the player picks, and a refusal is only shown.
    Developer,
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Mode::Consumer => "consumer",
            Mode::Developer => "dev",
        }
    }

    pub fn from_key(k: &str) -> Option<Mode> {
        match k.trim() {
            "consumer" => Some(Mode::Consumer),
            "dev" | "developer" => Some(Mode::Developer),
            _ => None,
        }
    }
}

/// A join that has not come back yet, or came back refused.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pending {
    /// `host:port`.
    pub server: String,
    /// The server is listed as a 3D one.
    pub three_d: bool,
    /// The version this join tried last.
    pub tried: String,
    /// Versions the server already refused in this join.
    pub refused: Vec<String>,
}

impl Pending {
    fn need(&self) -> Need {
        Need {
            three_d: self.three_d,
            refused: self
                .refused
                .iter()
                .filter_map(|l| ClientVersion::from_label(l))
                .collect(),
        }
    }
}

/// The version to try first on a server: the remembered one, or core's
/// first try for a server Seihai has not joined.
pub fn first_version(remembered: Option<&str>, three_d: bool) -> ClientVersion {
    remembered
        .and_then(ClientVersion::from_label)
        .unwrap_or_else(|| {
            version_pick::first_try(&Need {
                three_d,
                ..Need::default()
            })
        })
}

/// The text after `Allowed:` in a version refusal, as the server wrote it.
pub fn allowed_text(message: &str) -> Option<String> {
    AllowedList::from_disconnect(message)?;
    let at = message.to_ascii_lowercase().find("allowed:")?;
    let rest = &message[at + "allowed:".len()..];
    Some(
        rest.split(['\r', '\n'])
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
    )
}

/// What the start window does when a join comes back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    /// Not a version refusal: the join is over; show the message as before.
    Done,
    /// Join again as this version, at once.
    Join(ClientVersion),
    /// Ask the player: one option per era, the recommended one first.
    Ask(Vec<ClientVersion>),
    /// Stop and say why.
    Stop(String),
}

/// A join came back with `message`. Updates `pending` (the refused
/// version) and says what to do next.
pub fn after_refusal(pending: &mut Pending, message: &str) -> Next {
    let Some(list) = AllowedList::from_disconnect(message) else {
        return Next::Done;
    };
    if !pending.tried.is_empty() && !pending.refused.contains(&pending.tried) {
        pending.refused.push(pending.tried.clone());
    }
    let shown = allowed_text(message).unwrap_or_default();
    if pending.refused.len() >= MAX_TRIES {
        return Next::Stop(format!(
            "The server refused {}. It takes: {shown}. Turn on developer mode to pick a version yourself.",
            pending.refused.join(", ")
        ));
    }
    match version_pick::pick(&list, &pending.need()) {
        Pick::Auto(v) => Next::Join(v),
        Pick::Ask { options, .. } => Next::Ask(options),
        Pick::Nothing => Next::Stop(format!(
            "This server takes only {shown}, and Seihai has no version that it accepts."
        )),
    }
}

/// A button's text on the question card.
pub fn option_text(v: ClientVersion) -> String {
    format!("{}: {}", Era::of(v).name(), v.label())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClientVersion::*;

    const REFUSAL: &str =
        "disconnected: Your client version is not allowed on this server.\nAllowed: ";

    fn pending(tried: ClientVersion) -> Pending {
        Pending {
            server: "h:14900".into(),
            three_d: false,
            tried: tried.label().into(),
            refused: vec![],
        }
    }

    #[test]
    fn a_new_server_starts_at_the_newest_2d_build_and_a_remembered_one_wins() {
        assert_eq!(first_version(None, false), V603R7);
        assert!(first_version(None, true).profile().three_d.is_some());
        assert_eq!(first_version(Some("2.22"), false), V222);
        assert_eq!(first_version(Some("no such"), false), V603R7);
    }

    #[test]
    fn a_refusal_with_a_list_joins_again_as_the_pick() {
        let mut p = pending(V603R7);
        let next = after_refusal(&mut p, &format!("{REFUSAL}2.22, 5.07"));
        assert_eq!(next, Next::Join(V222));
        assert_eq!(p.refused, ["6.03r7"]);
        // 2.22 refused too (the server's table differs): 5.00r7.
        p.tried = "2.22".into();
        let next = after_refusal(&mut p, &format!("{REFUSAL}2.22, 5.07"));
        assert_eq!(next, Next::Join(V500R7));
    }

    #[test]
    fn two_eras_with_a_gap_ask() {
        let mut p = pending(V222);
        let next = after_refusal(&mut p, &format!("{REFUSAL}1.41r1, 5.12 - 6.037"));
        assert_eq!(next, Next::Ask(vec![V603R7, V141]));
        assert_eq!(option_text(V141), "Classic (1.x): 1.41");
    }

    #[test]
    fn other_messages_end_the_join() {
        let mut p = pending(V222);
        assert_eq!(
            after_refusal(&mut p, "The server closed the connection."),
            Next::Done
        );
        assert!(p.refused.is_empty());
    }

    #[test]
    fn it_stops_after_three_versions_or_when_nothing_fits() {
        let mut p = pending(V603R7);
        p.refused = vec!["2.22".into(), "5.00r7".into()];
        match after_refusal(&mut p, &format!("{REFUSAL}2.22, 5.07, 6.037")) {
            Next::Stop(why) => assert!(why.contains("6.03r7"), "{why}"),
            n => panic!("{n:?}"),
        }
        let mut p = pending(V222);
        match after_refusal(&mut p, &format!("{REFUSAL}iPhone 1.5")) {
            Next::Stop(why) => assert!(why.contains("iPhone 1.5"), "{why}"),
            n => panic!("{n:?}"),
        }
    }

    #[test]
    fn the_list_is_read_whatever_the_line_break() {
        assert_eq!(
            allowed_text("Your client version is not allowed on this server.\rAllowed: 2.22, 5.07")
                .as_deref(),
            Some("2.22, 5.07")
        );
        assert_eq!(allowed_text("Account is already in use."), None);
    }
}
