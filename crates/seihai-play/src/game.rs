//! The game loop both windowed clients run, with no window of its own: the
//! level the server sent, drawn flat ([`crate::render`]), with the arrows
//! walking the player over it the way `seihai-mae` walks (core's
//! `OfflineSession::try_move`, the same walls and water).
//!
//! The walk goes to the server: the game keeps the pose in a
//! [`online::SharedPose`] and core's network thread sends it once a 50 ms
//! tick (`seihai_core::client::run_with`). A link the player walks into is
//! noted and asked for, and the level it leads to is drawn when it comes
//! in. Walking plays the footstep and water sounds of `walk.gani` and
//! `swim.gani` through core's sound engine ([`Game::play_sounds`]); a
//! sound not on disk is asked of the server (`WANTFILE`).
//!
//! This was `seihai-ato`'s `game` module. It moved here unchanged, less
//! the egui view ([`crate::view`]), so that Issho runs the same loop.

mod actions;
mod life;
mod signs;
pub use actions::{ItemList, ListKey, HAS_SPIN};
pub use signs::SignKey;

use crate::chars::{self, Characters, Figure};
use crate::effects::EffectSheets;
use crate::hud::{self, HudArt};
use crate::render::{self, Frame, NpcImages, Players, Sheet};
use crate::scripts::{self, ClientScripts};
use seihai_core::assets::{
    AssetResolver, AssetRoot, Gani, GaniPlayer, Level, LISTENER_DX, LISTENER_DY,
};
use seihai_core::client::level::Arrival;
use seihai_core::client::{self as online, Live};
use seihai_core::gmap::Gmap;
use seihai_core::offline::OfflineSession;
use seihai_core::protocol::{ChatLine, ClientVersion, SessionState};
use seihai_core::sound::{self, SoundEngine};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

/// Where a level with no start position puts the player.
pub const NO_START: (f32, f32) = (30.0, 30.0);

/// What the network thread hands the window.
pub enum Event {
    Level(Box<Level>, Arrival),
    Ended(String),
}

/// The level on screen and the walk over it.
pub struct Game {
    pub session: OfflineSession,
    sheet: Sheet,
    rx: Receiver<Event>,
    pub status: String,
    last_step: Instant,
    /// The frame needs drawing again.
    dirty: bool,
    /// Where the network thread reads the player's pose.
    pub pose: online::SharedPose,
    /// Where the network thread puts the other players.
    pub others: online::SharedOthers,
    /// The other players drawn last frame.
    drawn_others: Vec<online::Other>,
    /// The players' pose GANIs and pictures.
    chars: Characters,
    /// When the game started: the clock the walking poses run on.
    started: Instant,
    /// Where the network thread takes a link request from.
    pub link: Arc<Mutex<Option<online::LinkAsk>>>,
    /// The link asked for, until its level arrives.
    link_to: Option<online::LinkAsk>,
    /// Sound effects: core's engine and device output (the `audio`
    /// feature). Silent with no sounds folder, no device, or `--no-sound`.
    sound: SoundEngine,
    sound_clock: Instant,
    /// The 50 ms pose clock (the swim stroke and its splashes, the lift):
    /// when it last ran.
    pose_clock: Instant,
    /// The level the looped sounds belong to; a new one stops them.
    sound_serial: u64,
    /// Where missing sounds go for the network thread to ask for.
    pub wanted_sounds: Arc<Mutex<Vec<String>>>,
    /// Sounds the server sent (kept in memory by the network thread), for
    /// the engine to take.
    arrived_sounds: online::SharedSounds,
    /// The asset folders, for NPC images.
    resolver: Option<AssetResolver>,
    /// NPC images read so far, by name.
    npc_images: NpcImages,
    /// Playing level files with no server (`--level`): links load the
    /// next level from the asset folder instead of asking a server.
    offline: bool,
    /// Where the network thread puts the gmap it has the player on.
    pub world: Arc<Mutex<Option<online::WorldView>>>,
    /// Where the walk's gmap crossings go for the network thread to send.
    pub crossings: Arc<Mutex<Vec<seihai_core::gmap::Crossing>>>,
    /// Where the network thread puts the server's board changes.
    pub board_mods: online::SharedBoardMods,
    /// Where the network thread puts the server's NPC updates.
    pub npc_changes: online::SharedNpcChanges,
    /// Set when the game is dropped (its window closed): the network
    /// thread logs out instead of keeping the player on the server.
    closed: Arc<std::sync::atomic::AtomicBool>,
    /// The network thread's gmap, as last taken.
    net_world: Option<online::WorldView>,
    /// Crossings handed to the network thread so far.
    crossings_sent: u64,
    /// The name of the level `session.current_level` holds (online).
    level_here: Option<String>,
    /// Where the network thread puts the player's own counters.
    pub own: online::SharedOwn,
    /// The counters drawn last frame.
    drawn_own: online::OwnStats,
    /// Where typed lines go for the network thread to say.
    pub say: Arc<Mutex<Vec<String>>>,
    /// Where messages the game built go for the network thread to send.
    pub send: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Where the network thread puts the chat lines that come in.
    pub chat_in: online::SharedChat,
    /// The chat lines taken so far (toalls, PMs, admin lines and what was
    /// said in the level), oldest first, with when each came.
    chat: Vec<(ChatLine, Instant)>,
    /// The player's own bubble and when it was said.
    own_bubble: Option<(String, Instant)>,
    /// The status display's pictures, when the line draws one
    /// ([`Game::show_hud`]).
    hud: Option<HudArt>,
    /// Set while a Graal3D UDP link is up: the real 4.20r6 and 5.x clients
    /// ignore the TCP level boards then (the world comes over UDP), so the
    /// flat level is not drawn.
    pub boards_hidden: Arc<std::sync::atomic::AtomicBool>,
    /// What [`Game::boards_hidden`] was on the last tick.
    hidden_drawn: bool,
    /// The server NPCs' GS2 client scripts, once [`Game::set_version`]
    /// named the build whose VM runs them.
    scripts: Option<ClientScripts>,
    /// When the scripts' clock last ran.
    script_clock: Instant,
    /// Where the network thread puts the hits on the player.
    pub hits: online::SharedHits,
    /// Health and death ([`life`]).
    life: life::LifeState,
    /// The first line of the sign page on screen ([`signs`]).
    sign_first: usize,
    /// A sign was open last frame.
    sign_was_open: bool,
    /// Draws the view while [`Game::boards_hidden`] is set (Ato's Graal3D
    /// world), given the frame's size in pixels; black without one.
    pub view3d: Option<View3d>,
    /// Takes the arrows (`dx`, `dy`) while [`Game::boards_hidden`] is set,
    /// in place of the flat walk (Ato's Graal3D walk).
    pub keys3d: Option<Keys3d>,
    /// Takes what the player says while [`Game::boards_hidden`] is set, in
    /// place of the TCP chat (Ato's Graal3D chat section).
    pub say3d: Option<Say3d>,
    /// Where the network thread puts the player's weapons.
    pub weapons: online::SharedWeapons,
    /// Where the network thread puts other players' bombs and arrows.
    pub level_objects: online::SharedLevelObjects,
    /// Where the network thread puts other players' throws.
    pub throws: online::SharedThrows,
    /// The spin, the weapon key, the lay mode and the item list
    /// ([`actions`]).
    actions: actions::Actions,
    /// The sheets bombs, arrows and floor items are cut from.
    effect_sheets: EffectSheets,
}

/// [`Game::keys3d`].
pub type Keys3d = Box<dyn Fn(i8, i8) + Send>;

/// [`Game::say3d`].
pub type Say3d = Box<dyn Fn(&str) + Send>;

/// How many chat lines the game keeps.
pub const CHAT_KEEP: usize = 100;

/// How long the player's own bubble stays: core's bubble time.
pub const OWN_BUBBLE: std::time::Duration =
    std::time::Duration::from_millis(seihai_core::chatlists::BUBBLE_MS as u64);

/// [`Game::view3d`].
pub type View3d = Box<dyn Fn(u32, u32) -> Frame + Send>;

/// `walk.gani` and `swim.gani` into the session: their `PLAYSOUND` cues
/// are the footsteps and the water, as in `seihai-mae`.
fn load_motion_ganis(resolver: &AssetResolver, session: &mut OfflineSession) {
    for name in ["walk.gani", "swim.gani"] {
        let Some(path) = resolver.pose_gani(name) else {
            continue;
        };
        match Gani::load(&path) {
            Ok(g) if name == "swim.gani" => session.set_swim_gani(GaniPlayer::new(g)),
            Ok(g) => session.set_walk_gani(GaniPlayer::new(g)),
            Err(e) => eprintln!("sound: cannot read {name}: {e}"),
        }
    }
}

/// The pose GANIs core plays for the player (the sword swing and the spin,
/// hurt, death): from the asset folders, else the built-in ones.
fn load_pose_ganis(resolver: Option<&AssetResolver>, session: &mut OfflineSession) {
    for name in [
        "sword.gani",
        "spin.gani",
        "hurt.gani",
        "dead.gani",
        "idle.gani",
    ] {
        let Some(kind) = seihai_core::pose::PoseKind::from_file(name) else {
            continue;
        };
        let from_disk = resolver
            .and_then(|r| r.pose_gani(name))
            .and_then(|p| Gani::load(p).ok());
        let gani = from_disk.or_else(|| {
            let text = seihai_core::assets::builtin::bytes(&format!("gani/{name}"))?;
            Gani::parse(&String::from_utf8_lossy(text)).ok()
        });
        if let Some(g) = gani {
            session.set_pose_gani(kind, g);
        }
    }
}

