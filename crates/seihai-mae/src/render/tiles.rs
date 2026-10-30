//! Software tile renderer.
//!
//! Composites a level board into an RGB buffer by blitting 16×16 tiles
//! from a 2048×512 sheet (`pics1.png` layout). No SDL2 required.
//!
//! Sheet layout: 128 columns × 32 rows of 16 px tiles, stored as eight
//! 16-column blocks of 512 tiles. Tile id `n` lives at column
//! `n % 16 + (n / 512) * 16`, row `(n / 16) % 32`.

use crate::assets::{tile_id, Gani, Level};
use crate::offline::OfflineSession;

/// Width of a tile in pixels.
pub const TILE_PX: u32 = 16;
/// Columns on the default `pics1.png` sheet.
pub const SHEET_COLS: u32 = 128;
/// Rows on the default `pics1.png` sheet.
pub const SHEET_ROWS: u32 = 32;

/// An RGB framebuffer (row-major, 3 bytes per pixel).
#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width as usize) * (height as usize) * 3],
        }
    }

    /// Blit a 16×16 tile from `sheet` (RGB, `sheet_w` × `sheet_h`) into
    /// this buffer at pixel position `(dx, dy)`.
    pub fn blit_tile(
        &mut self,
        sheet: &[u8],
        sheet_w: u32,
        sheet_h: u32,
        tile: u16,
        dx: i32,
        dy: i32,
    ) {
        let t = tile as u32;
        let col = t % 16 + (t / 512) * 16;
        let row = (t / 16) % SHEET_ROWS;
        let sx = col * TILE_PX;
        let sy = row * TILE_PX;
        if sx + TILE_PX > sheet_w || sy + TILE_PX > sheet_h {
            return;
        }
        for ty in 0..TILE_PX as i32 {
            for tx in 0..TILE_PX as i32 {
                let px = dx + tx;
                let py = dy + ty;
                if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
                    continue;
                }
                let sidx = ((sy as i32 + ty) as usize * sheet_w as usize
                    + (sx as i32 + tx) as usize)
                    * 3;
                let didx = (py as usize * self.width as usize + px as usize) * 3;
                if sidx + 2 < sheet.len() && didx + 2 < self.pixels.len() {
                    self.pixels[didx..didx + 3].copy_from_slice(&sheet[sidx..sidx + 3]);
                }
            }
        }
    }

    /// Fill a rectangle with a solid colour (used for the player marker).
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, rgb: [u8; 3]) {
        for py in y..y + h {
            for px in x..x + w {
                if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
                    continue;
                }
                let i = (py as usize * self.width as usize + px as usize) * 3;
                if i + 2 < self.pixels.len() {
                    self.pixels[i..i + 3].copy_from_slice(&rgb);
                }
            }
        }
    }

    /// Blit a sprite from an RGB or RGBA image.
    ///
    /// Transparent pixels (alpha 0, or magenta 255,0,255) are skipped.
    pub fn blit_sprite(
        &mut self,
        src: &[u8],
        src_w: u32,
        bpp: usize,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        dx: i32,
        dy: i32,
    ) {
        for ty in 0..sh {
            for tx in 0..sw {
                let px = dx + tx;
                let py = dy + ty;
                if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
                    continue;
                }
                let sidx = ((sy + ty) as usize * src_w as usize + (sx + tx) as usize) * bpp;
                if sidx + bpp > src.len() {
                    continue;
                }
                if bpp == 4 && src[sidx + 3] == 0 {
                    continue;
                }
                if bpp == 3 && src[sidx] == 255 && src[sidx + 1] == 0 && src[sidx + 2] == 255 {
                    continue;
                }
                let didx = (py as usize * self.width as usize + px as usize) * 3;
                if didx + 2 < self.pixels.len() {
                    self.pixels[didx..didx + 3].copy_from_slice(&src[sidx..sidx + 3]);
                }
            }
        }
    }

    /// Draw a simple HUD: health bar and rupee bar.
    pub fn draw_hud(&mut self, health: i32, rupees: i32) {
        let hp = health.max(0).min(100) as i32;
        self.fill_rect(4, 4, 100, 8, [0x40, 0x00, 0x00]);
        self.fill_rect(4, 4, hp, 8, [0xC0, 0x20, 0x20]);
        let ru = rupees.max(0).min(100) as i32;
        self.fill_rect(110, 4, 50, 8, [0x00, 0x40, 0x00]);
        self.fill_rect(110, 4, ru / 2, 8, [0x20, 0xC0, 0x20]);
    }

    /// Draw a 64×64 minimap of the level in the top-right corner.
    pub fn draw_minimap(&mut self, level: &crate::assets::Level, px: f32, py: f32) {
        let ox = self.width as i32 - 68;
        let oy = 4;
        self.fill_rect(ox, oy, 64, 64, [0x20, 0x20, 0x20]);
        for run in &level.boards {
            if run.layer != 0 {
                continue;
            }
            for (i, _) in run.tiles.iter().enumerate() {
                let tx = run.x + i as i32;
                let ty = run.y;
                if (0..64).contains(&tx) && (0..64).contains(&ty) {
                    self.fill_rect(ox + tx, oy + ty, 1, 1, [0x30, 0x60, 0x30]);
                }
            }
        }
        let (mx, my) = (px as i32, py as i32);
        if (0..64).contains(&mx) && (0..64).contains(&my) {
            self.fill_rect(ox + mx, oy + my, 2, 2, [0xFF, 0xFF, 0x40]);
        }
    }

    /// Draw hearts in 0.5 increments using a HUD sheet.
    ///
    /// `sheet` is a 96×16 RGBA image: full, half, empty heart, rupee, bomb, arrow.
    /// `health` and `max_health` are in half-hearts.
    pub fn draw_hearts(
        &mut self,
        sheet: &[u8],
        sheet_w: u32,
        health: i32,
        max_health: i32,
    ) {
        let hearts = (max_health.max(1) + 1) / 2;
        let filled = health.max(0);
        for i in 0..hearts {
            let sx = if filled >= (i + 1) * 2 {
                0
            } else if filled >= i * 2 + 1 {
                16
            } else {
                32
            };
            self.blit_sprite(sheet, sheet_w, 16, sx, 0, 4 + i * 18, 4, 16, 16);
        }
    }

    /// Darken a rectangle to `keep`/256 of its brightness, for text
    /// backgrounds that leave the level visible.
    pub fn shade_rect(&mut self, x: i32, y: i32, w: i32, h: i32, keep: u16) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                let i = (py as usize * self.width as usize + px as usize) * 3;
                for c in &mut self.pixels[i..i + 3] {
                    *c = ((*c as u16 * keep) >> 8) as u8;
                }
            }
        }
    }

    /// Draw `text` with the 5×7 font, top-left at `(x, y)`.
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, rgb: [u8; 3]) {
        let mut cx = x;
        for ch in text.chars() {
            let g = super::font::glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits >> (4 - col) & 1 != 0 {
                        self.fill_rect(cx + col, y + row as i32, 1, 1, rgb);
                    }
                }
            }
            cx += super::font::CELL_W;
        }
    }

    /// Mix `rgb` into one pixel at `alpha`/255 (0 leaves it, 255 replaces it).
    pub fn blend_pixel(&mut self, x: i32, y: i32, rgb: [u8; 3], alpha: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let i = (y as usize * self.width as usize + x as usize) * 3;
        let a = u16::from(alpha);
        for (c, &n) in self.pixels[i..i + 3].iter_mut().zip(rgb.iter()) {
            *c = ((u16::from(*c) * (255 - a) + u16::from(n) * a + 127) / 255) as u8;
        }
    }

    /// [`Self::draw_text`] blended over what is already there at
    /// `alpha`/255, for text that fades.
    pub fn draw_text_alpha(&mut self, x: i32, y: i32, text: &str, rgb: [u8; 3], alpha: u8) {
        let mut cx = x;
        for ch in text.chars() {
            let g = super::font::glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits >> (4 - col) & 1 != 0 {
                        self.blend_pixel(cx + col, y + row as i32, rgb, alpha);
                    }
                }
            }
            cx += super::font::CELL_W;
        }
    }

    /// Draw `text` with a one-pixel dark outline, so it reads on any tile.
    pub fn draw_text_outlined(&mut self, x: i32, y: i32, text: &str, rgb: [u8; 3]) {
        for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            self.draw_text(x + ox, y + oy, text, [0, 0, 0]);
        }
        self.draw_text(x, y, text, rgb);
    }
}

