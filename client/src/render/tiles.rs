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
    let ox = session.player_x as i32 - (view_tiles_w as i32 / 2);
    let oy = session.player_y as i32 - (view_tiles_h as i32 / 2);

    for ty in 0..view_tiles_h as i32 {
        for tx in 0..view_tiles_w as i32 {
            let lx = ox + tx;
            let ly = oy + ty;
            if lx < 0 || ly < 0 || lx >= 64 || ly >= 64 {
                continue;
            }
            if let Some(id) = tile_at(level, lx, ly) {
                fb.blit_tile(
                    sheet,
                    sheet_w,
                    sheet_h,
                    id,
                    tx * TILE_PX as i32,
                    ty * TILE_PX as i32,
                );
            }
        }
    }

    // Chests, signs, NPCs.
    for chest in &level.chests {
        draw_marker(&mut fb, chest.x, chest.y, ox, oy, [0xC0, 0x80, 0x20]);
    }
    for sign in &level.signs {
        draw_marker(&mut fb, sign.x, sign.y, ox, oy, [0xFF, 0xFF, 0x40]);
    }
    for npc in &level.npcs {
        if npc.visible {
            draw_marker(&mut fb, npc.x as i32, npc.y as i32, ox, oy, [0x40, 0xC0, 0x40]);
        }
    }

    // Player: body + head if sheets are provided, else marker.
    let px = (view_tiles_w as i32 / 2) * TILE_PX as i32;
    let py = (view_tiles_h as i32 / 2) * TILE_PX as i32;
    fb
}

/// Draw the player body and head sprites for the given direction.
///
/// `body` and `head` are RGB or RGBA images. Direction: 0 south, 1 west,
/// 2 east, 3 north. Body sprites are 32×32 at column `dir`, row 0.
/// Head sprites are 32×32 at column 0, row `dir`.
pub fn draw_player_sprites(
    fb: &mut Framebuffer,
    body: &[u8],
    body_w: u32,
    body_bpp: usize,
    head: &[u8],
    head_w: u32,
    head_bpp: usize,
    dir: u8,
    dx: i32,
    dy: i32,
) {
    let d = (dir as i32) % 4;
    // Body: column d, row 0.
    fb.blit_sprite(body, body_w, body_bpp, d * 32, 0, 32, 32, dx - 8, dy - 16);
    // Head: column 0, row d.
    fb.blit_sprite(head, head_w, head_bpp, 0, d * 32, 32, 32, dx - 8, dy - 32);
}

fn draw_marker(fb: &mut Framebuffer, x: i32, y: i32, ox: i32, oy: i32, color: [u8; 3]) {
    let tx = x - ox;
    let ty = y - oy;
    if tx < 0 || ty < 0 {
        return;
    }
    fb.fill_rect(tx * TILE_PX as i32 + 2, ty * TILE_PX as i32 + 2, 12, 12, color);
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
