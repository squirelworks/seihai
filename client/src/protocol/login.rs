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

/// The weapon image the real 2.22 client asks for when the server sends a
/// default weapon (`PLO_DEFAULTWEAPON`, 43) with this id.
///
/// Only pairs a lab run showed: with the server's ids filtered to 7 alone the
/// client asked `wbowi1.png`; ids 12, 20, 21, 22 and 23 sent after login made
/// it ask `wbomb2.png`, `wbowi2.png`, `wbowi3.png`, `wbowi4.png` and
/// `wbomb3.png`. Id 8 is left out: `wbomb1.png` was asked whether or not id 8
/// was sent, so the capture does not tie it to that id. Nothing is invented
/// for other ids.
pub fn default_weapon_image(id: u8) -> Option<&'static str> {
    match id {
        7 => Some("wbowi1.png"),
        12 => Some("wbomb2.png"),
        20 => Some("wbowi2.png"),
        21 => Some("wbowi3.png"),
        22 => Some("wbowi4.png"),
        23 => Some("wbomb3.png"),
        _ => None,
    }
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

/// The first `PLI_PLAYERPROPS` of a gen-2 build (1.33 to 1.39), sent once
/// the first level has loaded: `NICKNAME` (0), `HEADGIF` (11), `COLORS`
/// (13), `STATUS` (18), `ADDITFLAGS` (33). Live runs of 1.33 and 1.39
/// (Graal_analysis `version-profiles/1.33-1.39.md`). Head, colours and
/// status are copied from the server's own props as they came, so a
/// numbered default head goes back as `c(number)`. The real clients also
/// send `UDPPORT` (31) before `ADDITFLAGS`; as in
/// [`login_props_message`], Seihai binds no UDP socket and leaves it out.
pub fn classic_login_props_message(own_props: &[u8], account: &str, accepted: Option<&IdSet>) -> Vec<u8> {
    let props = parse_player_props_in(own_props, false, accepted);
    let mut m = vec![PLI_PLAYERPROPS + 0x20];
    m.push(0x20);
    gstring(&mut m, account);
    for id in [11u8, 13, 18] {
        if let Some(p) = props.iter().find(|p| p.id == id) {
            m.push(id + 0x20);
            m.extend(&p.raw);
        }
    }
    m.extend([33 + 0x20, 0x20]);
    m.push(b'\n');
    m
}