/// The chest art of the real 2.05 and 2.22 tile sheets: `arrays.dat` array 13
/// (closed) and array 14 (open), as top-left, top-right, bottom-left,
/// bottom-right tile ids. The client draws a chest from its record as a 32x32
/// blit of these tiles (closed from (896, 240), open from (464, 304) of the
/// sheet), 2x2 tiles at the record's x and y.
pub const CHEST_CLOSED_TILES: [u16; 4] = [1784, 1785, 1800, 1801];
pub const CHEST_OPEN_TILES: [u16; 4] = [829, 830, 845, 846];

/// The four tiles to draw for a chest record. The built-in sheet has its
/// chest art at the same ids as the real sheets.
pub fn chest_art(opened: bool) -> [u16; 4] {
    if opened {
        CHEST_OPEN_TILES
    } else {
        CHEST_CLOSED_TILES
    }
}

/// Render a camera view of the level into a framebuffer.
///
/// `sheet` is the RGB tile sheet (2048×512 for `pics1.png`).
/// `view_tiles_w` / `view_tiles_h` is the visible window in tiles.
/// The player is drawn as a 16×16 marker. Chest records are drawn over the
/// board (see [`chest_art`]); signs are not drawn, their art is the board's.
pub fn render_view(
    session: &OfflineSession,
    sheet: &[u8],
    sheet_w: u32,
    sheet_h: u32,
    view_tiles_w: u32,
    view_tiles_h: u32,
) -> Framebuffer {
    let mut fb = Framebuffer::new(view_tiles_w * TILE_PX, view_tiles_h * TILE_PX);
    let level = match &session.current_level {
        Some(l) => l,
        None => return fb,
    };
    let (cam_x, cam_y) = camera_origin(session, view_tiles_w, view_tiles_h);
    let t = TILE_PX as i32;
    let (ox, oy) = (cam_x.div_euclid(t), cam_y.div_euclid(t));
    let (sub_x, sub_y) = (cam_x.rem_euclid(t), cam_y.rem_euclid(t));

    // One extra row and column cover the sub-tile scroll.
    for ty in 0..=view_tiles_h as i32 {
        for tx in 0..=view_tiles_w as i32 {
            let lx = ox + tx;
            let ly = oy + ty;
            // The current level, or on a gmap the neighbour the tile is in.
            let (kx, ky) = (lx.div_euclid(64), ly.div_euclid(64));
            let at = if (kx, ky) == (0, 0) {
                Some(level)
            } else {
                session.gmap.as_ref().and_then(|w| w.levels.get(&(w.cell.0 + kx, w.cell.1 + ky)))
            };
            let Some(at) = at else { continue };
            if let Some(id) = tile_at(at, lx.rem_euclid(64), ly.rem_euclid(64)) {
                fb.blit_tile(sheet, sheet_w, sheet_h, id, tx * t - sub_x, ty * t - sub_y);
            }
        }
    }

    // Chests are drawn from their records, over the board; signs have no
    // art of their own on the board. An NPC whose image is not there yet
    // draws nothing, as in the 2.05 client (see npc-images.md).
    draw_chests(&mut fb, session, level, sheet, sheet_w, sheet_h, cam_x, cam_y);

    fb
}

