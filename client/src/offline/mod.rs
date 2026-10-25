//! Offline play support.
//!
//! Allows loading and simulating levels, NPCs, and a local player
//! without a network connection. Behavior is provisional and will
//! be refined against the original client’s offline/tutorial paths.

use crate::assets::{GaniPlayer, Level, Link, Object2x2, ObjectTables, TileKind, TileTypes};
use crate::sound::SoundEvent;
use crate::dev::DevMode;
use crate::gmap::{Crossing, GmapWorld, Resolved};
use crate::life::{respawn_half_hearts, Life, LifeEvent};
use crate::protocol::{BoardModify, PlayerUpdate};
use std::collections::{HashMap, HashSet};

/// Facing, in Graal order (ganis and the 2.05 wire): 0 up, 1 left,
/// 2 down, 3 right.
pub const DIR_NORTH: u8 = 0;
pub const DIR_WEST: u8 = 1;
pub const DIR_SOUTH: u8 = 2;
pub const DIR_EAST: u8 = 3;

/// Where the feet are, from the sprite's top-left, in tiles. This is the
/// one point the original client tests for walls, water and links.
pub const FOOT_DX: f32 = 2.0;
pub const FOOT_DY: f32 = 2.75;

/// Where the 2.05 client tests for walls while walking, from the sprite's
/// top-left, in tiles, per facing (0 up, 1 left, 2 down, 3 right). The body is
/// a box 2 tiles wide and 2 tall (x + 0.5 to x + 2.5, y + 1 to y + 3); a step
/// is refused if any of the four points on its leading edge, taken at the
/// position the step would reach, is a wall. The points sit 0.025 tile either
/// side of the middle and of the two corners, so the middle of the edge and
/// each corner are tested. The client reads six points per facing; the last
/// two only steer its 0.1-tile corner nudge, which is not implemented (see
/// `documentation/walking-collision.md`).
pub const WALK_PROBES: [[(f32, f32); 4]; 4] = [
    [(0.525, 1.0), (1.475, 1.0), (1.525, 1.0), (2.475, 1.0)],
    [(0.5, 1.025), (0.5, 1.975), (0.5, 2.025), (0.5, 2.975)],
    [(0.525, 3.0), (1.475, 3.0), (1.525, 3.0), (2.475, 3.0)],
    [(2.5, 1.025), (2.5, 1.975), (2.5, 2.025), (2.5, 2.975)],
];

/// Walking and swimming step times. A step is half a tile; swimming is
/// half as fast.
pub const WALK_STEP_MS: u64 = 50;
pub const SWIM_STEP_MS: u64 = 100;

/// When the step just taken counts from, so the pace holds on a loop that
/// ticks every 16 ms or so. A step is due `interval` after the last one; if
/// the loop only noticed it late (a 50 ms step seen after 64 ms), the next
/// step still counts from the time it was due, not from when it was taken. That
/// keeps the average at one step per `interval` (20 per second, 10 tiles a
/// second, as the real clients walk). After a pause of two intervals or more
/// there is nothing to catch up on, and the clock starts from `now`.
pub fn next_step_clock(last: std::time::Instant, now: std::time::Instant, interval: std::time::Duration) -> std::time::Instant {
    if now.saturating_duration_since(last) >= interval * 2 {
        now
    } else {
        last + interval
    }
}

/// The box, as offsets from the player's position in tiles, that the real
/// 2.22 and 2.31 clients test against level links, by walking direction
/// (`DIR_NORTH`, `DIR_WEST`, `DIR_SOUTH`, `DIR_EAST`): `(x0, y0, x1, y1)`.
///
/// Measured in the private lab on 2026-10-25, with the player put on half-tile
/// positions and walking into a rectangle link and the edge links; the only
/// thing the captures show is *where on the half-tile grid the first warp
/// goes out*, so each edge is a value inside the range the captures allow
/// (see `documentation/link-geometry.md`):
///
/// * a left or right step fires once the leading edge, `x + 0.25` or
///   `x + 2.75`, is inside the link; the box is as tall as the feet strip
///   (`y + 2` to `y + 3`);
/// * an up or down step fires once the leading edge, `y + 0.75` or
///   `y + 3.25`, is inside the link; the box is one tile wide, centred on
///   `x + 1.5`.
pub const LINK_BOX: [(f32, f32, f32, f32); 4] = [
    (1.0, 0.75, 2.0, 1.25), // north
    (0.25, 2.0, 0.75, 3.0), // west
    (1.0, 2.75, 2.0, 3.25), // south
    (2.25, 2.0, 2.75, 3.0), // east
];

/// A locally simulated game session (no server).
#[derive(Debug)]
pub struct OfflineSession {
    pub current_level: Option<Level>,
    pub tiletypes: Option<TileTypes>,
    /// The cut and lift tables of `arrays.dat`, if one was loaded.
    pub object_tables: Option<ObjectTables>,
    /// What the player is carrying after a lift: the object's tiles.
    pub carried: Option<Carried>,
    /// Objects in flight, break effects playing, and thrown things that stay
    /// (see [`crate::throwing`]).
    pub thrown: Vec<crate::throwing::Thrown>,
    pub leaps: Vec<crate::throwing::Leap>,
    pub grounded: Vec<crate::throwing::Grounded>,
    /// Landings since the last [`Self::take_landings`].
    landings: Vec<Landing>,
    /// Milliseconds carried over between flight ticks.
    throw_ms: u64,
    pub player_x: f32,
    pub player_y: f32,
    pub player_dir: u8,
    /// True when the player is standing on a water tile (type == 11).
    pub in_water: bool,
    /// Optional walk animation.
    pub walk_gani: Option<GaniPlayer>,
    /// Optional swim animation, stepped instead of the walk one while the
    /// feet are in water.
    pub swim_gani: Option<GaniPlayer>,
    /// The sword, hurt and sit animations (see [`crate::pose`]).
    pub poses: crate::pose::PoseGanis,
    /// GANIs for animated NPCs, by lower-case name (see [`crate::npcart`]).
    pub npc_ganis: HashMap<String, crate::assets::Gani>,
    /// The feet are on a chair tile (type 3): the sit pose is drawn while the
    /// player is not walking. Set by [`Self::refresh_water`].
    pub sitting: bool,
    /// The last step moved the player and no stop has come since: the walk
    /// animation is drawn instead of idle. Cleared by [`Self::stop_walking`].
    pub walking: bool,
    /// Sounds from GANI frames entered since the last
    /// [`Self::take_sound_events`].
    pub sound_events: Vec<SoundEvent>,
    /// Player health (for `hurt`).
    pub player_health: i32,
    /// Last message from `say` / `message`.
    pub last_message: Option<String>,
    /// Flags set by `set` / `unset`.
    pub flags: HashMap<String, bool>,
    /// String variables (`setstring` / `addstring`).
    pub strings: HashMap<String, String>,
    /// Player properties (`setplayerprop`).
    pub props: HashMap<String, String>,
    /// Pending link destination if the player walked into a LINK area.
    pub pending_link: Option<(String, f32, f32)>,
    /// Rupees (gralats).
    pub rupees: i32,
    pub bombs: i32,
    pub darts: i32,
    /// Sword power 1–4 and shield power 1–3.
    pub sword_power: u8,
    pub shield_power: u8,
    /// Whether the glove is held, and its power.
    pub glove_power: u8,
    /// Heart containers, in half hearts as Graal counts them.
    pub max_hearts: i32,
    /// Chests opened in the current level, by tile position.
    pub opened_chests: HashSet<(i32, i32)>,
    /// Sign or chest text being read. The game waits until it is closed.
    pub reading: Option<String>,
    /// Developer-only switches (wallhack). Off at start, never saved.
    pub dev: DevMode,
    /// Set from the moment a level warp is asked for until the new level
    /// arrives. Nothing walks and no link fires meanwhile.
    pub warp_pending: Option<String>,
    /// On a gmap: the grid, the player's cell and the neighbouring levels.
    /// `player_x`/`player_y` stay local to that cell. Walking past an edge
    /// moves the player into the next cell (see [`Self::take_gmap_crossing`]).
    pub gmap: Option<GmapWorld>,
    /// False from an arrival until the player has moved: a link cannot
    /// fire on the spot the player was put on.
    armed: bool,
    /// Test links with [`LINK_BOX`] (the profile's `levels.link_boxes`)
    /// instead of the legacy foot point.
    pub link_boxes: bool,
    /// Tile the player arrived on. Links that contain it stay off until
    /// the player steps out of them.
    arrival_tile: Option<(i32, i32)>,
    /// Swimming state last reported to the server.
    swim_sent: bool,
    /// Counts levels loaded with [`OfflineSession::load_level`], so a
    /// caller can tell when its per-level state (scripts) is stale.
    pub level_serial: u64,
    /// The death sequence (timer, input lock, dead bit). It has no rules
    /// until the game loop gives it its version's (see
    /// [`Life::set_rules`]), so a bare session never dies.
    pub life: Life,
    /// The sword swing and hurt mode (see [`crate::combat`]).
    pub combat: crate::combat::CombatState,
    /// Alignment points (AP, 0–100): decide the hearts after a respawn.
    /// Starts at 50, the GServer default account; the server's
    /// `PLPROP_ALIGNMENT` replaces it online.
    pub alignment: u8,
    /// The level entry point a respawn returns to; see [`Self::note_entry`].
    pub entry: Option<EntryPoint>,
    /// The level entered last, whether or not it was saved as the entry
    /// point: the "level left" of the next [`Self::note_entry`].
    entered: Option<String>,
    /// The current level said `noplayerkilling`: sword and packet hits
    /// from other players do nothing. Cleared when a level loads.
    pub no_pk: bool,
    /// Per attacker id: the second of the day it was last judged (see
    /// `CombatRules::judge_throttle`).
    pub judged_at: std::collections::HashMap<u16, u32>,
    /// A hit's knockback moved the player since the last [`OfflineSession::take_knock_moved`].
    pub(crate) knock_moved: bool,
    /// File name of the level loaded last with [`Self::load_level`].
    loaded_file: Option<String>,
}

/// Where the player last entered a level: the point a respawn goes back to.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryPoint {
    pub level: String,
    pub x: f32,
    pub y: f32,
    pub dir: u8,
}

/// Where a respawn put the player.
#[derive(Debug, Clone, PartialEq)]
pub enum RespawnTo {
    /// The saved level entry point. The caller loads the level when it is
    /// not the current one and, online, re-sends the level at `(x, y)`.
    Saved(EntryPoint),
    /// Nothing saved and online: the profile's default position in the
    /// current level. Online, the current level is re-sent there.
    Default { x: f32, y: f32 },
    /// Nothing saved and offline: the player stays where they are. What the
    /// real client does here is not documented (see `health-death.md`).
    InPlace,
}

/// An object the player lifted: which lift entry it was and its tiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carried {
    pub kind: usize,
    pub tiles: Object2x2,
}

/// A thrown object coming down.
#[derive(Debug, Clone, PartialEq)]
pub struct Landing {
    /// Lift-table entry of the object.
    pub kind: usize,
    /// Break effect started (a row of the break table), `None` for an
    /// object that stays on the ground.
    pub effect: Option<usize>,
    pub x: f32,
    pub y: f32,
}

/// What an interact press did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interaction {
    /// An object (bush, rock, ...) was lifted; `kind` is its entry in the
    /// lift table.
    Lifted { kind: usize },
    /// The carried object was thrown; `kind` is its lift-table entry.
    Thrown { kind: usize },
    /// A closed chest was opened at `(x, y)`; `got` describes the item.
    Chest { x: i32, y: i32, item: String, got: String },
    /// A sign is being read.
    Sign(String),
    Nothing,
}

/// The message that tells a 2.1+ server the player started or stopped
/// swimming: `PLI_PLAYERPROPS` (2) with prop 10 (`PLPROP_GANI`) as a
/// GSTRING, `swim` or `idle`.
///
/// GServer keeps no swimming status. It stores the gani name and relays
/// it to the other players in the level, so that is how they see the
/// swim animation. Clients before 2.1 read prop 10 as the bow slot, so
/// nothing is sent for them.
pub fn swim_message(swimming: bool) -> Vec<u8> {
    let name: &[u8] = if swimming { b"swim" } else { b"idle" };
    let mut m = vec![2 + 0x20, 10 + 0x20, name.len() as u8 + 0x20];
    m.extend_from_slice(name);
    m.push(b'\n');
    m
}

/// The name group of a level name (2.05's save-entry rule): the name without
/// its folder, lower-cased, up to and including its last `§` or `$`; empty
/// when it has neither.
pub fn name_group(level: &str) -> String {
    let base = level.rsplit(['/', '\\', ':']).next().unwrap_or(level).to_lowercase();
    match base.rfind(['§', '$']) {
        Some(i) => base[..i + base[i..].chars().next().map_or(1, char::len_utf8)].to_string(),
        None => String::new(),
    }
}

impl OfflineSession {
    pub fn new() -> Self {
        Self {
            current_level: None,
            tiletypes: None,
            object_tables: None,
            carried: None,
            thrown: Vec::new(),
            leaps: Vec::new(),
            grounded: Vec::new(),
            landings: Vec::new(),
            throw_ms: 0,
            player_x: 30.0,
            player_y: 30.0,
            player_dir: DIR_SOUTH,
            in_water: false,
            walk_gani: None,
            swim_gani: None,
            poses: crate::pose::PoseGanis::default(),
            sitting: false,
            walking: false,
            npc_ganis: HashMap::new(),
            sound_events: Vec::new(),
            player_health: 100,
            last_message: None,
            flags: HashMap::new(),
            strings: HashMap::new(),
            props: HashMap::new(),
            pending_link: None,
            rupees: 0,
            bombs: 10,
            darts: 10,
            sword_power: 1,
            shield_power: 1,
            glove_power: 0,
            max_hearts: 6,
            opened_chests: HashSet::new(),
            reading: None,
            dev: DevMode::default(),
            warp_pending: None,
            gmap: None,
            armed: true,
            link_boxes: false,
            arrival_tile: None,
            swim_sent: false,
            level_serial: 0,
            life: Life::default(),
            combat: crate::combat::CombatState::default(),
            alignment: 50,
            entry: None,
            entered: None,
            no_pk: false,
            judged_at: HashMap::new(),
            knock_moved: false,
            loaded_file: None,
        }
    }

    /// Current hearts in half hearts, from the health percentage and
    /// `max_hearts`. 0 only when the health is 0.
    pub fn hearts_half(&self) -> i32 {
        if self.player_health <= 0 {
            return 0;
        }
        ((self.player_health as f64 * self.max_hearts as f64 / 100.0).round() as i32).clamp(1, self.max_hearts.max(1))
    }

    /// Set the hearts in half hearts (clamped to the maximum). A positive
    /// value never rounds down to 0 health, so rounding cannot kill.
    pub fn set_hearts_half(&mut self, half: i32) {
        let max = self.max_hearts.max(1);
        let half = half.clamp(0, max);
        self.player_health = if half == 0 {
            0
        } else {
            ((half as f64 / max as f64 * 100.0).round() as i32).clamp(1, 100)
        };
    }

