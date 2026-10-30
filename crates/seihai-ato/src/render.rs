//! Stage 3: the flat 2D fallback renderer. It draws the level's tiles the
//! way `seihai-mae` does (`render::tiles::render_view`): 16-pixel tiles from
//! a `pics1.png` sheet, the camera centred on the player's 48x48 box,
//! chest records over the board, and the player as a plain marker. No
//! NPCs, other players, GANI art or HUD.
//!
//! The 3.x clients drew their 2D levels from the same board and tile sheet
//! as 2.x, so this is a stand-in until Ato has its own 3.x view. The tile
//! sheet maths and the chest tiles are copied from `seihai-mae` rather than
//! shared, so the two lines stay apart until the drawing moves to core.

use seihai_core::assets::{builtin, read_png_rgb, tile_id, AssetResolver, Level, TileTypes, TileTypesFile};
use seihai_core::offline::OfflineSession;

/// Width of a tile in pixels.
pub const TILE_PX: u32 = 16;
/// Rows on the `pics1.png` sheet (128 columns of 16-column blocks).
const SHEET_ROWS: u32 = 32;
/// The 2x2 chest art, closed and open (`seihai-mae` `render::tiles`).
pub const CHEST_CLOSED_TILES: [u16; 4] = [1784, 1785, 1800, 1801];
pub const CHEST_OPEN_TILES: [u16; 4] = [829, 830, 845, 846];
/// The player marker: red, blue in water.
const MARKER: [u8; 3] = [0xFF, 0x40, 0x40];
const MARKER_WATER: [u8; 3] = [0x40, 0x80, 0xFF];

/// An RGB tile sheet.
pub struct Sheet {
    pub w: u32,
    pub h: u32,
    pub rgb: Vec<u8>,
}

impl Sheet {
    /// `pics1.png` from the asset folders, or the built-in sheet.
    pub fn load(resolver: Option<&AssetResolver>) -> Result<Sheet, String> {
        let data = match resolver.and_then(|r| r.image("pics1.png")) {
            Some(p) => std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?,
            None => builtin::bytes("pics1.png").ok_or("no built-in pics1.png")?.to_vec(),
        };
        let (w, h, rgb) = read_png_rgb(&data).map_err(|e| format!("pics1.png: {e}"))?;
        Ok(Sheet { w, h, rgb })
    }
}

/// The tile types (walls, water) from the asset folders, or the built-in
/// `tiledefs.txt`, which uses the same layout as the built-in sheet.
pub fn tile_types(resolver: Option<&AssetResolver>) -> Option<TileTypes> {
    match resolver.and_then(|r| r.tile_types()) {
        Some((p, TileTypesFile::Text)) => TileTypes::load(p).ok(),
        Some((p, TileTypesFile::ArraysDat)) => TileTypes::load_arrays(p).ok(),
        None => TileTypes::parse(&String::from_utf8_lossy(builtin::bytes("tiledefs.txt")?)).ok(),
    }
}

/// An RGB frame.
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub rgb: Vec<u8>,
}

impl Frame {
    fn new(w: u32, h: u32) -> Frame {
        Frame { w, h, rgb: vec![0; (w * h * 3) as usize] }
    }

    /// Copy tile `tile` of the sheet to `(dx, dy)`, clipped.
    fn blit_tile(&mut self, s: &Sheet, tile: u16, dx: i32, dy: i32) {
        let t = tile as u32;
        let (sx, sy) = ((t % 16 + (t / 512) * 16) * TILE_PX, ((t / 16) % SHEET_ROWS) * TILE_PX);
        if sx + TILE_PX > s.w || sy + TILE_PX > s.h {
            return;
        }
        for ty in 0..TILE_PX as i32 {
            let py = dy + ty;
            if py < 0 || py >= self.h as i32 {
                continue;
            }
            for tx in 0..TILE_PX as i32 {
                let px = dx + tx;
                if px < 0 || px >= self.w as i32 {
                    continue;
                }
                let si = ((sy as usize + ty as usize) * s.w as usize + sx as usize + tx as usize) * 3;
                let di = (py as usize * self.w as usize + px as usize) * 3;
                self.rgb[di..di + 3].copy_from_slice(&s.rgb[si..si + 3]);
            }
        }
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, rgb: [u8; 3]) {
        for py in y.max(0)..(y + h).min(self.h as i32) {
            for px in x.max(0)..(x + w).min(self.w as i32) {
                let di = (py as usize * self.w as usize + px as usize) * 3;
                self.rgb[di..di + 3].copy_from_slice(&rgb);
            }
        }
    }
}

/// Top-left of the view in level pixels: centred on the middle of the
/// player's 48x48 box (player x/y + 1.5 tiles).
pub fn camera_origin(s: &OfflineSession, view_w: u32, view_h: u32) -> (i32, i32) {
    let t = TILE_PX as f32;
    (((s.player_x + 1.5) * t - view_w as f32 * t / 2.0) as i32, ((s.player_y + 1.5) * t - view_h as f32 * t / 2.0) as i32)
}

