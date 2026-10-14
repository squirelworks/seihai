//! Minimal offline board view.
//!
//! SDL2 is not available in this environment. This module draws a
//! loaded level as ASCII (or a raw PPM) so offline movement can be
//! watched. A real SDL2/OpenGL path can replace this later.

use crate::assets::{tile_id, Level};
use crate::offline::OfflineSession;

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
            0 => 'V',
            1 => '{',
            2 => '}',
            3 => 'A',
            _ => 'W',
        }
    } else {
        dir_char(session.player_dir)
    }
}

fn dir_char(dir: u8) -> char {
    match dir {
        0 => 'v', // south
        1 => '<', // west
        2 => '>', // east
        3 => '^', // north
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