    /// New maximum in half hearts. Current hearts stay as they are, up to
    /// the new maximum.
    pub fn set_max_hearts_half(&mut self, max: i32) {
        let cur = self.hearts_half();
        self.max_hearts = max.clamp(0, 80);
        self.set_hearts_half(cur);
    }

    /// Hearts and AP the server gave: MAXPOWER (heart containers), CURPOWER
    /// (hearts × 2, so half hearts) and ALIGNMENT. Max first, so a CURPOWER
    /// in the same message is read against the new maximum.
    pub fn apply_server_power(&mut self, max_power: Option<u8>, cur_power: Option<u8>, alignment: Option<u8>) {
        if let Some(m) = max_power {
            self.set_max_hearts_half(i32::from(m) * 2);
        }
        if let Some(c) = cur_power {
            self.set_hearts_half(i32::from(c));
        }
        if let Some(a) = alignment {
            self.alignment = a.min(100);
        }
    }

    /// Take damage in half hearts from `attacker` (a player id) or from
    /// nothing known. A player who is dead cannot be hurt. Returns false
    /// when the hit did nothing. The death itself starts at the next
    /// [`Self::tick_life`].
    pub fn damage_half(&mut self, half: i32, attacker: Option<u16>) -> bool {
        if self.life.is_dead() || half <= 0 {
            return false;
        }
        let left = self.hearts_half() - half;
        self.set_hearts_half(left.max(0));
        self.life.note_attacker(attacker);
        true
    }

    /// Advance the death sequence by `ms` and return what happened. At the
    /// start of a death the player faces down. Controls stay locked until
    /// the timer is past the profile's limit (see [`Self::controls_locked`]).
    pub fn tick_life(&mut self, ms: u64) -> Vec<LifeEvent> {
        let zero = self.player_health <= 0 || self.hearts_half() <= 0;
        self.life.tick(ms, zero);
        let events = self.life.take_events();
        if events.iter().any(|e| matches!(e, LifeEvent::Died { .. })) {
            self.player_dir = DIR_SOUTH;
            self.reading = None;
        }
        events
    }

    /// The player cannot act: dead and the timer not yet past the lock.
    pub fn controls_locked(&self) -> bool {
        self.life.input_locked()
    }

    /// True when the level loaded last is the one named `name` (a file name,
    /// with or without `.nw`, any case).
    pub fn is_loaded(&self, name: &str) -> bool {
        let base = |n: &str| n.strip_suffix(".nw").or_else(|| n.strip_suffix(".NW")).unwrap_or(n).to_ascii_lowercase();
        self.loaded_file.as_deref().is_some_and(|f| base(f) == base(name))
    }

    /// A level was entered (or the session started in it): maybe remember
    /// where the player stands as the entry point a respawn returns to.
    /// The client saves when no level was left (the first entry), or when
    /// exactly one of the level left and the new level is in the current
    /// gmap's level list at an index above 0. (The client multiplies the
    /// two list indices, -1 for "not in the list", and saves on a negative
    /// product, so index 0 counts as "same side": a quirk, kept.) Otherwise
    /// builds with [`DeathRules::entry_name_group`] (2.05) save only when
    /// the two names are in different name groups ([`name_group`]); later
    /// builds do not save. A session without death rules saves every entry.
    pub fn note_entry(&mut self, level: &str) {
        let old = self.entered.replace(level.to_string());
        let save = match self.life.rules().copied() {
            None => true,
            Some(rules) => match old.as_deref().filter(|o| !o.is_empty()) {
                None => true,
                Some(old) => {
                    let list = |n: &str| match &self.gmap {
                        Some(g) => g.map.list_index(n).map_or(-1, |i| i as i64),
                        None => -1,
                    };
                    list(level) * list(old) < 0 || (rules.entry_name_group && name_group(old) != name_group(level))
                }
            },
        };
        if save {
            self.entry = Some(EntryPoint { level: level.to_string(), x: self.player_x, y: self.player_y, dir: self.player_dir });
        }
    }

    /// A movement or action key while dead: respawn if the lock is over.
    /// Hearts follow the profile's table by AP, the position is the saved
    /// entry point (or the default), and the death sequence ends. `online`
    /// picks the default when nothing was saved. `None` when the player
    /// could not respawn.
    pub fn respawn(&mut self, online: bool) -> Option<RespawnTo> {
        let rules = *self.life.rules()?;
        if !self.life.respawn() {
            return None;
        }
        let hearts = respawn_half_hearts(&rules, self.alignment, self.max_hearts);
        self.set_hearts_half(hearts.max(1));
        let to = match self.entry.clone() {
            Some(e) => {
                self.player_x = e.x;
                self.player_y = e.y;
                self.player_dir = e.dir;
                RespawnTo::Saved(e)
            }
            None if online => {
                let (x, y) = (f32::from(rules.default_x2) / 2.0, f32::from(rules.default_y2) / 2.0);
                self.player_x = x;
                self.player_y = y;
                self.player_dir = DIR_SOUTH;
                RespawnTo::Default { x, y }
            }
            None => RespawnTo::InPlace,
        };
        self.refresh_water();
        Some(to)
    }

    /// The feet, in tiles: the point used for links, water and scripts
    /// (walking tests the body box, see [`WALK_PROBES`]).
    pub fn foot(&self) -> (f32, f32) {
        (self.player_x + FOOT_DX, self.player_y + FOOT_DY)
    }

    /// The point the A key probes, in tiles: one tile beyond the middle of
    /// the body box's leading edge (north `(x + 1.5, y)`, west `(x - 0.5,
    /// y + 2)`, south `(x + 1.5, y + 4)`, east `(x + 3.5, y + 2)`).
    ///
    /// The 2.05 client reads this table for lifting and for the sword's cut.
    /// Chests and signs use it here too; that is the one assumption, since
    /// the point the client uses for them was not read. It fits the walk: a
    /// player stopped by a chest or sign has exactly this point inside it.
    pub fn facing_point(&self) -> (f32, f32) {
        let (x, y) = (self.player_x, self.player_y);
        match self.player_dir {
            DIR_NORTH => (x + 1.5, y),
            DIR_WEST => (x - 0.5, y + 2.0),
            DIR_EAST => (x + 3.5, y + 2.0),
            _ => (x + 1.5, y + 4.0),
        }
    }

    /// Interact with what the player faces: open a closed chest (2×2
    /// tiles), or read a sign (2×1 tiles). A chest is tried first.
    pub fn interact(&mut self) -> Interaction {
        if self.life.is_dead() {
            return Interaction::Nothing;
        }
        // The A key tries to lift first, then reads a sign or opens a chest.
        // With something in the hands it throws it.
        if let Some(c) = self.throw_carried() {
            return Interaction::Thrown { kind: c };
        }
        if let Some(kind) = self.try_lift() {
            return Interaction::Lifted { kind };
        }
        let Some(level) = &self.current_level else {
            return Interaction::Nothing;
        };
        let (px, py) = self.facing_point();
        let inside = |x: i32, y: i32, w: f32, h: f32| {
            px >= x as f32 && px < x as f32 + w && py >= y as f32 && py < y as f32 + h
        };
        // The real 2.22 client opens a chest only from below, facing up (lab
        // capture, `documentation/chest-drawing.md`); a player stopped against
        // a chest from the west, east or north side and pressing A sends
        // nothing.
        let chest = level
            .chests
            .iter()
            .find(|c| {
                self.player_dir == DIR_NORTH
                    && inside(c.x, c.y, 2.0, 2.0)
                    && !c.opened
                    && !self.opened_chests.contains(&(c.x, c.y))
            })
            .cloned();
        if let Some(c) = chest {
            // A chest's sign index points into the level's signs.
            let note = usize::try_from(c.sign_index)
                .ok()
                .filter(|_| c.sign_index >= 0)
                .and_then(|i| level.signs.get(i))
                .map(|s| s.text.clone());
            self.opened_chests.insert((c.x, c.y));
            let got = self.give_item(&c.item);
            self.reading = note;
            return Interaction::Chest { x: c.x, y: c.y, item: c.item, got };
        }
        if let Some(sign) = level.signs.iter().find(|s| inside(s.x, s.y, 2.0, 1.0)) {
            let text = sign.text.clone();
            self.reading = Some(text.clone());
            return Interaction::Sign(text);
        }
        Interaction::Nothing
    }

    /// Apply a chest item. Returns a short description.
    ///
    /// Item names are the CHEST record's (`greenrupee`, `fullheart`, ...).
    pub fn give_item(&mut self, item: &str) -> String {
        let item = item.to_ascii_lowercase();
        let mut gain = |what: &mut i32, n: i32, label: &str| {
            *what += n;
            format!("{n} {label}")
        };
        match item.as_str() {
            "greenrupee" => gain(&mut self.rupees, 1, "rupee"),
            "bluerupee" => gain(&mut self.rupees, 5, "rupees"),
            "redrupee" => gain(&mut self.rupees, 30, "rupees"),
            "goldrupee" => gain(&mut self.rupees, 100, "rupees"),
            "bombs" => gain(&mut self.bombs, 5, "bombs"),
            "darts" => gain(&mut self.darts, 5, "arrows"),
            "heart" => {
                self.player_health = 100;
                "a heart".into()
            }
            "fullheart" => {
                self.max_hearts = (self.max_hearts + 2).min(40);
                self.player_health = 100;
                "a heart container".into()
            }
            "spinattack" => {
                self.flags.insert("spinattack".into(), true);
                "the spin attack".into()
            }
            "glove1" | "glove2" => {
                self.glove_power = self.glove_power.max(if item == "glove1" { 1 } else { 2 });
                "a glove".into()
            }
            "sword" | "battleaxe" | "lizardsword" | "goldensword" => {
                let p = match item.as_str() {
                    "sword" => 1,
                    "battleaxe" => 2,
                    "lizardsword" => 3,
                    _ => 4,
                };
                self.sword_power = self.sword_power.max(p);
                format!("sword {p}")
            }
            "shield" | "mirrorshield" | "lizardshield" => {
                let p = match item.as_str() {
                    "shield" => 1,
                    "mirrorshield" => 2,
                    _ => 3,
                };
                self.shield_power = self.shield_power.max(p);
                format!("shield {p}")
            }
            other => {
                self.flags.insert(format!("item:{other}"), true);
                other.to_string()
            }
        }
    }

