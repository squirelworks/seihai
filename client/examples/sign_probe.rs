//! Render a level's first sign as the client shows it: the view, the player
//! below the sign, and the sign panel from `letters.png`, one PNG a page.
//!
//! Usage: cargo run -p seihai --example sign_probe -- <assets> <level.nw> <out_dir>

use seihai::assets::{read_png_rgb, read_png_rgba, AssetRoot};
use seihai::offline::{Interaction, OfflineSession, DIR_NORTH};
use seihai::render::{player_box_origin, render_view, signs, Sheet};
use std::path::Path;

const VIEW_W: u32 = 40;
const VIEW_H: u32 = 30;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = AssetRoot::new(&args[0]);
    let out = Path::new(&args[2]);
    std::fs::create_dir_all(out).unwrap();
    let (sw, sh, sheet) = read_png_rgb(&std::fs::read(root.pics1()).unwrap()).unwrap();
    let letters = root.find_image("", "letters.png").expect("no letters.png");
    let (lw, _, lpx) = read_png_rgba(&std::fs::read(letters).unwrap()).unwrap();

    let mut session = OfflineSession::new();
    session.load_level(Path::new(&args[1])).unwrap();
    let sign = session.current_level.as_ref().and_then(|l| l.signs.first().cloned()).expect("no sign");
    session.player_x = sign.x as f32 - 0.5;
    session.player_y = sign.y as f32 + 0.5;
    session.player_dir = DIR_NORTH;
    let Interaction::Sign(text) = session.interact() else { panic!("the sign was not read") };
    let mut page = Some(0);
    while let Some(first) = page {
        let mut fb = render_view(&session, &sheet, sw, sh, VIEW_W, VIEW_H);
        let row = player_box_origin(&session, VIEW_W, VIEW_H).1 as f32 / 16.0;
        signs::draw_sign(&mut fb, &Sheet { px: &lpx, w: lw, bpp: 4 }, &text, first, row);
        let name = format!("sign-page-{}.png", first / signs::LINES_PER_PAGE + 1);
        let mut file = std::fs::File::create(out.join(&name)).unwrap();
        seihai::assets::write_png_rgb(&mut file, fb.width, fb.height, &fb.pixels).unwrap();
        println!("{name}: lines from {first}");
        page = signs::next_page(&text, first);
    }
}
