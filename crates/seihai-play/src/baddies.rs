//! The level's baddies, drawn as `seihai-mae`'s `render::baddies` draws
//! them: each baddy's pieces (core's `Baddy::pieces`) cut from its own
//! picture, the archer's bow picture (`wbow1`) or `sprites.png`, with the
//! sword spark after a clash and the verse a baddy says over it.
//!
//! Which baddies there are is core's: offline the level's `BADDY` records
//! come alive, online the server's `PLO_BADDYPROPS` place them, and the
//! level leader runs their AI (`OfflineSession::tick_baddies`). A picture
//! that is not in the asset folders leaves its pieces out.

use crate::render::{Frame, Image, NpcImages, TILE_PX};
use seihai_core::assets::AssetResolver;
use seihai_core::baddies::{self as b, DrawTables, PieceSheet};
use seihai_core::offline::OfflineSession;
use seihai_draw::draw_sprite;

/// The pictures the session's baddies need, by name: each baddy's own and
/// the bow. A name looked for and not found is kept as `None`, so it is not
/// looked for again. Returns the names that were not found.
pub fn load_images(
    s: &OfflineSession,
    resolver: Option<&AssetResolver>,
    images: &mut NpcImages,
) -> Vec<String> {
    let ext = &s.baddy_image_ext;
    let mut names: Vec<String> = s.baddies.iter().map(|bd| bd.image_name(ext)).collect();
    if !s.baddies.is_empty() {
        names.push(format!("{}{}", b::BOW_IMAGE, ext));
    }
    let mut missing = Vec::new();
    for name in names {
        if images.contains_key(&name) {
            continue;
        }
        let image = resolver
            .and_then(|r| r.image(&name))
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|d| Image::decode(&name, &d));
        if image.is_none() {
            missing.push(name.clone());
        }
        images.insert(name, image);
    }
    missing
}

fn rect(s: &OfflineSession, id: u16) -> Option<(i32, i32, i32, i32)> {
    seihai_core::throwing::sprite_rect(s.object_tables.as_ref(), id).or_else(|| b::builtin_rect(id))
}

/// Piece `r` of `sheet` with its top-left at `(x, y)`; a rectangle outside
/// the sheet draws nothing.
fn blit(f: &mut Frame, sheet: &Image, r: (i32, i32, i32, i32), x: i32, y: i32) {
    if r.0 < 0 || r.1 < 0 || r.0 + r.2 > sheet.w as i32 || r.1 + r.3 > sheet.h as i32 {
        return;
    }
    draw_sprite(
        f,
        seihai_draw::Image::rgba(&sheet.rgba, sheet.w, sheet.h),
        r,
        x,
        y,
    );
}

/// Draw the session's baddies from the camera `cam`, and the spark on the
/// player's sword after it hit a baddy's. `sprites` is `sprites.png`;
/// `frame` counts 50 ms frames (the hurt blink).
pub fn draw(
    f: &mut Frame,
    s: &OfflineSession,
    images: &NpcImages,
    sprites: Option<&Image>,
    cam: (i32, i32),
    frame: u64,
) {
    if s.baddies.is_empty() && s.sword_spark.is_none() {
        return;
    }
    let draw = DrawTables::from_tables(s.object_tables.as_ref());
    let ext = &s.baddy_image_ext;
    let picture = |name: &str| images.get(name).and_then(Option::as_ref);
    let bow = picture(&format!("{}{}", b::BOW_IMAGE, ext));
    let t = TILE_PX as f64;
    for bd in &s.baddies {
        let own = picture(&bd.image_name(ext));
        let (bx, by) = (
            (bd.x * t).round() as i32 - cam.0,
            (bd.y * t).round() as i32 - cam.1,
        );
        for p in bd.pieces(&draw, frame) {
            let sheet = match p.sheet {
                PieceSheet::Baddy => own,
                PieceSheet::Bow => bow,
                PieceSheet::Sprites => sprites,
            };
            if let (Some(sheet), Some(r)) = (sheet, rect(s, p.sprite)) {
                blit(f, sheet, r, bx + p.dx, by + p.dy);
            }
        }
    }
    if let (Some(spark), Some(sheet)) = (s.sword_spark, sprites) {
        let (sx, sy) = (
            (spark.x * t).round() as i32 - cam.0,
            (spark.y * t).round() as i32 - cam.1,
        );
        for p in spark.pieces(&draw, s.player_dir) {
            if let Some(r) = rect(s, p.sprite) {
                blit(f, sheet, r, sx + p.dx, sy + p.dy);
            }
        }
    }
}

/// The verse each baddy says, with where it goes in view pixels: centred
/// over `(x + 1, y - 2)`, as Mae draws it.
pub fn verses(s: &OfflineSession, cam_x: i32, cam_y: i32) -> Vec<(String, f32, f32)> {
    let t = TILE_PX as f32;
    s.baddies
        .iter()
        .filter_map(|bd| {
            let text = bd.shown_text()?;
            Some((
                text.to_string(),
                (bd.x as f32 + 1.0) * t - cam_x as f32,
                (bd.y as f32 - 2.0) * t - cam_y as f32,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::assets::Level;

    /// An offline level's `BADDY` record comes alive on the first tick and
    /// is drawn from `sprites.png` pieces or its picture; with nothing to
    /// cut from, nothing is drawn and nothing breaks.
    #[test]
    fn a_level_baddy_comes_alive_and_is_drawn() {
        let text = "GLEVNW01\nBADDY 20 20 0\n\n\n\nBADDYEND\n";
        let mut s = OfflineSession::new();
        s.current_level = Some(Level::parse(text).expect("level"));
        s.level_serial += 1;
        s.player_x = 40.0;
        s.player_y = 40.0;
        assert!(s.baddies.is_empty());
        s.tick_baddies(50, &[]);
        assert_eq!(s.baddies.len(), 1, "the record came alive offline");
        let mut images = NpcImages::new();
        let missing = load_images(&s, None, &mut images);
        assert_eq!(missing.len(), 2, "its picture and the bow: {missing:?}");
        assert!(
            load_images(&s, None, &mut images).is_empty(),
            "looked for once"
        );
        let mut f = Frame::filled(64 * 16, 64 * 16, [0, 0, 0, 255]);
        let before = f.rgba.clone();
        draw(&mut f, &s, &images, None, (0, 0), 0);
        assert_eq!(f.rgba, before, "no sheets: nothing drawn");
        let sprites = Image::decode(
            "sprites.png",
            seihai_core::assets::builtin::bytes("sprites.png").unwrap(),
        );
        draw(&mut f, &s, &images, sprites.as_ref(), (0, 0), 0);
        assert!(verses(&s, 0, 0).len() <= 1);
    }
}
