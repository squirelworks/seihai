//! Stage 3: the flat 2D fallback renderer. It draws the level's tiles the
//! way `seihai-mae` does (`render::tiles::render_view`): 16-pixel tiles from
//! a `pics1.png` sheet, the camera centred on the player's 48x48 box,
//! chest records over the board, each link as an outline of its rectangle,
//! the neighbouring levels of a gmap cell past its edges, the level's NPCs
//! as their images (or an outline where the image is not
//! on disk), other players and the player as plain markers. No GANI art
//! or HUD.
//!
//! The 3.x clients drew their 2D levels from the same board and tile sheet
//! as 2.x, so this is a stand-in until Ato has its own 3.x view. The tile
//! sheet maths and the chest tiles are copied from `seihai-mae` rather than
//! shared, so the two lines stay apart until the drawing moves to core.

use seihai_core::assets::gif::read_gif_rgba;
use seihai_core::assets::{
    builtin, read_png_rgb, read_png_rgba, tile_id, AssetResolver, Level, TileTypes, TileTypesFile,
};
use seihai_core::offline::OfflineSession;
use std::collections::HashMap;

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
/// Another player's marker.
const MARKER_OTHER: [u8; 3] = [0xFF, 0xD0, 0x40];
/// The outline drawn where an NPC's image is not on disk.
pub const NPC_MISSING: [u8; 3] = [0xC0, 0x60, 0xFF];
/// The box an NPC with a missing image is outlined in, in pixels.
const NPC_MISSING_PX: i32 = 32;

/// A decoded NPC image.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    /// A PNG or GIF file's bytes, by the name's extension. Other formats
    /// (`.mng`, `.bmp`) are not read here and give `None`.
    pub fn decode(name: &str, data: &[u8]) -> Option<Image> {
        let lower = name.to_ascii_lowercase();
        let (w, h, rgba) = if lower.ends_with(".png") {
            read_png_rgba(data).ok()?
        } else if lower.ends_with(".gif") {
            read_gif_rgba(data).ok()?
        } else {
            return None;
        };
        Some(Image { w, h, rgba })
    }
}

/// NPC images by name as the level spells them. `None` is an image looked
/// for and not found, so it is not looked for again.
pub type NpcImages = HashMap<String, Option<Image>>;

/// Read every NPC image of the level not looked for yet, from the asset
/// folders. Returns the names that were not found.
pub fn load_npc_images(
    level: &Level,
    resolver: Option<&AssetResolver>,
    images: &mut NpcImages,
) -> Vec<String> {
    let mut missing = Vec::new();
    for npc in &level.npcs {
        let name = npc.image.trim();
        if name.is_empty() || images.contains_key(name) {
            continue;
        }
        let image = resolver
            .and_then(|r| r.image(name))
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|d| Image::decode(name, &d));
        if image.is_none() {
            missing.push(name.to_string());
        }
        images.insert(name.to_string(), image);
    }
    missing
}

