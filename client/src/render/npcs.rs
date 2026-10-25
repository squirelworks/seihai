//! Drawing the level's NPCs: their image (whole or a part of it), their
//! animation (`setcharani`, `setani`) and the images their scripts put up
//! with `showimg`.

use std::collections::HashMap;

use super::tiles::{draw_gani_frame, Framebuffer, PoseSheets, TILE_PX};
use crate::assets::Npc;
use crate::offline::OfflineSession;

/// The facing an animated NPC is drawn with (down; NPCs have no direction
/// of their own in Seihai yet).
pub const NPC_DIR: u8 = 2;

/// What the NPC drawing reads from: the decoded NPC images by file name
/// (`width`, RGBA pixels) and the sheets a GANI names (`BODY`, `HEAD`, ...).
#[derive(Clone, Copy)]
pub struct NpcArt<'a> {
    pub images: &'a HashMap<String, (u32, Vec<u8>)>,
    pub sheets: PoseSheets<'a>,
}

/// Blit an RGBA image, or a part `(x, y, w, h)` of it, with its top-left at
/// `(dx, dy)`. The part is limited to the image.
fn blit_image(fb: &mut Framebuffer, img: &(u32, Vec<u8>), part: Option<(i32, i32, i32, i32)>, dx: i32, dy: i32) {
    let (w, px) = (img.0 as i32, &img.1);
    if w <= 0 {
        return;
    }
    let h = (px.len() / (w as usize * 4)) as i32;
    let (sx, sy, sw, sh) = match part {
        Some((x, y, pw, ph)) => {
            let (x, y) = (x.clamp(0, w), y.clamp(0, h));
            (x, y, pw.clamp(0, w - x), ph.clamp(0, h - y))
        }
        None => (0, 0, w, h),
    };
    fb.blit_sprite(px, img.0, 4, sx, sy, sw, sh, dx, dy);
}

/// Draw every NPC of the session's level, then their `showimg` images.
/// `(cam_x, cam_y)` is the camera origin in level pixels. A hidden NPC
/// (`hide`) draws nothing of its own; the images it put up with `showimg`
/// stay until `hideimg` removes them.
pub fn draw_npcs(fb: &mut Framebuffer, session: &OfflineSession, art: &NpcArt, cam_x: i32, cam_y: i32) {
    let Some(level) = &session.current_level else { return };
    let at = |x: f32, y: f32| ((x * TILE_PX as f32) as i32 - cam_x, (y * TILE_PX as f32) as i32 - cam_y);
    for npc in level.npcs.iter().filter(|n| n.visible) {
        let (dx, dy) = at(npc.x, npc.y);
        if !draw_animation(fb, session, npc, art, dx, dy) {
            if let Some(img) = art.images.get(&npc.image) {
                blit_image(fb, img, npc.image_part, dx, dy);
            }
        }
    }
    for npc in &level.npcs {
        for im in &npc.images {
            if let Some(img) = art.images.get(&im.file) {
                let (dx, dy) = at(im.x, im.y);
                blit_image(fb, img, im.part, dx, dy);
            }
        }
    }
}

/// Draw the NPC as its animation. False when it has none, or the animation
/// or the sheets it needs are not loaded (the image is drawn then).
fn draw_animation(fb: &mut Framebuffer, session: &OfflineSession, npc: &Npc, art: &NpcArt, dx: i32, dy: i32) -> bool {
    let Some(name) = &npc.ani else { return false };
    let Some(gani) = session.npc_ganis.get(name) else { return false };
    if gani.frame_count() == 0 || art.sheets.body.is_none() || art.sheets.head.is_none() {
        return false;
    }
    draw_gani_frame(fb, gani, npc.ani_frame, NPC_DIR, &art.sheets, dx, dy);
    true
}