/// The cut and lift tables, from the build's `arrays.dat` where it has
/// one. Without them the sword cuts nothing.
fn load_object_tables(resolver: &AssetResolver, session: &mut OfflineSession) {
    if let Some((p, seihai_core::assets::TileTypesFile::ArraysDat)) = resolver.tile_types() {
        session.object_tables = std::fs::read(p)
            .ok()
            .and_then(|d| seihai_core::assets::ObjectTables::parse(&d));
    }
}

/// The sound engine for this run: core's default backend over the asset
/// folders' sounds, or off.
fn sound_engine(resolver: Option<&AssetResolver>, sound_on: bool) -> SoundEngine {
    let Some(r) = resolver.filter(|_| sound_on) else {
        return SoundEngine::off();
    };
    let dirs = r.sound_dirs();
    if dirs.is_empty() {
        eprintln!("sound: no sounds folder; sound is silent");
        return SoundEngine::off();
    }
    let (out, why) = sound::default_backend();
    let list: Vec<String> = dirs.iter().map(|d| d.display().to_string()).collect();
    eprintln!("sound: {why}; sounds from {}", list.join(", "));
    SoundEngine::with_dirs(out, dirs, true)
}

impl Game {
    pub fn new(
        rx: Receiver<Event>,
        resolver: Option<&AssetResolver>,
        sound_on: bool,
    ) -> Result<Game, String> {
        let mut session = OfflineSession::default();
        session.tiletypes = render::tile_types(resolver);
        if let Some(r) = resolver {
            load_motion_ganis(r, &mut session);
            load_object_tables(r, &mut session);
        }
        load_pose_ganis(resolver, &mut session);
        Ok(Game {
            session,
            sheet: Sheet::load(resolver)?,
            rx,
            status: "Waiting for the level.".into(),
            last_step: Instant::now(),
            dirty: true,
            pose: online::SharedPose::default(),
            others: online::SharedOthers::default(),
            drawn_others: Vec::new(),
            chars: Characters::load(resolver),
            started: Instant::now(),
            link: Default::default(),
            link_to: None,
            sound: sound_engine(resolver, sound_on),
            sound_clock: Instant::now(),
            pose_clock: Instant::now(),
            sound_serial: 0,
            wanted_sounds: Default::default(),
            arrived_sounds: Default::default(),
            resolver: resolver.cloned(),
            npc_images: NpcImages::new(),
            offline: false,
            world: Default::default(),
            crossings: Default::default(),
            board_mods: Default::default(),
            npc_changes: Default::default(),
            closed: Default::default(),
            net_world: None,
            crossings_sent: 0,
            level_here: None,
            own: Default::default(),
            drawn_own: Default::default(),
            say: Default::default(),
            send: Default::default(),
            chat_in: Default::default(),
            chat: Vec::new(),
            own_bubble: None,
            scripts: None,
            script_clock: Instant::now(),
            hits: Default::default(),
            life: Default::default(),
            sign_first: 0,
            sign_was_open: false,
            hud: None,
            boards_hidden: Default::default(),
            hidden_drawn: false,
            view3d: None,
            keys3d: None,
            say3d: None,
            weapons: Default::default(),
            level_objects: Default::default(),
            throws: Default::default(),
            actions: actions::Actions::new(),
            effect_sheets: EffectSheets::load(resolver),
        })
    }

    /// A game with no server: level `name` from the asset folder (or the
    /// path `name` with no asset folder), and links that load their level
    /// the same way, as `seihai-mae --level` plays offline.
    pub fn open_offline(
        resolver: Option<&AssetResolver>,
        name: &str,
        sound_on: bool,
    ) -> Result<Game, String> {
        // No network thread: the channel's sender is dropped at once.
        let (_, rx) = mpsc::channel();
        let mut g = Game::new(rx, resolver, sound_on)?;
        g.offline = true;
        g.enter_offline(name, None)?;
        Ok(g)
    }

    /// Find level or gmap `name` offline: in the asset folder, or as a path
    /// when there is none.
    fn offline_file(&self, name: &str) -> Option<PathBuf> {
        match &self.resolver {
            Some(r) => r.level(name),
            None => Some(PathBuf::from(name)).filter(|p| p.is_file()),
        }
    }

    /// Open gmap `name` offline. `at` is a position on the whole map in
    /// tiles (a link into the gmap); without it the player starts at the
    /// default start of the first cell that has a level.
    fn enter_offline_gmap(&mut self, name: &str, at: Option<(f32, f32)>) -> Result<(), String> {
        let path = self
            .offline_file(name)
            .ok_or_else(|| "not in the asset folder".to_string())?;
        let text = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let map = Gmap::parse(&String::from_utf8_lossy(&text))
            .ok_or_else(|| format!("{}: not a gmap", path.display()))?;
        let (cell, local) = match at {
            Some((x, y)) => {
                let cell = ((x / 64.0).floor() as i32, (y / 64.0).floor() as i32);
                (cell, (x - cell.0 as f32 * 64.0, y - cell.1 as f32 * 64.0))
            }
            None => {
                let first = (0..map.height as i32)
                    .flat_map(|cy| (0..map.width as i32).map(move |cx| (cx, cy)))
                    .find(|&(cx, cy)| map.level_at(cx, cy).is_some())
                    .ok_or_else(|| format!("{name}: the gmap names no level"))?;
                (first, NO_START)
            }
        };
        let level = map
            .level_at(cell.0, cell.1)
            .ok_or_else(|| format!("{name}: no level at cell {}, {}", cell.0, cell.1))?
            .to_string();
        let path = self
            .offline_file(&level)
            .ok_or_else(|| format!("{level}: not in the asset folder"))?;
        self.session
            .load_level(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        self.session.enter_gmap(name.to_string(), map, cell);
        self.session.note_entry(name);
        self.session.arrive(local.0, local.1);
        self.load_gmap_neighbours();
        self.level_entered();
        Ok(())
    }

    /// On a gmap, keep only the levels next to the player's cell and load
    /// the missing ones from the asset folder, so the view draws them past
    /// the edges and core's walk can cross into them. A neighbour that is
    /// not on disk stays missing, and its edge stays a wall.
    fn load_gmap_neighbours(&mut self) {
        let Some(w) = self.session.gmap.as_ref() else {
            return;
        };
        let around = w.map.neighbours(w.cell);
        let wanted: Vec<((i32, i32), String)> = w
            .missing()
            .into_iter()
            .filter_map(|c| Some((c, w.map.level_at(c.0, c.1)?.to_string())))
            .collect();
        let loaded: Vec<((i32, i32), Level)> = wanted
            .into_iter()
            .filter_map(|(c, n)| Some((c, Level::load(self.offline_file(&n)?).ok()?)))
            .collect();
        let w = self.session.gmap.as_mut().expect("checked above");
        w.levels.retain(|c, _| around.contains(c));
        w.levels.extend(loaded);
        let neighbours: Vec<Level> = w.levels.values().cloned().collect();
        for level in &neighbours {
            render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images);
        }
    }

