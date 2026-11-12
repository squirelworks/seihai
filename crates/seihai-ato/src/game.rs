//! Stage 3: the start window, then the level the server sent, drawn flat
//! ([`crate::render`]), with the arrow keys walking the player over it the
//! way `seihai-mae` walks (core's `OfflineSession::try_move`, the same walls
//! and water).
//!
//! The walk goes to the server: the view keeps the pose in a
//! [`online::SharedPose`] and the network thread sends it once a 50 ms
//! tick ([`online::run_with_pose`]). A link the player walks into is noted
//! and asked for, and the level it leads to is drawn when it comes in.
//! Walking plays the footstep and water sounds of `walk.gani` and
//! `swim.gani` through core's sound engine ([`Game::play_sounds`]).

use crate::level::Arrival;
use crate::online::{self, Login};
use crate::render::{self, NpcImages, Sheet};
use crate::server_list::{Pick, ServerList};
use crate::start_window::{Next, StartWindow, CANVAS};
use seihai_core::assets::{
    AssetResolver, AssetRoot, Gani, GaniPlayer, Level, LISTENER_DX, LISTENER_DY,
};
use seihai_core::gmap::Gmap;
use seihai_core::offline::OfflineSession;
use seihai_core::protocol::ClientVersion;
use seihai_core::sound::{self, SoundEngine};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The view, in tiles: 640 x 480 pixels, the 3.x window size.
pub const VIEW_TILES: (u32, u32) = (
    CANVAS.0 as u32 / render::TILE_PX,
    CANVAS.1 as u32 / render::TILE_PX,
);

