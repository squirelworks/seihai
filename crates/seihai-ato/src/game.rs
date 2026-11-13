//! Stage 3: the start window, then the level the server sent, drawn flat
//! and walked with the arrow keys.
//!
//! The loop itself (levels, the walk, links, gmaps, the other players,
//! board and NPC changes, the walk's sounds and asking the server for the
//! missing ones) is `seihai-play`'s [`Game`], shared with Issho. Ato adds
//! the start window, the server list, the 3.x window size and its Torque
//! field lines under each level.

use crate::online::Login;
use crate::server_list::{self, Pick, ServerList};
use crate::start_window::{Next, StartWindow, CANVAS};
use seihai_core::protocol::ClientVersion;
use seihai_play::game;
use seihai_play::view::{view_tiles, View};
use std::path::PathBuf;

pub use seihai_play::game::{Event, Game};

/// The view, in tiles: 640 x 480 pixels, the 3.x window size.
pub fn view_size() -> (u32, u32) {
    view_tiles(CANVAS)
}

/// The start window, then the game view once Start logs in.
pub struct AtoApp {
    start: StartWindow,
    game: Option<View>,
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

    /// As [`AtoApp::new`], but with `skip_start` log in at once with the
    /// form as given, as if Start had been pressed. For lab runs that cannot
    /// press Start (see [`skips_start`]).
    pub fn new_skipping_start(
        start: StartWindow,
        assets: Option<PathBuf>,
        skip_start: bool,
    ) -> AtoApp {
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
        let resolver = game::resolver(self.assets.as_ref(), version);
        let repaint = ctx.clone();
        let (rx, live) = game::spawn_online(
            addr,
            version,
            login,
            self.assets.clone(),
            crate::fields::lines,
            move || repaint.request_repaint(),
        );
        match Game::new(rx, resolver.as_ref(), self.sound_on) {
            Ok(mut g) => {
                g.attach(live);
                self.game = Some(View::new(g, CANVAS));
            }
            Err(e) => self.start.status = e,
        }
    }
}

impl AtoApp {
    /// Act on where Start went.
    fn next(&mut self, next: Next, ctx: &egui::Context) {
        if next != Next::Stay {
            self.start
                .remembered(seihai_play::start::Saved::load())
                .save();
        }
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
            match server_list::show(list, ctx) {
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
    let resolver = game::resolver(assets.as_ref(), version);
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
    app.game = Some(View::new(game, CANVAS));
    seihai_play::window::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app))))
        .map_err(|e| e.to_string())
}

/// True when the command line names a server and an account, so the start
/// window can be skipped. Decided before the saved form fills the window,
/// so a remembered account never logs in on its own.
pub fn skips_start(start: &StartWindow) -> bool {
    start.online.is_some() && !start.account.is_empty()
}

/// Open the window and block until it is closed. With `skip_start` the
/// start window is skipped (see [`skips_start`]).
pub fn run(
    version: ClientVersion,
    start: StartWindow,
    assets: Option<PathBuf>,
    sound_on: bool,
    skip_start: bool,
) -> Result<(), String> {
    let title = format!("Graal {} (Seihai Ato)", version.label());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([CANVAS.0, CANVAS.1])
            .with_resizable(false),
        ..Default::default()
    };
    let mut app = AtoApp::new_skipping_start(start, assets, skip_start);
    app.sound_on = sound_on;
    seihai_play::window::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app))))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(view_size(), (40, 30));
    }
}