    /// After a level (or gmap cell) became the current one offline: its
    /// NPC images, and the status line.
    fn level_entered(&mut self) {
        if let Some(level) = &self.session.current_level {
            for missing in
                render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images)
            {
                eprintln!("npc image {missing}: not in the asset folders; outlined");
            }
        }
        let here = self.session.level_name().unwrap_or_default();
        self.status = match &self.session.gmap {
            Some(w) => format!("{here} on {} (offline)", w.name),
            None => format!("{here} (offline)"),
        };
        self.dirty = true;
    }

    /// Load level `name` offline and stand at `at` (the link's arrival), or
    /// at the default start. A `.gmap` name opens the gmap.
    fn enter_offline(&mut self, name: &str, at: Option<(f32, f32)>) -> Result<(), String> {
        if name.to_ascii_lowercase().ends_with(".gmap") {
            return self.enter_offline_gmap(name, at);
        }
        let path = self.offline_file(name);
        // Checked first: core's `load_level` names the session after the
        // file before reading it, so a failed read would rename the level
        // the player is still on.
        let Some(path) = path else {
            return Err("not in the asset folder".into());
        };
        self.session
            .load_level(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        self.session.leave_gmap();
        self.session.note_entry(name);
        let at = at.unwrap_or(NO_START);
        self.session.arrive(at.0, at.1);
        self.level_entered();
        Ok(())
    }

    /// A level named `name` from the server is the one the view already
    /// holds on a gmap: the cell the walk crossed into (and not a link).
    fn same_gmap_level(&self, name: &str) -> bool {
        self.session.gmap.is_some()
            && self.link_to.is_none()
            && self
                .level_here
                .as_deref()
                .is_some_and(|h| h.eq_ignore_ascii_case(name))
    }

    /// The server's copy of the level the view is on: its board and NPCs
    /// replace the neighbour copy the walk crossed into.
    fn refresh_level(&mut self, level: Level, a: &Arrival) {
        self.session.current_level = Some(level);
        for (id, p) in &a.server_npc_props {
            self.session.take_server_npc(*id, p);
        }
        // The scripts already running go on; new ones start.
        self.start_scripts(a);
        if let Some(level) = &self.session.current_level {
            for name in render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images)
            {
                eprintln!("npc image {name}: not in the asset folders; outlined");
            }
        }
        self.status = a.line();
    }

    /// A new level from the server while the view is on a gmap: a level
    /// of the gmap moves the view to its cell (a warp inside the gmap), any
    /// other leaves the gmap.
    fn gmap_cell_for(&mut self, name: &str) {
        let Some(w) = self.session.gmap.as_mut() else {
            return;
        };
        match w.map.cell_of(name) {
            Some(c) if c != w.cell => {
                w.cell = c;
                w.levels.clear();
            }
            Some(_) => {}
            None => self.session.leave_gmap(),
        }
    }

    /// The walk crossed a gmap edge: hand the crossing to the network
    /// thread, and note the level the view is on now.
    fn crossed(&mut self, c: seihai_core::gmap::Crossing) {
        self.crossings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(c);
        self.crossings_sent += 1;
        self.level_here = self.session.level_name();
        if let Some(level) = &self.session.current_level {
            render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images);
        }
        self.apply_world();
    }

    /// Bring the view's gmap in line with the network thread's: enter it
    /// once the view holds the level of its cell, take a server move to
    /// another cell, and fill in the neighbours the view is missing. While
    /// the network thread has not sent all of the view's crossings its
    /// cell is behind, so only its levels are used.
    fn apply_world(&mut self) {
        let Some(v) = self.net_world.as_ref() else {
            return;
        };
        let holds = |cell: (i32, i32)| {
            let here = self.level_here.as_deref();
            v.map
                .level_at(cell.0, cell.1)
                .is_some_and(|n| here.is_some_and(|h| h.eq_ignore_ascii_case(n)))
        };
        let caught_up = v.crossings >= self.crossings_sent;
        match self.session.gmap.as_mut() {
            Some(w) if w.name.eq_ignore_ascii_case(&v.name) => {
                if caught_up && w.cell != v.cell {
                    if !holds(v.cell) {
                        // The server moved the player: wait for the level.
                        return;
                    }
                    w.cell = v.cell;
                    w.levels.clear();
                }
            }
            _ => {
                if !holds(v.cell) {
                    return;
                }
                self.session
                    .enter_gmap(v.name.clone(), v.map.clone(), v.cell);
            }
        }
        let w = self.session.gmap.as_mut().expect("entered above");
        let around = w.map.neighbours(w.cell);
        w.levels.retain(|c, _| around.contains(c));
        let mut added = Vec::new();
        for (c, l) in &v.levels {
            if *c != w.cell && around.contains(c) && !w.levels.contains_key(c) {
                w.levels.insert(*c, l.clone());
                added.push(*c);
            }
        }
        for c in added {
            if let Some(level) = w.levels.get(&c) {
                render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images);
            }
        }
        self.dirty = true;
    }

    /// Play the sounds the session queued: the GANI cues of the steps just
    /// taken. A new level stops the looped ones. A sound not on disk is
    /// handed to the network thread to ask the server for. The file comes
    /// back through [`Live::arrived_sounds`] (kept in memory, not saved),
    /// goes into the engine, and plays on the next cue.
    pub fn play_sounds(&mut self) {
        let arrived = std::mem::take(
            &mut *self
                .arrived_sounds
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
        );
        let on = self.sound.is_enabled();
        for (name, bytes) in arrived.into_iter().filter(|_| on) {
            if self.sound.add(&name, &bytes).is_ok() {
                eprintln!("sound: {name} arrived");
            }
        }
        if !self.sound.is_enabled() {
            self.session.sound_events.clear();
            self.session.sound_stops.clear();
            return;
        }
        if self.session.level_serial != self.sound_serial {
            self.sound_serial = self.session.level_serial;
            self.sound.stop_all_loops();
        }
        let now = self.sound_clock.elapsed().as_millis() as u64;
        let listener = (
            self.session.player_x + LISTENER_DX,
            self.session.player_y + LISTENER_DY,
        );
        for name in self.session.take_sound_stops() {
            self.sound.stop(&name);
        }
        for ev in self.session.take_sound_events() {
            if self.sound.play(&ev, listener, now) == sound::Outcome::Missing {
                self.wanted_sounds
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(sound::wanted_name(&ev.file));
            }
        }
    }

    /// Take what the network thread sent.
    pub fn poll(&mut self) {
        let taken = self.world.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(v) = taken {
            self.net_world = Some(v);
            self.apply_world();
        }
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Level(level, a) => {
                    if self.same_gmap_level(&a.name) {
                        // The server's copy of the cell the walk crossed
                        // into: the player stays where the walk put them
                        // (the server sends no position with it).
                        self.refresh_level(*level, &a);
                        self.apply_world();
                        self.dirty = true;
                        continue;
                    }
                    self.gmap_cell_for(&a.name);
                    self.session.current_level = Some(*level);
                    self.session.level_serial += 1;
                    // The server's start position; after a link, where the
                    // link said to arrive; else the default.
                    let at = a
                        .start
                        .or(self.link_to.take().map(|l| (l.x, l.y)))
                        .unwrap_or(NO_START);
                    self.session.arrive(at.0, at.1);
                    // The server's NPCs for this level join the file's,
                    // through core's linking (a file NPC with the same
                    // image and place is the same NPC).
                    for (id, p) in &a.server_npc_props {
                        self.session.take_server_npc(*id, p);
                    }
                    if let Some(sc) = self.scripts.as_mut() {
                        sc.clear();
                    }
                    self.start_scripts(&a);
                    if let Some(level) = &self.session.current_level {
                        for name in render::load_npc_images(
                            level,
                            self.resolver.as_ref(),
                            &mut self.npc_images,
                        ) {
                            eprintln!("npc image {name}: not in the asset folders; outlined");
                        }
                    }
                    self.status = a.line();
                    self.level_here = Some(a.name.clone());
                    self.set_pose(false);
                    self.apply_world();
                }
                Event::Ended(why) => self.status = why,
            }
            self.dirty = true;
        }
        // The server's board changes, after the levels they may belong to.
        // A change is a plain write, so one the level already had is
        // harmless.
        let mods = std::mem::take(&mut *self.board_mods.lock().unwrap_or_else(|e| e.into_inner()));
        if !mods.is_empty() {
            seihai_core::gmap::apply_board_mods_to(
                &mut self.session,
                self.level_here.as_deref(),
                &mods,
            );
            self.dirty = true;
        }
        self.apply_npc_changes();
    }

    /// The server's NPC updates since the level arrived (`PLO_NPCPROPS`),
    /// onto the NPCs of the level the view is on, as core links them (a
    /// record for an NPC the level does not have yet needs its image and
    /// place), and its deletes (`PLO_NPCDEL2`), which take the NPC off. An update for a level the view has left is dropped: the
    /// server's NPCs come again with the level. The arrival's records hold
    /// every update up to it, so an update the arrival already had is
    /// applied twice in order and ends the same.
    fn apply_npc_changes(&mut self) {
        let changes =
            std::mem::take(&mut *self.npc_changes.lock().unwrap_or_else(|e| e.into_inner()));
        let Some(here) = self.level_here.as_deref() else {
            return;
        };
        let mut applied = false;
        for c in changes
            .iter()
            .filter(|c| c.level.eq_ignore_ascii_case(here))
        {
            match &c.props {
                Some(p) => {
                    self.session.take_server_npc(c.id, p);
                }
                None => {
                    self.session.drop_server_npc(c.id);
                }
            }
            applied = true;
        }
        if !applied {
            return;
        }
        if let Some(level) = &self.session.current_level {
            for name in render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images)
            {
                eprintln!("npc image {name}: not in the asset folders; outlined");
            }
        }
        self.dirty = true;
    }

    /// Hand the current pose to the network thread.
    fn set_pose(&self, walking: bool) {
        let s = &self.session;
        // The GANI the others see: the swing or spin, the lift, carrying.
        let anim = s.bow_slot_anim();
        *self.pose.lock().unwrap_or_else(|e| e.into_inner()) = Some(online::Pose {
            x: s.player_x,
            y: s.player_y,
            dir: s.player_dir % 4,
            walking,
            sprite: Some(s.sprite_value()),
            gani: (anim != "idle").then_some(anim),
        });
    }

    /// Take the build's rules: combat (the swing and being hit), death,
    /// the knockback and how the build shows a swing to the others (its
    /// GANI prop, or the sprite), as `seihai-mae` sets them.
    pub fn set_version(&mut self, version: ClientVersion) {
        let p = version.profile();
        self.scripts = Some(ClientScripts::new(version));
        self.life.version = Some(version);
        self.session.life.set_rules(p.life.death);
        self.session.combat.set_rules(p.life.combat);
        self.session.set_mannerisms(p.mannerisms);
        self.session.walk_sprite = !p.ui.ganis;
        self.session.gani_swing = matches!(version, ClientVersion::V222 | ClientVersion::V304)
            || seihai_core::protocol::lobby::LobbyBurst::for_version(version).is_some();
        self.set_action_version(version);
    }

    /// Swing the sword (the sword key). The swing cuts what is in front of
    /// it (core's `OfflineSession::try_cut`); online the cut goes to the
    /// server as a board change and the swing as the player's GANI or
    /// sprite. False when no swing started: no combat rules for the build,
    /// no sword, in water, or already swinging.
    pub fn swing(&mut self) -> bool {
        if self.list_open() {
            return false;
        }
        // The sword key while dead respawns, once the death allows it.
        if self.session.life.is_dead() {
            self.respawn();
            return false;
        }
        // S while carrying throws what is carried (2.05).
        if self.session.carried.is_some() && self.session.current_level.is_some() {
            self.interact();
            return false;
        }
        // S in the lay mode leaves it.
        if self.session.stop_laying() {
            self.dirty = true;
            return false;
        }
        if self.session.current_level.is_none() || !self.session.start_swing() {
            return false;
        }
        self.send_board_changes();
        self.set_pose(self.session.walking);
        self.dirty = true;
        true
    }

    /// The board changes the player made (a cut): to the server online,
    /// as `PLI_BOARDMODIFY`. Offline the level already holds them.
    fn send_board_changes(&mut self) {
        let changes = self.session.take_board_changes();
        if self.offline || changes.is_empty() {
            return;
        }
        let in_range = |v: i32| u8::try_from(v).ok().filter(|v| *v < 64);
        let msgs: Vec<Vec<u8>> = changes
            .iter()
            .filter_map(|c| {
                Some(seihai_core::net::board_modify_message(
                    in_range(c.x)?,
                    in_range(c.y)?,
                    u8::try_from(c.width).ok()?,
                    u8::try_from(c.height).ok()?,
                    &c.tiles,
                ))
            })
            .collect();
        self.send
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(msgs);
    }

    /// The player's position in tiles, for the view, once a level is in.
    pub fn position(&self) -> Option<String> {
        self.session.current_level.as_ref()?;
        Some(format!(
            "{:.1}, {:.1}",
            self.session.player_x, self.session.player_y
        ))
    }

    /// Runs core's pose clock up to `now`: the swim stroke goes on in the
    /// water, moving or not, and queues its `PLAYSOUND` cues (the splash
    /// of `swim.gani`) as each step is entered, as `seihai-mae` does.
    pub fn tick_poses(&mut self, now: Instant) {
        let ms = now.saturating_duration_since(self.pose_clock).as_millis() as u64;
        if ms == 0 {
            return;
        }
        self.pose_clock += std::time::Duration::from_millis(ms);
        if self.session.tick_poses(ms) {
            self.dirty = true;
        }
        // A ledge jump in the air, a pull that brought a block along, or a
        // hit's knockback: the new position goes out like a step.
        let moved = self.session.tick_pushpull(ms);
        if (moved | self.session.take_knock_moved()) && self.session.current_level.is_some() {
            self.set_pose(self.session.walking);
            self.dirty = true;
        }
        if self.session.jumping() || self.session.pushpull.pull_pose() {
            self.dirty = true;
        }
        // The bombs, arrows, floor items and weapon scripts too.
        if self.session.current_level.is_some() && self.tick_actions(ms) {
            self.dirty = true;
        }
        // The swing and the spin run on the same clock; their end goes to
        // the server.
        let swinging =
            self.session.combat.is_swinging() || self.session.combat.spin_elapsed_ms().is_some();
        if self.session.tick_combat(ms)
            || swinging
                != (self.session.combat.is_swinging()
                    || self.session.combat.spin_elapsed_ms().is_some())
        {
            self.dirty = true;
            if self.session.current_level.is_some() {
                self.set_pose(self.session.walking);
            }
        }
    }

    /// One walk step for the arrows held, at the session's pace. `(dx, dy)`
    /// are -1, 0 or 1 per axis.
    pub fn walk(&mut self, dx: i8, dy: i8, now: Instant) {
        // The open item list takes the arrows.
        let (dx, dy) = if self.list_open() { (0, 0) } else { (dx, dy) };
        // An arrow in the lay mode leaves it without a step.
        if (dx, dy) != (0, 0) && self.session.stop_laying() {
            self.dirty = true;
            return;
        }
        if (dx, dy) == (0, 0) {
            if self.session.stop_walking() {
                self.dirty = true;
                if self.session.current_level.is_some() {
                    self.set_pose(false);
                }
            }
            return;
        }
        if self.session.current_level.is_none()
            || now.duration_since(self.last_step) < self.session.step_interval()
        {
            return;
        }
        self.last_step = now;
        self.session.try_move(dx as f32 * 0.5, dy as f32 * 0.5);
        // Offline on a gmap: core already moved the player into the next
        // cell; its neighbours change with it.
        if self.offline && self.session.take_gmap_crossing().is_some() {
            self.load_gmap_neighbours();
            self.level_entered();
        }
        // Online: the network thread sends the crossing, and the server
        // follows with the new cell's level.
        if !self.offline {
            if let Some(c) = self.session.take_gmap_crossing() {
                self.crossed(c);
            }
        }
        self.set_pose(true);
        // A link the feet are now in: ask the server once and wait for the
        // level (no walking meanwhile, core's warp-pending state).
        if self.session.check_links() && self.follow_link() {
            return;
        }
        self.dirty = true;
    }

    /// Go where `session.pending_link` says (a link the feet are in, or a
    /// script's `setlevel`): offline the level loads now; online the
    /// server is asked once, and the walk waits for the level. True when
    /// an offline level was entered (or failed to load).
    fn follow_link(&mut self) -> bool {
        let Some((dest, x, y)) = self.session.pending_link.take() else {
            return false;
        };
        if self.offline {
            if let Err(e) = self.enter_offline(&dest, Some((x, y))) {
                self.status = format!("Link to {dest}: {e}");
            }
            return true;
        }
        self.ask_link(&dest, x, y, true);
        false
    }

    /// Ask the server for level `dest` at `(x, y)`. With `warp` the walk
    /// waits for the level (core's warp-pending state); without it (a
    /// respawn on the level the player is on, where the server only moves
    /// the player) it does not.
    fn ask_link(&mut self, dest: &str, x: f32, y: f32, warp: bool) {
        let ask = online::LinkAsk {
            dest: dest.to_string(),
            x,
            y,
            dir: self.session.player_dir % 4,
        };
        *self.link.lock().unwrap_or_else(|e| e.into_inner()) = Some(ask.clone());
        if warp {
            self.session.begin_warp(dest);
            self.link_to = Some(ask);
            self.status = format!("Link to {dest}: asking the server.");
        }
        self.dirty = true;
    }

    /// Start the GS2 scripts a level brought (with a build set).
    fn start_scripts(&mut self, a: &Arrival) {
        let Some(sc) = self.scripts.as_mut() else {
            return;
        };
        if sc.start(&mut self.session, &a.scripts) > 0 {
            self.script_clock = Instant::now();
        }
        self.after_scripts();
    }

    /// Run the GS2 scripts' clock up to `now`.
    pub fn tick_scripts(&mut self, now: Instant) {
        let Some(sc) = self.scripts.as_mut() else {
            return;
        };
        let ms = now.saturating_duration_since(self.script_clock).as_millis() as u64;
        if ms < scripts::FRAME_MS {
            return;
        }
        self.script_clock = now;
        if sc.is_empty() || self.session.current_level.is_none() {
            return;
        }
        sc.tick(&mut self.session, ms);
        self.after_scripts();
        self.dirty = true;
    }

    /// What the scripts' effects ask of the game: the server messages for
    /// the flags they changed, new NPC images, and a `setlevel`.
    fn after_scripts(&mut self) {
        let Some(sc) = self.scripts.as_mut() else {
            return;
        };
        let msgs = sc.take_server_messages(&mut self.session);
        for line in sc.log.drain(..) {
            eprintln!("gs2: {line}");
        }
        let images = std::mem::take(&mut sc.host.images_changed);
        if !msgs.is_empty() && !self.offline {
            self.send
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend(msgs);
        }
        if images {
            if let Some(level) = &self.session.current_level {
                render::load_npc_images(level, self.resolver.as_ref(), &mut self.npc_images);
            }
        }
        if self.session.pending_link.is_some() {
            self.follow_link();
        }
    }

    /// The network thread's shared state: hand [`Live`] to
    /// `seihai_core::client::run_with` and keep its other half here.
    pub fn attach(&mut self, live: Live) {
        self.pose = live.pose;
        self.others = live.others;
        self.link = live.link;
        self.wanted_sounds = live.wanted_sounds;
        self.arrived_sounds = live.arrived_sounds;
        self.world = live.world;
        self.crossings = live.crossings;
        self.board_mods = live.board_mods;
        self.npc_changes = live.npc_changes;
        self.closed = live.closed;
        self.own = live.own;
        self.say = live.say;
        self.send = live.send;
        self.chat_in = live.chat;
        self.hits = live.hits;
        self.weapons = live.weapons;
        self.level_objects = live.level_objects;
        self.throws = live.throws;
    }

    /// Draw the status display over the level (hearts, gralats, bombs,
    /// arrows, AP and the item slots), from the build's `state.png`.
    pub fn show_hud(&mut self) {
        self.hud = Some(HudArt::load(self.resolver.as_ref()));
        self.dirty = true;
    }

    /// True when the status display is drawn.
    pub fn has_hud(&self) -> bool {
        self.hud.is_some()
    }

    /// Say `text` in the level: the network thread sends it, and the
    /// player's own bubble shows it. Blank text says nothing.
    pub fn say(&mut self, text: &str, now: Instant) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if let (Some(s), true) = (&self.say3d, self.boards_hidden()) {
            s(text);
            self.own_bubble = Some((text.to_string(), now));
            return;
        }
        self.say
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(text.to_string());
        self.own_bubble = Some((text.to_string(), now));
        if let Some(sc) = self.scripts.as_mut() {
            sc.player_chats(&mut self.session);
            self.after_scripts();
        }
    }

    /// The player's own bubble, while it lasts.
    pub fn own_bubble(&self, now: Instant) -> Option<&str> {
        self.own_bubble
            .as_ref()
            .filter(|(_, at)| now.saturating_duration_since(*at) < OWN_BUBBLE)
            .map(|(t, _)| t.as_str())
    }

    /// The chat lines taken so far, oldest first, with when each came.
    pub fn chat(&self) -> &[(ChatLine, Instant)] {
        &self.chat
    }

    /// Take the chat lines and the counters the network thread put out.
    /// True when either changed.
    fn take_chat_and_counters(&mut self, now: Instant) -> bool {
        let lines = std::mem::take(&mut *self.chat_in.lock().unwrap_or_else(|e| e.into_inner()));
        let mut changed = !lines.is_empty();
        self.chat.extend(lines.into_iter().map(|l| (l, now)));
        if self.chat.len() > CHAT_KEEP {
            let extra = self.chat.len() - CHAT_KEEP;
            self.chat.drain(..extra);
        }
        let own = *self.own.lock().unwrap_or_else(|e| e.into_inner());
        if own != self.drawn_own {
            let was = self.drawn_own;
            self.take_own_counters(&was, &own);
            self.take_own_power(&was, &own);
            self.drawn_own = own;
            changed |= self.hud.is_some();
        }
        changed |= self.take_weapons();
        changed |= self.take_others_actions();
        changed
    }

    /// One turn of the loop: take what the network thread sent, walk one
    /// step for the arrows held (`dx`, `dy` are -1, 0 or 1), play the
    /// sounds the step queued, and pick up the other players. True when
    /// the picture changed and wants drawing again.
    pub fn tick(&mut self, dx: i8, dy: i8, now: Instant) -> bool {
        self.poll();
        let hidden = self.boards_hidden();
        if hidden != self.hidden_drawn {
            self.hidden_drawn = hidden;
            if hidden {
                self.status = "Graal3D: the world comes over the 3D link.".into();
            }
            self.dirty = true;
        }
        // The 3D world moves on its own thread: draw every frame.
        if hidden && self.view3d.is_some() {
            self.dirty = true;
        }
        match (&self.keys3d, hidden) {
            (Some(k), true) => k(dx, dy),
            _ => self.walk(dx, dy, now),
        }
        if (dx, dy) != (0, 0) && self.session.life.is_dead() {
            self.respawn();
        }
        self.tick_poses(now);
        self.tick_scripts(now);
        if self.take_hits() {
            self.dirty = true;
        }
        self.tick_life(now);
        self.notice_sign();
        self.play_sounds();
        if self.take_chat_and_counters(now) {
            self.dirty = true;
        }
        let others = self
            .others
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if others != self.drawn_others {
            self.drawn_others = others;
            self.dirty = true;
        }
        // The pictures the players are drawn with, read once each.
        let mut figures = self.other_figures();
        figures.push(render::own_figure(&self.session));
        if self.chars.load_pictures(self.resolver.as_ref(), &figures) {
            self.dirty = true;
        }
        // A pose that runs on the clock (another player walking, the swim
        // stroke) needs every frame drawn.
        if figures.iter().any(|f| f.step == chars::Step::Clock) {
            self.dirty = true;
        }
        std::mem::take(&mut self.dirty)
    }

    fn other_figures(&self) -> Vec<Figure> {
        self.drawn_others.iter().map(chars::other_figure).collect()
    }

    /// The level around the player, `view` tiles wide and high, with the
    /// other players as last taken.
    pub fn draw(&self, view: (u32, u32)) -> Frame {
        if self.boards_hidden() {
            if let Some(v) = &self.view3d {
                return v(view.0 * render::TILE_PX, view.1 * render::TILE_PX);
            }
            return Frame::filled(
                view.0 * render::TILE_PX,
                view.1 * render::TILE_PX,
                [0, 0, 0, 255],
            );
        }
        let others = self.other_figures();
        let players = Players {
            chars: &self.chars,
            others: &others,
            clock_ms: self.started.elapsed().as_millis() as u64,
            effects: Some(&self.effect_sheets),
        };
        let mut f = render::draw(
            &self.session,
            &self.sheet,
            &self.npc_images,
            players,
            view.0,
            view.1,
        );
        if let Some(art) = &self.hud {
            hud::draw(&mut f, art, &self.hud_values());
        }
        f
    }

    /// True while a Graal3D link is up and the TCP boards are not drawn.
    pub fn boards_hidden(&self) -> bool {
        self.boards_hidden
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// The other players as last taken.
    pub fn others(&self) -> &[online::Other] {
        &self.drawn_others
    }

    /// True when sound effects reach a device.
    pub fn sound_on(&self) -> bool {
        self.sound.is_enabled()
    }
}

