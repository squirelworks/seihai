//! Stage 3: the start window, then the level the server sent, drawn flat
//! ([`crate::render`]), with the arrow keys walking the player over it the
//! way `seihai-mae` walks (core's `OfflineSession::try_move`, the same walls
//! and water).
//!
//! The walk is local: Ato does not send its position yet, because what a
//! 3.x client sends while walking has not been captured. A link the player
//! walks into is noted and dropped; level changes are not in Ato yet.

use crate::level::Arrival;
use crate::online::{self, Login};
use crate::render::{self, Sheet};
use crate::start_window::{StartWindow, CANVAS};
use seihai_core::assets::{AssetResolver, AssetRoot, Level};
use seihai_core::offline::OfflineSession;
use seihai_core::protocol::ClientVersion;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// The view, in tiles: 640 x 480 pixels, the 3.x window size.
pub const VIEW_TILES: (u32, u32) = (CANVAS.0 as u32 / render::TILE_PX, CANVAS.1 as u32 / render::TILE_PX);

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
}

impl Game {
    pub fn new(rx: Receiver<Event>, resolver: Option<&AssetResolver>) -> Result<Game, String> {
        let mut session = OfflineSession::default();
        session.tiletypes = render::tile_types(resolver);
        Ok(Game {
            session,
            sheet: Sheet::load(resolver)?,
            rx,
            status: "Waiting for the level.".into(),
            last_step: Instant::now(),
            texture: None,
            dirty: true,
        })
    }

    /// Take what the network thread sent.
    pub fn poll(&mut self) {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Level(level, a) => {
                    self.session.current_level = Some(*level);
                    self.session.level_serial += 1;
                    (self.session.player_x, self.session.player_y) = a.start.unwrap_or(NO_START);
                    self.session.warp_pending = None;
                    self.session.refresh_water();
                    self.status = a.line();
                }
                Event::Ended(why) => self.status = why,
            }
            self.dirty = true;
        }
    }

    /// One walk step for the arrows held, at the session's pace. `(dx, dy)`
    /// are -1, 0 or 1 per axis.
    pub fn walk(&mut self, dx: i8, dy: i8, now: Instant) {
        if (dx, dy) == (0, 0) {
            if self.session.stop_walking() {
                self.dirty = true;
            }
            return;
        }
        if self.session.current_level.is_none() || now.duration_since(self.last_step) < self.session.step_interval() {
            return;
        }
        self.last_step = now;
        self.session.try_move(dx as f32 * 0.5, dy as f32 * 0.5);
        if let Some(dest) = self.session.warp_pending.take() {
            self.status = format!("Link to {dest}: level changes are not in Ato yet.");
        }
        self.dirty = true;
    }

    fn show(&mut self, ctx: &egui::Context) {
        self.poll();
        let held = |k| ctx.input(|i| i.key_down(k));
        let dx = held(egui::Key::ArrowRight) as i8 - held(egui::Key::ArrowLeft) as i8;
        let dy = held(egui::Key::ArrowDown) as i8 - held(egui::Key::ArrowUp) as i8;
        self.walk(dx, dy, Instant::now());
        if self.dirty || self.texture.is_none() {
            let f = render::draw(&self.session, &self.sheet, VIEW_TILES.0, VIEW_TILES.1);
            let image = egui::ColorImage::from_rgb([f.w as usize, f.h as usize], &f.rgb);
            match &mut self.texture {
                Some(t) => t.set(image, egui::TextureOptions::NEAREST),
                None => self.texture = Some(ctx.load_texture("level", image, egui::TextureOptions::NEAREST)),
            }
            self.dirty = false;
        }
        egui::CentralPanel::default().frame(egui::Frame::none().fill(egui::Color32::BLACK)).show(ctx, |ui| {
            let origin = ui.max_rect().min;
            if let Some(t) = &self.texture {
                let rect = egui::Rect::from_min_size(origin, egui::vec2(CANVAS.0, CANVAS.1));
                ui.painter().image(t.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            }
            let bar = egui::Rect::from_min_size(origin, egui::vec2(CANVAS.0, 18.0));
            ui.painter().rect_filled(bar, 0.0, egui::Color32::from_black_alpha(160));
            ui.put(bar, egui::Label::new(egui::RichText::new(&self.status).color(egui::Color32::WHITE).size(12.0)));
        });
        // Walking needs frames while a key is held; otherwise poll the
        // network now and then.
        ctx.request_repaint_after(Duration::from_millis(if dx != 0 || dy != 0 { 16 } else { 100 }));
    }
}

