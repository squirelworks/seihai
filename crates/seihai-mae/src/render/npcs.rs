//! Drawing the level's NPCs: their image (whole or a part of it), their
//! animation (`setcharani`, `setani`) and the images their scripts put up
//! with `showimg`.

use std::collections::HashMap;

use super::tiles::{compose_gani_frame, draw_gani_frame, Framebuffer, PoseSheets, Sheet, TILE_PX};
use crate::assets::{DrawLayer, Gani, ImageEffect, Npc, SCREEN_HEIGHT};
use crate::looks;
use crate::offline::OfflineSession;

/// The facing an NPC starts with (down) until its script or the server
/// turns it (`Npc::dir`).
pub const NPC_DIR: u8 = 2;

/// What the NPC drawing reads from: the decoded NPC images by file name
/// (`width`, RGBA pixels) and the sheets a GANI names (`BODY`, `HEAD`, ...).
#[derive(Clone, Copy)]
pub struct NpcArt<'a> {
    pub images: &'a HashMap<String, (u32, Vec<u8>)>,
    pub sheets: PoseSheets<'a>,
    /// The build draws GANI attributes (`ATTR1` … `ATTR5`, from 2.02).
    pub gani_attrs: bool,
}

/// The sheets a character NPC is drawn with: the head, body, shield, sword
/// and attributes its `setcharprop` set, when those pictures are loaded,
/// else `art.sheets`. The body is the one recoloured in the NPC's colours
/// (`looks::colored_body_key`) when that is loaded.
pub fn npc_sheets<'a>(npc: &Npc, gani: &Gani, art: &NpcArt<'a>) -> PoseSheets<'a> {
    let images = art.images;
    let sheet = |n: Option<&str>| n.and_then(|n| images.get(n)).map(|i| Sheet { px: i.1.as_slice(), w: i.0, bpp: 4 });
    let l = looks::npc_looks(npc);
    let colored = sheet(Some(&looks::colored_body_key(l.body_name(), &l.colors)));
    let mut attrs = PoseSheets::default().attrs;
    if art.gani_attrs {
        for (i, a) in attrs.iter_mut().enumerate() {
            *a = sheet(looks::attr_image(&l.attrs, gani, i));
        }
    }
    PoseSheets {
        body: colored.or(sheet(l.body.as_deref())).or(art.sheets.body),
        head: sheet(l.head.as_deref()).or(art.sheets.head),
        sprites: art.sheets.sprites,
        sword: sheet(npc.props.get("#1").map(String::as_str)).or(art.sheets.sword),
        shield: sheet(npc.shield.as_deref()).or(art.sheets.shield),
        attrs,
    }
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
    // that height (`documentation/npc-draw-order.md`): the NPCs' own, then
    // the ones weapon scripts put up for the player.
    let draw_images = |fb: &mut Framebuffer, heights: std::ops::RangeInclusive<i32>| {
        let all = level.npcs.iter().flat_map(|n| n.images.iter()).chain(&session.player_images);
        for im in all.filter(|i| heights.contains(&i.height)) {
            if let Some(img) = art.images.get(&im.file) {
                let (dx, dy) = if im.height >= SCREEN_HEIGHT { (im.x.round() as i32, im.y.round() as i32) } else { at(im.x, im.y) };
                blit_image(fb, img, im.part, dx, dy, im.effect, im.height == DrawLayer::Light.height());
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

/// Draw the `message` of each visible NPC of the session's level in the
/// chat style ([`super::chattext::draw_npc_message`]): centred over the
/// NPC's picture, the top of its last line one tile above the NPC. The
/// picture is the image part (or the whole image) for an image NPC and the
/// 48-pixel player box for a character, as for players. Checked against the
/// 2.05 rev 2 client: a 32-pixel crate and a 64-pixel image part both get
/// their text centred over them, glyphs just clear of the top edge.
pub fn draw_npc_messages(fb: &mut Framebuffer, session: &OfflineSession, art: &NpcArt, font: Option<&super::chattext::ChatFont>, cam_x: i32, cam_y: i32) {
    let Some(level) = &session.current_level else { return };
    let tile = TILE_PX as i32;
    for npc in level.npcs.iter().filter(|n| n.visible && !n.message.is_empty()) {
        let dx = (npc.x * TILE_PX as f32) as i32 - cam_x;
        let dy = (npc.y * TILE_PX as f32) as i32 - cam_y;
        super::chattext::draw_npc_message(fb, font, &npc.message, dx + message_width(npc, art) / 2, dy - tile);
    }
}

/// The width the NPC's message is centred over.
fn message_width(npc: &Npc, art: &NpcArt) -> i32 {
    if npc.ani.is_some() {
        return TILE_PX as i32 * 3;
    }
    let Some(w) = art.images.get(&npc.image).map(|i| i.0 as i32) else { return TILE_PX as i32 * 2 };
    match npc.image_part {
        Some((x, _, pw, _)) => pw.clamp(0, w - x.clamp(0, w)),
        None => w,
    }
}

/// Draw the NPC as its animation. False when it has none, or the animation
/// or the sheets it needs are not loaded (the image is drawn then).
fn draw_animation(fb: &mut Framebuffer, session: &OfflineSession, npc: &Npc, art: &NpcArt, dx: i32, dy: i32) -> bool {
    let Some(name) = &npc.ani else { return false };
    let Some(gani) = session.npc_gani(name) else { return false };
    let sheets = npc_sheets(npc, gani, art);
    if gani.frame_count() == 0 || sheets.body.is_none() || sheets.head.is_none() {
        return false;
    }
    // A character with `setcoloreffect` or `setzoomeffect`: the whole figure
    // is coloured and zoomed about its centre, as an image is.
    if npc.effect.is_on() {
        if let Some((ox, oy, w, px)) = compose_gani_frame(gani, npc.ani_frame, npc.dir, &sheets) {
            let h = (px.len() / (w as usize * 4)) as i32;
            blit_effect(fb, &px, w as i32, (0, 0, w as i32, h), dx + ox, dy + oy, npc.effect, npc.layer == DrawLayer::Light);
        }
        return true;
    }
    draw_gani_frame(fb, gani, npc.ani_frame, npc.dir, &sheets, dx, dy);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_centres_over_the_image_part_or_the_character_box() {
        let mut images = HashMap::new();
        images.insert("crate.gif".to_string(), (32u32, vec![0u8; 32 * 32 * 4]));
        images.insert("bomb1.png".to_string(), (128u32, vec![0u8; 128 * 16 * 4]));
        let art = NpcArt { images: &images, sheets: PoseSheets::default(), gani_attrs: true };
        let mut npc = Npc::new("crate.gif".into(), 30.0, 25.0, String::new());
        assert_eq!(message_width(&npc, &art), 32);
        npc.image = "bomb1.png".into();
        npc.image_part = Some((0, 0, 64, 16));
        assert_eq!(message_width(&npc, &art), 64);
        // A part running off the image is cut at its edge.
        npc.image_part = Some((100, 0, 64, 16));
        assert_eq!(message_width(&npc, &art), 28);
        npc.image = "missing.png".into();
        npc.image_part = None;
        assert_eq!(message_width(&npc, &art), 32);
        npc.ani = Some("idle".into());
        assert_eq!(message_width(&npc, &art), 48);
    }
}
