//! The v6 game loop in a window: log in with core's client session, draw
//! the level the server sends flat, walk it with the arrow keys, and play
//! the walk's sounds through core's sound engine, asking the server for
//! the ones not on disk (`WANTFILE`).
//!
//! The loop is `seihai-play`'s [`Game`], the one `seihai-ato` runs; Issho
//! does not depend on Ato. Issho has no start window yet: `--online` and
//! `--account` log in at once, and `--level` plays level files offline.
//!
//! This is the shared flat stand-in, not the 6.x playfield: no OpenGL
//! presenter, no `GuiGraalCtrl`, no FMOD and no GS2 (see [`crate::shell`]).

use crate::cli::Online;
use seihai_core::protocol::ClientVersion;
use seihai_play::game::{self, Game};
use seihai_play::view::View;

/// The window in pixels. The 6.x client opens at 640 x 480 as 3.x does
/// until a lab run of a 6.x build says otherwise.
pub const CANVAS: (f32, f32) = (640.0, 480.0);

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
    let g = Game::open_offline(resolver.as_ref(), level, sound_on)?;
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

    #[test]
    fn the_window_is_40_by_30_tiles() {
        assert_eq!(seihai_play::view::view_tiles(CANVAS), (40, 30));
    }
}