/// Where a level with no start position puts the player.
const NO_START: (f32, f32) = (30.0, 30.0);

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
    texture: Option<egui::TextureHandle>,
    /// The frame needs drawing again.
    dirty: bool,
    /// Where the network thread reads the player's pose.
    pub pose: online::SharedPose,
    /// Where the network thread puts the other players.
    pub others: online::SharedOthers,
    /// The other players drawn last frame.
    drawn_others: Vec<online::Other>,
    /// Where the network thread takes a link request from.
    pub link: Arc<Mutex<Option<online::LinkAsk>>>,
    /// The link asked for, until its level arrives.
    link_to: Option<online::LinkAsk>,
    /// Sound effects: core's engine and device output (the `audio`
    /// feature). Silent with no sounds folder, no device, or `--no-sound`.
    sound: SoundEngine,
    sound_clock: Instant,
    /// The level the looped sounds belong to; a new one stops them.
    sound_serial: u64,
    /// Where missing sounds go for the network thread to ask for.
    pub wanted_sounds: Arc<Mutex<Vec<String>>>,
    /// The asset folders, for NPC images.
    resolver: Option<AssetResolver>,
    /// NPC images read so far, by name.
    npc_images: NpcImages,
    /// Playing level files with no server (`--level`): links load the
    /// next level from the asset folder instead of asking a server.
    offline: bool,
}

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
        }
        Ok(Game {
            session,
            sheet: Sheet::load(resolver)?,
            rx,
            status: "Waiting for the level.".into(),
            last_step: Instant::now(),
            texture: None,
            dirty: true,
            pose: online::SharedPose::default(),
            others: online::SharedOthers::default(),
            drawn_others: Vec::new(),
            link: Default::default(),
            link_to: None,
            sound: sound_engine(resolver, sound_on),
            sound_clock: Instant::now(),
            sound_serial: 0,
            wanted_sounds: Default::default(),
            resolver: resolver.cloned(),
            npc_images: NpcImages::new(),
            offline: false,
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

    /// Play the sounds the session queued: the GANI cues of the steps just
    /// taken. A new level stops the looped ones. A sound not on disk is
    /// handed to the network thread to ask the server for, and plays on a
    /// later cue once it has arrived.
    pub fn play_sounds(&mut self) {
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
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Level(level, a) => {
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
                    self.set_pose(false);
                }
                Event::Ended(why) => self.status = why,
            }
            self.dirty = true;
        }
    }

    /// Hand the current pose to the network thread.
    fn set_pose(&self, walking: bool) {
        *self.pose.lock().unwrap_or_else(|e| e.into_inner()) = Some(online::Pose {
            x: self.session.player_x,
            y: self.session.player_y,
            dir: self.session.player_dir % 4,
            walking,
        });
    }

    /// The player's position in tiles, for the view, once a level is in.
    pub fn position(&self) -> Option<String> {
        self.session.current_level.as_ref()?;
        Some(format!(
            "{:.1}, {:.1}",
            self.session.player_x, self.session.player_y
        ))
    }

    /// One walk step for the arrows held, at the session's pace. `(dx, dy)`
    /// are -1, 0 or 1 per axis.
    pub fn walk(&mut self, dx: i8, dy: i8, now: Instant) {
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
        self.set_pose(true);
        // A link the feet are now in: ask the server once and wait for the
        // level (no walking meanwhile, core's warp-pending state).
        if self.session.check_links() {
            if let Some((dest, x, y)) = self.session.pending_link.take() {
                if self.offline {
                    if let Err(e) = self.enter_offline(&dest, Some((x, y))) {
                        self.status = format!("Link to {dest}: {e}");
                    }
                    return;
                }
                self.session.begin_warp(&dest);
                let ask = online::LinkAsk {
                    dest: dest.clone(),
                    x,
                    y,
                    dir: self.session.player_dir % 4,
                };
                *self.link.lock().unwrap_or_else(|e| e.into_inner()) = Some(ask.clone());
                self.link_to = Some(ask);
                self.status = format!("Link to {dest}: asking the server.");
            }
        }
        self.dirty = true;
    }

    fn show(&mut self, ctx: &egui::Context) {
        self.poll();
        let held = |k| ctx.input(|i| i.key_down(k));
        let dx = held(egui::Key::ArrowRight) as i8 - held(egui::Key::ArrowLeft) as i8;
        let dy = held(egui::Key::ArrowDown) as i8 - held(egui::Key::ArrowUp) as i8;
        self.walk(dx, dy, Instant::now());
        self.play_sounds();
        let others = self
            .others
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if others != self.drawn_others {
            self.drawn_others = others;
            self.dirty = true;
        }
        if self.dirty || self.texture.is_none() {
            let at: Vec<(f32, f32)> = self.drawn_others.iter().map(|o| (o.x, o.y)).collect();
            let f = render::draw(
                &self.session,
                &self.sheet,
                &self.npc_images,
                &at,
                VIEW_TILES.0,
                VIEW_TILES.1,
            );
            let image = egui::ColorImage::from_rgb([f.w as usize, f.h as usize], &f.rgb);
            match &mut self.texture {
                Some(t) => t.set(image, egui::TextureOptions::NEAREST),
                None => {
                    self.texture =
                        Some(ctx.load_texture("level", image, egui::TextureOptions::NEAREST))
                }
            }
            self.dirty = false;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let origin = ui.max_rect().min;
                if let Some(t) = &self.texture {
                    let rect = egui::Rect::from_min_size(origin, egui::vec2(CANVAS.0, CANVAS.1));
                    ui.painter().image(
                        t.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                let bar = egui::Rect::from_min_size(origin, egui::vec2(CANVAS.0, 18.0));
                ui.painter()
                    .rect_filled(bar, 0.0, egui::Color32::from_black_alpha(160));
                ui.put(
                    bar,
                    egui::Label::new(
                        egui::RichText::new(&self.status)
                            .color(egui::Color32::WHITE)
                            .size(12.0),
                    ),
                );
                // Other players' names under their markers, as the clients
                // write a nickname under the player.
                let (cam_x, cam_y) =
                    render::camera_origin(&self.session, VIEW_TILES.0, VIEW_TILES.1);
                for (nick, x, y) in name_tags(&self.drawn_others, cam_x, cam_y) {
                    ui.painter().text(
                        origin + egui::vec2(x, y),
                        egui::Align2::CENTER_TOP,
                        nick,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(pos) = self.position() {
                    let at = egui::Rect::from_min_size(
                        origin + egui::vec2(0.0, 18.0),
                        egui::vec2(CANVAS.0 - 6.0, 16.0),
                    );
                    ui.painter().text(
                        at.right_top(),
                        egui::Align2::RIGHT_TOP,
                        pos,
                        egui::FontId::monospace(12.0),
                        egui::Color32::WHITE,
                    );
                }
            });
        // Walking needs frames while a key is held; otherwise poll the
        // network now and then.
        ctx.request_repaint_after(Duration::from_millis(if dx != 0 || dy != 0 {
            16
        } else {
            100
        }));
    }
}

/// Where each named other player's name goes, in view pixels: centred
/// under the 48x48 box. Players with no nickname get no tag.
fn name_tags(others: &[online::Other], cam_x: i32, cam_y: i32) -> Vec<(String, f32, f32)> {
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

/// The start window, then the game view once Start logs in.
pub struct AtoApp {
    start: StartWindow,
    game: Option<Game>,
    /// The server list, while it is open.
    list: Option<ServerList>,
    assets: Option<PathBuf>,
    /// Log in on the first frame, without the start window.
    skip_start: bool,
    /// Sound effects on (off with `--no-sound`).
    pub sound_on: bool,
}

impl AtoApp {
    pub fn new(start: StartWindow, assets: Option<PathBuf>) -> AtoApp {
        AtoApp {
            start,
            game: None,
            list: None,
            assets,
            skip_start: false,
            sound_on: true,
        }
    }

    /// As [`AtoApp::new`], but log in at once with the form as given when it
    /// names a server and an account (`--online` and `--account`), as if
    /// Start had been pressed. For lab runs that cannot press Start.
    pub fn new_skipping_start(start: StartWindow, assets: Option<PathBuf>) -> AtoApp {
        let skip_start = skips_start(&start);
        AtoApp {
            start,
            game: None,
            list: None,
            assets,
            skip_start,
            sound_on: true,
        }
    }

    /// Start the network thread for `login` and switch to the game view.
    fn begin(&mut self, addr: String, login: Login, ctx: &egui::Context) {
        let version = self.start.version;
        let resolver = self
            .assets
            .clone()
            .map(|d| AssetResolver::new(AssetRoot::new(d), version.profile()));
        let (tx, rx) = mpsc::channel();
        let assets = self.assets.clone();
        let repaint = ctx.clone();
        let live = online::Live::default();
        let net_live = live.clone();
        std::thread::spawn(move || {
            let shown_tx = tx.clone();
            let shown_ctx = repaint.clone();
            let mut shown = move |l: &Level, a: &Arrival| {
                let _ = shown_tx.send(Event::Level(Box::new(l.clone()), a.clone()));
                shown_ctx.request_repaint();
            };
            let why = match online::run_with_pose(
                &addr,
                version,
                &login,
                assets,
                false,
                &mut std::io::stdout(),
                &mut shown,
                Some(net_live),
            ) {
                Ok(()) => "The server closed the connection.".to_string(),
                Err(e) => format!("Could not log in: {e}"),
            };
            let _ = tx.send(Event::Ended(why));
            repaint.request_repaint();
        });
        match Game::new(rx, resolver.as_ref(), self.sound_on) {
            Ok(mut g) => {
                g.pose = live.pose;
                g.others = live.others;
                g.link = live.link;
                g.wanted_sounds = live.wanted_sounds;
                self.game = Some(g);
            }
            Err(e) => self.start.status = e,
        }
    }
}

impl AtoApp {
    /// Act on where Start went.
    fn next(&mut self, next: Next, ctx: &egui::Context) {
        match next {
            Next::Stay => {}
            Next::Login(addr) => self.login_at(addr, ctx),
            Next::ServerList => {
                let login = self.start.started.lock().unwrap().clone();
                let Some(login) = login else {
                    return;
                };
                let repaint = ctx.clone();
                self.list = Some(ServerList::fetch(
                    self.start.listserver.clone(),
                    self.start.version,
                    login.account,
                    login.password,
                    move || repaint.request_repaint(),
                ));
            }
        }
    }

    /// Log in at `addr` with the form Start handed over.
    fn login_at(&mut self, addr: String, ctx: &egui::Context) {
        let login = self.start.started.lock().unwrap().take();
        if let Some(login) = login {
            self.list = None;
            self.begin(addr, login, ctx);
        }
    }
}

impl eframe::App for AtoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if std::mem::take(&mut self.skip_start) {
            let next = self.start.press_start();
            self.next(next, ctx);
        }
        if let Some(g) = &mut self.game {
            g.show(ctx);
            return;
        }
        if let Some(list) = &mut self.list {
            match list.show(ctx) {
                Some(Pick::Connect(addr)) => self.login_at(addr, ctx),
                Some(Pick::Back) => {
                    self.list = None;
                    self.start.started.lock().unwrap().take();
                }
                None => {}
            }
            return;
        }
        let next = self.start.show(ctx);
        self.next(next, ctx);
    }
}

/// Open the window on level `level` with no server (`--level`), and block
/// until it is closed.
pub fn run_offline(
    version: ClientVersion,
    level: &str,
    assets: Option<PathBuf>,
    sound_on: bool,
) -> Result<(), String> {
    let resolver = assets
        .clone()
        .map(|d| AssetResolver::new(AssetRoot::new(d), version.profile()));
    let game = Game::open_offline(resolver.as_ref(), level, sound_on)?;
    let title = format!("Graal {} (Seihai Ato, offline)", version.label());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([CANVAS.0, CANVAS.1])
            .with_resizable(false),
        ..Default::default()
    };
    let start = StartWindow::new(version, String::new(), String::new(), String::new());
    let mut app = AtoApp::new(start, assets);
    app.sound_on = sound_on;
    app.game = Some(game);
    eframe::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app))))
        .map_err(|e| e.to_string())
}

