//! Software tile renderer.
//!
//! Composites a level board into an RGB buffer by blitting 16×16 tiles
//! from a 2048×512 sheet (`pics1.png` layout). No SDL2 required.
//!
//! Sheet layout (cleaned): 128 columns × 32 rows of 16 px tiles.
//! Tile id `n` lives at column `n % 128`, row `n / 128`.

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
        let col = (tile as u32) % SHEET_COLS;
        let row = (tile as u32) / SHEET_COLS;
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

    // Player marker (centre of view).
    let px = (view_tiles_w as i32 / 2) * TILE_PX as i32;
    let py = (view_tiles_h as i32 / 2) * TILE_PX as i32;
    let color = if session.in_water {
        [0x40, 0x80, 0xFF]
    } else {
        [0xFF, 0x40, 0x40]
    };
    fb.fill_rect(px + 4, py + 4, 8, 8, color);
    fb
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
