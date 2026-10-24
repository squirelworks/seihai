//! Per-version client profiles.
//!
//! One table row per client version Seihai can present as. A row says how
//! that version logs in and frames its traffic, which server packets and
//! props it understands, which file and level features it has, and which
//! parts of the client it shows. The rest of Seihai reads these fields
//! instead of matching on versions, so picking a version in the GUI makes
//! the whole client behave as that build.
//!
//! Sources, all cleaned (no decompile material):
//! - Graal_analysis `client-internals/cross-version-timeline.md`
//!   (packet and prop switch sets read from each build, 1.33 to 2.1 tests)
//! - Graal_analysis `protocol/client-versions.md` and `player-props.md`
//!   (2.05, 2.17, 2.22, 2.31 read from the binaries)
//! - Graal_analysis `client-internals/feature-timeline.md` and the shipped
//!   release notes (features per version)
//! - GServer-v2 and gs2lib (public source): version tokens and the
//!   server's per-version branches
//!
//! Rows marked [`Basis::Inferred`] have no client binary behind them; their
//! values come from release notes and server source and should be checked
//! when a build turns up.

use super::showimg::ShowImgInfo;
use std::time::Duration;
use super::tampercheck::{TamperInfo, TamperResend, TamperWords};
use super::version::{ClientVersion, EncryptionGen};

/// A set of message or prop ids (0 to 255).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IdSet([u64; 4]);

impl IdSet {
    pub const EMPTY: IdSet = IdSet([0; 4]);

    /// Every id from `lo` to `hi`, inclusive.
    pub const fn range(lo: u8, hi: u8) -> Self {
        Self::EMPTY.with_range(lo, hi)
    }

    pub const fn with(self, id: u8) -> Self {
        let mut w = self.0;
        w[(id / 64) as usize] |= 1 << (id % 64);
        IdSet(w)
    }

    pub const fn without(self, id: u8) -> Self {
        let mut w = self.0;
        w[(id / 64) as usize] &= !(1 << (id % 64));
        IdSet(w)
    }

    pub const fn with_range(self, lo: u8, hi: u8) -> Self {
        let mut s = self;
        let mut i = lo as u16;
        while i <= hi as u16 {
            s = s.with(i as u8);
            i += 1;
        }
        s
    }

    pub const fn without_range(self, lo: u8, hi: u8) -> Self {
        let mut s = self;
        let mut i = lo as u16;
        while i <= hi as u16 {
            s = s.without(i as u8);
            i += 1;
        }
        s
    }

    pub const fn with_all(self, ids: &[u8]) -> Self {
        let mut s = self;
        let mut i = 0;
        while i < ids.len() {
            s = s.with(ids[i]);
            i += 1;
        }
        s
    }

    pub const fn without_all(self, ids: &[u8]) -> Self {
        let mut s = self;
        let mut i = 0;
        while i < ids.len() {
            s = s.without(ids[i]);
            i += 1;
        }
        s
    }

    /// Ids in both sets.
    pub const fn and(self, other: IdSet) -> Self {
        IdSet([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }

    pub const fn contains(&self, id: u8) -> bool {
        self.0[(id / 64) as usize] & (1 << (id % 64)) != 0
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..=255u8).filter(|&i| self.contains(i))
    }
}

impl std::fmt::Debug for IdSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// How well a row is backed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// Read from that version's client binary.
    Binary,
    /// No binary: release notes, server source, and the neighbouring builds.
    Inferred,
}

/// Where the client finds servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerList {
    /// No listserver: the client connects straight to one game server.
    Direct { default: &'static str },
    /// A listserver with one list.
    Single,
    /// A listserver with this many tabs (2.14: company and playerworlds;
    /// 2.22 adds under-construction worlds).
    Tabs(u8),
}

/// Listserver login.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listserver {
    pub port: u16,
    /// Token sent at listserver login. 2.22 and 2.31 send `GNW30123`
    /// inside id 7; older clients send their game token.
    pub token: Option<&'static str>,
}

/// Login, framing and the packets a version speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Protocol {
    /// First GCHAR of the login line.
    pub client_type: u8,
    pub encryption: EncryptionGen,
    pub listserver: Option<Listserver>,
    /// Server→client ids the client has a handler for. The client drops
    /// any other id without error.
    pub plo: IdSet,
    /// Client→server ids the build can send. Seihai sends nothing else.
    pub pli: IdSet,
    /// `PLI_LANGUAGE` (37) right after login.
    pub sends_language: bool,
    /// `PLI_TAMPERCHECK` (39) after the language.
    pub tampercheck: bool,
    /// The build's tamper-check constant and re-send rule. `Some` for every
    /// build with `tampercheck`; `words` is `None` where the constant was
    /// never captured (2.3). Seihai sends it only when asked
    /// (`--send-tampercheck`); see [`super::tampercheck`].
    pub tamper: Option<TamperInfo>,
    /// When the build sends `PLI_SHOWIMGPLAYER` (24) with its constant 9;
    /// `Some` only for the builds observed doing it (2.22 and 2.31).
    /// Seihai sends it only when asked (`--send-showimg`); see
    /// [`super::showimg`].
    pub showimg: Option<ShowImgInfo>,
    /// Peer-to-peer UDP channel present.
    pub udp: bool,
    /// UDP off by default (`noudp=true` in the shipped config).
    pub udp_off_by_default: bool,
}

/// Player prop parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Props {
    /// Ids the `PLO_PLAYERPROPS` (own player) parser accepts. The first
    /// id outside the set ends the stream.
    pub own: IdSet,
    /// Ids the `PLO_OTHERPLPROPS` parser accepts, same rule.
    pub other: IdSet,
    /// Highest NPC prop id parsed.
    pub npc_max: u8,
    /// Prop 10 is the old bow slot (`c(10 + len) + image`) instead of a
    /// gani name. GServer sends that form below 2.1.
    pub bow_prop10: bool,
    /// X2/Y2/Z2 (78–80) and `PLO_MOVE2` are used for positions.
    pub precise_pos: bool,
}

