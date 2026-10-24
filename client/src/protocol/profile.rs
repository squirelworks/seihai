//! Per-version client profiles.
//!
//! One table row per client version Seihai can present as. A row says how
//! that version logs in and frames its traffic, which server packets and
//! props it understands, which file and level features it has, and which
//! parts of the client it shows. The rest of Seihai reads these fields
//! instead of matching on versions, so picking a version in the GUI makes
//! the whole client behave as that build.
//!
//! The rows are not written here: [`PROFILES`] is the resolved version chain
//! in [`super::chain`] (each build is a small delta against the one before).
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

use super::login::LoginInfo;
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
    /// What the build sends once the server's login burst is in (image
    /// checks, process list, login props); `Some` only for the builds
    /// observed doing it (2.17, 2.22 and 2.31). See [`super::login`].
    pub login: Option<LoginInfo>,
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
    /// A downloaded `.gmap`, and a map-named `.txt`/`.gif`/`.png`/`.mng`,
    /// is saved in `maps/`. Read in 2.17.1, 2.22 and 2.31; 2.05 has no such
    /// rule (every download goes under `levels/`). 2.1 to 2.16 are inferred.
    /// See `filecache::download_dir`.
    pub download_maps: bool,
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
    /// A link fires when the walking direction's probe box (see
    /// `offline::LINK_BOX`) overlaps its rectangle, as on the real 2.22 and
    /// 2.31 clients (lab, 2026-10-25). Without it the legacy foot-point rule
    /// stays: builds nobody measured.
    pub link_boxes: bool,
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
    /// The level-entry save (the point a respawn returns to) skips an entry
    /// whose level is in the same *name group* as the level left: the name
    /// without its folder, lower-cased, up to its last `§` or `$`
    /// (delimiter included; names with neither all share the empty group).
    /// Only the 2.05 builds have this test; 2.1 test 2 and everything after
    /// it keep just the gmap rule. See `documentation/health-death.md`.
    pub entry_name_group: bool,
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
    entry_name_group: true,
};

/// 2.1 test 2 and later: the same death sequence, but the level-entry save
/// no longer has the name-group test (the delimiter set is not in those
/// binaries; the 2.17.1, 2.22 and 2.31 entry code was read, the 2.1 tests'
/// was inferred from the missing constant).
pub const DEATH_21: DeathRules = DeathRules { entry_name_group: false, ..DEATH_205 };

/// How a build's client judges and sends hits from other players (the
/// victim's side of combat). Sources: `decomp-findings-2026-10-24/
/// 04-sword-hitbox.md`, `01-death-respawn.md`, `constants-205.tsv` (2.05
/// rev 2, derived), and Graal_analysis `protocol/packet-layouts.md` "Hits".
/// Later builds take the same row as an inference, the builds before 2.05
/// have none because their handler was not traced. Distances are in half
/// tiles, the wire scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRules {
    /// The attacker's sprite number (SPRITE prop 17 is `dir + 4 * S`) is a
    /// sword swing from `swing_sprite_first` to `swing_sprite_last`
    /// (animation steps 9 to 13). Outside it a sword cannot hit.
    pub swing_sprite_first: u8,
    pub swing_sprite_last: u8,
    /// Seihai's own swing, one animation step each, in ticks of 50 ms.
    /// **Placeholder, not a client value**: the real frame times come from
    /// the sword gani file, which Seihai does not play yet.
    pub swing_step_ticks: u16,
    /// The victim must be `reach_min_half` to `reach_max_half` ahead of the
    /// attacker in the direction the attacker faces (0 to 4 tiles,
    /// inclusive) ...
    pub reach_min_half: u8,
    pub reach_max_half: u8,
    /// ... and strictly less than `lateral_below_half` to the side (under
    /// 2 tiles).
    pub lateral_below_half: u8,
    /// A victim with exactly this alignment (AP) cannot be hit by a sword.
    pub immune_ap: u8,
    /// Hurt mode (mode 3) lasts this many ticks; a hurt player cannot be
    /// hurt again until it ends (40 ticks, 2 s).
    pub hurt_ticks: u16,
    /// Sprite number others see while the player is hurt (S = 32).
    pub hurt_sprite: u8,
    /// `PLO_HURTPLAYER` (40) applies only when its attacker is within this
    /// many half tiles of the victim (10 tiles).
    pub hurtplayer_range_half: u8,
    /// A hit by id 40 or 46 takes at most this many half hearts (1 heart).
    pub packet_hit_cap_half: u8,
    /// A sword hit takes the attacker's sword power in half hearts, with the
    /// power limited to plus or minus this (20). The wire value is power + 30
    /// (10 to 50), and 0 to 4 for the default swords 1 to 4.
    pub sword_power_limit: u8,
    /// A negative sword power heals the victim, but only at this AP or more
    /// (40). Below it the hit hurts (hurt mode) without changing hearts.
    pub heal_min_ap: u8,
    /// An attacker whose STATUS (18) has any of these bits (0x22: hidden and
    /// one untraced) cannot land a sword hit.
    pub attacker_blocked_status: u8,
    /// The hit judge handles one swing or hit per attacker per value of the
    /// client's second-of-day clock; later ones in the same second are
    /// dropped.
    pub judge_throttle: bool,
}

/// The 2.05 rev 2 values.
pub const COMBAT_205: CombatRules = CombatRules {
    swing_sprite_first: 6,
    swing_sprite_last: 10,
    swing_step_ticks: 2,
    reach_min_half: 0,
    reach_max_half: 8,
    lateral_below_half: 4,
    immune_ap: 100,
    hurt_ticks: 40,
    hurt_sprite: 32,
    hurtplayer_range_half: 20,
    packet_hit_cap_half: 2,
    sword_power_limit: 20,
    heal_min_ap: 40,
    attacker_blocked_status: 0x22,
    judge_throttle: true,
};

/// Health, death and combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    /// The client-side death sequence, where it was traced.
    pub death: Option<DeathRules>,
    /// How hits from other players are judged, where it was traced.
    pub combat: Option<CombatRules>,
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

// Tamper-check constants (CRC words A and B), from the 2026-10-24 decompile
// reading; the encoded bytes match the lab captures of the real clients.
pub(super) const TAMPER_217: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0xab63_7cbe, b: 0x3e48_dc0c }),
    resend: TamperResend::Never,
};
pub(super) const TAMPER_222: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0x5dcb_77f6, b: 0xbc73_6311 }),
    resend: TamperResend::OnLevelChange,
};
pub(super) const TAMPER_231: TamperInfo = TamperInfo {
    words: Some(TamperWords { a: 0x9ecd_4bc0, b: 0xecea_65e2 }),
    resend: TamperResend::Timer,
};
/// 2.3: no binary, so no constant; it is taken to re-send like 2.31.
pub(super) const TAMPER_230: TamperInfo = TamperInfo { words: None, resend: TamperResend::Timer };

// Id 24 (lab, real clients, 2026-10-24): 2.22 about every 5.1 s while another
// player is in the level, 2.31 every 5.13 s always.
pub(super) const SHOWIMG_222: ShowImgInfo = ShowImgInfo { period: Duration::from_millis(5100), needs_others: true };
pub(super) const SHOWIMG_231: ShowImgInfo = ShowImgInfo { period: Duration::from_millis(5130), needs_others: false };

/// Every version Seihai can present as, oldest first. Resolved from
/// [`super::chain::CHAIN`]: each row is the row before it plus that build's
/// delta, so a new version is a step in the chain, not a new row here.
pub static PROFILES: &[VersionProfile] = &super::chain::RESOLVED;

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