/// Where each named other player's name goes, in view pixels: centred
/// under the 48x48 box. Players with no nickname get no tag.
pub fn name_tags(others: &[online::Other], cam_x: i32, cam_y: i32) -> Vec<(String, f32, f32)> {
    let t = render::TILE_PX as f32;
    others
        .iter()
        .filter_map(|o| {
            let nick = o.nick.as_deref()?.trim();
            (!nick.is_empty()).then(|| {
                (
                    nick.to_string(),
                    (o.x + 1.5) * t - cam_x as f32,
                    (o.y + 3.0) * t - cam_y as f32,
                )
            })
        })
        .collect()
}

/// Where each bubble goes, in view pixels: centred over the 48x48 box of
/// the player who said it, as the clients draw chat over the head.
pub fn bubbles(others: &[online::Other], cam_x: i32, cam_y: i32) -> Vec<(String, f32, f32)> {
    others
        .iter()
        .filter_map(|o| {
            let text = o.chat.as_deref()?.trim();
            (!text.is_empty()).then(|| {
                let (x, y) = bubble_at(o.x, o.y, cam_x, cam_y);
                (text.to_string(), x, y)
            })
        })
        .collect()
}

/// The messages of the level's visible NPCs (`message` in a script, or
/// the server's), as bubbles over each NPC, in view pixels.
pub fn npc_bubbles(level: &Level, cam_x: i32, cam_y: i32) -> Vec<(String, f32, f32)> {
    let t = render::TILE_PX as f32;
    level
        .npcs
        .iter()
        .filter(|n| n.visible && !n.message.trim().is_empty())
        .map(|n| {
            (
                n.message.trim().to_string(),
                (n.x + 1.0) * t - cam_x as f32,
                n.y * t - cam_y as f32,
            )
        })
        .collect()
}