/// File transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Files {
    /// `PLO_FILE` carries a 5-byte modtime (2.1+).
    pub modtime: bool,
    /// `PLO_LARGEFILESTART`/`END` (68/69): files over 64 KB (2.14+).
    pub large_files: bool,
    /// `PLO_LARGEFILESIZE` (84) (2.3+).
    pub large_file_size: bool,
    /// Default image extension. GServer adds `.gif` to bare image names for
    /// clients below 2.1.
    pub image_ext: &'static str,
}

/// Level formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Levels {
    /// `.graal` (GR-V1.0x) levels.
    pub graal: bool,
    /// `.nw` (GLEVNW01) levels.
    pub nw: bool,
    /// `.gmap` overworlds.
    pub gmap: bool,
    /// Level heights in the level file (`PLO_BOARDHEIGHTS`, 2.17+).
    pub heights: bool,
    /// The build asks for a level it already holds with `PLI_LEVELWARPMOD`
    /// (30, with the level's mod time) instead of `PLI_LEVELWARP` (0).
    /// Seen on real 2.22 and 2.31 clients (lab, 2026-10-24); written up for
    /// 2.05 in Graal_analysis `packet-layouts.md`; taken as the same for
    /// the builds between and after (inferred). Not claimed for 1.33 to
    /// 2.04: those binaries send id 30, but no document or capture says
    /// they use it for cached levels.
    pub cached_links: bool,
}

/// The HUD sheet (`state.png`) a build ships. A version's own asset folder
/// supplies the image; this says which layout to expect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudArt {
    /// 1.33–1.34: 224×81, A/S/D letters on the item buttons, digits in
    /// `sprites.png`.
    Letters,
    /// 1.35–1.37: as `Letters` with a CHAT label.
    LettersChat,
    /// 1.38+: 224×97, blank item buttons (keys are configurable) and the
    /// digits moved into `state.png`. [later builds inferred]
    BlankButtons,
}

/// Which parts of the client this version shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ui {
    pub server_list: ServerList,
    /// Up/Down recalls earlier typed chat lines (1.39+, last 100).
    pub chat_recall: bool,
    /// Which HUD art the build ships in `state.png`.
    pub hud: HudArt,
    /// Minimap set by the game server (2.1+).
    pub minimap: bool,
    /// The level editor (F4) and level loading (F5) are inside the client.
    /// 2.3 moved them to a separate GraalEditor.
    pub editor: bool,
    /// Ganis (`TGraalAni`, NewMain 1.42+).
    pub ganis: bool,
    /// Emotion icons on Ctrl+letter (1.38+).
    pub emoticons: bool,
    /// Configurable keys (1.38+).
    pub key_config: bool,
    /// Alt+0 log window (2.1+).
    pub log_window: bool,
    /// Ghost mode for free players (2.1+).
    pub ghost_mode: bool,
    /// Server name in the window title (2.05+).
    pub server_in_title: bool,
    /// Number of GS1 commands in the script table, for reference.
    pub gs1_commands: u16,
}

/// How a build's own client handles the player's death.
///
/// Read from the 2.05 rev 2 decompile (`decomp-findings-2026-10-24/
/// 01-death-respawn.md`, `constants-205.tsv`); the tick is 50 ms. Later
/// builds are taken as the same (inferred) and the builds before 2.05
/// have no row, because their handler was not traced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathRules {
    /// The death timer counts up one per tick and stops here (30 = 1.5 s).
    pub timer_cap: u16,
    /// Controls are ignored while the timer is at or below this (27); the
    /// first tick past it (28) lets the player act and respawn.
    pub input_lock_until: u16,
    /// The dead bit (0x08) goes into the STATUS prop (18) once the timer
    /// reaches this (28, 1.4 s). Before it, others see no death.
    pub status_bit_from: u16,
    /// Respawn hearts: below `low_ap_below` AP the player gets
    /// `low_hearts`; below `mid_ap_below` AP min(`mid_hearts_cap`, max
    /// hearts); otherwise max hearts.
    pub low_ap_below: u8,
    pub mid_ap_below: u8,
    pub low_hearts: u8,
    pub mid_hearts_cap: u8,
    /// Position used when no level entry point was saved and the client is
    /// online: x 30.5, y 30.0 in the current level, facing down. Kept in
    /// half tiles (61 and 60), the wire scale.
    pub default_x2: u8,
    pub default_y2: u8,
}

/// The 2.05 rev 2 values.
pub const DEATH_205: DeathRules = DeathRules {
    timer_cap: 30,
    input_lock_until: 27,
    status_bit_from: 28,
    low_ap_below: 20,
    mid_ap_below: 40,
    low_hearts: 3,
    mid_hearts_cap: 5,
    default_x2: 61,
    default_y2: 60,
};

/// Health and death.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    /// The client-side death sequence, where it was traced.
    pub death: Option<DeathRules>,
}

/// Everything that makes one client version itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionProfile {
    pub version: ClientVersion,
    /// Display name, e.g. "2.05".
    pub label: &'static str,
    /// Eight-character token sent at login.
    pub token: &'static str,
    /// Link date of the build, where known.
    pub date: &'static str,
    pub basis: Basis,
    pub protocol: Protocol,
    pub props: Props,
    pub files: Files,
    pub levels: Levels,
    pub life: Health,
    pub ui: Ui,
}

impl VersionProfile {
    /// Window title the way this version shows it.
    pub fn window_title(&self, server: Option<&str>) -> String {
        match server {
            Some(s) if self.ui.server_in_title && !s.is_empty() => format!("Graal {} - {s}", self.label),
            _ => format!("Graal {}", self.label),
        }
    }

    /// Level file extensions this version can open.
    pub fn level_extensions(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.levels.nw {
            v.push(".nw");
        }
        if self.levels.graal {
            v.push(".graal");
        }
        if self.levels.gmap {
            v.push(".gmap");
        }
        v
    }

