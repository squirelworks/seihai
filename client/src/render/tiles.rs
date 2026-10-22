//! Software tile renderer.
//!
//! Composites a level board into an RGB buffer by blitting 16×16 tiles
//! from a 2048×512 sheet (`pics1.png` layout). No SDL2 required.
//!
//! Sheet layout: 128 columns × 32 rows of 16 px tiles, stored as eight
//! 16-column blocks of 512 tiles. Tile id `n` lives at column
//! `n % 16 + (n / 512) * 16`, row `(n / 16) % 32`.

use crate::assets::{tile_id, Level};
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

    /// Draw a short nickname tag under a player box.
    ///
    /// Uses a simple 4×6 block font for ASCII. `dx`/`dy` is the
    /// top-left of the 48×48 player box.
    pub fn draw_nick(&mut self, nick: &str, dx: i32, dy: i32) {
        let mut x = dx + 8;
        let y = dy + 48;
        for ch in nick.chars().take(12) {
            let c = ch as u8;
            if c < 32 || c > 126 {
                continue;
            }
            // 4×6 block: fill a small rect per character.
            self.fill_rect(x, y, 4, 6, [0xFF, 0xFF, 0xFF]);
            x += 5;
        }
    }
}

/// Render a camera view of the level into a framebuffer.
///
/// `sheet` is the RGB tile sheet (2048×512 for `pics1.png`).
/// `view_tiles_w` / `view_tiles_h` is the visible window in tiles.
/// The player is drawn as a 16×16 marker.
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
            if lx < 0 || ly < 0 || lx >= 64 || ly >= 64 {
                continue;
            }
            if let Some(id) = tile_at(level, lx, ly) {
                fb.blit_tile(sheet, sheet_w, sheet_h, id, tx * t - sub_x, ty * t - sub_y);
            }
        }
    }

    // Chests, signs, NPCs.
    for chest in &level.chests {
        // Chest: brown box with a gold latch.
        let (cx, cy) = (chest.x * t - cam_x, chest.y * t - cam_y);
        fb.fill_rect(cx + 2, cy + 6, 28, 22, [0x80, 0x40, 0x10]);
        fb.fill_rect(cx + 2, cy + 4, 28, 6, [0xA0, 0x50, 0x18]);
        fb.fill_rect(cx + 12, cy + 10, 8, 6, [0xF0, 0xD0, 0x40]);
    }
    for sign in &level.signs {
        draw_marker(&mut fb, sign.x, sign.y, cam_x, cam_y, [0xFF, 0xFF, 0x40]);
    }
    for npc in &level.npcs {
        if npc.visible {
            draw_marker(&mut fb, npc.x as i32, npc.y as i32, cam_x, cam_y, [0x40, 0xC0, 0x40]);
        }
    }

    fb
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
        fb, body, body_w, body_bpp, head, head_w, head_bpp, sprites, dir, dx, dy,
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
    if let Some((spr, spr_w, spr_bpp)) = sprites {
        fb.blit_sprite(spr, spr_w, spr_bpp, 0, 0, 24, 12, dx + shadow.0, dy + shadow.1);
    }
    let d = d as i32;
    fb.blit_sprite(body, body_w, body_bpp, d * 32, 0, 32, 32, dx + b.0, dy + b.1);
    fb.blit_sprite(head, head_w, head_bpp, 0, d * 32, 32, 32, dx + h.0, dy + h.1);
}

fn draw_marker(fb: &mut Framebuffer, x: i32, y: i32, cam_x: i32, cam_y: i32, color: [u8; 3]) {
    let t = TILE_PX as i32;
    fb.fill_rect(x * t - cam_x + 2, y * t - cam_y + 2, 12, 12, color);
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
    fn blit_respects_bounds() {
        let sheet = vec![0xFFu8; (16 * 16 * 3) as usize];
        let mut fb = Framebuffer::new(32, 32);
        fb.blit_tile(&sheet, 16, 16, 0, 0, 0);
        assert_eq!(fb.pixels[0], 0xFF);
    }
}