/// Whether `version` sends [`classic_login_props_message`] once its first
/// level has loaded: the gen-2 builds that can send `PLI_PLAYERPROPS`.
pub fn sends_classic_login_props(version: super::ClientVersion) -> bool {
    let p = version.profile();
    p.protocol.encryption == super::EncryptionGen::Gen2 && p.protocol.pli.contains(PLI_PLAYERPROPS)
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
    /// The frame has not gone out yet.
    pub fn pending(&self) -> bool {
        !self.sent
    }

    pub fn due(&self, state: &SessionState) -> bool {
        !self.sent && state.process_list_requests > 0
    }

    /// Build the frame's messages (once). `check` turns an image name into
    /// the file messages the request rules give (`PLI_UPDATEFILE` or
    /// `PLI_WANTFILE`, or nothing when the name is queued or settled).
    /// `enter_level` gives the login `PLI_ENTERLEVEL` (46) when one is owed
    /// (2.31): the real client sends it right after the file checks.
    /// `flags` gives the flag messages scripts have queued so far (see
    /// [`super::flagset`]), which go between the file checks and the process
    /// list, where the real 2.22 put the `-gr_movement` weapon's first
    /// `gr.x`/`gr.y`; it runs only when the frame is built.
    pub fn take(
        &mut self,
        state: &SessionState,
        account: &str,
        mut check: impl FnMut(&str) -> Vec<Vec<u8>>,
        enter_level: impl FnOnce() -> Vec<Vec<u8>>,
        flags: impl FnOnce() -> Vec<Vec<u8>>,
    ) -> Vec<Vec<u8>> {
        if !self.due(state) {
            return Vec::new();
        }
        self.sent = true;
        let mut out = Vec::new();
        let mut names = wear_images(&state.player_props, state.own_prop_ids.as_ref());
        names.extend(weapon_images(&state.weapons));
        // Default weapons the server sent: the real client asks for their
        // images too (`wbowi1.png` for id 7).
        names.extend(
            state
                .default_weapons
                .iter()
                .filter_map(|&id| default_weapon_image(id))
                .map(str::to_string)
                .filter(|n| !names.contains(n))
                .collect::<Vec<_>>(),
        );
        if self.info.status_icons {
            names.extend(state.status_list.iter().map(|s| status_icon(s)));
        }
        for n in &names {
            out.extend(check(n));
        }
        out.extend(enter_level());
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
    fn classic_login_props_are_nick_head_colours_status_additflags() {
        use crate::protocol::ClientVersion;
        let v = ClientVersion::V139;
        let ids = v.profile().props.own.clone();
        // A numbered default head (c(0+32)): it goes back as it came.
        let own = [11 + 0x20, 0x20, 13 + 0x20, 0x22, 0x20, 0x2a, 0x24, 0x32, 18 + 0x20, 0x21];
        let m = classic_login_props_message(&own, "acct", Some(&ids));
        let mut want = vec![PLI_PLAYERPROPS + 0x20, 0x20, 0x24];
        want.extend(b"acct");
        want.extend([11 + 0x20, 0x20, 13 + 0x20, 0x22, 0x20, 0x2a, 0x24, 0x32, 18 + 0x20, 0x21, 33 + 0x20, 0x20, b'\n']);
        assert_eq!(m, want);
        assert!(sends_classic_login_props(ClientVersion::V133) && sends_classic_login_props(v));
        assert!(!sends_classic_login_props(ClientVersion::V141) && !sends_classic_login_props(ClientVersion::V205));
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

    #[test]
    fn default_weapon_images_follow_the_lab_pairs() {
        assert_eq!(default_weapon_image(7), Some("wbowi1.png"));
        assert_eq!(default_weapon_image(20), Some("wbowi2.png"));
        assert_eq!(default_weapon_image(23), Some("wbomb3.png"));
        // No pair seen: id 8 (wbomb1.png comes with or without it), others.
        assert_eq!(default_weapon_image(8), None);
        assert_eq!(default_weapon_image(0), None);
        assert_eq!(default_weapon_image(99), None);
    }

    #[test]
    fn login_burst_asks_the_image_of_default_weapon_7() {
        let mut b = LoginBurst::for_version(super::super::ClientVersion::V222).unwrap();
        let mut st = burst_state();
        st.default_weapons = vec![8, 7];
        st.process_list_requests = 1;
        let mut asked = Vec::new();
        b.take(&st, "acct", |n| {
            asked.push(n.to_string());
            vec![n.as_bytes().to_vec()]
        }, Vec::new, Vec::new);
        // wbowi1.png is already asked for by the Bow weapon: once only; a
        // server without the Bow weapon but with default id 7 gets it too.
        assert_eq!(asked.iter().filter(|n| *n == "wbowi1.png").count(), 1);
        let mut b = LoginBurst::for_version(super::super::ClientVersion::V222).unwrap();
        let mut st = burst_state();
        st.weapons.retain(|w| w.name != "Bow");
        st.default_weapons = vec![8, 7];
        st.process_list_requests = 1;
        let mut asked = Vec::new();
        b.take(&st, "acct", |n| {
            asked.push(n.to_string());
            vec![n.as_bytes().to_vec()]
        }, Vec::new, Vec::new);
        assert!(asked.contains(&"wbowi1.png".to_string()));
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
        assert!(b.pending());
        assert!(!b.due(&st));
        assert!(b.take(&st, "acct", |_| vec![b"x".to_vec()], Vec::new, Vec::new).is_empty());
        st.process_list_requests = 1;
        assert!(b.due(&st));
        let mut asked = Vec::new();
        let out = b.take(&st, "acct", |n| {
            asked.push(n.to_string());
            vec![n.as_bytes().to_vec()]
        }, || vec![b"enter".to_vec()], || vec![b"flag".to_vec()]);
        // images in order, then the icons, the login 46, the flags, the process list and the props
        assert_eq!(
            asked,
            ["sword1.png", "shield1.png", "head0.png", "body.png", "wbomb1.png", "wbowi1.png", "plisticononline.png", "plisticonno pms.png"]
        );
        assert_eq!(out.len(), asked.len() + 4);
        assert_eq!(out[asked.len()], b"enter");
        assert_eq!(out[asked.len() + 1], b"flag");
        assert_eq!(out[asked.len() + 2][0], PLI_PROCESSLIST + 0x20);
        assert_eq!(out[asked.len() + 3][0], PLI_PLAYERPROPS + 0x20);
        assert!(!b.pending(), "sent");
        assert!(!b.due(&st), "once");
        assert!(b.take(&st, "acct", |_| vec![b"x".to_vec()], Vec::new, Vec::new).is_empty());
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