/// Draw the level's chest records: a 2x2 block of the closed or open chest
/// tiles at the record's tile position. The client skips a chest whose x or y
/// is outside 0 to 62.
#[allow(clippy::too_many_arguments)]
fn draw_chests(fb: &mut Framebuffer, session: &OfflineSession, level: &crate::assets::Level, sheet: &[u8], sw: u32, sh: u32, cam_x: i32, cam_y: i32) {
    let t = TILE_PX as i32;
    for c in &level.chests {
        if !(0..=62).contains(&c.x) || !(0..=62).contains(&c.y) {
            continue;
        }
        let opened = c.opened || session.opened_chests.contains(&(c.x, c.y));
        for (i, id) in chest_art(opened).into_iter().enumerate() {
            let (dx, dy) = ((i % 2) as i32, (i / 2) as i32);
            fb.blit_tile(sheet, sw, sh, id, (c.x + dx) * t - cam_x, (c.y + dy) * t - cam_y);
        }
    }
}

/// Top-left pixel of the view in level pixels: the camera centres on
/// the middle of the player's 48×48 box (player x/y + 1.5 tiles).
fn camera_origin(session: &OfflineSession, view_tiles_w: u32, view_tiles_h: u32) -> (i32, i32) {
    let t = TILE_PX as f32;
    (
        ((session.player_x + 1.5) * t - (view_tiles_w as f32 * t) / 2.0) as i32,
        ((session.player_y + 1.5) * t - (view_tiles_h as f32 * t) / 2.0) as i32,
    )
}