/// The tile at `(x, y)`: layer 0, else any board run that covers it.
fn tile_at(level: &Level, x: i32, y: i32) -> Option<u16> {
    level.tile_id_at(x, y).or_else(|| {
        level
            .boards
            .iter()
            .find(|r| r.y == y && x >= r.x && x < r.x + r.width)
            .and_then(|r| r.tiles.get((x - r.x) as usize).copied())
            .map(tile_id)
    })
}

/// Draw the session's level around the player, `view_w` x `view_h` tiles.
/// Off the level is black.
pub fn draw(s: &OfflineSession, sheet: &Sheet, view_w: u32, view_h: u32) -> Frame {
    let mut f = Frame::new(view_w * TILE_PX, view_h * TILE_PX);
    let Some(level) = &s.current_level else { return f };
    let (cam_x, cam_y) = camera_origin(s, view_w, view_h);
    let t = TILE_PX as i32;
    let (ox, oy) = (cam_x.div_euclid(t), cam_y.div_euclid(t));
    let (sub_x, sub_y) = (cam_x.rem_euclid(t), cam_y.rem_euclid(t));
    // One extra row and column cover the sub-tile scroll.
    for ty in 0..=view_h as i32 {
        for tx in 0..=view_w as i32 {
            let (lx, ly) = (ox + tx, oy + ty);
            if !(0..64).contains(&lx) || !(0..64).contains(&ly) {
                continue;
            }
            if let Some(id) = tile_at(level, lx, ly) {
                f.blit_tile(sheet, id, tx * t - sub_x, ty * t - sub_y);
            }
        }
    }
    // Chests from their records, over the board; skipped outside 0 to 62
    // as the client does.
    for c in &level.chests {
        if !(0..=62).contains(&c.x) || !(0..=62).contains(&c.y) {
            continue;
        }
        let opened = c.opened || s.opened_chests.contains(&(c.x, c.y));
        let art = if opened { CHEST_OPEN_TILES } else { CHEST_CLOSED_TILES };
        for (i, id) in art.into_iter().enumerate() {
            let (dx, dy) = ((i % 2) as i32, (i / 2) as i32);
            f.blit_tile(sheet, id, (c.x + dx) * t - cam_x, (c.y + dy) * t - cam_y);
        }
    }
    // The player's box starts at its tile position; the marker sits where
    // the body would be.
    let (bx, by) = ((s.player_x * t as f32) as i32 - cam_x, (s.player_y * t as f32) as i32 - cam_y);
    f.fill_rect(bx + 16, by + 24, 16, 16, if s.in_water { MARKER_WATER } else { MARKER });
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(fill: u16) -> OfflineSession {
        let mut s = OfflineSession::default();
        s.current_level = Some(Level::from_board(&[fill; 4096]));
        s.player_x = 30.0;
        s.player_y = 30.0;
        s
    }

    fn px(f: &Frame, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * f.w + x) * 3) as usize;
        [f.rgb[i], f.rgb[i + 1], f.rgb[i + 2]]
    }

    #[test]
    fn the_builtin_sheet_and_types_load() {
        let s = Sheet::load(None).unwrap();
        assert_eq!((s.w, s.h), (2048, 512));
        assert!(tile_types(None).is_some());
    }

    #[test]
    fn tiles_come_from_the_sheet_position_mae_uses() {
        let sheet = Sheet::load(None).unwrap();
        // Tile 600: column 600 % 16 + (600 / 512) * 16 = 24, row (600 / 16) % 32 = 5.
        let f = draw(&session(600), &sheet, 40, 30);
        let (cam_x, cam_y) = camera_origin(&session(600), 40, 30);
        // A screen pixel at a tile corner is that sheet tile's corner pixel.
        let (sx, sy) = ((16 - cam_x.rem_euclid(16)) as u32, (16 - cam_y.rem_euclid(16)) as u32);
        let si = ((5 * 16 * sheet.w + 24 * 16) * 3) as usize;
        assert_eq!(px(&f, sx, sy), [sheet.rgb[si], sheet.rgb[si + 1], sheet.rgb[si + 2]]);
    }

    #[test]
    fn the_player_marker_is_at_the_centre() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(&session(0), &sheet, 40, 30);
        // The box (48x48) is centred; the marker is its middle-bottom 16x16.
        assert_eq!(px(&f, 320, 248), MARKER);
    }

    #[test]
    fn off_the_level_is_black() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(600);
        s.player_x = 0.0;
        s.player_y = 0.0;
        let f = draw(&s, &sheet, 40, 30);
        assert_eq!(px(&f, 0, 0), [0, 0, 0]);
    }

    #[test]
    fn no_level_draws_black() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(&OfflineSession::default(), &sheet, 4, 4);
        assert!(f.rgb.iter().all(|&b| b == 0));
    }
}