/// Where a bubble for a player at `(x, y)` tiles goes in view pixels: the
/// middle of the top of the 48x48 box.
pub fn bubble_at(x: f32, y: f32, cam_x: i32, cam_y: i32) -> (f32, f32) {
    let t = render::TILE_PX as f32;
    ((x + 1.5) * t - cam_x as f32, y * t - cam_y as f32)
}

/// The asset folders for `version`, when `--assets` named one.
pub fn resolver(assets: Option<&PathBuf>, version: ClientVersion) -> Option<AssetResolver> {
    assets.map(|d| AssetResolver::new(AssetRoot::new(d.clone()), version.profile()))
}

/// The log lines a client adds under each level (Ato's Torque fields);
/// [`no_lines`] for none.
pub type LevelLines = fn(ClientVersion, &Level, &str, &SessionState) -> Vec<String>;

/// No extra log lines.
pub fn no_lines(_: ClientVersion, _: &Level, _: &str, _: &SessionState) -> Vec<String> {
    Vec::new()
}

impl Drop for Game {
    /// The window is gone: tell the network thread to log out.
    fn drop(&mut self) {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Log in at `addr` as `version` on a thread of its own (core's client
/// session, logging to stdout) and return what the [`Game`] takes:
/// the levels as [`Event`]s and the [`Live`] state to [`Game::attach`].
/// `wake` runs after every event, so a window can repaint.
pub fn spawn_online(
    addr: String,
    version: ClientVersion,
    login: online::Login,
    assets: Option<PathBuf>,
    level_lines: LevelLines,
    wake: impl Fn() + Clone + Send + 'static,
) -> (Receiver<Event>, Live) {
    spawn_online_hooked(
        addr,
        version,
        login,
        assets,
        level_lines,
        online::Hooks::default(),
        || {},
        wake,
    )
}

/// As [`spawn_online`], with the client line's session [`online::Hooks`]
/// (Ato's Graal3D side). `ended` runs on the network thread once the
/// session is over, before the view hears of it.
#[allow(clippy::too_many_arguments)]
pub fn spawn_online_hooked(
    addr: String,
    version: ClientVersion,
    login: online::Login,
    assets: Option<PathBuf>,
    level_lines: LevelLines,
    hooks: online::Hooks,
    ended: impl FnOnce() + Send + 'static,
    wake: impl Fn() + Clone + Send + 'static,
) -> (Receiver<Event>, Live) {
    let (tx, rx) = mpsc::channel();
    let live = Live::default();
    let net_live = live.clone();
    std::thread::spawn(move || {
        let shown_tx = tx.clone();
        let shown_wake = wake.clone();
        let mut shown = move |l: &Level, a: &Arrival| {
            let _ = shown_tx.send(Event::Level(Box::new(l.clone()), a.clone()));
            shown_wake();
        };
        let why = match online::run_hooked(
            &addr,
            version,
            &login,
            assets,
            false,
            &mut std::io::stdout(),
            &mut shown,
            Some(net_live),
            &|level, name, st| level_lines(version, level, name, st),
            &hooks,
        ) {
            Ok(()) => "The server closed the connection.".to_string(),
            Err(e) => format!("Could not log in: {e}"),
        };
        ended();
        let _ = tx.send(Event::Ended(why));
        wake();
    });
    (rx, live)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> (Game, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        (Game::new(rx, None, true).unwrap(), tx)
    }

    /// While a Graal3D link is up the TCP board is not drawn, and the
    /// status says why; when it goes down the level comes back.
    /// While the 3D link is up, what the player says goes to the 3D side,
    /// not the TCP chat.
    #[test]
    fn a_graal3d_link_takes_what_is_said() {
        let (mut g, _tx) = game();
        let got = Arc::new(Mutex::new(Vec::new()));
        let sink = got.clone();
        g.say3d = Some(Box::new(move |t| sink.lock().unwrap().push(t.to_string())));
        g.say("flat", Instant::now());
        g.boards_hidden
            .store(true, std::sync::atomic::Ordering::Relaxed);
        g.say("in 3d", Instant::now());
        assert_eq!(*got.lock().unwrap(), ["in 3d"]);
        assert_eq!(*g.say.lock().unwrap(), ["flat"]);
    }

    #[test]
    fn a_graal3d_link_hides_the_tcp_board() {
        let (mut g, tx) = game();
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0x1FF; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.tick(0, 0, Instant::now());
        let drawn = g.draw((4, 4));
        let black = Frame::filled(4 * render::TILE_PX, 4 * render::TILE_PX, [0, 0, 0, 255]);
        assert_ne!(drawn, black, "the level is drawn first");
        g.boards_hidden
            .store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(g.tick(0, 0, Instant::now()));
        assert!(g.status.starts_with("Graal3D"), "{}", g.status);
        assert_eq!(g.draw((4, 4)), black);
        g.boards_hidden
            .store(false, std::sync::atomic::Ordering::Relaxed);
        assert!(g.tick(0, 0, Instant::now()));
        assert_eq!(g.draw((4, 4)), drawn);
    }

    fn arrival(start: Option<(f32, f32)>) -> Arrival {
        Arrival {
            name: "lab3x.nw".into(),
            source: seihai_core::client::level::Source::Board,
            start,
            server_npcs: 0,
            server_npc_props: Vec::new(),
            file_npcs: 0,
            scripts: Vec::new(),
        }
    }

    /// With a build set, the GS2 scripts a level brings run on arrival
    /// and tick with the game; their NPC messages show as bubbles.
    #[test]
    fn a_level_runs_its_gs2_scripts() {
        use seihai_core::protocol::npcprops::NpcProps;
        use seihai_core::scripting::gs2::compile::compile_gs2;
        let (mut g, tx) = game();
        g.set_version(ClientVersion::V600R2);
        let mut a = arrival(Some((30.0, 30.0)));
        a.server_npc_props = vec![(
            7,
            NpcProps {
                image: Some("block.png".into()),
                x: Some(10.0),
                y: Some(10.0),
                ..Default::default()
            },
        )];
        let src = "function onCreated() { message(\"hi\"); setTimer(0.05); }\n\
                   function onTimeout() { message(\"later\"); }\n";
        a.scripts = vec![(7, Ok(compile_gs2(src).unwrap()))];
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), a))
            .unwrap();
        g.tick(0, 0, Instant::now());
        let level = g.session.current_level.as_ref().unwrap();
        assert_eq!(npc_bubbles(level, 0, 0), vec![("hi".into(), 176.0, 160.0)]);
        g.tick(0, 0, Instant::now() + Duration::from_millis(60));
        let level = g.session.current_level.as_ref().unwrap();
        assert_eq!(level.npcs[0].message, "later");
    }