/// True when the form names a server and an account, so the start window
/// can be skipped.
pub fn skips_start(start: &StartWindow) -> bool {
    start.online.is_some() && !start.account.is_empty()
}

/// Open the window and block until it is closed. With a server and an
/// account given, the start window is skipped (see [`skips_start`]).
pub fn run(
    version: ClientVersion,
    start: StartWindow,
    assets: Option<PathBuf>,
    sound_on: bool,
) -> Result<(), String> {
    let title = format!("Graal {} (Seihai Ato)", version.label());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([CANVAS.0, CANVAS.1])
            .with_resizable(false),
        ..Default::default()
    };
    let mut app = AtoApp::new_skipping_start(start, assets);
    app.sound_on = sound_on;
    eframe::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app))))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> (Game, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        (Game::new(rx, None, true).unwrap(), tx)
    }

    fn arrival(start: Option<(f32, f32)>) -> Arrival {
        Arrival {
            name: "lab3x.nw".into(),
            source: crate::level::Source::Board,
            start,
            server_npcs: 0,
            server_npc_props: Vec::new(),
            file_npcs: 0,
            scripts: Vec::new(),
        }
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
    fn the_start_window_is_skipped_only_with_a_server_and_an_account() {
        let w = |online: Option<&str>, account: &str| {
            let mut w =
                StartWindow::new(ClientVersion::V3153, "n".into(), account.into(), "p".into());
            w.online = online.map(String::from);
            w
        };
        assert!(skips_start(&w(Some("127.0.0.1:14802"), "lab")));
        assert!(!skips_start(&w(None, "lab")));
        assert!(!skips_start(&w(Some("127.0.0.1:14802"), "")));
    }

    #[test]
    fn the_view_is_the_3x_window() {
        assert_eq!(VIEW_TILES, (40, 30));
    }

    /// The session's queued sounds are taken every frame, so they never pile
    /// up, whether sound is on or not; no asset folder means sound is off.
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

    #[test]
    fn names_go_under_named_players_only() {
        let other = |id, nick: Option<&str>| online::Other {
            id,
            x: 10.0,
            y: 20.0,
            dir: 2,
            nick: nick.map(Into::into),
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

    /// A folder with `levels/a.nw` (a link east to `b.nw`) and `levels/b.nw`.
    fn offline_levels(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("seihai-ato-offline-{tag}-{}", std::process::id()));
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
}