    /// One-line summary for the GUI.
    pub fn summary(&self) -> String {
        let gen = match self.protocol.encryption {
            EncryptionGen::Gen1 => "gen 1",
            EncryptionGen::Gen2 => "gen 2 (zlib)",
            EncryptionGen::Gen3 => "gen 3 (zlib + LCG byte)",
            EncryptionGen::Gen5 => "gen 5 (XOR + zlib/bz2)",
        };
        let list = match self.ui.server_list {
            ServerList::Direct { .. } => "no listserver".to_string(),
            ServerList::Single => "server list".to_string(),
            ServerList::Tabs(n) => format!("server list, {n} tabs"),
        };
        let basis = match self.basis {
            Basis::Binary => "",
            Basis::Inferred => ", inferred",
        };
        format!(
            "{} {}: {gen}, {list}, {} server packets{basis}",
            self.label,
            self.token,
            self.protocol.plo.len()
        )
    }
}

// Server→client id sets, from each build's receive switch.

/// 1.33 to 1.37: 0–41 and 100.
const PLO_133: IdSet = IdSet::range(0, 41).with(100);
const PLO_138: IdSet = PLO_133.with(43);
const PLO_139: IdSet = PLO_138.with(75);
const PLO_142: IdSet = PLO_139.with_all(&[42, 44, 45, 150, 151, 153]);
const PLO_200: IdSet = PLO_142.with_all(&[154, 155]);
const PLO_201: IdSet = PLO_200.with_all(&[46, 156]);
const PLO_202: IdSet = PLO_201.with(47);
const PLO_203: IdSet = PLO_202.with_all(&[48, 165]);
const PLO_204: IdSet = PLO_203.with(166).without(26);
/// 59 ids.
const PLO_205: IdSet = PLO_204.with(168);
/// 2.1: the 2.1 test builds add 49 and 169; ghost mode and the gserver
/// bigmap/minimap (170–174) arrive with 2.1. [inferred]
const PLO_210: IdSet = PLO_205.with(49).with_range(169, 174);
/// 2.14: large files (68/69) and ids 178 and 180, as the version-profile
/// TOML has them; the rest of 175–185 is first seen in 2.17. [inferred]
const PLO_214: IdSet = PLO_210.with_all(&[68, 69, 178, 180]);
/// 79 ids.
const PLO_217: IdSet = PLO_205.with_all(&[49, 68, 69]).with_range(169, 185);
/// 80 ids.
const PLO_222: IdSet = PLO_217.with(186);
/// 81 ids: drops `HASNPCSERVER`, adds `LARGEFILESIZE` and `MOVE2`.
const PLO_231: IdSet = PLO_222.without(44).with_all(&[84, 189]);

// Client→server id sets.

/// 1.33–1.37: 0–31.
const PLI_133: IdSet = IdSet::range(0, 31);
/// 1.38: adds ITEMTAKE (32) and WEAPONADD (33), drops 25.
const PLI_138: IdSet = PLI_133.without(25).with_all(&[32, 33]);
/// 1.39: adds PROFILEGET/SET (80, 81).
const PLI_139: IdSet = PLI_138.with_all(&[80, 81]);
/// 2.05 rev 2: 0–24, 26–30, 32–38, 80, 81.
const PLI_205: IdSet = IdSet::range(0, 24)
    .with_range(26, 30)
    .with_range(32, 38)
    .with_all(&[80, 81]);
/// 1.41: 1.39 without 31 and 34–36. Ids passed as constants to the send
/// method, read from the 1.41 binary.
const PLI_141: IdSet = PLI_139.without(31);
/// 1.42 and 2.00 add UPDATEFILE (34).
const PLI_142: IdSet = PLI_141.with(34);
/// 2.01 and 2.02 add ADJACENTLEVEL (35) and HITOBJECTS (36).
const PLI_201: IdSet = PLI_142.with_all(&[35, 36]);
/// 2.02 rev 2 adds LANGUAGE (37).
const PLI_202R2: IdSet = PLI_201.with(37);
/// 2.03 adds TRIGGERACTION (38). 2.04 and 2.05 send the same set.
const PLI_203: IdSet = PLI_202R2.with(38);
/// 2.14: adds SERVERWARP (41). [inferred]
const PLI_214: IdSet = PLI_205.with(41);
/// 2.17: 2.05's set plus TAMPERCHECK, SHOOT and SERVERWARP (39–41) and
/// MUTEPLAYER and the process-list pair (43–45).
const PLI_217: IdSet = PLI_205.with_range(39, 41).with_range(43, 45);
/// 2.22: 2.17 without 36, 45 and 81 (no send site was matched for 36 and
/// 45; 81 is not sent), so 0–24, 26–30, 32–35, 37–41, 43, 44 and 80.
const PLI_222: IdSet = PLI_217.without_all(&[36, 45, 81]);
/// 2.31: 2.22 minus PUTNPC (21), plus ENTERLEVEL (46). A real 2.31 client
/// answered the server's process-list request with PLI 44 (lab capture), so
/// 44 stays, as in the TOML's `pli_sent` list.
const PLI_231: IdSet = PLI_222.without(21).with(46);

// Own-player prop sets (`PLO_PLAYERPROPS`).
const OWN_133: IdSet = IdSet::range(0, 33);
const OWN_139: IdSet = IdSet::range(0, 34);
const OWN_141: IdSet = IdSet::range(0, 36);
const OWN_202: IdSet = IdSet::range(0, 41);
const OWN_204: IdSet = IdSet::range(0, 42);
const OWN_210: IdSet = IdSet::range(0, 45);
/// 2.16 adds GATTRIB10–30 (54–74). [inferred for 2.16/2.17]
const OWN_216: IdSet = IdSet::range(0, 74).without_all(&[50, 51, 53]);
const OWN_222: IdSet = IdSet::range(0, 74).without_all(&[50, 51, 53]);
const OWN_231: IdSet = IdSet::range(0, 80).without_all(&[50, 51, 53, 75, 76, 77]);