    /// Closing the window drops the game, which tells the network thread
    /// to log out.
    #[test]
    fn dropping_the_game_closes_the_connection() {
        let (mut g, _tx) = game();
        let live = Live::default();
        let closed = live.closed.clone();
        g.attach(live);
        assert!(!closed.load(std::sync::atomic::Ordering::SeqCst));
        drop(g);
        assert!(closed.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn a_level_puts_the_player_at_the_start() {
        let (mut g, tx) = game();
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((12.5, 40.0))),
        ))
        .unwrap();
        g.poll();
        assert_eq!((g.session.player_x, g.session.player_y), (12.5, 40.0));
        assert!(g.status.starts_with("level lab3x.nw"));
        assert_eq!(g.position().as_deref(), Some("12.5, 40.0"));
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(None),
        ))
        .unwrap();
        g.poll();
        assert_eq!((g.session.player_x, g.session.player_y), NO_START);
    }

    #[test]
    fn the_arrows_walk_at_the_session_pace() {
        let (mut g, tx) = game();
        // Tile 0 is open floor in the built-in tile types.
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.poll();
        let t0 = g.last_step + Duration::from_secs(1);
        g.walk(1, 0, t0);
        assert!(g.session.player_x > 30.0, "{}", g.session.player_x);
        let x = g.session.player_x;
        // Too soon for the next step.
        g.walk(1, 0, t0 + Duration::from_millis(1));
        assert_eq!(g.session.player_x, x);
        g.walk(0, 1, t0 + g.session.step_interval());
        assert!(g.session.player_y > 30.0);
    }

    /// The level sets the pose (standing), each step updates it (walking),
    /// and the stop sets it standing again: the network thread reads it.
    #[test]
    fn the_walk_reaches_the_shared_pose() {
        let (mut g, tx) = game();
        assert!(g.pose.lock().unwrap().is_none());
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.poll();
        let p = g.pose.lock().unwrap().unwrap();
        assert_eq!((p.x, p.y, p.walking), (30.0, 30.0, false));
        let t0 = g.last_step + Duration::from_secs(1);
        g.walk(1, 0, t0);
        let p = g.pose.lock().unwrap().unwrap();
        assert!(p.walking && p.x > 30.0, "{p:?}");
        assert_eq!(p.dir, 3, "facing right");
        g.walk(0, 0, t0 + Duration::from_millis(1));
        assert!(!g.pose.lock().unwrap().unwrap().walking);
    }

    /// Walking into a link asks the network thread for it once, holds the
    /// walk, and the next level arrives where the link said.
    #[test]
    fn a_link_is_asked_for_and_the_level_arrives_there() {
        let (mut g, tx) = game();
        let mut level = Level::from_board(&[0; 4096]);
        level.add_server_links(&["next.nw 34 28 2 4 5 6".to_string()]);
        tx.send(Event::Level(Box::new(level), arrival(Some((30.0, 30.0)))))
            .unwrap();
        g.poll();
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..20 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
            if g.link.lock().unwrap().is_some() {
                break;
            }
        }
        let ask = g.link.lock().unwrap().clone().expect("a link request");
        assert_eq!((ask.dest.as_str(), ask.x, ask.y), ("next.nw", 5.0, 6.0));
        assert_eq!(g.session.warp_pending.as_deref(), Some("next.nw"));
        // No walking while the warp is pending.
        let x = g.session.player_x;
        g.walk(1, 0, t + Duration::from_secs(1));
        assert_eq!(g.session.player_x, x);
        // The server's level, with no start of its own: the link's place.
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(None),
        ))
        .unwrap();
        g.poll();
        assert_eq!((g.session.player_x, g.session.player_y), (5.0, 6.0));
        assert!(g.session.warp_pending.is_none());
    }

    #[test]
    fn no_position_before_a_level() {
        assert_eq!(game().0.position(), None);
    }

    #[test]
    fn nothing_walks_before_a_level() {
        let (mut g, _tx) = game();
        g.walk(1, 0, Instant::now() + Duration::from_secs(1));
        assert_eq!(g.session.player_x, OfflineSession::default().player_x);
    }

    #[test]
    fn the_swim_stroke_splashes_on_every_loop_in_the_water() {
        let (mut g, tx) = game();
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.poll();
        // The stock swim.gani's shape: a splash on the first of eight
        // two-tick steps, looped (one splash every 0.8 s).
        let mut text =
            String::from("SINGLEDIRECTION\nLOOP\nANI\n1 0 0\nPLAYSOUND water.wav 0 0\nWAIT 1\n\n");
        text.push_str(&"1 0 0\nWAIT 1\n\n".repeat(7));
        text.push_str("ANIEND\n");
        g.session
            .set_swim_gani(GaniPlayer::new(Gani::parse(&text).unwrap()));
        g.session.in_water = true;
        let t0 = g.pose_clock;
        let mut splashes = 0;
        for i in 1..=100u64 {
            // 5 s of floating still, one frame every 50 ms.
            g.tick_poses(t0 + Duration::from_millis(50 * i));
            splashes += g
                .session
                .take_sound_events()
                .iter()
                .filter(|e| e.file == "water.wav")
                .count();
        }
        assert!((6..=7).contains(&splashes), "{splashes} splashes in 5 s");
    }

    #[test]
    fn queued_sounds_are_taken_each_frame() {
        let (mut g, _tx) = game();
        assert!(!g.sound.is_enabled());
        g.session.sound_events.push(seihai_core::sound::SoundEvent {
            file: "steps.wav".into(),
            x: 1.0,
            y: 1.0,
        });
        g.session.sound_stops.push("water.wav".into());
        g.play_sounds();
        assert!(g.session.sound_events.is_empty());
        assert!(g.session.sound_stops.is_empty());
    }

    /// Lab #441: a sound asked for with WANTFILE arrives in memory, goes
    /// into the engine, and the next cue for it plays.
    #[test]
    fn a_sound_from_the_server_plays_on_the_next_cue() {
        use seihai_core::sound::{Call, RecordingSound, SoundEngine, SoundEvent};
        let (mut g, _tx) = game();
        let rec = RecordingSound::default();
        g.sound = SoundEngine::with_dirs(Box::new(rec.clone()), Vec::new(), true);
        let cue = |g: &mut Game| {
            g.session.sound_events.push(SoundEvent {
                file: "steps2.wav".into(),
                x: 1.0,
                y: 1.0,
            })
        };
        cue(&mut g);
        g.play_sounds();
        assert_eq!(
            g.wanted_sounds.lock().unwrap().as_slice(),
            ["steps2.wav".to_string()]
        );
        // 8-bit mono 8 kHz PCM, 100 samples.
        let mut wav = Vec::new();
        wav.extend(b"RIFF");
        wav.extend(136u32.to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(8000u32.to_le_bytes());
        wav.extend(8000u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(8u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend(100u32.to_le_bytes());
        wav.extend([128u8; 100]);
        g.arrived_sounds
            .lock()
            .unwrap()
            .push(("steps2.wav".into(), wav));
        cue(&mut g);
        g.play_sounds();
        assert!(g.arrived_sounds.lock().unwrap().is_empty());
        assert!(matches!(rec.log().as_slice(), [Call::Play { name, .. }] if name == "steps2.wav"));
    }

    #[test]
    fn the_servers_npcs_join_the_level() {
        use seihai_core::protocol::npcprops::NpcProps;
        let (mut g, tx) = game();
        let mut a = arrival(Some((30.0, 30.0)));
        let p = NpcProps {
            image: Some("door.png".into()),
            x: Some(20.0),
            y: Some(12.5),
            ..Default::default()
        };
        a.server_npc_props = vec![(300, p)];
        a.server_npcs = 1;
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), a))
            .unwrap();
        g.poll();
        let npcs = &g.session.current_level.as_ref().unwrap().npcs;
        assert_eq!(npcs.len(), 1);
        assert_eq!(
            (npcs[0].image.as_str(), npcs[0].x, npcs[0].y),
            ("door.png", 20.0, 12.5)
        );
        assert_eq!(npcs[0].server_id, Some(300));
        // No asset folder: looked for once, kept as missing (outlined).
        assert_eq!(g.npc_images.get("door.png"), Some(&None));
    }

    /// After the level is in, a server update moves and re-images an NPC
    /// it has, adds one it did not have, and one for another level is
    /// dropped. A delete takes an NPC off its own level only.
    #[test]
    fn npc_updates_after_the_level_change_what_is_drawn() {
        use seihai_core::protocol::npcprops::NpcProps;
        let (mut g, tx) = game();
        let mut a = arrival(Some((30.0, 30.0)));
        a.server_npc_props = vec![(
            300,
            NpcProps {
                image: Some("door.png".into()),
                x: Some(20.0),
                y: Some(12.5),
                ..Default::default()
            },
        )];
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), a))
            .unwrap();
        g.poll();
        let change = |level: &str, id, props| online::NpcChange {
            level: level.into(),
            id,
            props,
        };
        g.npc_changes.lock().unwrap().extend([
            change(
                "lab3x.nw",
                300,
                Some(NpcProps {
                    image: Some("door-open.png".into()),
                    x: Some(22.0),
                    ..Default::default()
                }),
            ),
            change(
                "lab3x.nw",
                301,
                Some(NpcProps {
                    image: Some("sign.png".into()),
                    x: Some(5.0),
                    y: Some(6.0),
                    ..Default::default()
                }),
            ),
            change(
                "elsewhere.nw",
                302,
                Some(NpcProps {
                    image: Some("tree.png".into()),
                    x: Some(1.0),
                    y: Some(1.0),
                    ..Default::default()
                }),
            ),
        ]);
        g.dirty = false;
        g.poll();
        assert!(g.dirty);
        assert!(g.npc_changes.lock().unwrap().is_empty());
        let npcs = &g.session.current_level.as_ref().unwrap().npcs;
        let got: Vec<_> = npcs
            .iter()
            .map(|n| (n.server_id, n.image.as_str(), n.x, n.y))
            .collect();
        assert_eq!(
            got,
            [
                (Some(300), "door-open.png", 22.0, 12.5),
                (Some(301), "sign.png", 5.0, 6.0),
            ]
        );
        assert_eq!(g.npc_images.get("door-open.png"), Some(&None));
        // A delete for another level leaves this one alone; one for this
        // level takes the NPC off.
        g.npc_changes.lock().unwrap().extend([
            change("elsewhere.nw", 300, None),
            change("lab3x.nw", 301, None),
        ]);
        g.poll();
        let ids: Vec<_> = g
            .session
            .current_level
            .as_ref()
            .unwrap()
            .npcs
            .iter()
            .map(|n| n.server_id)
            .collect();
        assert_eq!(ids, [Some(300)]);
    }

    #[test]
    fn names_go_under_named_players_only() {
        let other = |id, nick: Option<&str>| online::Other {
            id,
            x: 10.0,
            y: 20.0,
            dir: 2,
            nick: nick.map(Into::into),
            ..Default::default()
        };
        let tags = name_tags(
            &[other(1, Some("Ann")), other(2, None), other(3, Some(" "))],
            100,
            200,
        );
        assert_eq!(
            tags,
            vec![("Ann".to_string(), 11.5 * 16.0 - 100.0, 23.0 * 16.0 - 200.0)]
        );
    }

    /// Bubbles go over the players who said something, and only them.
    #[test]
    fn bubbles_go_over_the_players_who_spoke() {
        let other = |id, chat: Option<&str>| online::Other {
            id,
            x: 10.0,
            y: 20.0,
            chat: chat.map(Into::into),
            ..Default::default()
        };
        let got = bubbles(&[other(1, Some("hi")), other(2, None)], 100, 200);
        assert_eq!(
            got,
            vec![("hi".to_string(), 11.5 * 16.0 - 100.0, 20.0 * 16.0 - 200.0)]
        );
    }

    /// A typed line goes to the network thread and shows as the player's
    /// own bubble for the bubble time; a blank one does neither.
    #[test]
    fn saying_queues_the_line_and_shows_the_bubble() {
        let (mut g, _tx) = game();
        let t0 = Instant::now();
        g.say("  ", t0);
        assert!(g.say.lock().unwrap().is_empty());
        assert_eq!(g.own_bubble(t0), None);
        g.say(" hello ", t0);
        assert_eq!(*g.say.lock().unwrap(), vec!["hello".to_string()]);
        assert_eq!(g.own_bubble(t0 + Duration::from_millis(100)), Some("hello"));
        assert_eq!(g.own_bubble(t0 + OWN_BUBBLE), None);
    }

    /// Chat lines from the network thread are kept in order, and new
    /// counters redraw the frame only when the status display is drawn.
    #[test]
    fn chat_and_counters_come_in() {
        use seihai_core::protocol::ChatKind;
        let (mut g, _tx) = game();
        let now = Instant::now();
        let line = |text: &str| ChatLine {
            kind: ChatKind::Toall,
            from: "Ann".into(),
            text: text.into(),
            born_ms: 0,
        };
        g.chat_in.lock().unwrap().extend([line("a"), line("b")]);
        assert!(g.take_chat_and_counters(now));
        let texts: Vec<_> = g.chat().iter().map(|(l, _)| l.text.as_str()).collect();
        assert_eq!(texts, ["a", "b"]);
        assert!(!g.take_chat_and_counters(now), "taken once");
        g.own.lock().unwrap().bombs = Some(9);
        assert!(!g.take_chat_and_counters(now), "no status display");
        g.show_hud();
        g.own.lock().unwrap().bombs = Some(8);
        assert!(g.take_chat_and_counters(now));
        assert!(g.has_hud());
    }

    /// S swings the sword on a 6.x build: the player is drawn in the sword
    /// pose, the server is told `sword` and then the swing's end.
    #[test]
    fn the_sword_swings_and_the_server_hears_of_it() {
        let (mut g, tx) = game();
        g.set_version(ClientVersion::V603R7);
        assert!(!g.swing(), "no level yet");
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.poll();
        assert!(g.swing());
        assert_eq!(render::own_figure(&g.session).pose, "sword");
        let pose = g.pose.lock().unwrap().unwrap();
        assert_eq!(pose.gani, Some("sword"));
        assert!(!g.swing(), "already swinging");
        // Run the clock until the swing is over.
        let t0 = Instant::now();
        for i in 1..=40 {
            g.tick_poses(t0 + Duration::from_millis(50 * i));
        }
        assert!(!g.session.combat.is_swinging());
        assert_ne!(render::own_figure(&g.session).pose, "sword");
        assert_eq!(g.pose.lock().unwrap().unwrap().gani, None);
    }

    /// A build with no combat rules does not swing.
    #[test]
    fn no_combat_rules_no_swing() {
        let (mut g, tx) = game();
        tx.send(Event::Level(
            Box::new(Level::from_board(&[0; 4096])),
            arrival(Some((30.0, 30.0))),
        ))
        .unwrap();
        g.poll();
        assert!(!g.swing());
    }

    /// A folder with `levels/a.nw` (a link east to `b.nw`) and `levels/b.nw`.
    fn offline_levels(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("seihai-play-offline-{tag}-{}", std::process::id()));
        let levels = dir.join("levels");
        std::fs::create_dir_all(&levels).unwrap();
        std::fs::write(levels.join("a.nw"), "GLEVNW01\nLINK b.nw 34 28 2 4 5 6\n").unwrap();
        std::fs::write(levels.join("b.nw"), "GLEVNW01\n").unwrap();
        dir
    }

    fn resolver(dir: &std::path::Path) -> AssetResolver {
        AssetResolver::new(
            AssetRoot::new(dir.to_path_buf()),
            ClientVersion::V304.profile(),
        )
    }

    #[test]
    fn offline_a_link_loads_the_next_level_file() {
        let dir = offline_levels("link");
        let r = resolver(&dir);
        let mut g = Game::open_offline(Some(&r), "a.nw", false).unwrap();
        assert_eq!(g.session.level_name().as_deref(), Some("a.nw"));
        assert_eq!((g.session.player_x, g.session.player_y), NO_START);
        assert_eq!(g.status, "a.nw (offline)");
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..20 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
            if g.session.level_name().as_deref() == Some("b.nw") {
                break;
            }
        }
        assert_eq!(g.session.level_name().as_deref(), Some("b.nw"));
        assert_eq!((g.session.player_x, g.session.player_y), (5.0, 6.0));
        // Nothing went to a server, and no warp is left waiting.
        assert!(g.link.lock().unwrap().is_none());
        assert!(g.session.warp_pending.is_none());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn offline_a_missing_level_is_an_error_and_a_missing_link_target_says_so() {
        let dir = offline_levels("missing");
        let r = resolver(&dir);
        assert!(Game::open_offline(Some(&r), "nowhere.nw", false).is_err());
        std::fs::write(
            dir.join("levels").join("a.nw"),
            "GLEVNW01\nLINK gone.nw 34 28 2 4 5 6\n",
        )
        .unwrap();
        let mut g = Game::open_offline(Some(&r), "a.nw", false).unwrap();
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..20 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
        }
        assert!(g.status.starts_with("Link to gone.nw:"), "{}", g.status);
        assert_eq!(g.session.level_name().as_deref(), Some("a.nw"));
        std::fs::remove_dir_all(dir).ok();
    }

    /// `levels/w.gmap` (two cells, `a.nw` and `b.nw`) and `levels/c.nw`,
    /// whose link leads into the gmap's east cell.
    fn offline_gmap(tag: &str) -> PathBuf {
        let dir = offline_levels(tag);
        let levels = dir.join("levels");
        std::fs::write(
            levels.join("w.gmap"),
            "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n",
        )
        .unwrap();
        std::fs::write(levels.join("a.nw"), "GLEVNW01\n").unwrap();
        std::fs::write(
            levels.join("c.nw"),
            "GLEVNW01\nLINK w.gmap 34 28 2 4 70 10\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn offline_a_gmap_opens_on_its_first_cell_and_the_walk_crosses_the_edge() {
        let dir = offline_gmap("gmap-cross");
        let r = resolver(&dir);
        let mut g = Game::open_offline(Some(&r), "w.gmap", false).unwrap();
        assert_eq!(g.session.level_name().as_deref(), Some("a.nw"));
        assert_eq!(g.status, "a.nw on w.gmap (offline)");
        let w = g.session.gmap.as_ref().unwrap();
        assert_eq!(w.cell, (0, 0));
        assert!(
            w.levels.contains_key(&(1, 0)),
            "the east neighbour is loaded"
        );
        g.session.player_x = 60.0;
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..40 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
            if g.session.gmap.as_ref().unwrap().cell == (1, 0) {
                break;
            }
        }
        let w = g.session.gmap.as_ref().unwrap();
        assert_eq!(w.cell, (1, 0));
        assert!(
            w.levels.contains_key(&(0, 0)),
            "the level left is kept beside"
        );
        assert!(
            g.session.player_x < 4.0,
            "local to the new cell: {}",
            g.session.player_x
        );
        assert_eq!(g.status, "b.nw on w.gmap (offline)");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn offline_a_link_into_a_gmap_lands_in_the_cell_its_position_names() {
        let dir = offline_gmap("gmap-link");
        let r = resolver(&dir);
        let mut g = Game::open_offline(Some(&r), "c.nw", false).unwrap();
        assert!(g.session.gmap.is_none());
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..20 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
            if g.session.gmap.is_some() {
                break;
            }
        }
        let w = g
            .session
            .gmap
            .as_ref()
            .unwrap_or_else(|| panic!("on the gmap: {}", g.status));
        assert_eq!(w.cell, (1, 0));
        assert_eq!(g.session.level_name().as_deref(), Some("b.nw"));
        assert_eq!((g.session.player_x, g.session.player_y), (6.0, 10.0));
        // A plain level link from the gmap leaves it.
        g.enter_offline("c.nw", Some((30.0, 30.0))).unwrap();
        assert!(g.session.gmap.is_none());
        std::fs::remove_dir_all(dir).ok();
    }

    /// The two-cell gmap of the online tests: `a.nw` west, `b.nw` east.
    fn world(cell: (i32, i32), with: &[(i32, i32)], crossings: u64) -> online::WorldView {
        online::WorldView {
            name: "w.gmap".into(),
            map: Gmap::parse(
                "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n",
            )
            .unwrap(),
            cell,
            levels: with
                .iter()
                .map(|c| (*c, Level::from_board(&[0; 4096])))
                .collect(),
            crossings,
        }
    }

    fn level_named(name: &str, start: Option<(f32, f32)>) -> Event {
        let mut a = arrival(start);
        a.name = name.into();
        Event::Level(Box::new(Level::from_board(&[0; 4096])), a)
    }

    /// Walk east until the walk crosses into the next cell.
    fn walk_east_across(g: &mut Game) {
        let mut t = g.last_step + Duration::from_secs(1);
        for _ in 0..40 {
            g.walk(1, 0, t);
            t += g.session.step_interval();
            if g.crossings_sent > 0 {
                return;
            }
        }
        panic!("no crossing; x = {}", g.session.player_x);
    }

    /// Online on a gmap: the network thread's world comes in with the
    /// level, the walk crosses into the neighbour it sent, the crossing
    /// goes to the network thread, and the server's copy of the new cell
    /// keeps the player where the walk put them.
    #[test]
    fn online_the_walk_crosses_a_gmap_edge_and_keeps_its_place() {
        let (mut g, tx) = game();
        tx.send(level_named("a.nw", Some((60.0, 30.0)))).unwrap();
        *g.world.lock().unwrap() = Some(world((0, 0), &[(1, 0)], 0));
        g.poll();
        let w = g.session.gmap.as_ref().expect("on the gmap");
        assert_eq!(w.cell, (0, 0));
        assert!(w.levels.contains_key(&(1, 0)));
        walk_east_across(&mut g);
        let sent = g.crossings.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        assert_eq!((sent[0].from, sent[0].to), ((0, 0), (1, 0)));
        assert_eq!(g.session.level_name().as_deref(), Some("b.nw"));
        let x = g.session.player_x;
        assert!(x < 4.0, "{x}");
        // A snapshot taken before the network thread sent the crossing
        // does not pull the view back.
        *g.world.lock().unwrap() = Some(world((0, 0), &[(1, 0)], 0));
        g.poll();
        assert_eq!(g.session.gmap.as_ref().unwrap().cell, (1, 0));
        // The server's copy of b.nw, with the login's stale position.
        tx.send(level_named("b.nw", Some((60.0, 30.0)))).unwrap();
        g.poll();
        assert_eq!(g.session.player_x, x);
        assert!(g.status.starts_with("level b.nw"), "{}", g.status);
        // The network thread caught up: the west cell is the neighbour.
        *g.world.lock().unwrap() = Some(world((1, 0), &[(0, 0)], 1));
        g.poll();
        let w = g.session.gmap.as_ref().unwrap();
        assert_eq!(w.cell, (1, 0));
        assert!(w.levels.contains_key(&(0, 0)));
    }

    /// The server moving the player to another cell (a warp inside the
    /// gmap) waits for that cell's level, which arrives at its start.
    #[test]
    fn online_a_server_move_inside_a_gmap_waits_for_its_level() {
        let (mut g, tx) = game();
        tx.send(level_named("a.nw", Some((30.0, 30.0)))).unwrap();
        *g.world.lock().unwrap() = Some(world((0, 0), &[(1, 0)], 0));
        g.poll();
        *g.world.lock().unwrap() = Some(world((1, 0), &[], 0));
        g.poll();
        assert_eq!(g.session.gmap.as_ref().unwrap().cell, (0, 0));
        tx.send(level_named("b.nw", Some((10.0, 20.0)))).unwrap();
        g.poll();
        assert_eq!(g.session.gmap.as_ref().unwrap().cell, (1, 0));
        assert_eq!((g.session.player_x, g.session.player_y), (10.0, 20.0));
    }

    /// A level outside the gmap leaves it.
    #[test]
    fn online_a_level_off_the_gmap_leaves_it() {
        let (mut g, tx) = game();
        tx.send(level_named("a.nw", Some((30.0, 30.0)))).unwrap();
        *g.world.lock().unwrap() = Some(world((0, 0), &[(1, 0)], 0));
        g.poll();
        assert!(g.session.gmap.is_some());
        tx.send(level_named("house.nw", Some((5.0, 5.0)))).unwrap();
        g.poll();
        assert!(g.session.gmap.is_none());
        assert_eq!((g.session.player_x, g.session.player_y), (5.0, 5.0));
    }

    /// The gmap waits for the level of its cell: a world that comes in
    /// before that level (the view still on the last one) is held.
    #[test]
    fn online_the_gmap_waits_for_the_level_of_its_cell() {
        let (mut g, tx) = game();
        tx.send(level_named("start.nw", Some((30.0, 30.0))))
            .unwrap();
        *g.world.lock().unwrap() = Some(world((1, 0), &[(0, 0)], 0));
        g.poll();
        assert!(g.session.gmap.is_none());
        tx.send(level_named("b.nw", Some((20.0, 30.0)))).unwrap();
        g.poll();
        let w = g.session.gmap.as_ref().expect("entered with its level");
        assert_eq!(w.cell, (1, 0));
        assert!(w.levels.contains_key(&(0, 0)));
        assert_eq!(g.session.player_x, 20.0);
    }

    fn tile_change(x: u8, y: u8, tile: u16) -> seihai_core::protocol::layouts::BoardModify2 {
        seihai_core::protocol::layouts::BoardModify2 {
            map_x: 0,
            map_y: 0,
            x,
            y,
            width: 1,
            height: 1,
            tiles: vec![tile],
            trailer: Vec::new(),
        }
    }

    /// The server's board changes land on the level the view is on and on
    /// a gmap neighbour it holds; one for a level it does not hold is
    /// dropped.
    #[test]
    fn online_board_changes_reach_the_levels_in_view() {
        let (mut g, tx) = game();
        tx.send(level_named("a.nw", Some((30.0, 30.0)))).unwrap();
        *g.world.lock().unwrap() = Some(world((0, 0), &[(1, 0)], 0));
        g.poll();
        g.board_mods.lock().unwrap().extend([
            ("a.nw".to_string(), tile_change(3, 4, 0x1FF)),
            ("b.nw".to_string(), tile_change(5, 6, 0x22)),
            ("elsewhere.nw".to_string(), tile_change(1, 1, 0x33)),
        ]);
        g.poll();
        let here = g.session.current_level.as_ref().unwrap();
        assert_eq!(here.tile_id_at(3, 4), Some(0x1FF));
        let east = &g.session.gmap.as_ref().unwrap().levels[&(1, 0)];
        assert_eq!(east.tile_id_at(5, 6), Some(0x22));
        assert!(g.board_mods.lock().unwrap().is_empty());
    }
}
