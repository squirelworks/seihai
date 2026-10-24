//! Render the player, chests and HUD on a level to PNG files.
//!
//! Usage: cargo run -p seihai --example render_sprites -- <assets> <level.nw> <out_dir>
//!
//! Writes `viewer-sprites.png` (standing, facing down) and one image
//! per step north, east, south and west. Without body/head sheets the
//! player falls back to a coloured marker.

use seihai::assets::{read_png_rgb, read_png_rgba, AssetRoot};
use seihai::offline::OfflineSession;
use seihai::render::{
    draw_player_marker, draw_player_sprites, player_box_origin, render_view, Framebuffer,
};
use std::path::{Path, PathBuf};

const VIEW_W: u32 = 40;
const VIEW_H: u32 = 30;

fn load_rgba(path: Option<PathBuf>) -> Option<(u32, Vec<u8>)> {
    let path = path?;
    match std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|d| read_png_rgba(&d).map_err(|e| e.to_string()))
    {
        Ok((w, _, px)) => Some((w, px)),
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            None
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = AssetRoot::new(&args[0]);
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out).unwrap();

    let (sw, sh, sheet) = read_png_rgb(&std::fs::read(root.pics1()).unwrap()).unwrap();
    let body = load_rgba(root.find_image("bodies", "body.png"));
    let head = load_rgba(root.find_image("heads", "head0.png"));
    let sprites = load_rgba(root.find_image("", "sprites.png"));
    println!(
        "body {} head {} sprites {}",
        body.is_some(),
        head.is_some(),
        sprites.is_some()
    );

    let mut session = OfflineSession::new();
    let level = root.level(&args[1]);
    let level = if level.is_file() {
        level
    } else {
        root.root.join(&args[1])
    };
    session.load_level(&level).unwrap();
    if root.tiledefs().is_file() {
        let _ = session.load_tiletypes(root.tiledefs());
    }
    if let Some(l) = &session.current_level {
        println!("chests {}", l.chests.len());
    }

    let snap = |session: &OfflineSession, name: &str| {
        let mut fb: Framebuffer = render_view(session, &sheet, sw, sh, VIEW_W, VIEW_H);
        let (dx, dy) = player_box_origin(session, VIEW_W, VIEW_H);
        match (&body, &head) {
            (Some((bw, b)), Some((hw, h))) => draw_player_sprites(
                &mut fb,
                b,
                *bw,
                4,
                h,
                *hw,
                4,
                sprites.as_ref().map(|(w, p)| (p.as_slice(), *w, 4)),
                session.player_dir,
                dx,
                dy,
                session.in_water,
            ),
            _ => draw_player_marker(&mut fb, session.in_water, dx, dy),
        }
        fb.draw_hud(session.player_health, 0);
        let mut file = std::fs::File::create(out.join(name)).unwrap();
        seihai::assets::write_png_rgb(&mut file, fb.width, fb.height, &fb.pixels).unwrap();
        println!(
            "{name}: x {} y {} dir {}",
            session.player_x, session.player_y, session.player_dir
        );
    };

    // The lab server's start position.
    session.player_x = 30.0;
    session.player_y = 30.5;
    session.player_dir = seihai::offline::DIR_SOUTH;
    snap(&session, "viewer-sprites.png");
    for (name, dx, dy) in [
        ("dir-north.png", 0.0, -1.0),
        ("dir-east.png", 1.0, 0.0),
        ("dir-south.png", 0.0, 1.0),
        ("dir-west.png", -1.0, 0.0),
    ] {
        session.try_move(dx, dy);
        snap(&session, name);
    }
}