    /// Load a .nw level from disk.
    pub fn load_level<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        self.loaded_file = path.as_ref().file_name().map(|n| n.to_string_lossy().into_owned());
        let level = Level::load(path)?;
        self.current_level = Some(level);
        self.opened_chests.clear();
        self.reading = None;
        self.no_pk = false;
        self.clear_throws();
        self.level_serial += 1;
        Ok(())
    }

    /// Load the 4096-byte collision table from a text tiledefs file.
    pub fn load_tiletypes<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        self.tiletypes = Some(TileTypes::load(path)?);
        self.refresh_water();
        Ok(())
    }

    /// Load type-0 collision from `arrays.dat` (first array = nowall list).
    /// Tiles in the list get type 0 (passable); everything else gets type 22.
    pub fn load_tiletypes_arrays<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        self.tiletypes = Some(TileTypes::load_arrays(path)?);
        // The same file holds the cut and lift tables.
        self.object_tables = std::fs::read(path).ok().and_then(|d| ObjectTables::parse(&d));
        self.refresh_water();
        Ok(())
    }

    /// The point the 2.05 client probes for lifting and for the sword's cut,
    /// from the sprite's top-left, by facing: north `(x + 1.5, y)`, west
    /// `(x - 0.5, y + 2)`, south `(x + 1.5, y + 4)`, east `(x + 3.5, y + 2)`.
    /// That is one tile beyond the middle of the walking body box's leading
    /// edge (see `documentation/walking-collision.md`).
    pub fn probe_point(&self) -> (f32, f32) {
        let (x, y) = (self.player_x, self.player_y);
        match self.player_dir {
            DIR_NORTH => (x + 1.5, y),
            DIR_WEST => (x - 0.5, y + 2.0),
            DIR_EAST => (x + 3.5, y + 2.0),
            _ => (x + 1.5, y + 4.0),
        }
    }

    /// The tile under [`Self::probe_point`].
    fn probe_tile(&self) -> (i32, i32) {
        let (px, py) = self.probe_point();
        (px.floor() as i32, py.floor() as i32)
    }

    /// Lift the object in front of the player, if the glove allows it.
    ///
    /// The glove power `g` allows the first `g + 2` entries of the lift
    /// table (a bare hand lifts the first two: the bush and the pot-like
    /// block; a glove the heavier ones). The object's four tiles on the board
    /// are replaced by the table's "left behind" tiles and the player carries
    /// it. Not while swimming, carrying, or dead.
    pub fn try_lift(&mut self) -> Option<usize> {
        if self.life.is_dead() || self.in_water || self.carried.is_some() {
            return None;
        }
        let tables = self.object_tables.as_ref()?;
        let max = usize::from(self.glove_power) + 2;
        let (px, py) = self.probe_tile();
        let (kind, ax, ay) = self.current_level.as_ref()?.find_object(&tables.lift, max, px, py)?;
        let tiles = tables.lift[kind];
        let left = tables.lift_into.get(kind).copied()?;
        self.current_level.as_mut()?.put_object(ax, ay, &left);
        self.carried = Some(Carried { kind, tiles });
        Some(kind)
    }

    /// Throw what the player carries: it leaves from above the head and flies
    /// the way the player faces ([`crate::throwing`]). Returns the lift-table
    /// entry thrown, `None` with empty hands or when dead.
    pub fn throw_carried(&mut self) -> Option<usize> {
        if self.life.is_dead() {
            return None;
        }
        let c = self.carried.take()?;
        self.thrown.push(crate::throwing::Thrown::launch(c.kind, self.player_x, self.player_y, self.player_dir));
        Some(c.kind)
    }

    /// Drop every object in flight and every effect (a level change).
    fn clear_throws(&mut self) {
        self.thrown.clear();
        self.leaps.clear();
        self.grounded.clear();
        self.landings.clear();
        self.throw_ms = 0;
    }

    /// Whether a thrown object's wall probe at `(fx, fy)` ends its flight:
    /// a wall tile, or off the board with no neighbour. Type 20 is the one
    /// wall type objects fly through. (Hits on NPCs, players and chests are
    /// open: see `documentation/throwing.md`.)
    fn throw_blocked(&self, fx: f32, fy: f32) -> bool {
        let Some(level) = &self.current_level else { return false };
        let outside = !(0.0..64.0).contains(&fx) || !(0.0..64.0).contains(&fy);
        let Some(types) = &self.tiletypes else { return outside && self.gmap.is_none() };
        if self.kind_at_point(level, types, fx, fy) != TileKind::Blocking {
            return false;
        }
        let inside = (0.0..64.0).contains(&fx) && (0.0..64.0).contains(&fy);
        if inside {
            if let Some(id) = level.tile_id_at(fx as i32, fy as i32) {
                return types.types.get(usize::from(id)).copied() != Some(crate::throwing::TYPE_THROW_THROUGH);
            }
        }
        true
    }

    /// Whether the water test point is on a water tile.
    fn throw_in_water(&self, fx: f32, fy: f32) -> bool {
        match (&self.current_level, &self.tiletypes) {
            (Some(level), Some(types)) => self.kind_at_point(level, types, fx, fy) == TileKind::Water,
            _ => false,
        }
    }

    /// Advance flights and break effects by `ms` on their 50 ms tick.
    /// Returns true if anything moved (the scene needs a redraw).
    pub fn tick_throw(&mut self, ms: u64) -> bool {
        if self.thrown.is_empty() && self.leaps.is_empty() {
            self.throw_ms = 0;
            return false;
        }
        self.throw_ms += ms;
        let mut moved = false;
        while self.throw_ms >= crate::throwing::TICK_MS {
            self.throw_ms -= crate::throwing::TICK_MS;
            self.throw_tick();
            moved = true;
        }
        moved
    }

    fn throw_tick(&mut self) {
        use crate::throwing as t;
        // Effects: one frame per tick, gone after the last.
        let tables = self.object_tables.as_ref();
        self.leaps.retain_mut(|l| {
            l.frame += 1;
            (l.frame as usize) < t::leap_frames(tables, l.kind).len()
        });
        for mut o in std::mem::take(&mut self.thrown) {
            o.step();
            let (wx, wy) = o.wall_probe();
            if o.ticks_left > 0 && !self.throw_blocked(wx, wy) {
                self.thrown.push(o);
                continue;
            }
            if t::stays_on_ground(o.sprite) {
                if let Some(sprite) = o.sprite {
                    self.grounded.push(t::Grounded { sprite, x: o.x, y: o.y });
                }
                self.landings.push(Landing { kind: o.kind, effect: None, x: o.x, y: o.y });
                continue;
            }
            let (ox, oy) = o.water_probe();
            let kind = t::leap_kind(o.sprite, self.throw_in_water(ox, oy));
            self.leaps.push(t::Leap { kind, x: o.x, y: o.ground, frame: -1 });
            self.landings.push(Landing { kind: o.kind, effect: Some(kind), x: o.x, y: o.ground });
        }
    }

    /// The landings since the last call.
    pub fn take_landings(&mut self) -> Vec<Landing> {
        std::mem::take(&mut self.landings)
    }

    /// The sword's cut: the cuttable object in front of the player (the same
    /// probe point as the lift) is replaced by what it leaves behind.
    /// Returns the table entry that was cut.
    pub fn try_cut(&mut self) -> Option<usize> {
        let tables = self.object_tables.as_ref()?;
        let (px, py) = self.probe_tile();
        let (kind, ax, ay) = self.current_level.as_ref()?.find_object(&tables.cut, tables.cut.len(), px, py)?;
        let left = tables.cut_into.get(kind).copied()?;
        self.current_level.as_mut()?.put_object(ax, ay, &left);
        Some(kind)
    }

    /// Current player state as a protocol update.
    pub fn player_update(&self) -> PlayerUpdate {
        PlayerUpdate {
            x: self.player_x,
            y: self.player_y,
            dir: self.player_dir,
        }
    }

    /// Apply a GS1 command against this offline session.
    ///
    /// Handles the high-frequency commands from the scraped set:
    /// `hide`/`show` (NPC visibility), `warp` (local position),
    /// `setlevel` (noted), `hurt` (health), `say`/`message` (last message),
    /// `set`/`unset` (flags).
    /// Returns true if the command was recognized and applied.
    pub fn handle_command(&mut self, name: &str, args: &[crate::scripting::Value]) -> bool {
        match name {
            "hide" | "hideimg" => {
                if let Some(level) = self.current_level.as_mut() {
                    for npc in &mut level.npcs {
                        npc.visible = false;
                    }
                }
                true
            }
            "show" | "showimg" => {
                if let Some(level) = self.current_level.as_mut() {
                    for npc in &mut level.npcs {
                        npc.visible = true;
                    }
                }
                true
            }
            "warp" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Num(x), crate::scripting::Value::Num(y)) =
                        (&args[0], &args[1])
                    {
                        self.player_x = *x;
                        self.player_y = *y;
                    }
                }
                true
            }
            "setlevel" => {
                // Level switch is recorded; actual load is done by the caller
                // once a level path or name is resolved.
                true
            }
            "hurt" => {
                let amt = args
                    .first()
                    .and_then(|v| match v {
                        crate::scripting::Value::Num(n) => Some(*n as i32),
                        _ => None,
                    })
                    .unwrap_or(1);
                self.player_health = (self.player_health - amt).max(0);
                true
            }
            "say" | "message" => {
                if let Some(crate::scripting::Value::Str(s)) = args.first() {
                    self.last_message = Some(s.clone());
                }
                true
            }
            "set" => {
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.flags.insert(name.clone(), true);
                }
                true
            }
            "unset" => {
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.flags.insert(name.clone(), false);
                }
                true
            }
            "setstring" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        self.strings.insert(name.clone(), val.clone());
                    }
                }
                true
            }
            "addstring" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        let entry = self.strings.entry(name.clone()).or_default();
                        entry.push_str(val);
                    }
                }
                true
            }
            "setplayerprop" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        self.props.insert(name.clone(), val.clone());
                    }
                }
                true
            }
            "setimg" | "setshape" => {
                // Image/shape change is recorded as a prop for now.
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.props.insert("image".into(), name.clone());
                }
                true
            }
            "move" => {
                // Relative move: args are dx, dy.
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Num(dx), crate::scripting::Value::Num(dy)) =
                        (&args[0], &args[1])
                    {
                        self.player_x += *dx;
                        self.player_y += *dy;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Place a named stamp from an object set onto the current level.
    /// Returns true if the stamp was found and written.
    pub fn place_named_stamp(&mut self, set: &crate::assets::ObjectSet, name: &str, x: i32, y: i32) -> bool {
        if let Some(stamp) = set.find(name) {
            if let Some(level) = self.current_level.as_mut() {
                level.place_stamp(stamp, x, y);
                return true;
            }
        }
        false
    }
    /// If the player's feet are in a link, record the destination in
    /// `pending_link`. Returns true if a link was entered.
    ///
    /// A link does not fire while a level is loading or a warp is
    /// pending, before the player has moved since arriving, or while the
    /// player is still inside a link that contained the arrival tile.
    pub fn check_links(&mut self) -> bool {
        self.pending_link = None;
        if self.warp_pending.is_some() || !self.armed {
            return false;
        }
        let (fx, fy) = self.foot();
        let (px, py) = (fx as i32, fy as i32);
        let Some(level) = &self.current_level else {
            return false;
        };
        let (px0, py0) = (self.player_x, self.player_y);
        let b = LINK_BOX[usize::from(self.player_dir & 3)];
        let boxes = self.link_boxes;
        let hit = |l: &Link| {
            if boxes {
                l.overlaps(px0 + b.0, py0 + b.1, px0 + b.2, py0 + b.3)
            } else {
                l.contains(px, py)
            }
        };
        if let Some(t) = self.arrival_tile {
            // Still on the way out of the arrival link?
            if level.links.iter().any(|l| l.contains(t.0, t.1) && hit(l)) {
                return false;
            }
            self.arrival_tile = None;
        }
        self.pending_link = level.links.iter().find(|l| hit(l)).map(|link| {
            (
                link.dest.clone(),
                link.new_x.unwrap_or(self.player_x + link.x_offset),
                link.new_y.unwrap_or(self.player_y + link.y_offset),
            )
        });
        self.pending_link.is_some()
    }

    /// The player is put at `(x, y)` in a level that is now loaded: the
    /// end of a warp, online or offline. Ends a pending warp, switches
    /// developer mode off (returns true if it was on), and holds links
    /// back until the player moves (see [`Self::check_links`]).
    pub fn arrive(&mut self, x: f32, y: f32) -> bool {
        self.player_x = x;
        self.player_y = y;
        self.warp_pending = None;
        self.pending_link = None;
        self.armed = false;
        let (fx, fy) = self.foot();
        self.arrival_tile = Some((fx as i32, fy as i32));
        let dev_was_on = self.dev.reset();
        self.refresh_water();
        dev_was_on
    }

    /// Ask for a warp: from now until [`Self::arrive`] or
    /// [`Self::cancel_warp`] no link fires and the player stays put.
    pub fn begin_warp(&mut self, dest: &str) {
        self.warp_pending = Some(dest.to_string());
        self.pending_link = None;
    }

    /// Give up a pending warp (the server refused or never answered).
    pub fn cancel_warp(&mut self) {
        self.warp_pending = None;
    }

    /// Recompute `in_water` and `sitting` from the feet.
    pub fn refresh_water(&mut self) {
        let (fx, fy) = self.foot();
        self.in_water = match (&self.current_level, &self.tiletypes) {
            (Some(level), Some(types)) => self.kind_at_point(level, types, fx, fy) == TileKind::Water,
            _ => false,
        };
        // Feet on a chair tile (a bench): drawn sitting once the player stops.
        self.sitting = match (&self.current_level, &self.tiletypes) {
            (Some(level), Some(types)) if (0.0..64.0).contains(&fx) && (0.0..64.0).contains(&fy) => {
                level.tile_id_at(fx as i32, fy as i32).is_some_and(|id| types.is_chair(id))
            }
            _ => false,
        };
    }

    /// Time between steps now: swimming is slower than walking.
    pub fn step_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(if self.in_water { SWIM_STEP_MS } else { WALK_STEP_MS })
    }

    /// `Some(swimming)` when the swimming state changed since the last
    /// call, so the caller can tell the server once per change.
    pub fn take_swim_change(&mut self) -> Option<bool> {
        if self.in_water != self.swim_sent {
            self.swim_sent = self.in_water;
            Some(self.in_water)
        } else {
            None
        }
    }

    /// The kind of tile under a point given relative to the current cell.
    /// On a gmap a point past the edge is in the next cell's level, and in
    /// no level (blocking) when that cell has none loaded.
    fn kind_at_point(&self, level: &Level, types: &TileTypes, fx: f32, fy: f32) -> TileKind {
        let Some(w) = &self.gmap else {
            return level.kind_at(types, fx, fy);
        };
        let side = crate::gmap::CELL_TILES as f32;
        let (kx, ky) = ((fx / side).floor() as i32, (fy / side).floor() as i32);
        if (kx, ky) == (0, 0) {
            return level.kind_at(types, fx, fy);
        }
        match w.levels.get(&(w.cell.0 + kx, w.cell.1 + ky)) {
            Some(l) => l.kind_at(types, fx - kx as f32 * side, fy - ky as f32 * side),
            None => TileKind::Blocking,
        }
    }

    /// Whether a sprite position (current-cell tiles) is a legal place to be
    /// as far as the gmap goes: inside the cell, or across an edge into a
    /// loaded level. A step that would cross both edges at once is refused
    /// (it was never seen from a real client; the walk slides along one
    /// axis instead).
    fn gmap_step_ok(&self, nx: f32, ny: f32) -> bool {
        let Some(w) = &self.gmap else { return true };
        match w.map.resolve(w.cell, (nx, ny)) {
            Resolved::Inside => true,
            Resolved::NoLevel => false,
            Resolved::Crossed(c) => !(c.x_changed() && c.y_changed()) && w.levels.contains_key(&c.to),
        }
    }

    /// Start a gmap walk: the grid, and the cell the player is in.
    /// `player_x`/`player_y` must be local to `cell`.
    pub fn enter_gmap(&mut self, name: String, map: crate::gmap::Gmap, cell: (i32, i32)) {
        self.gmap = Some(GmapWorld::new(name, map, cell));
    }

    /// Leave the gmap (a plain level warp).
    pub fn leave_gmap(&mut self) {
        self.gmap = None;
    }

    /// The edge crossing the last step made, if any. The caller sends the
    /// crossing frame ([`crate::gmap::crossing_frame`]); the player is
    /// already in the new cell with the wrapped position.
    pub fn take_gmap_crossing(&mut self) -> Option<Crossing> {
        self.gmap.as_mut()?.crossing.take()
    }

    /// Move into the cell `c` names: the loaded level there becomes the
    /// current one, the old one is kept as a neighbour.
    fn cross_to(&mut self, c: Crossing) {
        let Some(w) = self.gmap.as_mut() else { return };
        if let Some(old) = self.current_level.take() {
            w.levels.insert(c.from, old);
        }
        self.current_level = w.levels.remove(&c.to);
        w.cell = c.to;
        w.crossing = Some(c);
        // A new cell is a new level to the script engine too.
        self.clear_throws();
        self.level_serial += 1;
        self.player_x = c.local_x;
        self.player_y = c.local_y;
        self.opened_chests.clear();
        self.pending_link = None;
    }

    /// True if a step in facing `dir` may end with the sprite's top-left at
    /// `(x, y)`: none of the four [`WALK_PROBES`] of that facing is on a
    /// wall tile (or off the board) or inside a chest. Water and lava are
    /// open. Developer mode skips the test.
    fn can_walk(&self, x: f32, y: f32, dir: usize) -> bool {
        self.dev.wallhack() || !WALK_PROBES[dir].iter().any(|&(px, py)| self.blocked_point(x + px, y + py))
    }

    /// How many of the nine points inside the body box (x + 0.525, 1.5 and
    /// 2.475 across, y + 1.025, 2.0 and 2.975 down) are on a wall or a chest.
    /// Zero for any position the walking test lets a player reach: a body
    /// that touches a wall with its right or bottom edge is not inside it.
    /// (The edge probes of [`WALK_PROBES`] are not used for this: the right
    /// and bottom ones sit on the tile line just outside the box, so a body
    /// resting against a wall would count as inside it.)
    fn body_overlap(&self, x: f32, y: f32) -> usize {
        const XS: [f32; 3] = [0.525, 1.5, 2.475];
        const YS: [f32; 3] = [1.025, 2.0, 2.975];
        XS.iter().flat_map(|&px| YS.iter().map(move |&py| (px, py))).filter(|&(px, py)| self.blocked_point(x + px, y + py)).count()
    }

    /// Move the player by (dx, dy) if the body box can go there.
    ///
    /// Uses the body-box wall test of the 2.05 client ([`WALK_PROBES`]);
    /// updates facing from the dominant axis. A diagonal is two axis steps
    /// tested one after the other, so it slides along the open axis.
    /// A player already standing in a blocked tile (put there by a
    /// warp, say) may step to any place that is less inside it. Nothing moves while a warp is pending.
    /// Returns the emitted `PlayerUpdate` if the move succeeded.
    pub fn try_move(&mut self, dx: f32, dy: f32) -> Option<PlayerUpdate> {
        if dx == 0.0 && dy == 0.0 || self.life.is_dead() {
            return None;
        }
        // Facing from dominant axis.
        if dx.abs() > dy.abs() {
            self.player_dir = if dx > 0.0 { DIR_EAST } else { DIR_WEST };
        } else {
            self.player_dir = if dy > 0.0 { DIR_SOUTH } else { DIR_NORTH };
        }
        if self.warp_pending.is_some() {
            return None;
        }

        // The client takes the arrow keys one at a time, in the order up,
        // left, down, right, each with its own wall test at the position the
        // earlier ones left. A diagonal is two steps, so it slides along a
        // wall without any extra rule.
        let (x0, y0) = (self.player_x, self.player_y);
        let (mut x, mut y) = (x0, y0);
        let mut try_step = |s: &Self, mx: f32, my: f32, dir: usize| {
            // A body that is inside a wall (put there by a warp) may only
            // step to a place that is less inside it; every other body
            // takes the plain edge test.
            let ok = s.can_walk(x + mx, y + my, dir) || {
                let here = s.body_overlap(x, y);
                here > 0 && s.body_overlap(x + mx, y + my) < here
            };
            if s.gmap_step_ok(x + mx, y + my) && ok {
                x += mx;
                y += my;
            }
        };
        if dy < 0.0 {
            try_step(self, 0.0, dy, DIR_NORTH as usize);
        }
        if dx < 0.0 {
            try_step(self, dx, 0.0, DIR_WEST as usize);
        }
        if dy > 0.0 {
            try_step(self, 0.0, dy, DIR_SOUTH as usize);
        }
        if dx > 0.0 {
            try_step(self, dx, 0.0, DIR_EAST as usize);
        }
        let (mx, my) = (x - x0, y - y0);
        if mx == 0.0 && my == 0.0 {
            return None;
        }

        self.player_x += mx;
        self.player_y += my;
        if let Some(w) = &self.gmap {
            if let Resolved::Crossed(c) = w.map.resolve(w.cell, (self.player_x, self.player_y)) {
                self.cross_to(c);
            }
        }
        self.armed = true;
        self.walking = true;
        self.refresh_water();
        // One GANI step per move; each frame entered with a PLAYSOUND cue
        // queues that sound once, at the cue's offset from the player.
        let swimming = self.in_water && self.swim_gani.is_some();
        let gani = if swimming { self.swim_gani.as_mut() } else { self.walk_gani.as_mut() };
        if let Some(gani) = gani {
            let cues = gani.step_sounds();
            for c in cues {
                self.sound_events.push(SoundEvent { file: c.file, x: self.player_x + c.x, y: self.player_y + c.y });
            }
        }
        Some(self.player_update())
    }

    /// True if the point `(fx, fy)` (tiles, current-cell coordinates) is
    /// blocked for a knocked-back body: a wall tile (or off the board) or
    /// inside a chest. Developer mode skips the test.
    fn blocked_point(&self, fx: f32, fy: f32) -> bool {
        if self.dev.wallhack() {
            return false;
        }
        if let (Some(level), Some(types)) = (&self.current_level, &self.tiletypes) {
            if self.kind_at_point(level, types, fx, fy) == TileKind::Blocking {
                return true;
            }
        }
        match &self.current_level {
            Some(level) => level.chests.iter().any(|c| fx >= c.x as f32 && fx < c.x as f32 + 2.0 && fy >= c.y as f32 && fy < c.y as f32 + 2.0),
            None => false,
        }
    }

    /// One knockback step along the unit vector `(ux, uy)`: one tile, then
    /// the twelve probe points of the body box ([`crate::combat::KNOCKBACK_PROBES`])
    /// must all be free, or the step is taken back. Returns true if the
    /// player moved. Not while a warp is pending.
    pub(crate) fn knock_step(&mut self, ux: f32, uy: f32) -> bool {
        if self.warp_pending.is_some() || self.life.is_dead() {
            return false;
        }
        let (nx, ny) = (self.player_x + ux * crate::combat::KNOCKBACK_STEP_TILES, self.player_y + uy * crate::combat::KNOCKBACK_STEP_TILES);
        if !self.gmap_step_ok(nx, ny) {
            return false;
        }
        if crate::combat::KNOCKBACK_PROBES.iter().any(|&(px, py)| self.blocked_point(nx + px, ny + py)) {
            return false;
        }
        self.player_x = nx;
        self.player_y = ny;
        if let Some(w) = &self.gmap {
            if let Resolved::Crossed(c) = w.map.resolve(w.cell, (self.player_x, self.player_y)) {
                self.cross_to(c);
            }
        }
        self.armed = true;
        self.refresh_water();
        true
    }

    /// True once if a hit's knockback moved the player since the last call:
    /// the caller sends the new position like a step.
    pub fn take_knock_moved(&mut self) -> bool {
        std::mem::take(&mut self.knock_moved)
    }

    /// Attach a walk GANI. Stepped on every successful move.
    pub fn set_walk_gani(&mut self, gani: GaniPlayer) {
        self.walk_gani = Some(gani);
    }

    /// Attach a swim GANI, stepped instead of the walk one in water.
    pub fn set_swim_gani(&mut self, gani: GaniPlayer) {
        self.swim_gani = Some(gani);
    }

    /// Sounds queued by GANI frames since the last call.
    pub fn take_sound_events(&mut self) -> Vec<SoundEvent> {
        std::mem::take(&mut self.sound_events)
    }

    /// Current walk-frame index, if a GANI is attached.
    pub fn walk_frame(&self) -> Option<usize> {
        self.walk_gani.as_ref().map(|g| g.frame)
    }

    /// Apply a board update to the current level (if any).
    pub fn apply_board_update(&mut self, m: &BoardModify) {
        if let Some(level) = self.current_level.as_mut() {
            level.apply_board_modify(m);
        }
    }

    /// Whether a level is currently loaded.
    pub fn has_level(&self) -> bool {
        self.current_level.is_some()
    }
}