/// A link's outline. The real clients draw nothing for a link; this only
/// shows where the level's links are.
pub const LINK_OUTLINE: [u8; 3] = [0xFF, 0xE0, 0x40];

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
            None => builtin::bytes("pics1.png")
                .ok_or("no built-in pics1.png")?
                .to_vec(),
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
        Frame {
            w,
            h,
            rgb: vec![0; (w * h * 3) as usize],
        }
    }

    /// Copy tile `tile` of the sheet to `(dx, dy)`, clipped.
    fn blit_tile(&mut self, s: &Sheet, tile: u16, dx: i32, dy: i32) {
        let t = tile as u32;
        let (sx, sy) = (
            (t % 16 + (t / 512) * 16) * TILE_PX,
            ((t / 16) % SHEET_ROWS) * TILE_PX,
        );
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
                let si =
                    ((sy as usize + ty as usize) * s.w as usize + sx as usize + tx as usize) * 3;
                let di = (py as usize * self.w as usize + px as usize) * 3;
                self.rgb[di..di + 3].copy_from_slice(&s.rgb[si..si + 3]);
            }
        }
    }

    /// Copy `part` (`x, y, w, h` in the image; the whole image when
    /// `None`) of `img` to `(dx, dy)`. A pixel is drawn when its alpha is
    /// at least half, as the 2D clients mask GIF and PNG images.
    fn blit_image(&mut self, img: &Image, part: Option<(i32, i32, i32, i32)>, dx: i32, dy: i32) {
        let (sx, sy, w, h) = part.unwrap_or((0, 0, img.w as i32, img.h as i32));
        for y in 0..h {
            let (iy, py) = (sy + y, dy + y);
            if iy < 0 || iy >= img.h as i32 || py < 0 || py >= self.h as i32 {
                continue;
            }
            for x in 0..w {
                let (ix, px) = (sx + x, dx + x);
                if ix < 0 || ix >= img.w as i32 || px < 0 || px >= self.w as i32 {
                    continue;
                }
                let si = (iy as usize * img.w as usize + ix as usize) * 4;
                if img.rgba[si + 3] < 128 {
                    continue;
                }
                let di = (py as usize * self.w as usize + px as usize) * 3;
                self.rgb[di..di + 3].copy_from_slice(&img.rgba[si..si + 3]);
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

    /// A one-pixel outline of the rectangle, clipped.
    fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, rgb: [u8; 3]) {
        self.fill_rect(x, y, w, 1, rgb);
        self.fill_rect(x, y + h - 1, w, 1, rgb);
        self.fill_rect(x, y, 1, h, rgb);
        self.fill_rect(x + w - 1, y, 1, h, rgb);
    }
}

/// Top-left of the view in level pixels: centred on the middle of the
/// player's 48x48 box (player x/y + 1.5 tiles).
pub fn camera_origin(s: &OfflineSession, view_w: u32, view_h: u32) -> (i32, i32) {
    let t = TILE_PX as f32;
    (
        ((s.player_x + 1.5) * t - view_w as f32 * t / 2.0) as i32,
        ((s.player_y + 1.5) * t - view_h as f32 * t / 2.0) as i32,
    )
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

/// Draw the session's level around the player, `view_w` x `view_h` tiles,
/// with a marker for each other player at `others` (tile positions). Off
/// the level is black.
pub fn draw(
    s: &OfflineSession,
    sheet: &Sheet,
    npc_images: &NpcImages,
    others: &[(f32, f32)],
    view_w: u32,
    view_h: u32,
) -> Frame {
    let mut f = Frame::new(view_w * TILE_PX, view_h * TILE_PX);
    let Some(level) = &s.current_level else {
        return f;
    };
    let (cam_x, cam_y) = camera_origin(s, view_w, view_h);
    let t = TILE_PX as i32;
    let (ox, oy) = (cam_x.div_euclid(t), cam_y.div_euclid(t));
    let (sub_x, sub_y) = (cam_x.rem_euclid(t), cam_y.rem_euclid(t));
    // One extra row and column cover the sub-tile scroll.
    for ty in 0..=view_h as i32 {
        for tx in 0..=view_w as i32 {
            let (lx, ly) = (ox + tx, oy + ty);
            // Past an edge of a gmap cell: the neighbour's tile, when that
            // level is loaded; otherwise black, as off a plain level.
            let (from, lx, ly) = if (0..64).contains(&lx) && (0..64).contains(&ly) {
                (Some(level), lx, ly)
            } else {
                let off = (lx.div_euclid(64), ly.div_euclid(64));
                let near = s
                    .gmap
                    .as_ref()
                    .and_then(|w| w.levels.get(&(w.cell.0 + off.0, w.cell.1 + off.1)));
                (near, lx.rem_euclid(64), ly.rem_euclid(64))
            };
            let Some(from) = from else {
                continue;
            };
            if let Some(id) = tile_at(from, lx, ly) {
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
        let art = if opened {
            CHEST_OPEN_TILES
        } else {
            CHEST_CLOSED_TILES
        };
        for (i, id) in art.into_iter().enumerate() {
            let (dx, dy) = ((i % 2) as i32, (i / 2) as i32);
            f.blit_tile(sheet, id, (c.x + dx) * t - cam_x, (c.y + dy) * t - cam_y);
        }
    }
    // Links over the tiles and chests, under the player.
    for l in &level.links {
        f.stroke_rect(
            l.x * t - cam_x,
            l.y * t - cam_y,
            l.width * t,
            l.height * t,
            LINK_OUTLINE,
        );
    }
    // The level's visible NPCs (file and server), in level order, as their
    // images. One whose image is not on disk is outlined instead; one with
    // no image name draws nothing, as in the clients.
    for npc in level.npcs.iter().filter(|n| n.visible) {
        let name = npc.image.trim();
        if name.is_empty() {
            continue;
        }
        let (nx, ny) = (
            (npc.x * t as f32) as i32 - cam_x,
            (npc.y * t as f32) as i32 - cam_y,
        );
        match npc_images.get(name) {
            Some(Some(img)) => f.blit_image(img, npc.image_part, nx, ny),
            _ => f.stroke_rect(nx, ny, NPC_MISSING_PX, NPC_MISSING_PX, NPC_MISSING),
        }
    }
    // Other players: the same marker as the player's, in their own colour.
    for &(x, y) in others {
        let (ox, oy) = ((x * t as f32) as i32 - cam_x, (y * t as f32) as i32 - cam_y);
        f.fill_rect(ox + 16, oy + 24, 16, 16, MARKER_OTHER);
    }
    // The player's box starts at its tile position; the marker sits where
    // the body would be.
    let (bx, by) = (
        (s.player_x * t as f32) as i32 - cam_x,
        (s.player_y * t as f32) as i32 - cam_y,
    );
    f.fill_rect(
        bx + 16,
        by + 24,
        16,
        16,
        if s.in_water { MARKER_WATER } else { MARKER },
    );
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
        let f = draw(&session(600), &sheet, &NpcImages::new(), &[], 40, 30);
        let (cam_x, cam_y) = camera_origin(&session(600), 40, 30);
        // A screen pixel at a tile corner is that sheet tile's corner pixel.
        let (sx, sy) = (
            (16 - cam_x.rem_euclid(16)) as u32,
            (16 - cam_y.rem_euclid(16)) as u32,
        );
        let si = ((5 * 16 * sheet.w + 24 * 16) * 3) as usize;
        assert_eq!(
            px(&f, sx, sy),
            [sheet.rgb[si], sheet.rgb[si + 1], sheet.rgb[si + 2]]
        );
    }

    #[test]
    fn the_player_marker_is_at_the_centre() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(&session(0), &sheet, &NpcImages::new(), &[], 40, 30);
        // The box (48x48) is centred; the marker is its middle-bottom 16x16.
        assert_eq!(px(&f, 320, 248), MARKER);
    }

    #[test]
    fn a_link_is_outlined_on_its_rectangle() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(0);
        let level = s.current_level.as_mut().unwrap();
        level.add_server_links(&["next.nw 20 25 2 3 30 30".to_string()]);
        let f = draw(&s, &sheet, &NpcImages::new(), &[], 40, 30);
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        let (x0, y0) = ((20 * 16 - cam_x) as u32, (25 * 16 - cam_y) as u32);
        // Its corners and edges, 2 x 3 tiles; not its inside.
        assert_eq!(px(&f, x0, y0), LINK_OUTLINE);
        assert_eq!(px(&f, x0 + 31, y0 + 47), LINK_OUTLINE);
        assert_eq!(px(&f, x0 + 16, y0), LINK_OUTLINE);
        assert_ne!(px(&f, x0 + 16, y0 + 24), LINK_OUTLINE);
    }

    #[test]
    fn another_player_gets_a_marker() {
        let sheet = Sheet::load(None).unwrap();
        let s = session(0);
        let (ox, oy) = (s.player_x + 3.0, s.player_y + 2.0);
        let f = draw(&s, &sheet, &NpcImages::new(), &[(ox, oy)], 40, 30);
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        let x = (ox * 16.0) as i32 - cam_x + 20;
        let y = (oy * 16.0) as i32 - cam_y + 28;
        assert_eq!(px(&f, x as u32, y as u32), MARKER_OTHER);
    }

    #[test]
    fn off_the_level_is_black() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(600);
        s.player_x = 0.0;
        s.player_y = 0.0;
        let f = draw(&s, &sheet, &NpcImages::new(), &[], 40, 30);
        assert_eq!(px(&f, 0, 0), [0, 0, 0]);
    }

    #[test]
    fn no_level_draws_black() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(
            &OfflineSession::default(),
            &sheet,
            &NpcImages::new(),
            &[],
            4,
            4,
        );
        assert!(f.rgb.iter().all(|&b| b == 0));
    }

    /// A 4x4 image: opaque red on its left half, clear on its right.
    fn half_red() -> Image {
        let mut rgba = Vec::new();
        for _ in 0..4 {
            for x in 0..4 {
                rgba.extend_from_slice(if x < 2 {
                    &[255, 0, 0, 255]
                } else {
                    &[0, 0, 255, 0]
                });
            }
        }
        Image { w: 4, h: 4, rgba }
    }

    fn with_npc(image: &str, dx: f32, dy: f32) -> (OfflineSession, i32, i32) {
        let mut s = session(0);
        let (x, y) = (s.player_x + dx, s.player_y + dy);
        s.current_level
            .as_mut()
            .unwrap()
            .npcs
            .push(seihai_core::assets::Npc::new(
                image.into(),
                x,
                y,
                String::new(),
            ));
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        (s, (x * 16.0) as i32 - cam_x, (y * 16.0) as i32 - cam_y)
    }

    #[test]
    fn an_npc_is_drawn_as_its_image_with_the_clear_pixels_left_out() {
        let sheet = Sheet::load(None).unwrap();
        let (s, x, y) = with_npc("block.png", 5.0, -4.0);
        let images = NpcImages::from([("block.png".to_string(), Some(half_red()))]);
        let under = px(
            &draw(&session(0), &sheet, &NpcImages::new(), &[], 40, 30),
            x as u32 + 3,
            y as u32,
        );
        let f = draw(&s, &sheet, &images, &[], 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), [255, 0, 0]);
        assert_eq!(px(&f, x as u32 + 1, y as u32 + 3), [255, 0, 0]);
        // The clear half shows the tiles under it.
        assert_eq!(px(&f, x as u32 + 3, y as u32), under);
    }

    #[test]
    fn an_image_part_draws_only_that_part() {
        let sheet = Sheet::load(None).unwrap();
        let (mut s, x, y) = with_npc("block.png", 5.0, -4.0);
        // Only the clear half: nothing red is drawn.
        s.current_level.as_mut().unwrap().npcs[0].image_part = Some((2, 0, 2, 4));
        let images = NpcImages::from([("block.png".to_string(), Some(half_red()))]);
        let f = draw(&s, &sheet, &images, &[], 40, 30);
        assert_ne!(px(&f, x as u32, y as u32), [255, 0, 0]);
    }

    #[test]
    fn a_missing_image_is_outlined_and_a_hidden_npc_is_not_drawn() {
        let sheet = Sheet::load(None).unwrap();
        let (mut s, x, y) = with_npc("gone.png", 5.0, -4.0);
        let images = NpcImages::from([("gone.png".to_string(), None)]);
        let f = draw(&s, &sheet, &images, &[], 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), NPC_MISSING);
        assert_eq!(px(&f, x as u32 + 31, y as u32 + 31), NPC_MISSING);
        s.current_level.as_mut().unwrap().npcs[0].visible = false;
        let f = draw(&s, &sheet, &images, &[], 40, 30);
        assert_ne!(px(&f, x as u32, y as u32), NPC_MISSING);
    }

    #[test]
    fn an_npc_with_no_image_draws_nothing() {
        let sheet = Sheet::load(None).unwrap();
        let (s, x, y) = with_npc("", 5.0, -4.0);
        let plain = draw(&session(0), &sheet, &NpcImages::new(), &[], 40, 30);
        let f = draw(&s, &sheet, &NpcImages::new(), &[], 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), px(&plain, x as u32, y as u32));
    }

    #[test]
    fn png_and_gif_names_decode_and_others_do_not() {
        let mut png = Vec::new();
        seihai_core::assets::write_png_rgb(&mut png, 2, 1, &[1, 2, 3, 4, 5, 6]).unwrap();
        let img = Image::decode("Thing.PNG", &png).unwrap();
        assert_eq!((img.w, img.h), (2, 1));
        assert_eq!(&img.rgba[..4], &[1, 2, 3, 255]);
        assert!(Image::decode("thing.mng", &png).is_none());
        assert!(Image::decode("thing.gif", &png).is_none(), "not a GIF");
    }

    #[test]
    fn images_are_looked_for_once_and_missing_ones_reported() {
        let (s, _, _) = with_npc("nowhere.png", 1.0, 1.0);
        let level = s.current_level.as_ref().unwrap();
        let mut images = NpcImages::new();
        assert_eq!(load_npc_images(level, None, &mut images), ["nowhere.png"]);
        assert_eq!(images.get("nowhere.png"), Some(&None));
        assert!(load_npc_images(level, None, &mut images).is_empty());
    }

    #[test]
    fn a_gmap_neighbour_is_drawn_past_the_edge() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(0);
        s.player_x = 60.0;
        let grid = "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n";
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        // The middle of tile (64, 30): one tile past the east edge.
        let (ex, ey) = ((64 * 16 - cam_x + 8) as u32, (30 * 16 - cam_y + 8) as u32);
        let f = draw(&s, &sheet, &NpcImages::new(), &[], 40, 30);
        assert_eq!(px(&f, ex, ey), [0, 0, 0], "no gmap: black past the edge");
        s.enter_gmap(
            "w.gmap".into(),
            seihai_core::gmap::Gmap::parse(grid).unwrap(),
            (0, 0),
        );
        s.gmap
            .as_mut()
            .unwrap()
            .levels
            .insert((1, 0), Level::from_board(&[600; 4096]));
        let f = draw(&s, &sheet, &NpcImages::new(), &[], 40, 30);
        let want = draw(&session(600), &sheet, &NpcImages::new(), &[], 40, 30);
        // Tile 600's middle pixel, wherever it is drawn.
        let (cx, cy) = camera_origin(&session(600), 40, 30);
        let (wx, wy) = ((30 * 16 - cx + 8) as u32, (30 * 16 - cy + 8) as u32);
        assert_eq!(px(&f, ex, ey), px(&want, wx, wy));
    }
}
