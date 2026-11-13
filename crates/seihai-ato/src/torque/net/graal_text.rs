//! Graal text over BigData: the flags, weapons and scripts a Graal3D
//! server sends once the mission has started, and the `events` text the
//! client sends back (protocols 21 and 24).
//!
//! Layout from Graal_analysis `torque/udp-session.md` §7 and
//! `graal3d/torque-engine-4x.md` ("Graal text"). A transfer has three
//! strings: `s1` (a weapon name, or empty), `s2` (the action) and `s3`
//! (its argument). With `s1` set the text goes to that weapon's
//! `onReceiveText`. Otherwise `s2` selects what the client does with it.
//! Weapon scripts are kept here and run by [`super::scripts3d`].

use super::events::{big_data_events, BigData, Event};
use std::collections::BTreeMap;

/// One thing a server transfer did, for the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    FlagSet {
        name: String,
        value: String,
    },
    FlagUnset(String),
    /// `t <target>,<kind>,<action>,<params>` from an `events` text.
    Trigger {
        target: String,
        kind: String,
        action: String,
        params: String,
    },
    WeaponAdded(String),
    WeaponScript {
        weapon: String,
        bytes: usize,
    },
    WeaponDeleted(String),
    /// `addnpc`: the script of a client-side object.
    NpcScript {
        object: String,
        bytes: usize,
    },
    Materials(String),
    /// `rpg`: the text for `onRPGMessage`.
    Rpg(String),
    /// A transfer with `s1` set: that weapon's `onReceiveText`.
    ReceiveText {
        weapon: String,
        kind: String,
        option: String,
        text: String,
    },
    /// An `s2` this client does not know.
    Unknown(String),
}

/// A weapon the server gave the player.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Weapon {
    /// The `addweapon` data (a 5-byte Graal integer, a time stamp).
    pub added: Vec<u8>,
    /// The `weaponscript` text, once sent.
    pub script: Option<Vec<u8>>,
}

/// What the server's text transfers have set so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraalText {
    pub flags: BTreeMap<String, String>,
    pub weapons: BTreeMap<String, Weapon>,
    /// `addnpc` scripts by object name.
    pub npc_scripts: BTreeMap<String, Vec<u8>>,
    pub materials: Option<String>,
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// `name=value`, or a bare `name` (value empty).
fn split_flag(s: &str) -> (String, String) {
    match s.split_once('=') {
        Some((n, v)) => (n.trim().to_string(), v.to_string()),
        None => (s.trim().to_string(), String::new()),
    }
}

impl GraalText {
    /// Apply one finished transfer.
    pub fn apply(&mut self, d: &BigData) -> Vec<Applied> {
        let kind = text(&d.kind);
        let option = text(&d.option);
        if !d.weapon.is_empty() {
            return vec![Applied::ReceiveText {
                weapon: text(&d.weapon),
                kind,
                option,
                text: text(&d.text),
            }];
        }
        let body = text(&d.text);
        match kind.as_str() {
            "set" => {
                let (name, value) = split_flag(body.trim_end_matches(['\r', '\n']));
                self.flags.insert(name.clone(), value.clone());
                vec![Applied::FlagSet { name, value }]
            }
            "unset" => {
                let name = body.trim().to_string();
                self.flags.remove(&name);
                vec![Applied::FlagUnset(name)]
            }
            "events" => self.events(&body),
            "addweapon" => {
                self.weapons.entry(option.clone()).or_default().added = d.text.clone();
                vec![Applied::WeaponAdded(option)]
            }
            "weaponscript" => {
                self.weapons.entry(option.clone()).or_default().script = Some(d.text.clone());
                vec![Applied::WeaponScript {
                    weapon: option,
                    bytes: d.text.len(),
                }]
            }
            "deleteweapon" => {
                self.weapons.remove(&option);
                vec![Applied::WeaponDeleted(option)]
            }
            "addnpc" => {
                self.npc_scripts.insert(option.clone(), d.text.clone());
                vec![Applied::NpcScript {
                    object: option,
                    bytes: d.text.len(),
                }]
            }
            "setmaterials" => {
                self.materials = Some(body.clone());
                vec![Applied::Materials(body)]
            }
            "rpg" => vec![Applied::Rpg(body)],
            _ => vec![Applied::Unknown(kind)],
        }
    }