/// The start window, then the game view once Start logs in.
pub struct AtoApp {
    start: StartWindow,
    game: Option<Game>,
    assets: Option<PathBuf>,
}

impl AtoApp {
    pub fn new(start: StartWindow, assets: Option<PathBuf>) -> AtoApp {
        AtoApp { start, game: None, assets }
    }

    /// Start the network thread for `login` and switch to the game view.
    fn begin(&mut self, addr: String, login: Login, ctx: &egui::Context) {
        let version = self.start.version;
        let resolver = self.assets.clone().map(|d| AssetResolver::new(AssetRoot::new(d), version.profile()));
        let (tx, rx) = mpsc::channel();
        let assets = self.assets.clone();
        let repaint = ctx.clone();
        std::thread::spawn(move || {
            let shown_tx = tx.clone();
            let shown_ctx = repaint.clone();
            let mut shown = move |l: &Level, a: &Arrival| {
                let _ = shown_tx.send(Event::Level(Box::new(l.clone()), a.clone()));
                shown_ctx.request_repaint();
            };
            let why = match online::run(&addr, version, &login, assets, false, &mut std::io::stdout(), &mut shown) {
                Ok(()) => "The server closed the connection.".to_string(),
                Err(e) => format!("Could not log in: {e}"),
            };
            let _ = tx.send(Event::Ended(why));
            repaint.request_repaint();
        });
        match Game::new(rx, resolver.as_ref()) {
            Ok(g) => self.game = Some(g),
            Err(e) => self.start.status = e,
        }
    }
}

impl eframe::App for AtoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.game {
            Some(g) => g.show(ctx),
            None => {
                if self.start.show(ctx) {
                    let login = self.start.started.lock().unwrap().take();
                    if let (Some(addr), Some(login)) = (self.start.online.clone(), login) {
                        self.begin(addr, login, ctx);
                    }
                }
            }
        }
    }
}

/// Open the window and block until it is closed.
pub fn run(version: ClientVersion, start: StartWindow, assets: Option<PathBuf>) -> Result<(), String> {
    let title = format!("Graal {} (Seihai Ato)", version.label());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([CANVAS.0, CANVAS.1]).with_resizable(false),
        ..Default::default()
    };
    let app = AtoApp::new(start, assets);
    eframe::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app)))).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> (Game, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        (Game::new(rx, None).unwrap(), tx)
    }

    fn arrival(start: Option<(f32, f32)>) -> Arrival {
        Arrival { name: "lab3x.nw".into(), source: crate::level::Source::Board, start, server_npcs: 0, file_npcs: 0 }
    }

    #[test]
    fn a_level_puts_the_player_at_the_start() {
        let (mut g, tx) = game();
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), arrival(Some((12.5, 40.0))))).unwrap();
        g.poll();
        assert_eq!((g.session.player_x, g.session.player_y), (12.5, 40.0));
        assert!(g.status.starts_with("level lab3x.nw"));
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), arrival(None))).unwrap();
        g.poll();
        assert_eq!((g.session.player_x, g.session.player_y), NO_START);
    }

    #[test]
    fn the_arrows_walk_at_the_session_pace() {
        let (mut g, tx) = game();
        // Tile 0 is open floor in the built-in tile types.
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), arrival(Some((30.0, 30.0))))).unwrap();
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

    #[test]
    fn nothing_walks_before_a_level() {
        let (mut g, _tx) = game();
        g.walk(1, 0, Instant::now() + Duration::from_secs(1));
        assert_eq!(g.session.player_x, OfflineSession::default().player_x);
    }

    #[test]
    fn the_view_is_the_3x_window() {
        assert_eq!(VIEW_TILES, (40, 30));
    }
}
