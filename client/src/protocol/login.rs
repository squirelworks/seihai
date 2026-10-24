//! What a real 2.17, 2.22 or 2.31 client sends once the server's login burst
//! is in: the image checks, the process list and the login props.
//!
//! Source: the lab, 2026-10-24 and 2026-10-25, real clients on a private
//! loopback server (`documentation/login-burst.md`). All three builds send,
//! in one frame about 60 ms after the burst (the burst ends with
//! `PLO_LISTPROCESSES`, 182):
//!
//! 1. one file check per image the player wears or carries, in this order:
//!    sword image, shield image, head image, body image, then the image of
//!    each weapon in the order the server gave the weapons. A file the client
//!    has gets `PLI_UPDATEFILE` (34) with its mod time; a missing one gets
//!    `PLI_WANTFILE` (23);
//! 2. `PLI_PROCESSLIST` (44), the answer to 182;
//! 3. `PLI_PLAYERPROPS` (2) with the player's identity props.
//!
//! The status icons (`plisticon<status>.png`, one per `PLO_STATUSLIST` entry)
//! are asked for as well; they ride the same request queue (four in flight).
//!
//! What Seihai leaves out, and why, is in the document.

use super::dispatch::{latin1, Weapon};
use super::props::parse_player_props_in;
use super::{IdSet, SessionState};

/// `PLI_PLAYERPROPS`.
pub const PLI_PLAYERPROPS: u8 = 2;
/// `PLI_PROCESSLIST`.
pub const PLI_PROCESSLIST: u8 = 44;
/// `PLO_LISTPROCESSES`, which the burst ends with.
pub const PLO_LISTPROCESSES: u8 = 182;

/// What a profile records about the post-login frame. `Some` only for the
/// builds observed doing it (2.17, 2.22 and 2.31).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoginInfo {
    /// The props carry `OSTYPE` (75) and `TEXTCODEPAGE` (76): 2.22 and 2.31;
    /// the 2.17 message stops at `PLANGUAGE`.
    pub os_props: bool,
    /// `WANTFILE idle.gani` goes out at once with the first frame after the
    /// login line (2.17 and 2.22; 2.31 does not ask).
    pub idle_gani: bool,
    /// The status icons are asked for after the burst.
    pub status_icons: bool,
}

pub const LOGIN_217: LoginInfo = LoginInfo { os_props: false, idle_gani: true, status_icons: true };
pub const LOGIN_222: LoginInfo = LoginInfo { os_props: true, idle_gani: true, status_icons: true };
pub const LOGIN_231: LoginInfo = LoginInfo { os_props: true, idle_gani: false, status_icons: true };

/// The windows-1252 code page, as `TEXTCODEPAGE` carries it.
const CODEPAGE_1252: u32 = 1252;
/// What the clients put in `OSTYPE`. Every real client observed ran on
/// Windows; Seihai has no value of its own to report, so it sends the one
/// servers know.
const OSTYPE: &str = "wind";
/// The language the post-login `PLI_LANGUAGE` names.
pub const LANGUAGE: &str = "English";

fn gstring(out: &mut Vec<u8>, text: &str) {
    let b: Vec<u8> = text.chars().map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' }).take(223).collect();
    out.push(b.len() as u8 + 0x20);
    out.extend(b);
}

/// The image names in the own player's `PLO_PLAYERPROPS` (9) payload, in
/// check order: `SWORDPOWER` (8), `SHIELDPOWER` (9), `HEADGIF` (11), then
/// `BODYIMG` (35). A prop whose value has no image gives nothing.
pub fn wear_images(own_props: &[u8], accepted: Option<&IdSet>) -> Vec<String> {
    let mut sword = None;
    let mut shield = None;
    let mut head = None;
    let mut body = None;
    for p in parse_player_props_in(own_props, false, accepted) {
        let r = &p.raw;
        let name = |bytes: Option<&[u8]>| bytes.map(latin1).filter(|s| !s.is_empty());
        match p.id {
            // power byte, then a GSTRING image when the power byte says so
            8 | 9 => {
                let limit = if p.id == 8 { 4 } else { 3 };
                if r.first().map_or(false, |b| b.wrapping_sub(0x20) > limit) {
                    let n = r.get(1).map_or(0, |b| b.wrapping_sub(0x20) as usize);
                    let v = name(r.get(2..2 + n));
                    if p.id == 8 { sword = v } else { shield = v }
                }
            }
            // length + 100, then the image
            11 => {
                let v = r.first().map_or(0, |b| b.wrapping_sub(0x20) as usize);
                if v >= 100 {
                    head = name(r.get(1..1 + (v - 100)));
                }
            }
            35 => {
                let n = r.first().map_or(0, |b| b.wrapping_sub(0x20) as usize);
                body = name(r.get(1..1 + n));
            }
            _ => {}
        }
    }
    [sword, shield, head, body].into_iter().flatten().collect()
}

