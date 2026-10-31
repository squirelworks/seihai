//! Run a level's baddies offline next to a standing player and write frames.
//!
//! Usage: cargo run -p seihai --example baddy_probe -- <assets> <level.nw> <out_dir> [px py] [ticks] [swing]
//!
//! Every 10th player tick (500 ms) writes `baddies-NNN.png` and prints each
//! baddy's mode, position and power, and the player's health.

use seihai::assets::{read_png_rgb, read_png_rgba, AssetRoot};
use seihai::offline::OfflineSession;
use seihai::render::{draw_baddies, draw_player_marker, player_box_origin, render_view, BaddyArt, Framebuffer, Sheet};
use std::collections::HashMap;
use std::path::Path;

const VIEW_W: u32 = 40;
const VIEW_H: u32 = 30;

fn load_rgba(path: &Path) -> Option<(u32, Vec<u8>)> {
    let d = std::fs::read(path).ok()?;
    read_png_rgba(&d).ok().map(|(w, _, px)| (w, px))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = AssetRoot::new(&args[0]);
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out).unwrap();
    let px: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(30.0);
    let py: f32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(30.0);
    let ticks: u32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(200);

    let (sw, sh, sheet) = read_png_rgb(&std::fs::read(root.pics1()).unwrap()).unwrap();
    let sprites = root.find_image("", "sprites.png").and_then(|p| load_rgba(&p));
    let mut images: HashMap<String, (u32, Vec<u8>)> = HashMap::new();
    for (dir, name) in seihai::baddies::IMAGES.iter().map(|i| ("baddies", format!("{i}.png"))).chain([("weapons", format!("{}.png", seihai::baddies::BOW_IMAGE))]) {
        if let Some(img) = root.find_image(dir, &name).and_then(|p| load_rgba(&p)) {
            images.insert(name, img);
        }
    }
    println!("sprites {} images {}", sprites.is_some(), images.len());

    let mut session = OfflineSession::new();
    session.load_level(Path::new(&args[1])).unwrap();
    if root.arrays_dat().is_file() {
        let _ = session.load_tiletypes_arrays(root.arrays_dat());
    }
    session.player_x = px;
    session.player_y = py;
    session.player_dir = seihai::offline::DIR_SOUTH;
    let profile = seihai::protocol::ClientVersion::V205.profile();
    session.life.set_rules(profile.life.death);
    session.combat.set_rules(profile.life.combat);
    session.sword_power = 1;
    let swing = args.get(6).is_some_and(|s| s == "swing");

    let snap = |session: &OfflineSession, n: u32| {
        let mut fb: Framebuffer = render_view(session, &sheet, sw, sh, VIEW_W, VIEW_H);
        let cam_x = ((session.player_x + 1.5) * 16.0 - (VIEW_W as f32 * 16.0) / 2.0) as i32;
        let cam_y = ((session.player_y + 1.5) * 16.0 - (VIEW_H as f32 * 16.0) / 2.0) as i32;
        let art = BaddyArt { sprites: sprites.as_ref().map(|(w, p)| Sheet { px: p.as_slice(), w: *w, bpp: 4 }), images: &images };
        draw_baddies(&mut fb, session, &art, cam_x, cam_y, u64::from(n));
        let (dx, dy) = player_box_origin(session, VIEW_W, VIEW_H);
        draw_player_marker(&mut fb, false, dx, dy);
        fb.draw_hud(session.player_health, 0);
        let mut file = std::fs::File::create(out.join(format!("baddies-{n:03}.png"))).unwrap();
        seihai::assets::write_png_rgb(&mut file, fb.width, fb.height, &fb.pixels).unwrap();
    };

    for n in 0..=ticks {
        // With `swing`: face the nearest baddy within two tiles and swing.
        if swing && session.combat.swing_elapsed_ms().is_none() {
            let (cx, cy) = (f64::from(session.player_x) + 1.5, f64::from(session.player_y) + 2.0);
            let near = session.baddies.iter().filter(|b| !b.removed).map(|b| (b.x + 1.0 - cx, b.y + 1.0 - cy)).find(|(dx, dy)| dx.abs() < 2.5 && dy.abs() < 2.5);
            if let Some((dx, dy)) = near {
                session.player_dir = if dx.abs() > dy.abs() { if dx < 0.0 { 1 } else { 3 } } else if dy < 0.0 { 0 } else { 2 };
                session.start_swing();
            }
        }
        session.tick_combat(50);
        session.tick_baddies(50, &[]);
        if n % 10 == 0 {
            snap(&session, n);
            let list: Vec<String> =
                session.baddies.iter().map(|b| format!("#{} k{} m{} ({:.2},{:.2}) p{}", b.id, b.kind, b.mode, b.x, b.y, b.power)).collect();
            println!("{n:3}: health {} {}", session.player_health, list.join("  "));
        }
    }
}
