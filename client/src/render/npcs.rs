//! Drawing the level's NPCs: their image (whole or a part of it), their
//! animation (`setcharani`, `setani`) and the images their scripts put up
//! with `showimg`.

use std::collections::HashMap;

use super::tiles::{draw_gani_frame, Framebuffer, PoseSheets, TILE_PX};
use crate::assets::{DrawLayer, ImageEffect, Npc, SCREEN_HEIGHT};
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
/// `(dx, dy)`. The part is limited to the image. With an effect on, the
/// picture is coloured and scaled about its centre (see [`blit_effect`]).
fn blit_image(fb: &mut Framebuffer, img: &(u32, Vec<u8>), part: Option<(i32, i32, i32, i32)>, dx: i32, dy: i32, effect: ImageEffect, light: bool) {
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
    if effect.is_on() {
        blit_effect(fb, px, w, (sx, sy, sw, sh), dx, dy, effect, light);
    } else {
        fb.blit_sprite(px, img.0, 4, sx, sy, sw, sh, dx, dy);
    }
}

/// Draw the part `(sx, sy, sw, sh)` of an RGBA picture scaled by the
/// effect's zoom about the part's centre, its colour multiplied by the
/// effect's red, green and blue, and blended by its alpha. A light
/// (`drawaslight`) adds its colour to what is under it; anything else is
/// laid over it. Seihai's reading of "drawn using alpha blending"; how the
/// 2.05 3D path blends was not read.
#[allow(clippy::too_many_arguments)]
pub fn blit_effect(fb: &mut Framebuffer, px: &[u8], w: i32, (sx, sy, sw, sh): (i32, i32, i32, i32), dx: i32, dy: i32, e: ImageEffect, light: bool) {
    let zoom = e.zoom.max(0.0);
    let (dw, dh) = ((sw as f32 * zoom).round() as i32, (sh as f32 * zoom).round() as i32);
    if dw <= 0 || dh <= 0 {
        return;
    }
    let (ox, oy) = (dx + (sw - dw) / 2, dy + (sh - dh) / 2);
    let [r, g, b, a] = e.color.map(|c| c.clamp(0.0, 1.0));
    for ty in 0..dh {
        let py = oy + ty;
        if py < 0 || py >= fb.height as i32 {
            continue;
        }
        let src_y = sy + ((ty as f32 / zoom) as i32).min(sh - 1);
        for tx in 0..dw {
            let qx = ox + tx;
            if qx < 0 || qx >= fb.width as i32 {
                continue;
            }
            let src_x = sx + ((tx as f32 / zoom) as i32).min(sw - 1);
            let si = ((src_y * w + src_x) * 4) as usize;
            let Some(p) = px.get(si..si + 4) else { continue };
            if p[3] == 0 {
                continue;
            }
            let di = (py as usize * fb.width as usize + qx as usize) * 3;
            let src = [p[0] as f32 * r, p[1] as f32 * g, p[2] as f32 * b];
            for (k, c) in src.iter().enumerate() {
                let d = fb.pixels[di + k] as f32;
                let v = if light { d + c * a } else { d * (1.0 - a) + c * a };
                fb.pixels[di + k] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// `seteffect`: lay the colour `(r, g, b)` over the whole picture with
/// alpha `a` (0 leaves it as it is).
pub fn shade_screen(fb: &mut Framebuffer, [r, g, b, a]: [f32; 4]) {
    let a = a.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let col = [r, g, b].map(|c| c.clamp(0.0, 1.0) * 255.0);
    for px in fb.pixels.chunks_exact_mut(3) {
        for (k, v) in px.iter_mut().enumerate() {
            *v = (*v as f32 * (1.0 - a) + col[k] * a).round() as u8;
        }
    }
}

/// Which part of the level's NPCs a [`draw_npcs`] call draws. The frame
/// draws [`NpcPass::BelowPlayers`] before the players and
/// [`NpcPass::AbovePlayers`] after them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcPass {
    /// The `drawunderplayer` NPCs, then the ones with the default height.
    BelowPlayers,
    /// The `drawoverplayer` NPCs, the `showimg` images (whose default
    /// height is over the player), then the `drawaslight` NPCs.
    AbovePlayers,
    /// The images `changeimgvis i,4` put on the screen layer, at screen
    /// pixels, drawn after the status bar.
    Screen,
}

/// Draw the NPCs of the session's level that belong to `pass`.
/// `(cam_x, cam_y)` is the camera origin in level pixels. A hidden NPC
/// (`hide`) draws nothing of its own; the images it put up with `showimg`
/// stay until `hideimg` removes them.
pub fn draw_npcs(fb: &mut Framebuffer, session: &OfflineSession, art: &NpcArt, cam_x: i32, cam_y: i32, pass: NpcPass) {
    let Some(level) = &session.current_level else { return };
    let at = |x: f32, y: f32| ((x * TILE_PX as f32) as i32 - cam_x, (y * TILE_PX as f32) as i32 - cam_y);
    let draw_layer = |fb: &mut Framebuffer, layer: DrawLayer| {
        for npc in level.npcs.iter().filter(|n| n.visible && n.layer == layer) {
            let (dx, dy) = at(npc.x, npc.y);
            if !draw_animation(fb, session, npc, art, dx, dy) {
                if let Some(img) = art.images.get(&npc.image) {
                    blit_image(fb, img, npc.image_part, dx, dy, npc.effect, layer == DrawLayer::Light);
                }
            }
        }
    };
    // The `showimg` images whose height is in `heights`, after the NPCs of
    // that height (`documentation/npc-draw-order.md`).
    let draw_images = |fb: &mut Framebuffer, heights: std::ops::RangeInclusive<i32>| {
        for npc in &level.npcs {
            for im in npc.images.iter().filter(|i| heights.contains(&i.height)) {
                if let Some(img) = art.images.get(&im.file) {
                    let (dx, dy) = if im.height >= SCREEN_HEIGHT { (im.x.round() as i32, im.y.round() as i32) } else { at(im.x, im.y) };
                    blit_image(fb, img, im.part, dx, dy, im.effect, im.height == DrawLayer::Light.height());
                }
            }
        }
    };
    match pass {
        NpcPass::BelowPlayers => {
            draw_layer(fb, DrawLayer::Under);
            draw_images(fb, i32::MIN..=-1);
            draw_layer(fb, DrawLayer::Player);
            draw_images(fb, 0..=0);
        }
        NpcPass::AbovePlayers => {
            draw_layer(fb, DrawLayer::Over);
            draw_images(fb, 1..=8);
            // Day and night shading lies under the lights.
            shade_screen(fb, session.screen_effect);
            draw_layer(fb, DrawLayer::Light);
            draw_images(fb, 9..=9);
        }
        NpcPass::Screen => draw_images(fb, SCREEN_HEIGHT..=i32::MAX),
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