/// Pixel position of the player's 48×48 box (top-left) in a view made
/// by [`render_view`] with the same size.
pub fn player_box_origin(session: &OfflineSession, view_tiles_w: u32, view_tiles_h: u32) -> (i32, i32) {
    let (cam_x, cam_y) = camera_origin(session, view_tiles_w, view_tiles_h);
    // The box starts at the player's tile position, the same as for other
    // players. The 2.05 client draws it there too (checked against a
    // screenshot at onlinestartlocal.nw 30, 30.5).
    (
        (session.player_x * TILE_PX as f32) as i32 - cam_x,
        (session.player_y * TILE_PX as f32) as i32 - cam_y,
    )
}

/// Fallback when no body/head sheets are loaded: a coloured square
/// where the body would be.
pub fn draw_player_marker(fb: &mut Framebuffer, in_water: bool, dx: i32, dy: i32) {
    let color = if in_water {
        [0x40, 0x80, 0xFF]
    } else {
        [0xFF, 0x40, 0x40]
    };
    fb.fill_rect(dx + 16, dy + 24, 16, 16, color);
}

/// Draw another player at a level position.
///
/// Uses the same body/head blit as the local player. `(cam_x, cam_y)` is
/// the camera origin in level pixels.
pub fn draw_other_player(
    fb: &mut Framebuffer,
    body: &[u8],
    body_w: u32,
    body_bpp: usize,
    head: &[u8],
    head_w: u32,
    head_bpp: usize,
    sprites: Option<(&[u8], u32, usize)>,
    dir: u8,
    level_x: f32,
    level_y: f32,
    cam_x: i32,
    cam_y: i32,
) {
    let dx = (level_x * TILE_PX as f32) as i32 - cam_x;
    let dy = (level_y * TILE_PX as f32) as i32 - cam_y;
    draw_player_sprites(
        fb, body, body_w, body_bpp, head, head_w, head_bpp, sprites, dir, dx, dy, false,
    );
}
///
/// `body` and `head` are images with `*_bpp` bytes per pixel (4 keeps
/// palette transparency). Direction is Graal order: 0 up, 1 left,
/// 2 down, 3 right. Body frames are 32×32 at column `dir`, row 0 of
/// `body.png`; head frames are 32×32 at row `dir` of the single-column
/// head sheet. `(dx, dy)` is the top-left of the 48×48 player box.
/// Offsets are the 2.05 `idle.gani` ones; `sprites` (if given) supplies
/// the shadow at 0,0 24×12.
pub fn draw_player_sprites(
    fb: &mut Framebuffer,
    body: &[u8],
    body_w: u32,
    body_bpp: usize,
    head: &[u8],
    head_w: u32,
    head_bpp: usize,
    sprites: Option<(&[u8], u32, usize)>,
    dir: u8,
    dx: i32,
    dy: i32,
    in_water: bool,
) {
    // (shadow, body, head) offsets per direction, from idle.gani.
    const OFFSETS: [((i32, i32), (i32, i32), (i32, i32)); 4] = [
        ((12, 34), (8, 16), (8, 2)),
        ((12, 36), (12, 16), (12, 2)),
        ((12, 34), (8, 16), (8, 2)),
        ((12, 36), (4, 16), (4, 2)),
    ];
    let d = (dir % 4) as usize;
    let (shadow, b, h) = OFFSETS[d];
    let d = d as i32;
    if in_water {
        // Swimming: only the head, lowered to the water line.
        fb.blit_sprite(head, head_w, head_bpp, 0, d * 32, 32, 32, dx + h.0, dy + h.1 + 14);
    } else {
        if let Some((spr, spr_w, spr_bpp)) = sprites {
            fb.blit_sprite(spr, spr_w, spr_bpp, 0, 0, 24, 12, dx + shadow.0, dy + shadow.1);
        }
        fb.blit_sprite(body, body_w, body_bpp, d * 32, 0, 32, 32, dx + b.0, dy + b.1);
        fb.blit_sprite(head, head_w, head_bpp, 0, d * 32, 32, 32, dx + h.0, dy + h.1);
    }
}