    /// An `events` text: `s <flag>` lines set a flag, `t` lines trigger.
    fn events(&mut self, body: &str) -> Vec<Applied> {
        let mut out = Vec::new();
        for line in body.lines() {
            if let Some(f) = line.strip_prefix("s ") {
                let (name, value) = split_flag(f);
                self.flags.insert(name.clone(), value.clone());
                out.push(Applied::FlagSet { name, value });
            } else if let Some(t) = line.strip_prefix("t ") {
                let mut parts = t.splitn(4, ',');
                let mut next = || parts.next().unwrap_or_default().to_string();
                out.push(Applied::Trigger {
                    target: next(),
                    kind: next(),
                    action: next(),
                    params: next(),
                });
            }
        }
        out
    }
}

impl Applied {
    /// The log line, without the `3d: ` prefix.
    pub fn describe(&self) -> String {
        match self {
            Applied::FlagSet { name, value } if value.is_empty() => format!("flag {name} set"),
            Applied::FlagSet { name, value } => format!("flag {name}={value}"),
            Applied::FlagUnset(n) => format!("flag {n} unset"),
            Applied::Trigger {
                target,
                kind,
                action,
                params,
            } => format!("trigger {target},{kind},{action},{params}"),
            Applied::WeaponAdded(w) => format!("weapon {w} added"),
            Applied::WeaponScript { weapon, bytes } => {
                format!("weapon {weapon} script ({bytes} bytes)")
            }
            Applied::WeaponDeleted(w) => format!("weapon {w} deleted"),
            Applied::NpcScript { object, bytes } => {
                format!("object {object} script ({bytes} bytes, kept; not run yet)")
            }
            Applied::Materials(m) => format!("terrain materials {m}"),
            Applied::Rpg(t) => format!("rpg {t}"),
            Applied::ReceiveText {
                weapon,
                kind,
                option,
                text,
            } => format!("text for weapon {weapon}: {kind},{option},{text}"),
            Applied::Unknown(k) => format!("text transfer {k:?} not handled"),
        }
    }
}

/// The client's `events` text: the lines its game queues, sent as one
/// transfer with `s1` and `s3` empty and `s2` `events`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientEvents {
    lines: Vec<String>,
}

impl ClientEvents {
    /// `w <weapon>`: the player picked up a weapon (getweapon).
    pub fn get_weapon(&mut self, name: &str) {
        self.lines.push(format!("w {name}"));
    }

    /// `s <name>=<value>`: a flag or client variable was set.
    pub fn set(&mut self, name: &str, value: &str) {
        self.lines.push(format!("s {name}={value}"));
    }

    /// `t weapon,<name>,<params>`: a serverside trigger from a weapon.
    pub fn trigger(&mut self, weapon: &str, params: &str) {
        self.lines.push(format!("t weapon,{weapon},{params}"));
    }

