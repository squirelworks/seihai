//! The v6 game loop in a window: log in with core's client session, draw
//! the level the server sends flat, walk it with the arrow keys, and play
//! the walk's sounds through core's sound engine, asking the server for
//! the ones not on disk (`WANTFILE`).
//!
//! The loop is `seihai-play`'s [`Game`], the one `seihai-ato` runs; Issho
//! does not depend on Ato. Issho has no start window of its own: the
//! `seihai` program's shared start window hands a 6.x pick over with
//! `--online` and `--account`, which log in at once, as they do from the
//! command line. `--level` plays level files offline.
//!
//! The window is the 6.x size, 1024 x 768, with the 6.x default status
//! display (`seihai_play::hud`), both from a lab run of the real 6.00r2
//! and 6.03r7 clients (#444). The level is still drawn by the shared flat
//! renderer: no OpenGL presenter, no `GuiGraalCtrl`, no FMOD and no GS2
//! (see [`crate::shell`]).

use crate::cli::Online;
use seihai_core::protocol::ClientVersion;
use seihai_play::game::{self, Game};
use seihai_play::view::View;

/// The window in pixels: 1024 x 768, the client area of the real
/// Windows 6.00r2 and Linux 6.03r7 (lab, #444). The playfield fills it.
pub const CANVAS: (f32, f32) = (1024.0, 768.0);

/// The game in the window.
pub struct IsshoApp {
    view: View,
}

impl eframe::App for IsshoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.view.show(ctx);
    }
}

fn options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([CANVAS.0, CANVAS.1])
            .with_resizable(false),
        ..Default::default()
    }
}

/// Log in as `o` asks, open the window on the game, and block until it is
/// closed.
pub fn run(o: &Online, sound_on: bool) -> Result<(), String> {
    let title = format!("Graal {} (Seihai Issho)", o.version.label());
    let o = o.clone();
    seihai_play::window::run_native(
        &title,
        options(),
        Box::new(move |cc| {
            let repaint = cc.egui_ctx.clone();
            let resolver = game::resolver(o.assets.as_ref(), o.version);
            let (rx, live) = game::spawn_online(
                o.addr.clone(),
                o.version,
                o.login.clone(),
                o.assets.clone(),
                game::no_lines,
                move || repaint.request_repaint(),
            );
            let mut g = Game::new(rx, resolver.as_ref(), sound_on)?;
            g.attach(live);
            g.set_version(o.version);
            g.show_hud();
            Ok(Box::new(IsshoApp {
                view: View::new(g, CANVAS),
            }))
        }),
    )
    .map_err(|e| e.to_string())
}

/// Open the window on level `level` with no server, and block until it is
/// closed. Links load their level from the asset folder.
pub fn run_offline(
    version: ClientVersion,
    level: &str,
    assets: Option<std::path::PathBuf>,
    sound_on: bool,
) -> Result<(), String> {
    let resolver = game::resolver(assets.as_ref(), version);
    let mut g = Game::open_offline(resolver.as_ref(), level, sound_on)?;
    g.set_version(version);
    g.show_hud();
    let title = format!("Graal {} (Seihai Issho, offline)", version.label());
    let app = IsshoApp {
        view: View::new(g, CANVAS),
    };
    seihai_play::window::run_native(&title, options(), Box::new(move |_| Ok(Box::new(app))))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every 6.x row swings: its profile carries combat rules.
    #[test]
    fn every_6x_row_has_combat_rules() {
        for v in ClientVersion::all_versions()
            .iter()
            .filter(|v| crate::is_v6(**v))
        {
            assert!(v.profile().life.combat.is_some(), "{}", v.label());
        }
    }

    #[test]
    fn the_window_is_64_by_48_tiles() {
        assert_eq!(seihai_play::view::view_tiles(CANVAS), (64, 48));
    }
}