/// An RGBA (or RGB) image the GANI renderer draws from.
#[derive(Debug, Clone, Copy)]
pub struct Sheet<'a> {
    pub px: &'a [u8],
    pub w: u32,
    pub bpp: usize,
}

/// The sheets a player GANI names symbolically in its `SPRITE` lines.
#[derive(Debug, Clone, Copy, Default)]
pub struct PoseSheets<'a> {
    pub body: Option<Sheet<'a>>,
    pub head: Option<Sheet<'a>>,
    pub sprites: Option<Sheet<'a>>,
    pub sword: Option<Sheet<'a>>,
    pub shield: Option<Sheet<'a>>,
    /// The pictures of the `ATTR1` … `ATTR5` sprites (ATTR1 is the hat).
    pub attrs: [Option<Sheet<'a>>; 5],
}

/// Draw one step of a player GANI for direction `dir` (Graal order: 0 up,
/// 1 left, 2 down, 3 right) with the top-left of the 48x48 player box at
/// `(dx, dy)`. The placements of the direction's row are drawn in order, so
/// later ones are on top. A placement whose sprite is not defined, or whose
/// sheet is not loaded, is skipped. A GANI with a single direction row uses
/// it for every direction.
pub fn draw_gani_frame(fb: &mut Framebuffer, gani: &Gani, frame: usize, dir: u8, sheets: &PoseSheets, dx: i32, dy: i32) {
    let Some(step) = gani.frame(frame) else { return };
    let Some(row) = step.rows.get(usize::from(dir % 4)).or_else(|| step.rows.first()) else { return };
    for pl in row {
        let Some(sp) = gani.sprites.iter().find(|s| s.id == pl.sprite_id) else { continue };
        let sheet = match sp.sheet.to_ascii_uppercase().as_str() {
            "BODY" => sheets.body,
            "HEAD" => sheets.head,
            "SPRITES" => sheets.sprites,
            "SWORD" => sheets.sword,
            "SHIELD" => sheets.shield,
            "ATTR1" => sheets.attrs[0],
            "ATTR2" => sheets.attrs[1],
            "ATTR3" => sheets.attrs[2],
            "ATTR4" => sheets.attrs[3],
            "ATTR5" => sheets.attrs[4],
            _ => None,
        };
        if let Some(sh) = sheet {
            if sp.x >= 0 && sp.y >= 0 {
                fb.blit_sprite(sh.px, sh.w, sh.bpp, sp.x, sp.y, sp.w, sp.h, dx + pl.x, dy + pl.y);
            }
        }
    }
}

/// One step of a GANI drawn into its own RGBA picture, for drawing it again
/// with an effect: `(x, y, w, px)`, where `(x, y)` is the picture's top-left
/// against the GANI's origin. Transparent where nothing was drawn. `None`
/// when the step draws nothing.
pub fn compose_gani_frame(gani: &Gani, frame: usize, dir: u8, sheets: &PoseSheets) -> Option<(i32, i32, u32, Vec<u8>)> {
    let step = gani.frame(frame)?;
    let row = step.rows.get(usize::from(dir % 4)).or_else(|| step.rows.first())?;
    let placed: Vec<_> = row
        .iter()
        .filter_map(|pl| {
            let sp = gani.sprites.iter().find(|s| s.id == pl.sprite_id)?;
            let sheet = match sp.sheet.to_ascii_uppercase().as_str() {
                "BODY" => sheets.body,
                "HEAD" => sheets.head,
                "SPRITES" => sheets.sprites,
                "SWORD" => sheets.sword,
                "SHIELD" => sheets.shield,
                _ => None,
            }?;
            (sp.x >= 0 && sp.y >= 0 && sp.w > 0 && sp.h > 0).then_some((pl, sp, sheet))
        })
        .collect();
    let x0 = placed.iter().map(|(pl, _, _)| pl.x).min()?;
    let y0 = placed.iter().map(|(pl, _, _)| pl.y).min()?;
    let x1 = placed.iter().map(|(pl, sp, _)| pl.x + sp.w).max()?;
    let y1 = placed.iter().map(|(pl, sp, _)| pl.y + sp.h).max()?;
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let mut px = vec![0u8; w * h * 4];
    for (pl, sp, sheet) in placed {
        for ty in 0..sp.h {
            for tx in 0..sp.w {
                let si = ((sp.y + ty) as usize * sheet.w as usize + (sp.x + tx) as usize) * sheet.bpp;
                let Some(src) = sheet.px.get(si..si + sheet.bpp) else { continue };
                let rgba = match sheet.bpp {
                    4 if src[3] == 0 => continue,
                    4 => [src[0], src[1], src[2], src[3]],
                    3 if src == [255, 0, 255] => continue,
                    3 => [src[0], src[1], src[2], 255],
                    _ => continue,
                };
                let di = ((pl.y + ty - y0) as usize * w + (pl.x + tx - x0) as usize) * 4;
                px[di..di + 4].copy_from_slice(&rgba);
            }
        }
    }
    Some((x0, y0, w as u32, px))
}