    /// `t <line>`: a `triggerserver` call, its arguments joined by commas.
    pub fn trigger_line(&mut self, line: &str) {
        self.lines.push(format!("t {line}"));
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// The queued lines as the events of transfer `transfer`, or nothing
    /// when none are queued. The queue is emptied.
    pub fn take(&mut self, transfer: u32) -> Vec<Event> {
        if self.lines.is_empty() {
            return Vec::new();
        }
        let text = std::mem::take(&mut self.lines).join("\n");
        big_data_events(transfer, b"", b"events", b"", text.as_bytes())
    }
}

/// Unpack a transfer the server compressed: zlib (1) is unpacked with
/// core's reader; bzip2 (2) is not read by this client yet.
pub fn unpack(mut d: BigData, compression: u32) -> Result<BigData, String> {
    match compression {
        0 => Ok(d),
        1 => {
            d.text = seihai_core::protocol::zlib_decompress(&d.text)
                .map_err(|e| format!("zlib text transfer: {e}"))?;
            Ok(d)
        }
        2 => Err("a bzip2 text transfer (not unpacked yet)".into()),
        n => Err(format!("a text transfer with compression {n}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(weapon: &str, kind: &str, option: &str, t: &[u8]) -> BigData {
        BigData {
            weapon: weapon.into(),
            kind: kind.into(),
            option: option.into(),
            text: t.to_vec(),
        }
    }

    #[test]
    fn flags_are_set_and_unset() {
        let mut g = GraalText::default();
        assert_eq!(
            g.apply(&data("", "set", "", b"gotkey=1")),
            [Applied::FlagSet {
                name: "gotkey".into(),
                value: "1".into()
            }]
        );
        g.apply(&data("", "set", "", b"visited"));
        assert_eq!(g.flags.get("visited").map(String::as_str), Some(""));
        g.apply(&data("", "unset", "", b"gotkey"));
        assert!(!g.flags.contains_key("gotkey"));
    }

    #[test]
    fn an_events_text_sets_flags_and_triggers() {
        let mut g = GraalText::default();
        let got = g.apply(&data(
            "",
            "events",
            "",
            b"s door=open\nt weapon,-Move3D,jump,1,2",
        ));
        assert_eq!(
            got,
            [
                Applied::FlagSet {
                    name: "door".into(),
                    value: "open".into()
                },
                Applied::Trigger {
                    target: "weapon".into(),
                    kind: "-Move3D".into(),
                    action: "jump".into(),
                    params: "1,2".into()
                }
            ]
        );
    }

    #[test]
    fn weapons_are_added_scripted_and_deleted() {
        let mut g = GraalText::default();
        g.apply(&data("", "addweapon", "-Move3D", &[1, 2, 3, 4, 5]));
        g.apply(&data("", "weaponscript", "-Move3D", b"//#CLIENTSIDE"));
        let w = &g.weapons["-Move3D"];
        assert_eq!(w.added, [1, 2, 3, 4, 5]);
        assert_eq!(w.script.as_deref(), Some(&b"//#CLIENTSIDE"[..]));
        assert_eq!(
            g.apply(&data("", "deleteweapon", "-Move3D", b"")),
            [Applied::WeaponDeleted("-Move3D".into())]
        );
        assert!(g.weapons.is_empty());
    }

    #[test]
    fn a_weapon_name_sends_the_text_to_that_weapon() {
        let mut g = GraalText::default();
        let got = g.apply(&data("-Chat", "say", "x", b"hello"));
        assert_eq!(
            got,
            [Applied::ReceiveText {
                weapon: "-Chat".into(),
                kind: "say".into(),
                option: "x".into(),
                text: "hello".into()
            }]
        );
        assert!(g.flags.is_empty());
    }

    #[test]
    fn the_client_events_go_as_one_events_transfer() {
        let mut c = ClientEvents::default();
        assert!(c.take(1).is_empty());
        c.get_weapon("-Move3D");
        c.set("client.speed", "2");
        c.trigger("-Move3D", "jump,1");
        let evs = c.take(7);
        assert!(c.is_empty());
        let Event::BigDataStart {
            transfer,
            weapon,
            kind,
            option,
            total,
            compression,
        } = &evs[0]
        else {
            panic!("a start event first")
        };
        let text = b"w -Move3D\ns client.speed=2\nt weapon,-Move3D,jump,1";
        assert_eq!(
            (
                *transfer,
                weapon.as_slice(),
                kind.as_slice(),
                option.as_slice()
            ),
            (7, &b""[..], &b"events"[..], &b""[..])
        );
        assert_eq!((*total as usize, *compression), (text.len(), 0));
        assert_eq!(
            evs[1],
            Event::BigDataChunk {
                transfer: 7,
                bytes: text.to_vec()
            }
        );
    }

    #[test]
    fn a_zlib_transfer_is_unpacked() {
        let packed = seihai_core::protocol::zlib_compress(b"gotkey=1").unwrap();
        let d = unpack(data("", "set", "", &packed), 1).unwrap();
        assert_eq!(d.text, b"gotkey=1");
        assert!(unpack(data("", "set", "", b"x"), 2).is_err());
    }
}