// Other-player prop sets (`PLO_OTHERPLPROPS`). Only the 2.05, 2.22 and
// 2.31 sets are read from binaries; older ones are the 2.05 set cut to the
// own-prop range of that build. [inferred]
const OTHER_205: IdSet = IdSet::EMPTY
    .with(0)
    .with(2)
    .with_range(8, 13)
    .with_range(15, 21)
    .with(24)
    .with_range(30, 42)
    .with_all(&[50, 51]);
/// 2.1 test builds: the 2.05 set plus 43–45 (GMAPLEVELX/Y, Z).
const OTHER_210: IdSet = OTHER_205.with_range(43, 45);
const OTHER_222: IdSet = IdSet::range(0, 76).without_all(&[1, 3, 4, 5, 6, 7, 14, 22, 25, 26, 27, 28, 29]);
const OTHER_231: IdSet = IdSet::range(0, 80).without_all(&[1, 3, 4, 5, 6, 7, 14, 22, 25, 26, 27, 28, 29, 77]);

/// 1.33: the 2.05 set below 33, plus 50 and 51.
const OTHER_133: IdSet = OTHER_205.and(IdSet::range(0, 32)).with_all(&[50, 51]);
/// 1.34–1.38 add ACCOUNTNAME (34).
const OTHER_134: IdSet = OTHER_133.with(34);
/// 1.39 adds 33.
const OTHER_139: IdSet = OTHER_134.with(33);

/// The 2.05 other-player set cut to `0..=max`, keeping 50 and 51.
const fn other_upto(max: u8) -> IdSet {
    OTHER_205.and(IdSet::range(0, max).with_all(&[50, 51]))
}

// Tamper-check constants (CRC words A and B), from the 2026-10-24 decompile
// reading; the encoded bytes match the lab captures of the real clients.
const TAMPER_217: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0xab63_7cbe, b: 0x3e48_dc0c }),
    resend: TamperResend::Never,
};
const TAMPER_222: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0x5dcb_77f6, b: 0xbc73_6311 }),
    resend: TamperResend::OnLevelChange,
};
const TAMPER_231: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0x9ecd_4bc0, b: 0xecea_65e2 }),
    resend: TamperResend::Timer,
};
/// 2.3: no binary, so no constant; it is taken to re-send like 2.31.
const TAMPER_230: TamperInfo = TamperInfo { words: None, resend: TamperResend::Timer };

// Id 24 (lab, real clients, 2026-10-24): 2.22 about every 5.1 s while another
// player is in the level, 2.31 every 5.13 s always.
const SHOWIMG_222: ShowImgInfo = ShowImgInfo { period: Duration::from_millis(5100), needs_others: true };
const SHOWIMG_231: ShowImgInfo = ShowImgInfo { period: Duration::from_millis(5130), needs_others: false };

/// The base every row starts from: a 2.05 rev 2 client.
const BASE: VersionProfile = VersionProfile {
    version: ClientVersion::V205,
    label: "2.05",
    token: "GNW03081",
    date: "",
    basis: Basis::Binary,
    protocol: Protocol {
        client_type: 0,
        encryption: EncryptionGen::Gen3,
        listserver: Some(Listserver { port: 14921, token: None }),
        plo: PLO_205,
        pli: PLI_205,
        sends_language: true,
        tampercheck: false,
        tamper: None,
        showimg: None,
        udp: true,
        udp_off_by_default: false,
    },
    props: Props {
        own: OWN_204,
        other: OTHER_205,
        npc_max: 40,
        bow_prop10: true,
        precise_pos: false,
    },
    files: Files {
        modtime: false,
        large_files: false,
        large_file_size: false,
        image_ext: ".gif",
    },
    levels: Levels {
        graal: true,
        nw: true,
        gmap: true,
        heights: false,
        cached_links: true,
    },
    life: Health { death: Some(DEATH_205) },
    ui: Ui {
        server_list: ServerList::Single,
        chat_recall: true,
        hud: HudArt::BlankButtons,
        minimap: false,
        editor: true,
        ganis: true,
        emoticons: true,
        key_config: true,
        log_window: false,
        ghost_mode: false,
        server_in_title: true,
        gs1_commands: 161,
    },
};

/// 1.33 to 1.39: gen 2, no key byte, no listserver, `.graal` levels only.
const CLASSIC: VersionProfile = VersionProfile {
    protocol: Protocol {
        encryption: EncryptionGen::Gen2,
        listserver: None,
        plo: PLO_133,
        pli: PLI_133,
        sends_language: false,
        ..BASE.protocol
    },
    props: Props {
        own: OWN_133,
        other: OTHER_133,
        npc_max: 34,
        ..BASE.props
    },
    levels: Levels {
        graal: true,
        nw: false,
        gmap: false,
        heights: false,
        cached_links: false,
    },
    life: Health { death: None },
    ui: Ui {
        server_list: ServerList::Direct { default: "www.graalonline.com:14900" },
        chat_recall: false,
        hud: HudArt::Letters,
        ganis: false,
        emoticons: false,
        key_config: false,
        server_in_title: false,
        gs1_commands: 88,
        ..BASE.ui
    },
    ..BASE
};

/// 2.1 and later: the server treats the client as 2.1+ (gani prop 10,
/// `.png` defaults, modtime in `PLO_FILE`, gserver minimap).
const NEW_2_1: VersionProfile = VersionProfile {
    protocol: Protocol {
        plo: PLO_210,
        // 2.1 test builds and later default to port 14922.
        listserver: Some(Listserver { port: 14922, token: None }),
        ..BASE.protocol
    },
    props: Props {
        own: OWN_210,
        other: OTHER_210,
        npc_max: 43,
        bow_prop10: false,
        ..BASE.props
    },
    files: Files {
        modtime: true,
        image_ext: ".png",
        ..BASE.files
    },
    ui: Ui {
        minimap: true,
        log_window: true,
        ghost_mode: true,
        ..BASE.ui
    },
    basis: Basis::Inferred,
    ..BASE
};