fn tile_at(level: &Level, x: i32, y: i32) -> Option<u16> {
    level.tile_id_at(x, y).or_else(|| {
        // Fallback for boards that store runs.
        level
            .boards
            .iter()
            .find(|r| r.y == y && x >= r.x && x < r.x + r.width)
            .and_then(|r| r.tiles.get((x - r.x) as usize).copied())
            .map(tile_id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_player_box_matches_other_players() {
        let mut s = OfflineSession::default();
        s.player_x = 30.0;
        s.player_y = 30.5;
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        // Other players are drawn at level * 16 - camera; the local
        // player must land on the same pixel for the same position.
        assert_eq!(
            player_box_origin(&s, 40, 30),
            ((30.0 * 16.0) as i32 - cam_x, (30.5 * 16.0) as i32 - cam_y)
        );
    }

    #[test]
    fn blit_respects_bounds() {
        let sheet = vec![0xFFu8; (16 * 16 * 3) as usize];
        let mut fb = Framebuffer::new(32, 32);
        fb.blit_tile(&sheet, 16, 16, 0, 0, 0);
        assert_eq!(fb.pixels[0], 0xFF);
    }

    /// A 2048×512 sheet where tile `n` is a flat colour: `n` in the red byte.
    fn flat_sheet() -> Vec<u8> {
        let mut sheet = vec![0u8; 2048 * 512 * 3];
        for n in 0..512u32 {
            let (col, row) = (n % 16, (n / 16) % SHEET_ROWS);
            for y in 0..16 {
                for x in 0..16 {
                    let i = (((row * 16 + y) * 2048) + col * 16 + x) as usize * 3;
                    sheet[i] = n as u8;
                }
            }
        }
        sheet
    }

    fn red_at(fb: &Framebuffer, x: u32, y: u32) -> u8 {
        fb.pixels[((y * fb.width + x) * 3) as usize]
    }

    /// A 2048×512 sheet where tile `id` is a flat colour: the id's low byte
    /// in red, its high byte in green.
    fn id_sheet() -> Vec<u8> {
        let mut sheet = vec![0u8; 2048 * 512 * 3];
        for id in 0..4096u32 {
            let (col, row) = (id % 16 + (id / 512) * 16, (id / 16) % SHEET_ROWS);
            for y in 0..16 {
                for x in 0..16 {
                    let i = (((row * 16 + y) * 2048) + col * 16 + x) as usize * 3;
                    sheet[i] = id as u8;
                    sheet[i + 1] = (id >> 8) as u8;
                }
            }
        }
        sheet
    }

    fn id_at(fb: &Framebuffer, x: u32, y: u32) -> u16 {
        let i = ((y * fb.width + x) * 3) as usize;
        fb.pixels[i] as u16 | (fb.pixels[i + 1] as u16) << 8
    }

    /// A level of plain tile 1 with the given chest records, the player
    /// standing so that the camera sits at (0, 0).
    fn chest_session(chests: Vec<crate::assets::Chest>) -> OfflineSession {
        let mut level = crate::assets::Level::from_board(&vec![1u16; 4096]);
        level.chests = chests;
        let mut s = OfflineSession::default();
        s.current_level = Some(level);
        s.player_x = 10.0;
        s.player_y = 10.0;
        s
    }

    fn chest(x: i32, y: i32, opened: bool) -> crate::assets::Chest {
        crate::assets::Chest { x, y, item: String::new(), sign_index: -1, opened }
    }

    /// The tile ids drawn in the 2x2 tiles at level tile (x, y), for a view
    /// of 20x20 tiles centred on the player.
    fn drawn_at(s: &OfflineSession, x: i32, y: i32) -> [u16; 4] {
        let sheet = id_sheet();
        let fb = render_view(s, &sheet, 2048, 512, 20, 20);
        let (cx, cy) = camera_origin(s, 20, 20);
        let (px, py) = (x * 16 - cx, y * 16 - cy);
        let at = |dx: i32, dy: i32| id_at(&fb, (px + dx * 16 + 8) as u32, (py + dy * 16 + 8) as u32);
        [at(0, 0), at(1, 0), at(0, 1), at(1, 1)]
    }

    /// A chest record is drawn from the record: the closed art of arrays 13,
    /// or the open art of array 14 once it is opened. The built-in sheet has
    /// its chest art at these ids too.
    #[test]
    fn a_chest_record_is_drawn_closed_or_open() {
        let mut s = chest_session(vec![chest(12, 12, false), chest(16, 12, true)]);
        assert_eq!(drawn_at(&s, 12, 12), [1784, 1785, 1800, 1801], "a closed chest");
        assert_eq!(drawn_at(&s, 16, 12), [829, 830, 845, 846], "the server said it is already opened");
        // Opening the closed one in play swaps it.
        s.opened_chests.insert((12, 12));
        assert_eq!(drawn_at(&s, 12, 12), [829, 830, 845, 846]);
        // Beside the chests the board is untouched.
        assert_eq!(drawn_at(&s, 14, 12), [1, 1, 1, 1]);
    }

    /// The client draws a chest only for x and y from 0 to 62.
    #[test]
    fn chests_outside_the_board_are_not_drawn() {
        assert_eq!(chest_art(false), CHEST_CLOSED_TILES);
        let s = chest_session(vec![chest(63, 12, false), chest(-1, 12, false)]);
        let sheet = id_sheet();
        let fb = render_view(&s, &sheet, 2048, 512, 70, 20);
        assert!(fb.pixels.chunks(3).all(|p| !(p[0] == 0x18 && p[1] == 7)), "no closed-chest tile anywhere");
    }

    /// On a gmap the view shows the neighbour's tiles past the edge: tile
    /// column 64 of the view is column 0 of the next level.
    #[test]
    fn neighbour_levels_are_drawn_past_the_edge() {
        use crate::assets::Level;
        let lv = |mark: u16| {
            let tiles: Vec<u16> = (0..64 * 64).map(|i| if i == 30 * 64 { mark } else { 1 }).collect();
            Level::from_board(&tiles)
        };
        let grid = "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n";
        let mut s = OfflineSession::default();
        s.current_level = Some(lv(20));
        s.player_x = 60.0;
        s.player_y = 30.5;
        let sheet = flat_sheet();
        // Without a gmap the right of the edge stays blank.
        let fb = render_view(&s, &sheet, 2048, 512, 40, 30);
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        // Tile (64, 30) in cell-local tiles, and (0, 30) of this level.
        let px = |tx: i32, ty: i32| ((tx * 16 - cam_x + 8) as u32, (ty * 16 - cam_y + 8) as u32);
        let (ex, ey) = px(64, 30);
        assert_eq!(red_at(&fb, ex, ey), 0, "no gmap: nothing past the edge");
        s.enter_gmap("g.gmap".into(), crate::gmap::Gmap::parse(grid).unwrap(), (0, 0));
        s.gmap.as_mut().unwrap().levels.insert((1, 0), lv(21));
        let fb = render_view(&s, &sheet, 2048, 512, 40, 30);
        assert_eq!(red_at(&fb, ex, ey), 21, "the east level's tile (0, 30)");
        let (ix, iy) = px(63, 30);
        assert_eq!(red_at(&fb, ix, iy), 1, "this level's own tile");
    }
}
