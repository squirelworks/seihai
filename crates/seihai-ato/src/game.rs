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
use crate::render::{self, Sheet};
use crate::server_list::{Pick, ServerList};
use crate::start_window::{Next, StartWindow, CANVAS};
use seihai_core::assets::{
    AssetResolver, AssetRoot, Gani, GaniPlayer, Level, LISTENER_DX, LISTENER_DY,
};
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
        })
    }

    /// Play the sounds the session queued: the GANI cues of the steps just
    /// taken. A new level stops the looped ones. A sound not on disk stays
    /// silent (Ato does not ask the server for it yet).
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
            self.sound.play(&ev, listener, now);
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
        self.set_pose(true);
        // A link the feet are now in: ask the server once and wait for the
        // level (no walking meanwhile, core's warp-pending state).
        if self.session.check_links() {
            if let Some((dest, x, y)) = self.session.pending_link.take() {
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
            let f = render::draw(&self.session, &self.sheet, &at, VIEW_TILES.0, VIEW_TILES.1);
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
}
