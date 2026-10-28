//! Minimal offline board view.
//!
//! SDL2 is not available in this environment. This module draws a
//! loaded level as ASCII (or a raw PPM) so offline movement can be
//! watched. `tiles` composites a 16×16 tile view into an RGB buffer
//! from a 2048×512 sheet. A real SDL2/OpenGL path can replace this later.

pub mod font;
mod npcs;
mod throw;
pub mod hud;
pub mod signs;
mod tiles;

pub use throw::{draw_sprite, draw_throw_layer};
pub use npcs::{draw_npcs, NpcArt, NpcPass, NPC_DIR};
pub use tiles::{
    draw_gani_frame, draw_other_player, draw_player_marker, draw_player_sprites, PoseSheets, Sheet, player_box_origin, render_view, render_view_with, ViewOptions, BUILTIN_CHEST_CLOSED, BUILTIN_CHEST_GRASS, CHEST_CLOSED_TILES, CHEST_OPEN_TILES, chest_art,
    Framebuffer, TILE_PX,
};

use crate::assets::{read_png_rgb, tile_id, Level};
use crate::offline::OfflineSession;
use std::fs;
use std::path::Path;

/// Load a `pics1.png` (or any 2048×512 RGB sheet) and render a camera view.
///
/// Returns the framebuffer. The sheet is expected to be 8-bit RGB.
pub fn render_with_sheet<P: AsRef<Path>>(
    session: &OfflineSession,
    sheet_path: P,
    view_tiles_w: u32,
    view_tiles_h: u32,
) -> Result<Framebuffer, String> {
    let data = fs::read(sheet_path.as_ref()).map_err(|e| e.to_string())?;
    let (w, h, sheet) = read_png_rgb(&data).map_err(|e| e.to_string())?;
    Ok(render_view(session, &sheet, w, h, view_tiles_w, view_tiles_h))
}

/// ASCII preview of the current board and player.
///
/// Tiles are shown as `.` (passable), `~` (water), or `#` (wall) when a
/// tile-type table is loaded; otherwise as a hex nibble of the tile id.
/// Player is a direction character.
pub fn ascii_board(session: &OfflineSession, width: usize, height: usize) -> String {
    let mut out = String::new();
    let level = match &session.current_level {
        Some(l) => l,
        None => return "(no level)\n".into(),
    };
    let types = session.tiletypes.as_ref();

    for y in 0..height {
        for x in 0..width {
            if (x as f32 - session.player_x).abs() < 0.5
                && (y as f32 - session.player_y).abs() < 0.5
            {
                out.push(player_char(session));
                continue;
            }
            let id = tile_at(level, x as i32, y as i32);
            out.push(tile_char(types, id));
        }
        out.push('\n');
    }
    out
}

/// ASCII preview with camera centered on the player.
///
/// `view_w` / `view_h` is the visible window. Out-of-bounds cells are ` `.
pub fn ascii_camera(session: &OfflineSession, view_w: usize, view_h: usize) -> String {
    let mut out = String::new();
    let level = match &session.current_level {
        Some(l) => l,
        None => return "(no level)\n".into(),
    };
    let types = session.tiletypes.as_ref();
    let ox = session.player_x as i32 - (view_w as i32 / 2);
    let oy = session.player_y as i32 - (view_h as i32 / 2);

    for dy in 0..view_h as i32 {
        for dx in 0..view_w as i32 {
            let x = ox + dx;
            let y = oy + dy;
            if (x as f32 - session.player_x).abs() < 0.5
                && (y as f32 - session.player_y).abs() < 0.5
            {
                out.push(player_char(session));
                continue;
            }
            if x < 0 || y < 0 || x >= 64 || y >= 64 {
                out.push(' ');
                continue;
            }
            let id = tile_at(level, x, y);
            out.push(tile_char(types, id));
        }
        out.push('\n');
    }
    out
}

fn player_char(session: &OfflineSession) -> char {
    if session.in_water {
        match session.player_dir {
            0 => 'A',
            1 => '{',
            2 => 'V',
            3 => '}',
            _ => 'W',
        }
    } else {
        dir_char(session.player_dir)
    }
}

fn dir_char(dir: u8) -> char {
    match dir {
        0 => '^', // up
        1 => '<', // left
        2 => 'v', // down
        3 => '>', // right
        _ => 'P',
    }
}

fn tile_char(types: Option<&crate::assets::TileTypes>, id: u16) -> char {
    if let Some(t) = types {
        if t.is_wall(id) {
            '#'
        } else if t.is_water(id) {
            '~'
        } else {
            '.'
        }
    } else {
        char::from_digit((id & 0xF) as u32, 16).unwrap_or('.')
    }
}

/// ASCII preview of a single stamp (for blit-path proof).
/// Uses the same wall/water/passable characters as the board view.
pub fn ascii_stamp(stamp: &crate::assets::Stamp, types: Option<&crate::assets::TileTypes>) -> String {
    let mut out = String::new();
    out.push_str(&format!("stamp {} ({}x{})\n", stamp.name, stamp.width, stamp.height));
    for row in 0..stamp.height {
        for col in 0..stamp.width {
            let id = stamp.tile_at(col, row).map(crate::assets::tile_id).unwrap_or(0);
            out.push(tile_char(types, id));
        }
        out.push('\n');
    }
    out
}

fn tile_at(level: &Level, x: i32, y: i32) -> u16 {
    for run in &level.boards {
        if run.layer != 0 {
            continue;
        }
        if x >= run.x && y >= run.y && x < run.x + run.width {
            let col = (x - run.x) as usize;
            if col < run.tiles.len() {
                return tile_id(run.tiles[col]);
            }
        }
    }
    0
}

/// Write a 64×64 PPM (P6) of the board. Walls are dark, water blue,
/// passable light, player red.
pub fn write_ppm(session: &OfflineSession, path: &str) -> Result<(), String> {
    let mut pixels = vec![0u8; 64 * 64 * 3];
    let level = session
        .current_level
        .as_ref()
        .ok_or("no level loaded")?;
    let types = session.tiletypes.as_ref();

    for y in 0..64 {
        for x in 0..64 {
            let id = tile_at(level, x, y);
            let (r, g, b) = if (x as f32 - session.player_x).abs() < 0.5
                && (y as f32 - session.player_y).abs() < 0.5
            {
                if session.in_water {
                    (40, 120, 220)
                } else {
                    (220, 40, 40)
                }
            } else if types.map(|t| t.is_wall(id)).unwrap_or(false) {
                (40, 40, 40)
            } else if types.map(|t| t.is_water(id)).unwrap_or(false) {
                (40, 80, 180)
            } else {
                (180, 180, 160)
            };
            let i = ((y * 64 + x) * 3) as usize;
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
        }
    }

    let mut out = format!("P6\n64 64\n255\n").into_bytes();
    out.extend_from_slice(&pixels);
    std::fs::write(path, out).map_err(|e| e.to_string())
}