/// 2.22 and 2.31: type 5, gen 5, listserver id 7 with `GNW30123`.
const GEN5: VersionProfile = VersionProfile {
    protocol: Protocol {
        client_type: 5,
        encryption: EncryptionGen::Gen5,
        listserver: Some(Listserver { port: 14922, token: Some("GNW30123") }),
        plo: PLO_222,
        pli: PLI_222,
        tampercheck: true,
        tamper: Some(TAMPER_222),
        showimg: Some(SHOWIMG_222),
        udp_off_by_default: true,
        ..NEW_2_1.protocol
    },
    props: Props {
        own: OWN_222,
        other: OTHER_222,
        npc_max: 73,
        ..NEW_2_1.props
    },
    files: Files {
        large_files: true,
        ..NEW_2_1.files
    },
    levels: Levels { heights: true, ..BASE.levels },
    ui: Ui {
        server_list: ServerList::Tabs(3),
        ..NEW_2_1.ui
    },
    basis: Basis::Binary,
    ..NEW_2_1
};

/// Every version Seihai can present as, oldest first.
pub static PROFILES: &[VersionProfile] = &[
    VersionProfile {
        version: ClientVersion::V133,
        label: "1.33",
        token: "GNW20020",
        date: "2000-02-21",
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V134,
        label: "1.34",
        token: "GNW22030",
        date: "2000-03-23",
        props: Props { other: OTHER_134, ..CLASSIC.props },
        ui: Ui {
            gs1_commands: 93,
            hud: HudArt::Letters,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V135,
        label: "1.35",
        token: "GNW13040",
        date: "2000-04-13",
        props: Props { other: OTHER_134, ..CLASSIC.props },
        ui: Ui {
            gs1_commands: 93,
            hud: HudArt::LettersChat,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V136,
        label: "1.36",
        token: "GNW10050",
        date: "2000-05-12",
        props: Props { other: OTHER_134, ..CLASSIC.props },
        ui: Ui {
            gs1_commands: 104,
            hud: HudArt::LettersChat,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V137,
        label: "1.37",
        token: "GNW21050",
        date: "2000-05-23",
        props: Props { other: OTHER_134, ..CLASSIC.props },
        ui: Ui {
            gs1_commands: 106,
            hud: HudArt::LettersChat,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V138,
        label: "1.38",
        token: "GNW22060",
        date: "2000-06-23",
        protocol: Protocol {
            plo: PLO_138,
            pli: PLI_138,
            ..CLASSIC.protocol
        },
        props: Props { other: OTHER_134, ..CLASSIC.props },
        ui: Ui {
            hud: HudArt::BlankButtons,
            emoticons: true,
            key_config: true,
            gs1_commands: 109,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    VersionProfile {
        version: ClientVersion::V139,
        label: "1.39",
        token: "GNW07080",
        date: "2000-08-07",
        protocol: Protocol {
            plo: PLO_139,
            pli: PLI_139,
            ..CLASSIC.protocol
        },
        props: Props {
            own: OWN_139,
            other: OTHER_139,
            ..CLASSIC.props
        },
        ui: Ui {
            chat_recall: true,
            hud: HudArt::BlankButtons,
            emoticons: true,
            key_config: true,
            gs1_commands: 111,
            ..CLASSIC.ui
        },
        ..CLASSIC
    },
    // 1.41: first gen 3 build with a key byte and a listserver client.
    VersionProfile {
        version: ClientVersion::V141,
        label: "1.41",
        token: "GNW13110",
        date: "",
        protocol: Protocol {
            encryption: EncryptionGen::Gen3,
            listserver: Some(Listserver { port: 14923, token: None }),
            plo: PLO_139,
            pli: PLI_141,
            sends_language: false,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_141,
            other: other_upto(36),
            npc_max: 35,
            ..BASE.props
        },
        levels: CLASSIC.levels,
        ui: Ui {
            ganis: false,
            server_in_title: false,
            gs1_commands: 126,
            ..BASE.ui
        },
        ..BASE
    },
    // NewMain 1.42: DirectDraw 7, ganis, 51 server ids.
    VersionProfile {
        version: ClientVersion::V142,
        label: "1.42",
        token: "GNW08011",
        date: "2001-01-10",
        protocol: Protocol {
            plo: PLO_142,
            pli: PLI_142,
            sends_language: false,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_141,
            other: other_upto(36),
            npc_max: 35,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 134,
            ..BASE.ui
        },
        ..BASE
    },
    VersionProfile {
        // Listserver (not implemented): a real 2.00 client's first frame is
        // `GCHAR 0` + `newmain`, with no account (lab, 2026-10-24); the
        // reply format is unobserved. After the list it opens no second
        // listserver connection and logs in to the gserver directly.
        version: ClientVersion::V200,
        label: "2.00",
        token: "GNW24011",
        date: "2001-02-01",
        protocol: Protocol {
            plo: PLO_200,
            pli: PLI_142,
            sends_language: false,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_141,
            other: other_upto(36),
            npc_max: 35,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 142,
            ..BASE.ui
        },
        ..BASE
    },
    VersionProfile {
        version: ClientVersion::V201,
        label: "2.01",
        token: "GNW23021",
        date: "2001-03-08",
        protocol: Protocol {
            plo: PLO_201,
            pli: PLI_201,
            sends_language: false,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_141,
            other: other_upto(36),
            npc_max: 35,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 144,
            ..BASE.ui
        },
        ..BASE
    },
    // 2.02 (base build and rev 1): no LANGUAGE send yet.
    VersionProfile {
        version: ClientVersion::V202,
        label: "2.02",
        token: "GNW15041",
        date: "2001-04-15",
        protocol: Protocol {
            plo: PLO_202,
            pli: PLI_201,
            sends_language: false,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_202,
            other: other_upto(41),
            npc_max: 40,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 146,
            ..BASE.ui
        },
        ..BASE
    },
    // 2.02 rev 2: first build that sends LANGUAGE (37).
    VersionProfile {
        version: ClientVersion::V202R2,
        label: "2.02 rev2",
        token: "GNW20041",
        date: "2001-04-20",
        protocol: Protocol {
            plo: PLO_202,
            pli: PLI_202R2,
            sends_language: true,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_202,
            other: other_upto(41),
            npc_max: 40,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 146,
            ..BASE.ui
        },
        ..BASE
    },
    // 2.03: TRIGGERACTION (38), MOVE (165) and PLO 48.
    VersionProfile {
        version: ClientVersion::V203,
        label: "2.03",
        token: "GNW06051",
        date: "2001-05-07",
        protocol: Protocol {
            plo: PLO_203,
            pli: PLI_203,
            sends_language: true,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_202,
            other: other_upto(41),
            npc_max: 40,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 148,
            ..BASE.ui
        },
        ..BASE
    },
    // 2.04: SHOWIMGNPC (166), PLO 26 dropped, ATTACHNPC (prop 42).
    VersionProfile {
        version: ClientVersion::V204,
        label: "2.04",
        token: "GNW24061",
        date: "2001-06-25",
        protocol: Protocol {
            plo: PLO_204,
            pli: PLI_203,
            sends_language: true,
            ..BASE.protocol
        },
        props: Props {
            own: OWN_204,
            other: other_upto(42),
            npc_max: 40,
            ..BASE.props
        },
        levels: Levels { nw: false, gmap: false, cached_links: false, ..BASE.levels },
        life: Health { death: None },
        ui: Ui {
            server_in_title: false,
            gs1_commands: 157,
            ..BASE.ui
        },
        ..BASE
    },
    // 2.05 rev 2 (the base row): .nw and .gmap levels, server name in the title.
    VersionProfile { date: "2001-08-16", ..BASE },
    VersionProfile {
        version: ClientVersion::V210,
        label: "2.1",
        token: "GNW31101",
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V212,
        label: "2.12",
        token: "GNW01012",
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V213,
        label: "2.13",
        token: "GNW23012",
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V214,
        label: "2.14",
        token: "GNW30042",
        protocol: Protocol { plo: PLO_214, pli: PLI_214, ..NEW_2_1.protocol },
        files: Files { large_files: true, ..NEW_2_1.files },
        ui: Ui { server_list: ServerList::Tabs(2), ..NEW_2_1.ui },
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V215,
        label: "2.15",
        token: "GNW19052",
        protocol: Protocol { plo: PLO_214, pli: PLI_214, ..NEW_2_1.protocol },
        files: Files { large_files: true, ..NEW_2_1.files },
        ui: Ui { server_list: ServerList::Tabs(2), ..NEW_2_1.ui },
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V216,
        label: "2.16",
        token: "GNW12102",
        protocol: Protocol { plo: PLO_214, pli: PLI_214, ..NEW_2_1.protocol },
        props: Props {
            own: OWN_216,
            other: OTHER_222,
            ..NEW_2_1.props
        },
        files: Files { large_files: true, ..NEW_2_1.files },
        ui: Ui { server_list: ServerList::Tabs(2), ..NEW_2_1.ui },
        ..NEW_2_1
    },
    // 2.17: still type 0 and gen 3; adds tampercheck and level heights.
    VersionProfile {
        version: ClientVersion::V217,
        label: "2.17",
        token: "GNW22122",
        basis: Basis::Binary,
        protocol: Protocol {
            plo: PLO_217,
            pli: PLI_217,
            tampercheck: true,
            tamper: Some(TAMPER_217),
            ..NEW_2_1.protocol
        },
        props: Props {
            own: OWN_216,
            other: OTHER_222,
            npc_max: 73,
            ..NEW_2_1.props
        },
        files: Files { large_files: true, ..NEW_2_1.files },
        levels: Levels { heights: true, ..NEW_2_1.levels },
        ui: Ui { server_list: ServerList::Tabs(2), ..NEW_2_1.ui },
        ..NEW_2_1
    },
    VersionProfile {
        version: ClientVersion::V222,
        label: "2.22",
        token: "GNW03014",
        ..GEN5
    },
    // 2.3: editor split out, precise positions. No binary; taken as 2.31.
    VersionProfile {
        version: ClientVersion::V230,
        label: "2.3",
        token: "GNW14015",
        basis: Basis::Inferred,
        // 2.3: no id 24 observation, so none is sent.
        protocol: Protocol {
            plo: PLO_231,
            pli: PLI_231,
            tamper: Some(TAMPER_230),
            showimg: None,
            ..GEN5.protocol
        },
        props: Props {
            own: OWN_231,
            other: OTHER_231,
            npc_max: 77,
            precise_pos: true,
            ..GEN5.props
        },
        files: Files { large_file_size: true, ..GEN5.files },
        ui: Ui { editor: false, ..GEN5.ui },
        ..GEN5
    },
    VersionProfile {
        version: ClientVersion::V231,
        label: "2.31",
        token: "GNW28015",
        protocol: Protocol {
            plo: PLO_231,
            pli: PLI_231,
            tamper: Some(TAMPER_231),
            showimg: Some(SHOWIMG_231),
            ..GEN5.protocol
        },
        props: Props {
            own: OWN_231,
            other: OTHER_231,
            npc_max: 77,
            precise_pos: true,
            ..GEN5.props
        },
        files: Files { large_file_size: true, ..GEN5.files },
        ui: Ui { editor: false, ..GEN5.ui },
        ..GEN5
    },
];

/// The profile row for a version.
pub fn profile(v: ClientVersion) -> &'static VersionProfile {
    PROFILES
        .iter()
        .find(|p| p.version == v)
        .expect("every ClientVersion has a profile row")
}

/// The profile with this GUI label, e.g. "2.05".
pub fn by_label(label: &str) -> Option<&'static VersionProfile> {
    PROFILES.iter().find(|p| p.label == label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_version_has_one_row() {
        for v in ClientVersion::all_versions() {
            assert_eq!(PROFILES.iter().filter(|p| p.version == *v).count(), 1, "{v:?}");
        }
        assert_eq!(PROFILES.len(), ClientVersion::all_versions().len());
    }

    #[test]
    fn tokens_and_labels() {
        for p in PROFILES {
            assert_eq!(p.token.len(), 8, "{}", p.label);
            assert!(p.token.starts_with("GNW"), "{}", p.label);
        }
        let mut labels: Vec<_> = PROFILES.iter().map(|p| p.label).collect();
        labels.dedup();
        assert_eq!(labels.len(), PROFILES.len());
    }

    /// Counts read from the binaries.
    #[test]
    fn server_id_counts() {
        let n = |v| profile(v).protocol.plo.len();
        assert_eq!(n(ClientVersion::V133), 43);
        assert_eq!(n(ClientVersion::V139), 45);
        assert_eq!(n(ClientVersion::V142), 51);
        assert_eq!(n(ClientVersion::V205), 59);
        assert_eq!(n(ClientVersion::V217), 79);
        assert_eq!(n(ClientVersion::V222), 80);
        assert_eq!(n(ClientVersion::V231), 81);
        assert!(!profile(ClientVersion::V205).protocol.plo.contains(26));
        assert!(!profile(ClientVersion::V231).protocol.plo.contains(44));
        assert!(!profile(ClientVersion::V139).protocol.plo.contains(42));
    }

    #[test]
    fn other_prop_sets() {
        let o = profile(ClientVersion::V205).props.other;
        assert_eq!(o.len(), 31);
        for id in [1, 3, 14, 22, 25, 43] {
            assert!(!o.contains(id));
        }
        assert!(profile(ClientVersion::V205).props.own.contains(42));
        assert!(!profile(ClientVersion::V205).props.own.contains(43));
    }

    #[test]
    fn generations() {
        use EncryptionGen::*;
        let g = |v| profile(v).protocol.encryption;
        assert_eq!(g(ClientVersion::V139), Gen2);
        assert_eq!(g(ClientVersion::V141), Gen3);
        assert_eq!(g(ClientVersion::V217), Gen3);
        assert_eq!(g(ClientVersion::V222), Gen5);
        assert_eq!(profile(ClientVersion::V222).protocol.client_type, 5);
        assert_eq!(profile(ClientVersion::V217).protocol.client_type, 0);
    }

    /// GServer's 2.1 line: modtime, gani prop 10 and `.png` defaults.
    #[test]
    fn the_2_1_line() {
        for p in PROFILES {
            let new = p.files.modtime;
            assert_eq!(new, !p.props.bow_prop10, "{}", p.label);
            assert_eq!(new, p.files.image_ext == ".png", "{}", p.label);
        }
        assert!(!profile(ClientVersion::V205).files.modtime);
        assert!(profile(ClientVersion::V210).files.modtime);
    }

    #[test]
    fn classic_sets() {
        let p = |v| profile(v);
        assert_eq!(p(ClientVersion::V133).protocol.pli, IdSet::range(0, 31));
        assert!(!p(ClientVersion::V138).protocol.pli.contains(25));
        assert!(p(ClientVersion::V139).protocol.pli.contains(80));
        assert!(p(ClientVersion::V133).props.other.contains(51));
        assert!(!p(ClientVersion::V133).props.other.contains(34));
        assert!(p(ClientVersion::V134).props.other.contains(34));
        assert!(!p(ClientVersion::V138).props.other.contains(33));
        assert!(p(ClientVersion::V139).props.other.contains(33));
        assert_eq!(p(ClientVersion::V135).ui.hud, HudArt::LettersChat);
        assert_eq!(p(ClientVersion::V138).ui.hud, HudArt::BlankButtons);
    }

    /// A version only sends language if it can send id 37.
    #[test]
    fn language_needs_pli_37() {
        for p in PROFILES {
            assert_eq!(p.protocol.sends_language, p.protocol.pli.contains(37), "{}", p.label);
            if p.protocol.tampercheck {
                assert!(p.protocol.pli.contains(39), "{}", p.label);
            }
            // The tamper-check record exists exactly where the build sends it,
            // and every build that was read from a binary carries its words.
            assert_eq!(p.protocol.tamper.is_some(), p.protocol.tampercheck, "{}", p.label);
            if p.protocol.tampercheck && p.basis == Basis::Binary {
                assert!(p.protocol.tamper.unwrap().words.is_some(), "{}", p.label);
            }
        }
    }

    /// The per-version data from Graal_analysis' version-profile TOMLs
    /// (`profile_toml.tsv`, made by `tools/gen_profile_fixture.py`).
    struct TomlRow {
        label: String,
        token: String,
        inferred: bool,
        plo: IdSet,
        pli: IdSet,
        own_max: u8,
        npc_max: u8,
        other: Option<IdSet>,
        tampercheck: bool,
        list_port: Option<u16>,
    }

    fn parse_ids(s: &str) -> Option<IdSet> {
        if s == "-" {
            return None;
        }
        let mut set = IdSet::EMPTY;
        for part in s.split(',') {
            let (lo, hi) = part.split_once('-').unwrap_or((part, part));
            set = set.with_range(lo.parse().unwrap(), hi.parse().unwrap());
        }
        Some(set)
    }

    fn toml_rows() -> Vec<TomlRow> {
        include_str!("profile_toml.tsv")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| {
                let c: Vec<&str> = l.split('\t').collect();
                assert_eq!(c.len(), 10, "{l}");
                TomlRow {
                    label: c[0].to_string(),
                    token: c[1].to_string(),
                    inferred: c[2] == "inf",
                    plo: parse_ids(c[3]).unwrap(),
                    pli: parse_ids(c[4]).unwrap(),
                    own_max: c[5].parse().unwrap(),
                    npc_max: c[6].parse().unwrap(),
                    other: parse_ids(c[7]),
                    tampercheck: c[8] == "1",
                    list_port: c[9].parse().ok(),
                }
            })
            .collect()
    }

    /// Every row agrees with the version-profile TOMLs on token, basis,
    /// server and client id sets, prop limits, tampercheck and listserver
    /// port. To fix a failure, correct the row, or (if the TOML changed)
    /// regenerate `profile_toml.tsv`.
    #[test]
    fn rows_match_version_profile_toml() {
        let rows = toml_rows();
        assert_eq!(rows.len(), PROFILES.len(), "one TOML row per profile row");
        let mut bad = Vec::new();
        for t in &rows {
            let Some(p) = by_label(&t.label) else {
                bad.push(format!("{}: no profile row", t.label));
                continue;
            };
            let mut check = |name: &str, ok: bool| {
                if !ok {
                    bad.push(format!("{}: {name}", t.label));
                }
            };
            check("token", p.token == t.token);
            check("basis", (p.basis == Basis::Inferred) == t.inferred);
            check("plo", p.protocol.plo == t.plo);
            check("pli", p.protocol.pli == t.pli);
            check("npc_max", p.props.npc_max == t.npc_max);
            check("tampercheck", p.protocol.tampercheck == t.tampercheck);
            check("sends_language == pli has 37", p.protocol.sends_language == t.pli.contains(37));
            check("listserver port", p.protocol.listserver.map(|l| l.port) == t.list_port);
            // 2.16: its TOML entry keeps the 2.15 prop limit but notes that
            // 54-74 arrive in 2.16; the row follows the note.
            if t.label != "2.16" {
                check("own_max", p.props.own.iter().max() == Some(t.own_max));
                if let Some(o) = t.other {
                    check("other props", p.props.other == o);
                }
            }
        }
        assert!(bad.is_empty(), "rows differ from the TOML: {bad:#?}");
    }

    /// The corrections this table was regenerated for.
    #[test]
    fn client_send_sets_after_toml_sync() {
        let pli = |v| profile(v).protocol.pli;
        // 2.00 predates ADJACENTLEVEL and HITOBJECTS (2.01).
        assert!(!pli(ClientVersion::V200).contains(35));
        assert!(!pli(ClientVersion::V200).contains(36));
        assert!(pli(ClientVersion::V201).contains(35));
        // 2.17 sends the 2.17 additions.
        for id in [34, 35, 36, 40, 41, 43, 44, 45] {
            assert!(pli(ClientVersion::V217).contains(id), "2.17 id {id}");
        }
        // 2.22 follows the binary list: no 36/45/81, adds 40/41/43/44.
        for id in [36, 45, 81] {
            assert!(!pli(ClientVersion::V222).contains(id), "2.22 id {id}");
        }
        for id in [40, 41, 43, 44] {
            assert!(pli(ClientVersion::V222).contains(id), "2.22 id {id}");
        }
        // 2.31: PUTNPC (21) gone, ENTERLEVEL (46) new.
        assert!(pli(ClientVersion::V222).contains(21));
        assert!(!pli(ClientVersion::V231).contains(21));
        assert!(pli(ClientVersion::V231).contains(46));
        // A real 2.31 client answers the process-list request with 44.
        assert!(pli(ClientVersion::V231).contains(44));
        assert_eq!(pli(ClientVersion::V230), pli(ClientVersion::V231));
        // 2.17's listserver default is 14922.
        assert_eq!(ClientVersion::V217.listserver_port(), 14922);
        assert_eq!(ClientVersion::V205.listserver_port(), 14921);
        assert_eq!(profile(ClientVersion::V217).props.npc_max, 73);
    }

    /// Cached level links (`PLI_LEVELWARPMOD`): rows with evidence or an
    /// inferred successor of it, never the builds before 2.05, and always
    /// with id 30 in the send set.
    #[test]
    fn cached_links_by_profile() {
        for p in PROFILES {
            if p.levels.cached_links {
                assert!(p.protocol.pli.contains(crate::net::PLI_LEVELWARPMOD), "{}", p.label);
                assert!(p.levels.nw, "{}: cached levels need .nw support", p.label);
            }
        }
        for v in [ClientVersion::V205, ClientVersion::V217, ClientVersion::V222, ClientVersion::V231] {
            assert!(v.sends_cached_links(), "{}", v.label());
        }
        for v in [ClientVersion::V133, ClientVersion::V139, ClientVersion::V200, ClientVersion::V204] {
            assert!(!v.sends_cached_links(), "{}", v.label());
        }
    }

    /// 2.02 rev 2, 2.03 and 2.04 each differ from the build before.
    #[test]
    fn the_2_0x_builds() {
        let p = |v| profile(v);
        let (a, b, c, d) = (
            p(ClientVersion::V202),
            p(ClientVersion::V202R2),
            p(ClientVersion::V203),
            p(ClientVersion::V204),
        );
        assert!(!a.protocol.sends_language && b.protocol.sends_language);
        assert!(!b.protocol.pli.contains(38) && c.protocol.pli.contains(38));
        assert!(!b.protocol.plo.contains(48) && c.protocol.plo.contains(48));
        assert!(c.protocol.plo.contains(26) && !d.protocol.plo.contains(26));
        assert!(d.protocol.plo.contains(166) && !c.protocol.plo.contains(166));
        assert!(d.props.own.contains(42) && !c.props.own.contains(42));
        assert_eq!((a.token, b.token, c.token, d.token), ("GNW15041", "GNW20041", "GNW06051", "GNW24061"));
        // 2.05 is the first build with .nw levels and the server name in the title.
        assert!(!d.levels.nw && p(ClientVersion::V205).levels.nw);
        assert_eq!(b.window_title(Some("X")), "Graal 2.02 rev2");
        assert_eq!(by_label("2.02 rev2").unwrap().version, ClientVersion::V202R2);
    }

    #[test]
    fn ui_switches() {
        assert!(matches!(profile(ClientVersion::V139).ui.server_list, ServerList::Direct { .. }));
        assert!(profile(ClientVersion::V139).ui.chat_recall);
        assert!(!profile(ClientVersion::V138).ui.chat_recall);
        assert!(!profile(ClientVersion::V205).ui.minimap);
        assert!(profile(ClientVersion::V217).ui.minimap);
        assert!(!profile(ClientVersion::V231).ui.editor);
        assert_eq!(profile(ClientVersion::V139).level_extensions(), vec![".graal"]);
        assert_eq!(
            profile(ClientVersion::V205).window_title(Some("Classic")),
            "Graal 2.05 - Classic"
        );
        assert_eq!(profile(ClientVersion::V139).window_title(Some("Classic")), "Graal 1.39");
    }
}
