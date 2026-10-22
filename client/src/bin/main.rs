//! Seihai offline viewer.
//!
//! Cross-platform (Windows / Linux / macOS). Uses a software framebuffer
//! and minifb, so no system SDL2 is required.
//!
//! Usage:
//!   seihai --assets <dir> [--level <name.nw>]
//!
//! The asset dir should contain pics1.png and optionally tiledefs.txt
//! and a levels/ subdirectory.

use seihai::assets::{read_png_rgb, read_png_rgba, AssetRoot, Level};
use seihai::offline::OfflineSession;
use seihai::render::{
    draw_player_marker, draw_player_sprites, player_box_origin, render_view, Framebuffer,
};
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut assets: Option<PathBuf> = None;
    let mut level_name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--assets" | "-a" => {
                i += 1;
                assets = args.get(i).map(PathBuf::from);
            }
            "--level" | "-l" => {
                i += 1;
                level_name = args.get(i).cloned();
            }
            "--help" | "-h" => {
                println!("seihai --assets <dir> [--level <name.nw>]");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let root = match assets {
        Some(p) => AssetRoot::new(p),
        None => {
            eprintln!("usage: seihai --assets <dir> [--level <name.nw>]");
            return;
        }
    };
    if !root.is_valid() {
        eprintln!("no pics1.png in {}", root.pics1().display());
        return;
    }

    let sheet_data = match std::fs::read(root.pics1()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read pics1.png: {e}");
            return;
        }
    };
    let (sw, sh, sheet) = match read_png_rgb(&sheet_data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse pics1.png: {e}");
            return;
        }
    };

    let mut session = OfflineSession::new();
    if let Some(name) = level_name {
        let path = root.level(&name);
        if let Err(e) = session.load_level(&path) {
            eprintln!("failed to load level: {e}");
        }
    }
    if root.tiledefs().is_file() {
        let _ = session.load_tiletypes(root.tiledefs());
    } else if root.arrays_dat().is_file() {
        let _ = session.load_tiletypes_arrays(root.arrays_dat());
    }

    let view_w = 40u32;
    let view_h = 30u32;
    let load = |dir: &str, name: &str| {
        let data = std::fs::read(root.find_image(dir, name)?).ok()?;
        read_png_rgba(&data).ok().map(|(w, _, px)| (w, px))
    };
    let body = load("bodies", "body.png");
    let head = load("heads", "head0.png");
    let sprites = load("", "sprites.png");
    if body.is_none() || head.is_none() {
        eprintln!("no body.png / head0.png found; drawing a marker for the player");
    }
    let draw = |session: &OfflineSession| -> (Framebuffer, Vec<u32>) {
        let mut fb = render_view(session, &sheet, sw, sh, view_w, view_h);
        let (dx, dy) = player_box_origin(session, view_w, view_h);
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
            ),
            _ => draw_player_marker(&mut fb, session.in_water, dx, dy),
        }
        fb.draw_hud(session.player_health, 0);
        // Convert RGB to minifb's 0RGB u32.
        let buffer = fb
            .pixels
            .chunks(3)
            .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
            .collect();
        (fb, buffer)
    };
    let (fb, mut buffer) = draw(&session);

    let mut window = match minifb::Window::new(
        "Seihai",
        fb.width as usize,
        fb.height as usize,
        minifb::WindowOptions {
            resize: false,
            ..minifb::WindowOptions::default()
        },
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("failed to open window: {e}");
            return;
        }
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        // Arrow keys move the player.
        let mut moved = false;
        if window.is_key_down(minifb::Key::Left) {
            session.try_move(-1.0, 0.0);
            moved = true;
        }
        if window.is_key_down(minifb::Key::Right) {
            session.try_move(1.0, 0.0);
            moved = true;
        }
        if window.is_key_down(minifb::Key::Up) {
            session.try_move(0.0, -1.0);
            moved = true;
        }
        if window.is_key_down(minifb::Key::Down) {
            session.try_move(0.0, 1.0);
            moved = true;
        }
        if moved {
            buffer = draw(&session).1;
        }
        let _ = window.update_with_buffer(&buffer, fb.width as usize, fb.height as usize);
    }
}