impl Default for OfflineSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    /// A 16 ms loop (the frame limit) and a 50 ms step: counting from the due time gives 20 steps
    /// a second (the real clients' pace); counting from the moment of the step
    /// gave a step every 64 ms (15.6 a second).
    #[test]
    fn step_clock_holds_the_pace_on_a_16_ms_loop() {
        use std::time::{Duration, Instant};
        let interval = Duration::from_millis(WALK_STEP_MS);
        let t0 = Instant::now();
        let simulate = |catch_up: bool| {
            let mut last = t0;
            let mut steps = 0;
            // ticks of 16 ms for ten seconds
            for k in 1..=625u64 {
                let now = t0 + Duration::from_millis(k * 16);
                if now - last >= interval {
                    last = if catch_up { next_step_clock(last, now, interval) } else { now };
                    steps += 1;
                }
            }
            steps
        };
        let old = simulate(false);
        let new = simulate(true);
        assert_eq!(old, 156, "the old rule steps every fourth tick (64 ms)");
        assert!((199..=201).contains(&new), "{new} steps in ten seconds");
    }

    #[test]
    fn step_clock_restarts_after_a_pause() {
        use std::time::{Duration, Instant};
        let interval = Duration::from_millis(50);
        let t0 = Instant::now();
        assert_eq!(next_step_clock(t0, t0 + Duration::from_millis(60), interval), t0 + interval);
        let late = t0 + Duration::from_millis(100);
        assert_eq!(next_step_clock(t0, late, interval), late);
    }

    use super::*;
    use crate::assets::ObjectTables;

    #[test]
    fn chest_and_sign_interact() {
        let text = "GLEVNW01\nCHEST 20 24 greenrupee 0\nCHEST 10 10 fullheart -1\nSIGN 30 10\nWelcome\nSIGNEND\n";
        let mut s = OfflineSession::new();
        s.current_level = Some(Level::parse(text).unwrap());

        // Walk up into the chest at (20, 24): it stops the player below it.
        s.player_x = 18.5;
        s.player_y = 25.0;
        while s.try_move(0.0, -0.5).is_some() {}
        // The body's top edge (y + 1) stops at the chest's bottom row.
        assert_eq!(s.player_y, 25.0);
        assert_eq!(s.player_dir, DIR_NORTH);
        let got = s.interact();
        assert_eq!(
            got,
            Interaction::Chest { x: 20, y: 24, item: "greenrupee".into(), got: "1 rupee".into() }
        );
        assert_eq!(s.rupees, 1);
        // Sign 0 is the chest's note.
        assert_eq!(s.reading.as_deref(), Some("Welcome"));
        // Opened once only.
        s.reading = None;
        assert_eq!(s.interact(), Interaction::Nothing);

        // Sign at (30, 10), read from below.
        s.player_x = 28.5;
        s.player_y = 10.0;
        assert_eq!(s.interact(), Interaction::Sign("Welcome".into()));

        // A heart container with no note.
        s.reading = None;
        s.player_x = 8.5;
        s.player_y = 11.0;
        assert!(matches!(s.interact(), Interaction::Chest { x: 10, y: 10, .. }));
        assert_eq!(s.max_hearts, 8);
        assert!(s.reading.is_none());
    }

    #[test]
    fn chest_opens_only_facing_up() {
        // Lab capture: the real client opens a chest only when the player is
        // below it facing up; west, east and south sides send nothing.
        let text = "GLEVNW01\nCHEST 20 24 greenrupee -1\n";
        let new = || {
            let mut s = OfflineSession::new();
            s.current_level = Some(Level::parse(text).unwrap());
            s
        };
        // Facing north under the chest: opens.
        let mut s = new();
        s.player_x = 18.5;
        s.player_y = 25.0;
        s.player_dir = DIR_NORTH;
        assert!(matches!(s.interact(), Interaction::Chest { x: 20, y: 24, .. }));
        // Chest to the east of the player, facing east: probe point is inside, nothing happens.
        let mut s = new();
        s.player_x = 16.5;
        s.player_y = 23.0;
        s.player_dir = DIR_EAST;
        let (px, py) = s.facing_point();
        assert!(px >= 20.0 && px < 22.0 && py >= 24.0 && py < 26.0);
        assert_eq!(s.interact(), Interaction::Nothing);
        assert!(s.opened_chests.is_empty());
        // West side, facing west.
        let mut s = new();
        s.player_x = 22.0;
        s.player_y = 23.0;
        s.player_dir = DIR_WEST;
        let (px, py) = s.facing_point();
        assert!(px >= 20.0 && px < 22.0 && py >= 24.0 && py < 26.0);
        assert_eq!(s.interact(), Interaction::Nothing);
        // Above the chest, facing south.
        let mut s = new();
        s.player_x = 18.5;
        s.player_y = 20.0;
        s.player_dir = DIR_SOUTH;
        let (_, py) = s.facing_point();
        assert!(py >= 24.0 && py < 26.0);
        assert_eq!(s.interact(), Interaction::Nothing);
    }

    // ---- tile movement and warp arrival ------------------------------------


    const OPEN: u16 = 0;
    const WALL: u16 = 1;
    const CHAIR: u16 = 7;
    const WATER: u16 = 2;
    const LAVA: u16 = 3;

    fn types() -> TileTypes {
        let mut t = TileTypes { types: [0; 4096] };
        t.types[WALL as usize] = 22;
        t.types[WATER as usize] = 11;
        t.types[LAVA as usize] = 12;
        t.types[CHAIR as usize] = 3;
        t
    }

    /// A 64x64 level where `f(x, y)` picks each tile.
    fn level(f: impl Fn(i32, i32) -> u16) -> Level {
        let tiles: Vec<u16> = (0..64 * 64).map(|i| f(i % 64, i / 64)).collect();
        Level::from_board(&tiles)
    }

    /// A session whose feet stand at tile-space `(fx, fy)`.
    fn session_at(level: Level, fx: f32, fy: f32) -> OfflineSession {
        let mut s = OfflineSession::new();
        s.current_level = Some(level);
        s.tiletypes = Some(types());
        s.player_x = fx - FOOT_DX;
        s.player_y = fy - FOOT_DY;
        s
    }

    /// A session whose sprite's top-left is at `(x, y)`.
    fn body_at(level: Level, x: f32, y: f32) -> OfflineSession {
        let mut s = OfflineSession::new();
        s.current_level = Some(level);
        s.tiletypes = Some(types());
        s.player_x = x;
        s.player_y = y;
        s
    }

    fn walk(s: &mut OfflineSession, dx: f32, dy: f32, steps: usize) -> usize {
        (0..steps).filter(|_| s.try_move(dx, dy).is_some()).count()
    }

    // ---- cutting and lifting ------------------------------------------------

    /// An `arrays.dat` with the structure of the client's: 9 walking lists
    /// (here empty), then the cut table (array 9), what a cut leaves (10),
    /// the lift table (11) and what a lift leaves (12), each a nested list
    /// of 2 x 2 objects stored `[column][row]`.
    fn object_arrays(cut: &[[[u32; 2]; 2]], cut_into: &[[[u32; 2]; 2]], lift: &[[[u32; 2]; 2]], lift_into: &[[[u32; 2]; 2]]) -> Vec<u8> {
        fn objs(out: &mut Vec<u8>, list: &[[[u32; 2]; 2]]) {
            out.extend_from_slice(&((1u32 << 16) | list.len() as u32).to_be_bytes());
            for o in list {
                out.extend_from_slice(&((1u32 << 16) | 2).to_be_bytes());
                for col in o {
                    out.extend_from_slice(&2u32.to_be_bytes());
                    for v in col {
                        out.extend_from_slice(&v.to_be_bytes());
                    }
                }
            }
        }
        let mut out = Vec::new();
        for _ in 0..9 {
            out.extend_from_slice(&0u32.to_be_bytes());
        }
        objs(&mut out, cut);
        objs(&mut out, cut_into);
        objs(&mut out, lift);
        objs(&mut out, lift_into);
        out
    }

    const BUSH: [[u32; 2]; 2] = [[2, 18], [3, 19]];
    const BUSH_CUT: [[u32; 2]; 2] = [[677, 693], [678, 694]];
    const BLOCK: [[u32; 2]; 2] = [[512, 528], [513, 529]];
    const BLOCK_LEFT: [[u32; 2]; 2] = [[1802, 1818], [1803, 1819]];
    const ROCK: [[u32; 2]; 2] = [[34, 50], [35, 51]];
    const ROCK_LEFT: [[u32; 2]; 2] = [[1834, 1850], [1835, 1851]];

    fn put(l: &mut Level, ax: i32, ay: i32, o: [[u32; 2]; 2]) {
        for i in 0..2 {
            for j in 0..2 {
                assert!(l.set_tile_id(ax + i as i32, ay + j as i32, o[i][j] as u16));
            }
        }
    }

    fn get(l: &Level, ax: i32, ay: i32) -> [[u32; 2]; 2] {
        let mut o = [[0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                o[i][j] = u32::from(l.tile_id_at(ax + i as i32, ay + j as i32).unwrap());
            }
        }
        o
    }

    fn object_session(glove: u8) -> OfflineSession {
        let data = object_arrays(&[BUSH, [[420, 436], [421, 437]]], &[BUSH_CUT, [[679, 695], [680, 696]]], &[BUSH, BLOCK, ROCK], &[BUSH_CUT, BLOCK_LEFT, ROCK_LEFT]);
        let mut lvl = level(|_, _| 2047);
        put(&mut lvl, 20, 20, BUSH);
        put(&mut lvl, 30, 20, BLOCK);
        put(&mut lvl, 40, 20, ROCK);
        let mut s = OfflineSession::new();
        s.current_level = Some(lvl);
        s.tiletypes = Some(types());
        s.object_tables = ObjectTables::parse(&data);
        s.glove_power = glove;
        s
    }

    /// Stand below the object whose top-left tile is `(ax, ay)`, facing it:
    /// the probe point (x + 1.5, y) falls in the object's bottom row.
    fn face_up_at(s: &mut OfflineSession, ax: i32, ay: i32) {
        s.player_x = ax as f32 - 0.5;
        s.player_y = ay as f32 + 1.5;
        s.player_dir = DIR_NORTH;
    }

    #[test]
    fn object_tables_parse_the_nested_arrays() {
        let t = ObjectTables::parse(&object_arrays(&[BUSH], &[BUSH_CUT], &[ROCK], &[ROCK_LEFT])).unwrap();
        assert_eq!(t.cut, vec![[[2, 18], [3, 19]]]);
        assert_eq!(t.cut_into, vec![[[677, 693], [678, 694]]]);
        assert_eq!(t.lift[0], [[34, 50], [35, 51]]);
        assert_eq!(t.lift_into[0], [[1834, 1850], [1835, 1851]]);
        // A file with only walking lists has no tables; a cut-off one is refused.
        assert!(ObjectTables::parse(&[0, 0, 0, 0]).is_none());
        let mut bad = object_arrays(&[BUSH], &[BUSH_CUT], &[ROCK], &[ROCK_LEFT]);
        bad.truncate(bad.len() - 3);
        assert!(ObjectTables::parse(&bad).is_none());
    }

    #[test]
    fn the_sword_cuts_a_bush_by_its_tiles_and_not_grass_or_a_rock() {
        let mut s = object_session(0);
        face_up_at(&mut s, 20, 20);
        assert_eq!(s.try_cut(), Some(0));
        assert_eq!(get(s.current_level.as_ref().unwrap(), 20, 20), BUSH_CUT);
        // Nothing left to cut.
        assert_eq!(s.try_cut(), None);
        // A point beside the object cuts nothing.
        let mut s = object_session(0);
        face_up_at(&mut s, 22, 20);
        assert_eq!(s.try_cut(), None);
        // A rock is not in the cut table.
        face_up_at(&mut s, 40, 20);
        assert_eq!(s.try_cut(), None);
    }

    #[test]
    fn a_swing_cuts_when_it_starts() {
        let mut s = object_session(0);
        s.life.set_rules(Some(crate::protocol::DEATH_205));
        s.combat.set_rules(Some(crate::protocol::COMBAT_205));
        s.sword_power = 1;
        face_up_at(&mut s, 20, 20);
        assert!(s.start_swing());
        assert_eq!(get(s.current_level.as_ref().unwrap(), 20, 20), BUSH_CUT);
    }

    #[test]
    fn a_bare_hand_lifts_a_bush_and_the_block_but_not_the_rock() {
        let mut s = object_session(0);
        face_up_at(&mut s, 20, 20);
        assert_eq!(s.interact(), Interaction::Lifted { kind: 0 });
        assert_eq!(get(s.current_level.as_ref().unwrap(), 20, 20), BUSH_CUT, "the bush leaves its stump");
        assert_eq!(s.carried.as_ref().unwrap().tiles, [[2u16, 18], [3, 19]]);
        // Hands full: A throws it.
        assert_eq!(s.interact(), Interaction::Thrown { kind: 0 });
        assert!(s.carried.is_none());
        assert_eq!(s.thrown.len(), 1);
        s.thrown.clear();

        face_up_at(&mut s, 30, 20);
        assert_eq!(s.interact(), Interaction::Lifted { kind: 1 });
        assert_eq!(get(s.current_level.as_ref().unwrap(), 30, 20), BLOCK_LEFT);
        s.carried = None;

        face_up_at(&mut s, 40, 20);
        assert_eq!(s.interact(), Interaction::Nothing, "the rock needs a glove");
        assert_eq!(get(s.current_level.as_ref().unwrap(), 40, 20), ROCK);
    }

    #[test]
    fn a_glove_lifts_the_rock() {
        let mut s = object_session(1);
        face_up_at(&mut s, 40, 20);
        assert_eq!(s.interact(), Interaction::Lifted { kind: 2 });
        assert_eq!(get(s.current_level.as_ref().unwrap(), 40, 20), ROCK_LEFT);
        assert_eq!(s.carried.as_ref().unwrap().kind, 2);
    }

    #[test]
    fn no_lift_while_swimming_or_without_the_tables() {
        let mut s = object_session(2);
        face_up_at(&mut s, 20, 20);
        s.in_water = true;
        assert_eq!(s.try_lift(), None);
        s.in_water = false;
        s.object_tables = None;
        assert_eq!(s.try_lift(), None);
        assert_eq!(s.try_cut(), None);
    }

    /// A bench (chair tiles, walkable): standing with the feet on it sits,
    /// walking off it stands. The flag only says "on a chair"; the pose is
    /// drawn once the player stops (`pose::current_pose`).
    #[test]
    fn feet_on_a_chair_tile_sit() {
        // Chair tiles in column 20, rows 10..=11 (a 1 x 2 bench).
        let tiles = |x: i32, y: i32| if x == 20 && (10..=11).contains(&y) { CHAIR } else { OPEN };
        // feet at (x + 2.0, y + 2.75): put the player so the feet are at (19.75, 10.5) -> tile 19: not a chair
        let mut s = session_at(level(tiles), 19.5, 10.5);
        s.refresh_water();
        assert!(!s.sitting);
        // one step right: the feet are at 20.0, 10.5: on the bench
        assert!(s.try_move(0.5, 0.0).is_some());
        assert!(s.sitting, "feet at ({}, {})", s.player_x + FOOT_DX, s.player_y + FOOT_DY);
        // walking on: still on it at 20.5, then off at 21.0
        assert!(s.try_move(0.5, 0.0).is_some());
        assert!(s.sitting);
        assert!(s.try_move(0.5, 0.0).is_some());
        assert!(!s.sitting);
        // the bench is not a wall: it did not block the walk
    }

    /// A player who spawns on a bench sits as soon as the tile types are loaded.
    #[test]
    fn a_player_placed_on_a_chair_sits() {
        let tiles = |x: i32, y: i32| if (20..=21).contains(&x) && (10..=11).contains(&y) { CHAIR } else { OPEN };
        let mut s = OfflineSession::new();
        s.current_level = Some(level(tiles));
        s.player_x = 20.0 - FOOT_DX + 0.5;
        s.player_y = 10.0 - FOOT_DY + 0.5;
        s.tiletypes = Some(types());
        s.refresh_water();
        assert!(s.sitting);
        // off the board is not a chair
        s.player_x = -10.0;
        s.refresh_water();
        assert!(!s.sitting);
    }

    #[test]
    fn wall_blocks_a_walk() {
        let mut s = body_at(level(|x, _| if x == 20 { WALL } else { OPEN }), 16.5, 8.0);
        // The right edge of the body is x + 2.5: 19.0 -> 19.5 is tile 19, 20.0 is the wall.
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 1);
        assert_eq!(s.player_x, 17.0);
        assert_eq!(s.player_dir, DIR_EAST);
        // and it is open again going back
        assert_eq!(walk(&mut s, -0.5, 0.0, 2), 2);
    }

    #[test]
    fn last_row_and_column_are_floor_not_wall() {
        // The original client truncates its test point to a tile, so tile
        // 63 runs from 63.0 to 64.0; only 64.0 and beyond is off the board.
        // The body's right edge is x + 2.5 and its left edge x + 0.5.
        let mut s = body_at(level(|_, _| OPEN), 60.5, 10.0);
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 1, "edge 63.0 -> 63.5; 64.0 is outside");
        assert_eq!(s.player_x, 61.0);
        let mut s = body_at(level(|_, _| OPEN), 0.0, 10.0);
        assert_eq!(walk(&mut s, -0.5, 0.0, 4), 1, "edge 0.5 -> 0.0 ok, -0.5 is off the board");
        assert_eq!(s.player_x, -0.5);
    }

    #[test]
    fn wall_edge_is_the_tile_edge_not_half_a_tile_earlier() {
        // Wall column at x = 20: a right edge at 19.5 must be allowed, since
        // tile 19 is floor (a rounded test point would call it tile 20).
        let mut s = body_at(level(|x, _| if x == 20 { WALL } else { OPEN }), 16.5, 8.0);
        assert!(s.try_move(0.5, 0.0).is_some());
        assert_eq!(s.player_x + 2.5, 19.5);
        assert!(s.try_move(0.5, 0.0).is_none());
    }

    #[test]
    fn water_is_swimmable_not_a_wall() {
        // Water at x 10..=13 on every row.
        let mut s = session_at(level(|x, _| if (10..=13).contains(&x) { WATER } else { OPEN }), 8.5, 20.5);
        assert!(!s.in_water);
        assert_eq!(s.take_swim_change(), None);
        assert!(s.try_move(0.5, 0.0).is_some()); // 9.0
        assert!(!s.in_water);
        assert!(s.try_move(0.5, 0.0).is_some()); // 9.5
        assert!(s.try_move(0.5, 0.0).is_some()); // 10.0: in the water
        assert!(s.in_water);
        assert_eq!(s.take_swim_change(), Some(true));
        assert_eq!(s.take_swim_change(), None, "reported once per change");
        // swim across and leave
        walk(&mut s, 0.5, 0.0, 8); // 10.5 .. 14.0
        assert!(!s.in_water);
        assert_eq!(s.take_swim_change(), Some(false));
        assert_eq!(s.foot().0, 14.0);
    }

    #[test]
    fn swimming_is_slower_and_reports_the_gani() {
        let mut s = session_at(level(|_, _| WATER), 30.5, 30.5);
        assert!(s.try_move(0.5, 0.0).is_some());
        assert!(s.in_water);
        assert!(s.step_interval() > std::time::Duration::from_millis(WALK_STEP_MS));
        let mut dry = session_at(level(|_, _| OPEN), 30.5, 30.5);
        dry.try_move(0.5, 0.0);
        assert_eq!(dry.step_interval(), std::time::Duration::from_millis(WALK_STEP_MS));
        // PLI_PLAYERPROPS, PLPROP_GANI, GSTRING
        assert_eq!(swim_message(true), b"\x22\x2a\x24swim\n");
        assert_eq!(swim_message(false), b"\x22\x2a\x24idle\n");
    }

    #[test]
    fn lava_does_not_block() {
        let mut s = session_at(level(|x, _| if x >= 12 { LAVA } else { OPEN }), 10.5, 20.5);
        assert_eq!(walk(&mut s, 0.5, 0.0, 6), 6);
        assert!(!s.in_water, "lava is not water");
    }

    #[test]
    fn diagonal_into_a_corner_goes_nowhere() {
        // A wall row at y = 30 and a wall column at x = 20: the body below
        // the row and left of the column cannot go down-right, since both of
        // its axis steps are blocked.
        let mut s = body_at(level(|x, y| if y == 30 || x == 20 { WALL } else { OPEN }), 17.0, 26.5);
        let before = (s.player_x, s.player_y);
        assert!(s.try_move(0.5, 0.5).is_none());
        assert_eq!((s.player_x, s.player_y), before);
    }

    #[test]
    fn diagonal_slides_along_a_wall() {
        // Wall row at y = 30: the bottom edge (y + 3) is at 29.5, so going
        // down-right slides right.
        let mut s = body_at(level(|_, y| if y == 30 { WALL } else { OPEN }), 10.0, 26.5);
        let r = s.try_move(0.5, 0.5);
        assert!(r.is_some());
        assert_eq!((s.player_x, s.player_y), (10.5, 26.5));
    }

    #[test]
    fn the_leading_edge_is_four_points_wide_not_one() {
        // A single wall tile at (21, 20), two tiles left of where the old
        // foot point (x + 2.0) would have met it: the left half of the body
        // walks into it from below. Going up, the edge is y + 1 and spans
        // x + 0.5 .. x + 2.5.
        let tiles = |x: i32, y: i32| if (x, y) == (21, 20) { WALL } else { OPEN };
        let mut s = body_at(level(tiles), 19.0, 20.5);
        // top edge 21.5, x range 19.5..21.5: the point at 21.475 is in tile 21.
        assert!(s.try_move(0.0, -0.5).is_some()); // edge 21.0: row 21, open
        assert!(s.try_move(0.0, -0.5).is_none(), "edge 20.5 is row 20 and 21.475 is the wall");
        // One tile to the left it is clear.
        let mut s = body_at(level(tiles), 18.0, 20.5);
        assert_eq!(walk(&mut s, 0.0, -0.5, 4), 4);
    }

    #[test]
    fn rocks_block_with_a_nested_arrays_file() {
        // The client's arrays.dat: array 0 is the passable list, and the
        // rock's tiles (34 35 50 51) are not in it.
        let mut data = Vec::new();
        for list in [vec![0u32, 2047], vec![], vec![], vec![]] {
            data.extend_from_slice(&(list.len() as u32).to_be_bytes());
            for v in list {
                data.extend_from_slice(&v.to_be_bytes());
            }
        }
        let t = TileTypes::parse_arrays(&data).unwrap();
        let tiles = |x: i32, y: i32| match (x, y) {
            (40, 10) => 34,
            (41, 10) => 35,
            (40, 11) => 50,
            (41, 11) => 51,
            _ => 2047,
        };
        let mut s = body_at(level(tiles), 36.5, 9.0);
        s.tiletypes = Some(t.clone());
        // Right edge 39.0 -> 39.5 free, 40.0 is the rock.
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 1);
        // From above, the bottom edge y + 3 reaches the rock's top row (10).
        let mut s = body_at(level(tiles), 39.0, 6.0);
        s.tiletypes = Some(t);
        assert_eq!(walk(&mut s, 0.0, 0.5, 10), 1);
        assert_eq!(s.player_y, 6.5);
    }

    #[test]
    fn a_player_inside_a_wall_can_walk_out() {
        // Walls in columns 20 and 21; a body at x = 19.0 spans 19.5 to 21.5,
        // so it is half in the wall. It may step back out (left) ...
        let mut s = body_at(level(|x, _| if (20..=21).contains(&x) { WALL } else { OPEN }), 19.0, 10.0);
        assert!(s.try_move(-0.5, 0.0).is_some());
        assert_eq!(s.player_x, 18.5);
        // ... and, once out, the wall blocks it again.
        assert_eq!(walk(&mut s, -0.5, 0.0, 3), 3);
        assert_eq!(s.player_x, 17.0);
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 0, "clear of the wall, it blocks again");
    }

    /// The screenshots' bug: a body resting against a wall by its right or
    /// bottom edge was counted as "inside" it (the right and bottom probes
    /// sit on the tile line outside the box), and a body counted as inside
    /// walked through anything. Walking down along a wall at x = 17.5 (right
    /// edge on the tile line, wall column 20) must not open the wall.
    #[test]
    fn resting_against_a_wall_does_not_unlock_walking_through_it() {
        let tiles = |x: i32, y: i32| if x == 20 && (10..=12).contains(&y) { WALL } else { OPEN };
        let mut s = body_at(level(tiles), 17.5, 5.0);
        // walk down past the wall: nothing in the way for a body at 17.5 (right probes are not used going down)
        assert_eq!(walk(&mut s, 0.0, 0.5, 20), 20);
        assert_eq!(s.player_x, 17.5);
        // now beside the wall's rows again from below; go back up and stop next to it
        let mut s = body_at(level(tiles), 17.5, 8.0);
        assert!(s.try_move(0.0, 0.5).is_some());
        assert!(s.try_move(0.0, 0.5).is_some());
        // beside the wall now (box rows overlap 10..12): the wall still blocks stepping right
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 0, "x = {}", s.player_x);
        assert_eq!(s.player_x, 17.5);
        // and diagonal pushes slide along it instead of through it
        assert_eq!(walk(&mut s, 0.5, 0.5, 3), 3);
        assert_eq!(s.player_x, 17.5);
    }

    /// Same at the bottom edge: a body whose bottom edge rests on a wall row.
    #[test]
    fn resting_on_a_wall_row_does_not_unlock_walking_through_it() {
        let tiles = |x: i32, y: i32| if y == 20 && (10..=12).contains(&x) { WALL } else { OPEN };
        // y + 3.0 = 20.0: the bottom edge is on the tile line above the wall row.
        let mut s = body_at(level(tiles), 8.5, 16.5);
        assert_eq!(walk(&mut s, 0.0, 0.5, 10), 0);
        // slide sideways above the wall, then try down again
        assert_eq!(walk(&mut s, 0.5, 0.0, 2), 2);
        assert_eq!(walk(&mut s, 0.0, 0.5, 10), 0, "y = {}", s.player_y);
        assert_eq!(s.player_y, 16.5);
    }

    /// Rocks one tile apart: a two-tile body does not fit between them, and a
    /// body beside one is not freed by it.
    #[test]
    fn a_body_overlap_of_zero_for_every_position_reached_by_walking() {
        let tiles = |x: i32, y: i32| if (x * 7 + y * 13) % 5 == 0 { WALL } else { OPEN };
        let mut s = body_at(level(|x, y| if (6..40).contains(&x) && (6..40).contains(&y) { tiles(x, y) } else { OPEN }), 2.0, 2.0);
        // a long scripted walk of mixed directions
        let dirs = [(0.5, 0.0), (0.0, 0.5), (-0.5, 0.0), (0.0, -0.5), (0.5, 0.5), (-0.5, 0.5), (0.5, -0.5)];
        for i in 0..4000usize {
            let (dx, dy) = dirs[(i / 7 + i / 31) % dirs.len()];
            s.try_move(dx, dy);
            assert_eq!(s.body_overlap(s.player_x, s.player_y), 0, "step {i} at ({}, {})", s.player_x, s.player_y);
        }
    }

    #[test]
    fn chests_block_the_feet() {
        let mut s = body_at(level(|_, _| OPEN), 16.5, 8.0);
        s.current_level.as_mut().unwrap().chests.push(crate::assets::Chest {
            x: 20,
            y: 10,
            item: "greenrupee".into(),
            sign_index: -1,
            opened: false,
        });
        assert_eq!(walk(&mut s, 0.5, 0.0, 10), 1, "the body's right edge meets the chest at x = 20");
    }

    #[test]
    fn dev_mode_is_off_at_start_and_walks_through_walls() {
        let mut s = body_at(level(|x, _| if x == 20 { WALL } else { OPEN }), 17.0, 10.0);
        assert!(!s.dev.active());
        assert!(s.try_move(0.5, 0.0).is_none());
        if DevMode::AVAILABLE {
            s.dev.toggle();
            assert!(s.try_move(0.5, 0.0).is_some());
            // a warp or level change switches it off again
            assert!(s.arrive(10.0, 10.0));
            assert!(!s.dev.active());
        }
    }

    fn link(dest: &str, x: i32, y: i32, w: i32, h: i32, nx: Option<f32>, ny: Option<f32>) -> Link {
        Link { dest: dest.into(), x, y, width: w, height: h, new_x: nx, new_y: ny, x_offset: 0.0, y_offset: 0.0 }
    }

    #[test]
    fn link_uses_its_destination_and_keeps_player_axes() {
        let mut lvl = level(|_, _| OPEN);
        lvl.links.push(link("b.nw", 40, 0, 4, 64, Some(1.0), None)); // x = 1, y = playery
        lvl.links.push(link("c.nw", 0, 50, 64, 2, None, Some(2.5))); // x = playerx, y = 2.5
        let mut s = session_at(lvl, 41.0, 20.0);
        assert!(s.check_links());
        assert_eq!(s.pending_link, Some(("b.nw".into(), 1.0, s.player_y)));
        let mut s2 = session_at(s.current_level.clone().unwrap(), 12.0, 50.5);
        assert!(s2.check_links());
        assert_eq!(s2.pending_link, Some(("c.nw".into(), s2.player_x, 2.5)));
    }

    /// A Z3-V1 level read from disk drives the same link rules: a negative
    /// arrival (`-1` in the file) keeps the player's coordinate on that axis.
    #[test]
    fn zelda_level_links_work_in_the_engine() {
        use crate::assets::{ZeldaLevel, ZeldaLink, ZeldaMagic};
        let z = ZeldaLevel {
            magic: ZeldaMagic::V103,
            board: vec![0; 4096],
            links: vec![
                ZeldaLink { dest: "b.zelda".into(), x: 40.0, y: 0.0, width: 4.0, height: 64.0, new_x: 1.5, new_y: -1.0 },
                ZeldaLink { dest: "c.zelda".into(), x: 0.0, y: 50.0, width: 64.0, height: 2.0, new_x: -1.0, new_y: 2.0 },
            ],
            computers: vec![],
            signs: vec![],
            layout: Default::default(),
        };
        let dir = std::env::temp_dir().join(format!("seihai-zelda-engine-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.zelda");
        std::fs::write(&path, z.to_bytes().unwrap()).unwrap();
        let mut s = session_at(level(|_, _| OPEN), 41.0, 20.0);
        s.load_level(&path).unwrap();
        s.armed = true;
        assert!(s.check_links());
        assert_eq!(s.pending_link, Some(("b.zelda".into(), 1.5, s.player_y)));
        let mut s2 = session_at(level(|_, _| OPEN), 12.0, 50.5);
        s2.load_level(&path).unwrap();
        s2.armed = true;
        assert!(s2.check_links());
        assert_eq!(s2.pending_link, Some(("c.zelda".into(), s2.player_x, 2.0)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// B has a link back to A across its west edge, and the arrival spot
    /// from A's link is inside it.
    fn level_b() -> Level {
        let mut b = level(|_, _| OPEN);
        b.links.push(link("a.nw", 0, 10, 4, 6, Some(60.0), None));
        b
    }

    #[test]
    fn arriving_inside_a_link_does_not_warp_straight_back() {
        let mut s = session_at(level_b(), 40.0, 40.0);
        // The server puts the player at x = 1, y = 12: feet at (3.0, 14.75),
        // inside the return link.
        s.begin_warp("b.nw");
        assert!(!s.check_links(), "no link fires while a warp is pending");
        assert!(s.try_move(0.5, 0.0).is_none(), "and nothing walks");
        s.arrive(1.0, 12.0);
        assert_eq!((s.player_x, s.player_y), (1.0, 12.0), "arrival is the destination, exactly");
        assert!(s.warp_pending.is_none());
        assert!(!s.check_links(), "not before the player has moved");
        // Holding the key toward the edge keeps the player in that link.
        for _ in 0..6 {
            s.try_move(-0.5, 0.0);
            assert!(!s.check_links());
        }
        assert!(s.player_x <= 0.0, "the left edge (x + 0.5) stops at the board edge");
    }

    #[test]
    fn stepping_off_the_arrival_link_and_back_on_warps() {
        let mut s = session_at(level_b(), 40.0, 40.0);
        s.arrive(1.0, 12.0); // feet (3.0, 14.75)
        assert_eq!(walk(&mut s, 0.5, 0.0, 2), 2); // feet x 4.0: out of the link
        assert!(!s.check_links());
        assert!(s.try_move(-0.5, 0.0).is_some()); // feet x 3.5: back in
        assert!(s.check_links());
        assert_eq!(s.pending_link, Some(("a.nw".into(), 60.0, s.player_y)));
    }

    #[test]
    fn a_link_elsewhere_still_fires_after_the_first_move() {
        let mut b = level_b();
        b.links.push(link("c.nw", 30, 0, 4, 64, Some(5.0), Some(5.0)));
        let mut s = session_at(b, 28.0, 30.0);
        s.arrive(26.0, 27.25);
        assert!(!s.check_links());
        assert_eq!(walk(&mut s, 0.5, 0.0, 4), 4); // feet x 30.0
        assert!(s.check_links());
        assert_eq!(s.pending_link.as_ref().map(|l| l.0.as_str()), Some("c.nw"));
    }

    /// A level with the lab's links: a rectangle (10, 28, 2 by 5) and the
    /// four edge strips, and a session with probe boxes on that walks from
    /// `(x, y)` in direction `(dx, dy)`; returns the position the first link
    /// fires at, or `None` after `steps` steps.
    fn first_fire(x: f32, y: f32, dx: f32, dy: f32, steps: usize) -> Option<(f32, f32)> {
        let mut lvl = level(|_, _| OPEN);
        lvl.links.push(link("lv_b.nw", 0, 0, 64, 2, None, Some(60.0)));
        lvl.links.push(link("lv_c.nw", 10, 28, 2, 5, Some(60.0), Some(30.0)));
        lvl.links.push(link("lv_w.nw", 0, 62, 64, 2, None, Some(4.0)));
        lvl.links.push(link("lv_s.nw", 62, 0, 2, 64, Some(4.0), None));
        let mut s = session_at(lvl, 0.0, 0.0);
        s.link_boxes = true;
        s.player_x = x;
        s.player_y = y;
        s.arrive(x, y);
        for _ in 0..steps {
            s.try_move(dx, dy)?;
            if s.check_links() {
                return Some((s.player_x, s.player_y));
            }
        }
        None
    }

    // The positions below are the real 2.22 and 2.31 clients' first warps in
    // the lab's `linkgeom` and `linkgeom2` scenarios (the player is put on a
    // half tile and walks): the warp goes out at the step *after* the last
    // position the client reported.

    #[test]
    fn probe_boxes_fire_where_the_real_client_does_on_the_rectangle_link() {
        // Down at x 8.5 ... 10.5 reached the link after reporting y 24.5.
        for x in [8.5, 9.0, 9.5, 10.0, 10.5] {
            assert_eq!(first_fire(x, 20.0, 0.0, 0.5, 40), Some((x, 25.0)), "down at x {x}");
        }
        // Up at x 8.5 and 9.5 from y 40 went on after y 32.5.
        for x in [8.5, 9.5] {
            assert_eq!(first_fire(x, 40.0, 0.0, -0.5, 40), Some((x, 32.0)), "up at x {x}");
        }
        // Left from x 14 after x 12; right from x 6 after x 7.
        for y in [26.0, 26.5, 27.0, 27.5] {
            assert_eq!(first_fire(14.0, y, -0.5, 0.0, 40), Some((11.5, y)), "left at y {y}");
        }
        for y in [26.5, 27.0, 27.5] {
            assert_eq!(first_fire(6.0, y, 0.5, 0.0, 40), Some((7.5, y)), "right at y {y}");
        }
    }

    #[test]
    fn probe_boxes_miss_the_rectangle_link_where_the_real_client_does() {
        // Beside the link (x 8.0 and 11.0 or more going down; 7 and 7.5 and
        // 11 to 12 in the first lab run).
        for x in [7.0, 7.5, 8.0, 11.0, 11.5, 12.0] {
            assert_eq!(first_fire(x, 20.0, 0.0, 0.5, 20), None, "down at x {x}");
        }
        assert_eq!(first_fire(11.0, 40.0, 0.0, -0.5, 20), None);
        // Left and right below the link's bottom edge (y 32 and 33.5).
        for y in [32.0, 32.5, 33.0, 33.5] {
            assert_eq!(first_fire(14.0, y, -0.5, 0.0, 20), None, "left at y {y}");
        }
        for y in [32.5, 33.0] {
            assert_eq!(first_fire(6.0, y, 0.5, 0.0, 20), None, "right at y {y}");
        }
    }

    #[test]
    fn probe_boxes_fire_where_the_real_client_does_on_the_edges() {
        assert_eq!(first_fire(30.0, 56.0, 0.0, 0.5, 40), Some((30.0, 59.0))); // south edge
        assert_eq!(first_fire(30.0, 6.0, 0.0, -0.5, 40), Some((30.0, 1.0))); // north edge
        assert_eq!(first_fire(56.0, 30.5, 0.5, 0.0, 40), Some((59.5, 30.5))); // east edge
    }

    #[test]
    fn the_foot_point_stays_for_profiles_without_probe_boxes() {
        // The legacy rule passes by the rectangle link at its far edge.
        let mut lvl = level(|_, _| OPEN);
        lvl.links.push(link("lv_c.nw", 10, 28, 2, 5, Some(60.0), Some(30.0)));
        let mut s = session_at(lvl, 0.0, 0.0);
        s.player_x = 8.5;
        s.player_y = 20.0;
        s.arrive(8.5, 20.0);
        assert!(!s.link_boxes);
        let mut fired = None;
        for _ in 0..40 {
            s.try_move(0.0, 0.5);
            if s.check_links() {
                fired = Some(s.player_y);
                break;
            }
        }
        // foot y = y + 2.75 reaches row 28 at y 25.5, and foot x = x + 2 = 10.5
        assert_eq!(fired, Some(25.5));
    }

    #[test]
    fn no_link_while_the_level_is_loading() {
        let mut s = session_at(level_b(), 2.0, 12.0);
        s.arrive(0.0, 9.25);
        s.try_move(0.5, 0.0);
        s.current_level = None;
        assert!(!s.check_links());
    }

    #[test]
    fn a_cancelled_warp_frees_the_player() {
        let mut s = session_at(level(|_, _| OPEN), 10.0, 10.0);
        s.begin_warp("x.nw");
        assert!(s.try_move(0.5, 0.0).is_none());
        s.cancel_warp();
        assert!(s.try_move(0.5, 0.0).is_some());
    }

    // ---- gmap walking ------------------------------------------------------

    use crate::gmap::Gmap;

    const GRID: &str = "GRMAP001\nWIDTH 2\nHEIGHT 2\nLEVELNAMES\n\"nw.nw\",\"ne.nw\",\n\"sw.nw\",\"se.nw\",\nLEVELNAMESEND\n";
    /// A tile id no wall or water test confuses with the open floor: each
    /// cell's level is open floor with its own marker in the top-left tile.
    const MARK: [u16; 4] = [10, 11, 12, 13];

    fn marked(cell: usize, wall_at: Option<(i32, i32)>) -> Level {
        level(move |x, y| {
            if Some((x, y)) == wall_at {
                WALL
            } else if (x, y) == (0, 0) {
                MARK[cell]
            } else {
                OPEN
            }
        })
    }

    /// A 2x2 world, the player in the north-west cell with the sprite at
    /// `(x, y)`. `load` says which neighbour cells have a level.
    fn gmap_session(x: f32, y: f32, load: &[(i32, i32)], wall_ne: Option<(i32, i32)>) -> OfflineSession {
        let mut s = OfflineSession::new();
        s.tiletypes = Some(types());
        s.current_level = Some(marked(0, None));
        s.player_x = x;
        s.player_y = y;
        s.enter_gmap("g.gmap".into(), Gmap::parse(GRID).unwrap(), (0, 0));
        let w = s.gmap.as_mut().unwrap();
        for &(cx, cy) in load {
            let i = (cy * 2 + cx) as usize;
            w.levels.insert((cx, cy), marked(i, if (cx, cy) == (1, 0) { wall_ne } else { None }));
        }
        s
    }

    fn marker(s: &OfflineSession) -> Option<u16> {
        s.current_level.as_ref().and_then(|l| l.tile_id_at(0, 0))
    }

    #[test]
    fn walking_east_crosses_into_the_next_level() {
        let mut s = gmap_session(60.0, 30.5, &[(1, 0), (0, 1), (1, 1)], None);
        // 60.0 -> 63.5 is seven half-steps; the eighth reaches 64.0.
        assert_eq!(walk(&mut s, 0.5, 0.0, 7), 7);
        assert_eq!((s.player_x, marker(&s)), (63.5, Some(10)));
        assert!(s.take_gmap_crossing().is_none());
        assert!(s.try_move(0.5, 0.0).is_some());
        let c = s.take_gmap_crossing().expect("the step crossed");
        assert_eq!((c.from, c.to, c.local_x, c.local_y), ((0, 0), (1, 0), 0.0, 30.5));
        assert_eq!((s.player_x, s.player_y), (0.0, 30.5));
        assert_eq!(marker(&s), Some(11), "the north-east level is current");
        assert_eq!(s.gmap.as_ref().unwrap().cell, (1, 0));
        assert!(s.gmap.as_ref().unwrap().levels.contains_key(&(0, 0)), "the old level stays as a neighbour");
        assert!(s.take_gmap_crossing().is_none(), "taken once");
    }

    #[test]
    fn walking_back_west_returns_with_a_wrapped_position() {
        let mut s = gmap_session(63.0, 30.5, &[(1, 0), (0, 1), (1, 1)], None);
        walk(&mut s, 0.5, 0.0, 2);
        s.take_gmap_crossing().unwrap();
        assert_eq!((s.player_x, marker(&s)), (0.0, Some(11)));
        assert!(s.try_move(-0.5, 0.0).is_some());
        let c = s.take_gmap_crossing().unwrap();
        assert_eq!((c.to, c.local_x), ((0, 0), 63.5));
        assert_eq!((s.player_x, marker(&s)), (63.5, Some(10)));
    }

    #[test]
    fn walking_south_and_north_cross_the_y_edge() {
        let mut s = gmap_session(30.5, 63.0, &[(1, 0), (0, 1), (1, 1)], None);
        walk(&mut s, 0.0, 0.5, 2);
        let c = s.take_gmap_crossing().unwrap();
        assert_eq!((c.to, c.local_y, c.y_changed(), c.x_changed()), ((0, 1), 0.0, true, false));
        assert_eq!(marker(&s), Some(12));
        s.try_move(0.0, -0.5);
        let c = s.take_gmap_crossing().unwrap();
        assert_eq!((c.to, c.local_y), ((0, 0), 63.5));
    }

    #[test]
    fn the_grid_edge_stops_the_walk() {
        let mut s = gmap_session(61.0, 30.5, &[(1, 0), (0, 1), (1, 1)], None);
        walk(&mut s, 0.5, 0.0, 6);
        s.take_gmap_crossing().unwrap();
        // Now in the east column: nothing further east. The body's right
        // edge (x + 2.5) reaches 63.5 (sprite 61.0) and stops there.
        walk(&mut s, 0.5, 0.0, 200);
        assert_eq!(s.player_x, 61.0);
        assert!(s.take_gmap_crossing().is_none());
        // Same at the north edge.
        let mut s = gmap_session(30.5, 1.0, &[(1, 0), (0, 1), (1, 1)], None);
        walk(&mut s, 0.0, -0.5, 20);
        assert!(s.take_gmap_crossing().is_none());
        assert_eq!(s.player_y, 0.0);
    }

    #[test]
    fn a_neighbour_that_is_not_loaded_blocks() {
        let mut s = gmap_session(60.0, 30.5, &[], None);
        walk(&mut s, 0.5, 0.0, 20);
        assert!(s.take_gmap_crossing().is_none());
        assert_eq!(s.player_x, 61.0, "stops with the body edge at the seam");
        assert_eq!(marker(&s), Some(10));
        // Once it is loaded the walk goes on.
        s.gmap.as_mut().unwrap().levels.insert((1, 0), marked(1, None));
        walk(&mut s, 0.5, 0.0, 20);
        assert!(s.take_gmap_crossing().is_some());
    }

    /// The feet test reads the neighbour's tiles across the seam, so a wall
    /// in column 0 of the next level stops the player in this one.
    #[test]
    fn a_wall_across_the_seam_blocks() {
        let mut s = gmap_session(60.0, 30.5, &[(1, 0), (0, 1), (1, 1)], Some((0, 33)));
        // The body spans rows 31.5 to 33.5, so row 33 is under its right edge.
        walk(&mut s, 0.5, 0.0, 20);
        assert_eq!(s.player_x, 61.0);
        assert!(s.take_gmap_crossing().is_none());
        // A row up the same walk is free.
        let mut s = gmap_session(60.0, 29.5, &[(1, 0), (0, 1), (1, 1)], Some((0, 33)));
        walk(&mut s, 0.5, 0.0, 8);
        assert!(s.take_gmap_crossing().is_some());
    }

    #[test]
    fn water_across_the_seam_is_read_from_the_neighbour() {
        let mut s = gmap_session(60.0, 30.5, &[(1, 0), (0, 1), (1, 1)], None);
        s.gmap.as_mut().unwrap().levels.insert((1, 0), level(|_, _| WATER));
        walk(&mut s, 0.5, 0.0, 4);
        assert!(s.in_water, "feet at 64.0 are in the east level's water");
    }

    /// A step across both edges at once is refused; the walk slides along
    /// one axis, so the player crosses one edge at a time.
    #[test]
    fn a_corner_step_does_not_cross_both_edges() {
        let mut s = gmap_session(63.5, 63.5, &[(1, 0), (0, 1), (1, 1)], None);
        let r = s.try_move(0.5, 0.5);
        assert!(r.is_some(), "it slides");
        let c = s.take_gmap_crossing().unwrap();
        assert!(c.x_changed() != c.y_changed(), "one axis only: {c:?}");
    }

    #[test]
    fn without_a_gmap_the_edge_is_a_wall_as_before() {
        let mut s = session_at(level(|_, _| OPEN), 62.5, 10.5);
        assert!(s.gmap.is_none());
        walk(&mut s, 0.5, 0.0, 10);
        assert!(s.take_gmap_crossing().is_none());
        assert_eq!(s.player_x, 61.0);
    }

    #[test]
    fn crossing_clears_the_opened_chests_and_pending_link() {
        let mut s = gmap_session(63.0, 30.5, &[(1, 0), (0, 1), (1, 1)], None);
        s.opened_chests.insert((3, 3));
        walk(&mut s, 0.5, 0.0, 2);
        assert!(s.opened_chests.is_empty());
    }

    #[test]
    fn level_serial_counts_loaded_levels() {
        let dir = std::env::temp_dir().join(format!("seihai-serial-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.nw");
        std::fs::write(&path, "GLEVNW01\n").unwrap();
        let mut s = OfflineSession::new();
        assert_eq!(s.level_serial, 0);
        s.load_level(&path).unwrap();
        s.load_level(&path).unwrap();
        assert_eq!(s.level_serial, 2);
        assert!(s.load_level(dir.join("missing.nw")).is_err());
        assert_eq!(s.level_serial, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- health and death --------------------------------------------------

    use crate::life::LifeEvent;
    use crate::protocol::DEATH_205;

    fn mortal() -> OfflineSession {
        let mut s = OfflineSession::new();
        s.life.set_rules(Some(DEATH_205));
        s.current_level = Some(level(|_, _| OPEN));
        s.tiletypes = Some(types());
        s
    }

    fn die(s: &mut OfflineSession, attacker: Option<u16>) -> Vec<LifeEvent> {
        assert!(s.damage_half(s.max_hearts, attacker));
        s.tick_life(0)
    }

    #[test]
    fn half_hearts_round_trip_and_never_round_to_zero() {
        let mut s = OfflineSession::new();
        assert_eq!((s.max_hearts, s.hearts_half()), (6, 6));
        for h in 0..=6 {
            s.set_hearts_half(h);
            assert_eq!(s.hearts_half(), h);
        }
        s.set_max_hearts_half(60);
        s.set_hearts_half(1);
        assert!(s.player_health >= 1, "1 of 60 half hearts is still alive");
        assert_eq!(s.hearts_half(), 1);
    }

    #[test]
    fn hearts_zero_starts_the_death_and_faces_down() {
        let mut s = mortal();
        s.player_dir = DIR_EAST;
        assert!(s.damage_half(2, Some(9)));
        assert!(s.tick_life(50).is_empty(), "4 half hearts left");
        let ev = die(&mut s, Some(9));
        assert_eq!(ev, vec![LifeEvent::Died { attacker: Some(9) }]);
        assert!(s.life.is_dead());
        assert_eq!(s.player_dir, DIR_SOUTH);
    }

    #[test]
    fn no_rules_means_no_death() {
        let mut s = OfflineSession::new();
        s.damage_half(100, Some(1));
        assert!(s.tick_life(5000).is_empty());
        assert!(!s.life.is_dead() && !s.controls_locked());
        assert_eq!(s.player_health, 0);
    }

    #[test]
    fn nothing_moves_or_interacts_while_dead() {
        let mut s = mortal();
        die(&mut s, None);
        let (x, y) = (s.player_x, s.player_y);
        assert!(s.try_move(0.5, 0.0).is_none());
        assert_eq!((s.player_x, s.player_y), (x, y));
        assert_eq!(s.interact(), Interaction::Nothing);
        assert!(s.controls_locked());
        // The lock is over after the timer passes 27 ticks (1.4 s).
        s.tick_life(27 * 50);
        assert!(s.controls_locked());
        s.tick_life(50);
        assert!(!s.controls_locked());
        assert!(s.try_move(0.5, 0.0).is_none(), "dead until a respawn, even unlocked");
    }

    #[test]
    fn a_dead_player_cannot_be_hurt() {
        let mut s = mortal();
        die(&mut s, None);
        assert!(!s.damage_half(2, Some(3)));
    }

    #[test]
    fn respawn_waits_for_the_lock() {
        let mut s = mortal();
        die(&mut s, None);
        s.tick_life(27 * 50);
        assert!(s.respawn(true).is_none());
        assert!(s.life.is_dead());
    }

    #[test]
    fn respawn_hearts_follow_ap() {
        // (ap, max half hearts) -> half hearts after the respawn.
        for (ap, max, want) in [(0, 20, 6), (19, 20, 6), (20, 20, 10), (39, 8, 8), (40, 20, 20), (100, 12, 12)] {
            let mut s = mortal();
            s.set_max_hearts_half(max);
            s.set_hearts_half(max);
            s.alignment = ap;
            die(&mut s, None);
            s.tick_life(28 * 50);
            let _ = s.life.take_events();
            assert!(s.respawn(false).is_some(), "ap {ap}");
            assert_eq!(s.hearts_half(), want, "ap {ap} max {max}");
            assert!(!s.life.is_dead());
        }
    }

    #[test]
    fn respawn_goes_to_the_saved_entry_point() {
        let mut s = mortal();
        s.player_x = 12.5;
        s.player_y = 20.0;
        s.player_dir = DIR_WEST;
        s.note_entry("a.nw");
        s.player_x = 40.0;
        s.player_y = 41.0;
        die(&mut s, None);
        s.tick_life(28 * 50);
        let to = s.respawn(true).unwrap();
        let want = EntryPoint { level: "a.nw".into(), x: 12.5, y: 20.0, dir: DIR_WEST };
        assert_eq!(to, RespawnTo::Saved(want));
        assert_eq!((s.player_x, s.player_y, s.player_dir), (12.5, 20.0, DIR_WEST));
        // A later entry of the same name group (plain names share the empty
        // group) does not replace it in the 2.05 rules.
        s.player_x = 3.0;
        s.note_entry("b.nw");
        assert_eq!(s.entry.as_ref().map(|e| (e.level.as_str(), e.x)), Some(("a.nw", 12.5)));
        // A different group does.
        s.note_entry("cave$1.nw");
        assert_eq!(s.entry.as_ref().map(|e| (e.level.as_str(), e.x)), Some(("cave$1.nw", 3.0)));
    }

    #[test]
    fn name_groups_are_the_name_up_to_the_last_delimiter() {
        assert_eq!(name_group("Cave$1.nw"), "cave$");
        assert_eq!(name_group("cave$2.nw"), "cave$");
        assert_eq!(name_group("a§b§c.nw"), "a§b§");
        assert_eq!(name_group("levels/Town§x.graal"), "town§");
        assert_eq!(name_group("town.nw"), "", "no delimiter: the empty group");
        assert_eq!(name_group("a.nw"), name_group("b.nw"));
        assert_ne!(name_group("a$1.nw"), name_group("b$1.nw"));
    }

    fn gmap_world(names: &str) -> crate::gmap::GmapWorld {
        let text = format!("GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n{names}\nLEVELNAMESEND\n");
        crate::gmap::GmapWorld::new("w.gmap".into(), crate::gmap::Gmap::parse(&text).unwrap(), (0, 0))
    }

    #[test]
    fn the_first_entry_is_always_saved() {
        let mut s = mortal();
        s.player_x = 7.0;
        s.note_entry("a.nw");
        assert_eq!(s.entry.as_ref().map(|e| e.x), Some(7.0));
    }

    #[test]
    fn entering_or_leaving_a_gmap_saves_even_in_the_same_name_group() {
        // The list is [first, second]: "second" is at index 1, "outside" is not in it.
        let mut s = mortal();
        s.gmap = Some(gmap_world("first.nw,second.nw"));
        s.player_x = 1.0;
        s.note_entry("outside.nw");
        s.player_x = 2.0;
        s.note_entry("second.nw");
        assert_eq!(s.entry.as_ref().map(|e| (e.level.as_str(), e.x)), Some(("second.nw", 2.0)), "into the gmap");
        s.player_x = 3.0;
        s.note_entry("outside.nw");
        assert_eq!(s.entry.as_ref().map(|e| (e.level.as_str(), e.x)), Some(("outside.nw", 3.0)), "out of it");
        // Both in the list (indices 0 and 1, or 1 and 1): the product is not
        // negative, so the group test decides: same group, not saved.
        s.player_x = 4.0;
        s.note_entry("second.nw");
        s.player_x = 5.0;
        s.note_entry("first.nw");
        assert_eq!(s.entry.as_ref().map(|e| e.x), Some(4.0));
    }

    #[test]
    fn index_zero_of_the_gmap_list_counts_as_the_same_side() {
        // The client multiplies the indices; 0 times -1 is 0, not negative.
        let mut s = mortal();
        s.gmap = Some(gmap_world("first.nw,second.nw"));
        s.player_x = 1.0;
        s.note_entry("outside.nw");
        s.player_x = 2.0;
        s.note_entry("first.nw");
        assert_eq!(s.entry.as_ref().map(|e| e.x), Some(1.0), "the quirk: entering at index 0 does not save");
    }

    #[test]
    fn later_builds_have_no_name_group_test() {
        use crate::protocol::DEATH_21;
        let mut s = OfflineSession::new();
        s.life.set_rules(Some(DEATH_21));
        s.player_x = 1.0;
        s.note_entry("a$1.nw");
        s.player_x = 2.0;
        s.note_entry("b$1.nw");
        assert_eq!(s.entry.as_ref().map(|e| e.x), Some(1.0), "different groups, still not saved");
    }

    #[test]
    fn the_name_group_test_is_the_2_05_builds_only() {
        use crate::protocol::ClientVersion as V;
        assert!(V::V205.death_rules().unwrap().entry_name_group);
        for v in [V::V217, V::V222, V::V231] {
            assert!(!v.death_rules().unwrap().entry_name_group, "{}", v.label());
        }
        for l in ["2.1", "2.12", "2.13", "2.14", "2.15", "2.16", "2.3"] {
            let p = crate::protocol::by_label(l).unwrap_or_else(|| panic!("{l}"));
            assert!(!p.life.death.unwrap().entry_name_group, "{l}");
        }
    }

    #[test]
    fn respawn_without_an_entry_point() {
        let mut s = mortal();
        die(&mut s, None);
        s.tick_life(28 * 50);
        s.player_x = 5.0;
        assert_eq!(s.respawn(true), Some(RespawnTo::Default { x: 30.5, y: 30.0 }));
        assert_eq!((s.player_x, s.player_y, s.player_dir), (30.5, 30.0, DIR_SOUTH));
        let mut s = mortal();
        die(&mut s, None);
        s.tick_life(28 * 50);
        s.player_x = 5.0;
        assert_eq!(s.respawn(false), Some(RespawnTo::InPlace));
        assert_eq!(s.player_x, 5.0, "offline with nothing saved: not moved");
    }

    #[test]
    fn dying_twice_in_a_row() {
        let mut s = mortal();
        for _ in 0..2 {
            let ev = die(&mut s, Some(4));
            assert_eq!(ev, vec![LifeEvent::Died { attacker: Some(4) }]);
            s.tick_life(28 * 50);
            assert!(s.respawn(false).is_some());
            let _ = s.life.take_events();
        }
    }

    #[test]
    fn server_power_and_ap_apply() {
        let mut s = OfflineSession::new();
        // MAXPOWER 5 hearts, CURPOWER 7 half hearts, AP 60.
        s.apply_server_power(Some(5), Some(7), Some(60));
        assert_eq!((s.max_hearts, s.hearts_half(), s.alignment), (10, 7, 60));
        s.apply_server_power(None, Some(0), None);
        assert_eq!(s.player_health, 0);
        s.apply_server_power(None, None, Some(250));
        assert_eq!(s.alignment, 100);
    }

    #[test]
    fn is_loaded_ignores_extension_and_case() {
        let dir = std::env::temp_dir().join(format!("seihai-loaded-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Town.nw"), "GLEVNW01\n").unwrap();
        let mut s = OfflineSession::new();
        assert!(!s.is_loaded("town"));
        s.load_level(dir.join("Town.nw")).unwrap();
        assert!(s.is_loaded("town.nw") && s.is_loaded("TOWN") && !s.is_loaded("other.nw"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- GANI sounds -------------------------------------------------------

    use crate::assets::Gani;

    /// A single-direction gani with `n` frames; frame `k` has a cue for
    /// each `(k, name)` in `cues`.
    fn gani_with(n: usize, cues: &[(usize, &str)]) -> GaniPlayer {
        let mut text = String::from("SINGLEDIRECTION\nANI\n");
        for k in 0..n {
            text.push_str("1 0 0, 2 0 0\n");
            for (at, name) in cues {
                if *at == k {
                    text.push_str(&format!("PLAYSOUND {name} 1.5 2\n"));
                }
            }
            text.push('\n');
        }
        text.push_str("ANIEND\n");
        GaniPlayer::new(Gani::parse(&text).unwrap())
    }

    #[test]
    fn a_gani_frame_with_a_sound_queues_it_once_per_entry() {
        let mut s = session_at(level(|_, _| OPEN), 10.5, 10.5);
        s.set_walk_gani(gani_with(4, &[(2, "steps.wav")]));
        // Frames entered: 1, 2 (cue), 3, 0, 1, 2 (cue).
        walk(&mut s, 0.5, 0.0, 1);
        assert!(s.sound_events.is_empty(), "frame 1 has no cue");
        walk(&mut s, 0.5, 0.0, 1);
        let ev = s.take_sound_events();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].file, "steps.wav");
        // The cue is 1.5, 2 from the player: the listener's own point.
        assert_eq!((ev[0].x, ev[0].y), (s.player_x + 1.5, s.player_y + 2.0));
        assert!(s.take_sound_events().is_empty(), "taken events are gone");
        walk(&mut s, 0.5, 0.0, 3); // 3, 0, 1
        assert!(s.take_sound_events().is_empty());
        walk(&mut s, 0.5, 0.0, 1); // 2 again
        assert_eq!(s.take_sound_events().len(), 1);
    }

    #[test]
    fn a_blocked_move_does_not_step_the_gani_or_queue_a_sound() {
        let mut s = body_at(level(|x, _| if x == 20 { WALL } else { OPEN }), 17.0, 10.0);
        s.set_walk_gani(gani_with(1, &[(0, "steps.wav")]));
        assert!(s.try_move(0.5, 0.0).is_none());
        assert!(s.sound_events.is_empty());
        assert_eq!(s.walk_frame(), Some(0));
    }

    #[test]
    fn no_gani_means_no_sound_events() {
        let mut s = session_at(level(|_, _| OPEN), 10.5, 10.5);
        walk(&mut s, 0.5, 0.0, 4);
        assert!(s.sound_events.is_empty());
    }

    #[test]
    fn swimming_steps_the_swim_gani_not_the_walk_one() {
        let mut s = session_at(level(|x, _| if (10..=13).contains(&x) { WATER } else { OPEN }), 8.5, 20.5);
        s.set_walk_gani(gani_with(1, &[(0, "steps.wav")]));
        s.set_swim_gani(gani_with(1, &[(0, "water.wav")]));
        walk(&mut s, 0.5, 0.0, 1); // 9.0, still on land
        assert_eq!(s.take_sound_events()[0].file, "steps.wav");
        walk(&mut s, 0.5, 0.0, 4); // into the water
        assert!(s.in_water);
        let files: Vec<String> = s.take_sound_events().into_iter().map(|e| e.file).collect();
        assert_eq!(files.last().map(String::as_str), Some("water.wav"));
    }
    // ---- throwing -----------------------------------------------------

    /// Lift the bush at (20, 20) and stand at `(px, py)` facing `dir`.
    fn carrying_bush(px: f32, py: f32, dir: u8) -> OfflineSession {
        let mut s = object_session(0);
        face_up_at(&mut s, 20, 20);
        assert_eq!(s.interact(), Interaction::Lifted { kind: 0 });
        s.player_x = px;
        s.player_y = py;
        s.player_dir = dir;
        s
    }

    #[test]
    fn the_interact_key_throws_what_is_carried_and_nothing_else_when_empty() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        assert_eq!(s.interact(), Interaction::Thrown { kind: 0 });
        assert!(s.carried.is_none());
        assert_eq!(s.thrown.len(), 1);
        let o = &s.thrown[0];
        assert_eq!((o.x, o.y, o.ground, o.dir, o.ticks_left), (10.5, 28.625, 31.0, DIR_EAST, 9));
        // Empty hands, nothing in front: no throw.
        s.thrown.clear();
        s.player_x = 5.0;
        s.player_y = 5.0;
        assert_eq!(s.interact(), Interaction::Nothing);
        assert!(s.thrown.is_empty());
        assert_eq!(s.throw_carried(), None);
    }

    #[test]
    fn a_dead_player_does_not_throw() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.player_health = 0;
        s.life.set_rules(Some(crate::protocol::DEATH_205));
        s.tick_life(10);
        if s.life.is_dead() {
            assert_eq!(s.throw_carried(), None);
            assert!(s.carried.is_some());
        }
    }

    #[test]
    fn a_throw_flies_nine_ticks_then_the_bush_breaks_into_leaves() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.throw_carried();
        // Less than a tick: nothing moves.
        assert!(!s.tick_throw(49));
        assert_eq!(s.thrown[0].x, 10.5);
        assert!(s.tick_throw(1));
        assert_eq!(s.thrown[0].x, 11.5);
        for _ in 0..7 {
            s.tick_throw(50);
        }
        assert_eq!(s.thrown[0].ticks_left, 1);
        assert!(s.take_landings().is_empty());
        s.tick_throw(50);
        assert!(s.thrown.is_empty());
        let l = s.take_landings();
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].kind, l[0].effect), (0, Some(crate::throwing::LEAP_BUSH)));
        assert_eq!((l[0].x, l[0].y), (19.5, 31.0));
        // The effect shows its 7 frames, one per tick, then is gone.
        assert_eq!(s.leaps.len(), 1);
        assert_eq!(s.leaps[0].frame, -1);
        for f in 0..7 {
            s.tick_throw(50);
            assert_eq!(s.leaps.len(), 1, "frame {f} is shown");
            assert_eq!(s.leaps[0].frame, f);
        }
        s.tick_throw(50);
        assert!(s.leaps.is_empty(), "removed once past the last frame");
        assert!(!s.tick_throw(50));
    }

    #[test]
    fn a_wall_ends_the_flight_early() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        // After k steps the wall probe is (11.5 + k, 32): a wall at tile
        // (14, 32) is hit on the third step.
        s.current_level.as_mut().unwrap().set_tile_id(14, 32, WALL);
        s.throw_carried();
        for _ in 0..2 {
            s.tick_throw(50);
        }
        assert_eq!(s.thrown.len(), 1);
        s.tick_throw(50);
        assert!(s.thrown.is_empty());
        let l = s.take_landings();
        assert_eq!(l[0].x, 13.5);
    }

    #[test]
    fn the_type_the_throw_passes_is_not_a_wall_to_it() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.tiletypes.as_mut().unwrap().types[WALL as usize] = 20;
        s.current_level.as_mut().unwrap().set_tile_id(14, 32, WALL);
        s.throw_carried();
        for _ in 0..4 {
            s.tick_throw(50);
        }
        assert_eq!(s.thrown.len(), 1, "type 20 is flown through");
        // The same tile as a real wall (22) stops it.
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.current_level.as_mut().unwrap().set_tile_id(14, 32, WALL);
        s.throw_carried();
        for _ in 0..4 {
            s.tick_throw(50);
        }
        assert!(s.thrown.is_empty());
    }

    #[test]
    fn the_board_edge_ends_the_flight() {
        let mut s = carrying_bush(58.0, 30.0, DIR_EAST);
        s.throw_carried();
        for _ in 0..9 {
            s.tick_throw(50);
        }
        // the probe 59.5 + k leaves the board at k = 5
        assert!(s.thrown.is_empty());
        let l = s.take_landings();
        assert_eq!(l.len(), 1);
        assert!(l[0].x < 64.0, "stopped at the edge, x {}", l[0].x);
    }

    #[test]
    fn landing_in_water_splashes_and_a_rock_breaks_into_rock_pieces() {
        // Water where the object's own probe point ends up.
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.throw_carried();
        // end: x = 19.5, y = 28.625 + 1.8 = 30.425 -> probe (20.5, 31.425)
        s.current_level.as_mut().unwrap().set_tile_id(20, 31, WATER);
        for _ in 0..9 {
            s.tick_throw(50);
        }
        assert_eq!(s.take_landings()[0].effect, Some(crate::throwing::LEAP_SPLASH));
        // A rock (lift entry 2 -> sprite 265) breaks into the rock pieces.
        let mut s = object_session(1);
        face_up_at(&mut s, 40, 20);
        assert_eq!(s.interact(), Interaction::Lifted { kind: 2 });
        assert_eq!(s.interact(), Interaction::Thrown { kind: 2 });
        for _ in 0..9 {
            s.tick_throw(50);
        }
        assert_eq!(s.take_landings()[0].effect, Some(crate::throwing::LEAP_ROCK));
    }

    #[test]
    fn a_bomb_stays_where_it_lands() {
        let mut s = object_session(0);
        s.player_x = 10.0;
        s.player_y = 30.0;
        let mut o = crate::throwing::Thrown::launch(0, 10.0, 30.0, DIR_SOUTH);
        o.sprite = Some(65);
        s.thrown.push(o);
        for _ in 0..9 {
            s.tick_throw(50);
        }
        assert!(s.thrown.is_empty());
        assert!(s.leaps.is_empty(), "no break effect for a bomb");
        assert_eq!(s.grounded.len(), 1);
        assert_eq!(s.grounded[0].sprite, 65);
        assert_eq!(s.take_landings()[0].effect, None);
    }

    #[test]
    fn the_four_directions_fly_their_own_way() {
        for (dir, dx, dz) in [(DIR_NORTH, 0.0, -9.0), (DIR_WEST, -9.0, 0.0), (DIR_SOUTH, 0.0, 9.0), (DIR_EAST, 9.0, 0.0)] {
            let mut s = carrying_bush(30.0, 30.0, dir);
            s.throw_carried();
            let (x0, z0) = (s.thrown[0].x, s.thrown[0].ground);
            for _ in 0..4 {
                s.tick_throw(50);
            }
            let o = &s.thrown[0];
            assert!((o.x - x0 - dx * 4.0 / 9.0).abs() < 1e-3 && (o.ground - z0 - dz * 4.0 / 9.0).abs() < 1e-3, "dir {dir}");
        }
    }

    #[test]
    fn a_new_level_drops_flights_and_effects() {
        let mut s = carrying_bush(10.0, 30.0, DIR_EAST);
        s.throw_carried();
        s.tick_throw(50);
        s.clear_throws();
        assert!(s.thrown.is_empty() && s.leaps.is_empty() && s.take_landings().is_empty());
    }

    #[test]
    fn break_tables_come_from_arrays_dat_when_it_has_them() {
        // a minimal file: nine empty arrays, four object arrays, then a break table
        // (array 19) with one kind of one frame, and a sprite table (array 24)
        let mut data = object_arrays(&[BUSH], &[BUSH_CUT], &[BUSH], &[BUSH_CUT]);
        for _ in 13..19 {
            data.extend_from_slice(&0u32.to_be_bytes());
        }
        data.extend_from_slice(&((1u32 << 16) | 1).to_be_bytes()); // kinds
        data.extend_from_slice(&((1u32 << 16) | 1).to_be_bytes()); // frames
        data.extend_from_slice(&1u32.to_be_bytes()); // pieces
        data.extend_from_slice(&((7u32 << 16) | (3 << 8) | 0xfe).to_be_bytes());
        for _ in 20..24 {
            data.extend_from_slice(&0u32.to_be_bytes());
        }
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&((1u32 << 24) | (2 << 12) | (3 << 6) | 4).to_be_bytes());
        let t = ObjectTables::parse(&data).unwrap();
        assert_eq!(t.leaps, vec![vec![vec![(7u32 << 16) | (3 << 8) | 0xfe]]]);
        assert_eq!(crate::throwing::leap_frames(Some(&t), 0), vec![vec![(7u16, 3i8, -2i8)]]);
        assert_eq!(crate::throwing::sprite_rect(Some(&t), 0), Some((2, 4, 6, 8)));
        // ids the file lacks fall back to the built-in rect
        assert_eq!(crate::throwing::sprite_rect(Some(&t), 261), Some((0, 338, 32, 32)));
    }

}