/// The image of each weapon, in the order the server added them.
///
/// A name with no extension gets `.gif`: the lab's server gave a weapon the
/// image `-`, and the real 2.22 and 2.31 clients asked for `-.gif`.
pub fn weapon_images(weapons: &[Weapon]) -> Vec<String> {
    weapons
        .iter()
        .filter_map(|w| w.image.clone())
        .filter(|s| !s.is_empty())
        .map(|s| if s.contains('.') { s } else { format!("{s}.gif") })
        .collect()
}

/// The icon file for a status name: `plisticon` and the lower-case name
/// (`No PMs` gives `plisticonno pms.png`).
pub fn status_icon(status: &str) -> String {
    format!("plisticon{}.png", status.to_ascii_lowercase())
}

/// `PLI_PROCESSLIST`: the processes the client runs, one per line,
/// gtokenized (GServer reads it with `guntokenize` and splits on newlines).
/// Seihai lists itself and nothing else.
pub fn process_list_message(names: &[&str]) -> Vec<u8> {
    let mut m = vec![PLI_PROCESSLIST + 0x20];
    m.extend(
        names
            .iter()
            .map(|n| format!("\"{}\"", n.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",")
            .bytes(),
    );
    m.push(b'\n');
    m
}

/// The login `PLI_PLAYERPROPS`, in the order the real clients write it:
/// `NICKNAME` (0), `HEADGIF` (11), `COLORS` (13), `STATUS` (18),
/// `ADDITFLAGS` (33), `PLANGUAGE` (52), and with `os_props` `OSTYPE` (75) and
/// `TEXTCODEPAGE` (76). Head, colours and status are copied from the server's
/// own props as they came. A real client also sends `UDPPORT` (31) between
/// `STATUS` and `ADDITFLAGS`; Seihai binds no UDP socket, so it leaves the
/// prop out.
pub fn login_props_message(info: &LoginInfo, own_props: &[u8], account: &str, accepted: Option<&IdSet>) -> Vec<u8> {
    let props = parse_player_props_in(own_props, false, accepted);
    let raw_of = |id: u8| props.iter().find(|p| p.id == id).map(|p| p.raw.clone());
    let mut m = vec![PLI_PLAYERPROPS + 0x20];
    m.push(0x20);
    gstring(&mut m, account);
    for id in [11u8, 13, 18] {
        if let Some(raw) = raw_of(id) {
            m.push(id + 0x20);
            m.extend(raw);
        }
    }
    m.extend([33 + 0x20, 0x20]);
    m.push(52 + 0x20);
    gstring(&mut m, LANGUAGE);
    if info.os_props {
        m.push(75 + 0x20);
        gstring(&mut m, OSTYPE);
        m.push(76 + 0x20);
        let v = CODEPAGE_1252;
        m.extend([((v >> 14) & 0x7F) as u8 + 0x20, ((v >> 7) & 0x7F) as u8 + 0x20, (v & 0x7F) as u8 + 0x20]);
    }
    m.push(b'\n');
    m
}

/// Decides when the post-login frame goes out: once, when the server's
/// burst has ended (`PLO_LISTPROCESSES` seen). A build with no
/// [`LoginInfo`], or one that cannot send ids 34 and 44, never sends it.
#[derive(Debug, Clone)]
pub struct LoginBurst {
    info: LoginInfo,
    sent: bool,
}

impl LoginBurst {
    /// `None` for a profile without a recorded post-login frame.
    pub fn for_version(version: super::ClientVersion) -> Option<Self> {
        let p = version.profile();
        let info = p.protocol.login?;
        (p.protocol.pli.contains(PLI_PROCESSLIST) && p.protocol.pli.contains(PLI_PLAYERPROPS)).then_some(Self { info, sent: false })
    }

    pub fn info(&self) -> LoginInfo {
        self.info
    }

    /// The burst has ended and the frame is still owed.
    pub fn due(&self, state: &SessionState) -> bool {
        !self.sent && state.process_list_requests > 0
    }

    /// Build the frame's messages (once). `check` turns an image name into
    /// the file messages the request rules give (`PLI_UPDATEFILE` or
    /// `PLI_WANTFILE`, or nothing when the name is queued or settled).
    /// `flags` gives the `gr.x`/`gr.y` flag messages (see
    /// [`super::grflags`]), which go between the file checks and the process
    /// list; it runs only when the frame is built.
    pub fn take(
        &mut self,
        state: &SessionState,
        account: &str,
        mut check: impl FnMut(&str) -> Vec<Vec<u8>>,
        flags: impl FnOnce() -> Vec<Vec<u8>>,
    ) -> Vec<Vec<u8>> {
        if !self.due(state) {
            return Vec::new();
        }
        self.sent = true;
        let mut out = Vec::new();
        let mut names = wear_images(&state.player_props, state.own_prop_ids.as_ref());
        names.extend(weapon_images(&state.weapons));
        if self.info.status_icons {
            names.extend(state.status_list.iter().map(|s| status_icon(s)));
        }
        for n in &names {
            out.extend(check(n));
        }
        out.extend(flags());
        out.push(process_list_message(&["seihai"]));
        out.push(login_props_message(&self.info, &state.player_props, account, state.own_prop_ids.as_ref()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2.22-style own-props stream, built by hand: power bytes, image
    /// strings, head, colours, status, body.
    fn own_props() -> Vec<u8> {
        let mut p = Vec::new();
        p.extend([8 + 0x20, 31 + 0x20]);
        gstring(&mut p, "sword1.png");
        p.extend([9 + 0x20, 11 + 0x20 + 0]);
        gstring(&mut p, "shield1.png");
        p.push(11 + 0x20);
        p.push(0x20 + 100 + 9);
        p.extend(b"head0.png");
        p.extend([13 + 0x20, 0x22, 0x20, 0x2a, 0x24, 0x32]);
        p.extend([18 + 0x20, 0x20 + 20]);
        p.push(35 + 0x20);
        gstring(&mut p, "body.png");
        p
    }

    #[test]
    fn wear_images_come_in_prop_order() {
        assert_eq!(wear_images(&own_props(), None), ["sword1.png", "shield1.png", "head0.png", "body.png"]);
    }

    #[test]
    fn a_prop_without_image_gives_none() {
        // sword power 1 (byte 0x20 + 1), shield power 1 (0x20 + 1): no images.
        let p = [8 + 0x20, 0x21, 9 + 0x20, 0x21];
        assert!(wear_images(&p, None).is_empty());
    }

    #[test]
    fn status_icon_names() {
        assert_eq!(status_icon("Online"), "plisticononline.png");
        assert_eq!(status_icon("No PMs"), "plisticonno pms.png");
    }

    #[test]
    fn process_list_is_gtokenized() {
        assert_eq!(process_list_message(&["seihai"]), b"\x4c\"seihai\"\n");
        assert_eq!(process_list_message(&["a", "b"]), b"\x4c\"a\",\"b\"\n");
    }

    #[test]
    fn login_props_222_layout() {
        let m = login_props_message(&LOGIN_222, &own_props(), "acct", None);
        // id, NICKNAME, HEADGIF, COLORS, STATUS, ADDITFLAGS, PLANGUAGE, OSTYPE, TEXTCODEPAGE
        let mut want = vec![0x22, 0x20, 0x24];
        want.extend(b"acct");
        want.extend([0x2b, 0x20 + 109]);
        want.extend(b"head0.png");
        want.extend([0x2d, 0x22, 0x20, 0x2a, 0x24, 0x32]);
        want.extend([0x32, 0x20 + 20]);
        want.extend([0x41, 0x20]);
        want.extend([0x20 + 52, 0x20 + 7]);
        want.extend(b"English");
        want.extend([0x20 + 75, 0x24]);
        want.extend(b"wind");
        want.extend([0x20 + 76, 0x20, 0x29, 0x84]);
        want.push(b'\n');
        assert_eq!(m, want);
        // TEXTCODEPAGE 1252 reads back as the capture's three bytes.
        assert_eq!(&m[m.len() - 4..m.len() - 1], &[0x20, 0x29, 0x84]);
    }

    #[test]
    fn login_props_217_stop_at_language() {
        let m = login_props_message(&LOGIN_217, &own_props(), "acct", None);
        let props = parse_player_props_in(&m[1..], false, None);
        let ids: Vec<u8> = props.iter().map(|p| p.id).collect();
        assert_eq!(ids, [0, 11, 13, 18, 33, 52]);
    }

    #[test]
    fn login_props_are_read_by_the_prop_parser() {
        let m = login_props_message(&LOGIN_231, &own_props(), "acct", None);
        let ids: Vec<u8> = parse_player_props_in(&m[1..m.len() - 1], false, None).iter().map(|p| p.id).collect();
        assert_eq!(ids, [0, 11, 13, 18, 33, 52, 75, 76]);
    }

    fn burst_state() -> SessionState {
        let mut st = SessionState::for_version(super::super::ClientVersion::V222);
        st.player_props = own_props();
        st.weapons = vec![
            Weapon { name: "Bomb".into(), image: Some("wbomb1.png".into()), script: None },
            Weapon { name: "Bow".into(), image: Some("wbowi1.png".into()), script: None },
        ];
        st.status_list = vec!["Online".into(), "No PMs".into()];
        st
    }

    #[test]
    fn frame_waits_for_the_process_list_request_and_goes_once() {
        let mut b = LoginBurst::for_version(super::super::ClientVersion::V222).unwrap();
        let mut st = burst_state();
        assert!(!b.due(&st));
        assert!(b.take(&st, "acct", |_| vec![b"x".to_vec()], Vec::new).is_empty());
        st.process_list_requests = 1;
        assert!(b.due(&st));
        let mut asked = Vec::new();
        let out = b.take(&st, "acct", |n| {
            asked.push(n.to_string());
            vec![n.as_bytes().to_vec()]
        }, || vec![b"flag".to_vec()]);
        // images in order, then the icons, the flags, the process list and the props
        assert_eq!(
            asked,
            ["sword1.png", "shield1.png", "head0.png", "body.png", "wbomb1.png", "wbowi1.png", "plisticononline.png", "plisticonno pms.png"]
        );
        assert_eq!(out.len(), asked.len() + 3);
        assert_eq!(out[asked.len()], b"flag");
        assert_eq!(out[asked.len() + 1][0], PLI_PROCESSLIST + 0x20);
        assert_eq!(out[asked.len() + 2][0], PLI_PLAYERPROPS + 0x20);
        assert!(!b.due(&st), "once");
        assert!(b.take(&st, "acct", |_| vec![b"x".to_vec()], Vec::new).is_empty());
    }

    #[test]
    fn a_weapon_image_without_an_extension_gets_gif() {
        let w = |n: &str, i: &str| Weapon { name: n.into(), image: Some(i.into()), script: None };
        let list = [w("a", "wbomb1.png"), w("b", "-"), w("c", "")];
        assert_eq!(weapon_images(&list), ["wbomb1.png", "-.gif"]);
    }

    #[test]
    fn builds_without_a_recorded_frame_send_nothing() {
        use super::super::ClientVersion::*;
        for v in [V205, V210, V230] {
            assert!(LoginBurst::for_version(v).is_none(), "{v:?}");
        }
        for v in [V217, V222, V231] {
            assert!(LoginBurst::for_version(v).is_some(), "{v:?}");
        }
    }

    #[test]
    fn only_the_builds_that_ask_send_idle_gani() {
        use super::super::ClientVersion::*;
        assert!(LoginBurst::for_version(V217).unwrap().info().idle_gani);
        assert!(LoginBurst::for_version(V222).unwrap().info().idle_gani);
        assert!(!LoginBurst::for_version(V231).unwrap().info().idle_gani);
    }
}
