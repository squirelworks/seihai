//! Baddies: their pieces from the baddy's picture, the bow picture and the
//! sprite sheet (see `seihai_core::baddies`), and the verse over them.

use super::{Framebuffer, Sheet, TILE_PX};
use crate::baddies::{self as b, DrawTables, PieceSheet};
use crate::offline::OfflineSession;
use std::collections::HashMap;

/// The sheets baddies are cut from.
pub struct BaddyArt<'a> {
    pub sprites: Option<Sheet<'a>>,
    /// Pictures by name: each baddy's and the bow.
    pub images: &'a HashMap<String, (u32, Vec<u8>)>,
}

fn rect(tables: Option<&crate::assets::ObjectTables>, id: u16) -> Option<(i32, i32, i32, i32)> {
    crate::throwing::sprite_rect(tables, id).or_else(|| b::builtin_rect(id))
}

fn blit(fb: &mut Framebuffer, sheet: Sheet<'_>, r: (i32, i32, i32, i32), sx: i32, sy: i32) {
    let rows = sheet.px.len() / (sheet.w as usize * sheet.bpp).max(1);
    if r.0 < 0 || r.1 < 0 || r.0 + r.2 > sheet.w as i32 || r.1 + r.3 > rows as i32 {
        return;
    }
    fb.blit_sprite(sheet.px, sheet.w, sheet.bpp, r.0, r.1, r.2, r.3, sx, sy);
}

/// Draw the session's baddies from the camera `(cam_x, cam_y)`. `frame`
/// counts drawing frames (the hurt blink).
pub fn draw_baddies(fb: &mut Framebuffer, session: &OfflineSession, art: &BaddyArt, cam_x: i32, cam_y: i32, frame: u64) {
    if session.baddies.is_empty() {
        return;
    }
    let tables = session.object_tables.as_ref();
    let draw = DrawTables::from_tables(tables);
    let ext = &session.baddy_image_ext;
    let bow_name = format!("{}{}", b::BOW_IMAGE, ext);
    let sheet_of = |name: &str| art.images.get(name).map(|i| Sheet { px: i.1.as_slice(), w: i.0, bpp: 4 });
    for bd in &session.baddies {
        let own = sheet_of(&bd.image_name(ext));
        let bow = sheet_of(&bow_name);
        let (bx, by) = ((bd.x * TILE_PX as f64).round() as i32 - cam_x, (bd.y * TILE_PX as f64).round() as i32 - cam_y);
        for p in bd.pieces(&draw, frame) {
            let sheet = match p.sheet {
                PieceSheet::Baddy => own,
                PieceSheet::Bow => bow,
                PieceSheet::Sprites => art.sprites,
            };
            let (Some(sheet), Some(r)) = (sheet, rect(tables, p.sprite)) else { continue };
            blit(fb, sheet, r, bx + p.dx, by + p.dy);
        }
    }
}

/// The verse each baddy says, centred over `(x + 1, y - 2)`.
pub fn draw_baddy_verses(fb: &mut Framebuffer, session: &OfflineSession, font: Option<&super::chattext::ChatFont>, cam_x: i32, cam_y: i32) {
    let tile = TILE_PX as f64;
    for bd in &session.baddies {
        if let Some(text) = bd.shown_text() {
            let cx = ((bd.x + 1.0) * tile).round() as i32 - cam_x;
            let top = ((bd.y - 2.0) * tile).round() as i32 - cam_y;
            super::chattext::draw_npc_message(fb, font, text, cx, top);
        }
    }
}
