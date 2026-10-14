//! Level file parser (.nw / GLEVNW01 format).

use std::fs;
use std::path::Path;

/// A single tile (2-character code, e.g. "Id").
pub type Tile = [u8; 2];

/// Decode a 2-char base64 tile code into a 12-bit tile id (0–4095).
pub fn tile_id(tile: Tile) -> u16 {
    fn val(c: u8) -> u16 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u16,
            b'a'..=b'z' => (c - b'a' + 26) as u16,
            b'0'..=b'9' => (c - b'0' + 52) as u16,
            b'+' => 62,
            b'/' => 63,
            _ => 0,
        }
    }
    (val(tile[0]) << 6) | val(tile[1])
}

/// Source rectangle for a tile id on a 2048×512 sheet (128×32 of 16 px).
pub fn tile_src_rect(id: u16) -> (u32, u32, u32, u32) {
    let col = (id as u32) % 128;
    let row = (id as u32) / 128;
    (col * 16, row * 16, 16, 16)
}

/// One BOARD line: a horizontal run of tiles.
#[derive(Debug, Clone)]
pub struct BoardRun {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub layer: i32,
    pub tiles: Vec<Tile>,
}

/// An NPC defined in the level file.
#[derive(Debug, Clone)]
pub struct Npc {
    pub image: String,
    pub x: f32,
    pub y: f32,
    pub script: String,
}

/// A level link (LINK record).
#[derive(Debug, Clone)]
pub struct Link {
    pub dest: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub new_x: i32,
    pub new_y: i32,
}

/// A sign (SIGN / SIGNEND). Text is already plain for GLEVNW01.
#[derive(Debug, Clone)]
pub struct Sign {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

/// A baddy (BADDY / BADDYEND).
#[derive(Debug, Clone)]
pub struct Baddy {
    pub x: f32,
    pub y: f32,
    pub kind: String,
    pub verses: String,
}

/// A chest (CHEST x y item signindex).
#[derive(Debug, Clone)]
pub struct Chest {
    pub x: i32,
    pub y: i32,
    pub item: String,
    pub sign_index: i32,
}

/// 4096-byte collision table. wall >= 20, water == 11.
#[derive(Debug, Clone)]
pub struct TileTypes {
    pub types: [u8; 4096],
}

impl TileTypes {
    /// Load a text tiledefs file.
    /// Accepts either a flat comma-separated list or
    /// `index,x,y,type,name` rows.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let mut types = [0u8; 4096];
        let mut i = 0;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            // index,x,y,type,name
            if parts.len() >= 4 && parts[0].parse::<u16>().is_ok() {
                if let Ok(idx) = parts[0].parse::<usize>() {
                    if idx < 4096 {
                        types[idx] = parts[3].parse().unwrap_or(0);
                        i = i.max(idx + 1);
                        continue;
                    }
                }
            }
            // flat list
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if i >= 4096 {
                    break;
                }
                types[i] = part.parse().unwrap_or(0);
                i += 1;
            }
        }
        if i < 4096 {
            return Err(format!("tiledefs short: {i} values"));
        }
        Ok(TileTypes { types })
    }

    /// Load a Graal `arrays.dat`.
    ///
    /// Format (big-endian, from the Zelda Online `makeArrays` lineage):
    /// length-prefixed int arrays. The first array is the passable-tile
    /// list (`nowall`). Tiles in that list get type 0; everything else
    /// gets type 22 (hard wall).
    pub fn load_arrays<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        Self::parse_arrays(&data)
    }

    pub fn parse_arrays(data: &[u8]) -> Result<Self, String> {
        if data.len() < 4 {
            return Err("arrays.dat too short".into());
        }
        let count = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
        if data.len() < 4 + count * 4 {
            return Err(format!("arrays.dat truncated: need {count} ints"));
        }
        let mut types = [22u8; 4096];
        for i in 0..count {
            let off = 4 + i * 4;
            let id = u32::from_be_bytes(data[off..off + 4].try_into().unwrap()) as usize;
            if id < 4096 {
                types[id] = 0;
            }
        }
        Ok(TileTypes { types })
    }

    pub fn is_wall(&self, id: u16) -> bool {
        self.types.get(id as usize).copied().unwrap_or(20) >= 20
    }

    pub fn is_water(&self, id: u16) -> bool {
        self.types.get(id as usize).copied().unwrap_or(0) == 11
    }
}

#[cfg(test)]
mod tiletypes_tests {
    use super::*;

    #[test]
    fn arrays_marks_nowall_passable() {
        // count=2, ids 0 and 8
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&8u32.to_be_bytes());
        let t = TileTypes::parse_arrays(&data).unwrap();
        assert!(!t.is_wall(0));
        assert!(!t.is_wall(8));
        assert!(t.is_wall(1));
    }

    #[test]
    fn text_index_type_rows() {
        let text = "0,0,0,22,blocking\n1,1,0,0,non-blocking\n";
        // pad to 4096 so parse accepts
        let mut full = text.to_string();
        for i in 2..4096 {
            full.push_str(&format!("{i},0,0,0,x\n"));
        }
        let t = TileTypes::parse(&full).unwrap();
        assert!(t.is_wall(0));
        assert!(!t.is_wall(1));
    }
}


use crate::protocol::BoardModify;

/// A parsed level.
#[derive(Debug, Clone)]
pub struct Level {
    pub boards: Vec<BoardRun>,
    pub npcs: Vec<Npc>,
    pub links: Vec<Link>,
    pub signs: Vec<Sign>,
    pub baddies: Vec<Baddy>,
    pub chests: Vec<Chest>,
}

impl Level {
    /// Parse a .nw file from disk.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&text)
    }

    /// Parse .nw content.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut boards = Vec::new();
        let mut npcs = Vec::new();
        let mut links = Vec::new();
        let mut signs = Vec::new();
        let mut baddies = Vec::new();
        let mut chests = Vec::new();
        let mut lines = text.lines().peekable();

        // Header
        let header = lines.next().unwrap_or("").trim();
        if header != "GLEVNW01" {
            return Err(format!("unexpected header: {header}"));
        }

        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("BOARD ") {
                if let Some(run) = parse_board(line) {
                    boards.push(run);
                }
            } else if line.starts_with("NPC ") {
                let mut npc_lines = vec![line.to_string()];
                while let Some(l) = lines.next() {
                    npc_lines.push(l.to_string());
                    if l.trim() == "NPCEND" {
                        break;
                    }
                }
                if let Some(npc) = parse_npc(&npc_lines) {
                    npcs.push(npc);
                }
            } else if line.starts_with("LINK ") {
                if let Some(link) = parse_link(line) {
                    links.push(link);
                }
            } else if line.starts_with("SIGN ") {
                let mut sign_lines = vec![line.to_string()];
                while let Some(l) = lines.next() {
                    sign_lines.push(l.to_string());
                    if l.trim() == "SIGNEND" {
                        break;
                    }
                }
                if let Some(sign) = parse_sign(&sign_lines) {
                    signs.push(sign);
                }
            } else if line.starts_with("BADDY ") {
                let mut baddy_lines = vec![line.to_string()];
                while let Some(l) = lines.next() {
                    baddy_lines.push(l.to_string());
                    if l.trim() == "BADDYEND" {
                        break;
                    }
                }
                if let Some(baddy) = parse_baddy(&baddy_lines) {
                    baddies.push(baddy);
                }
            } else if line.starts_with("CHEST ") {
                if let Some(chest) = parse_chest(line) {
                    chests.push(chest);
                }
            }
        }

        Ok(Level {
            boards,
            npcs,
            links,
            signs,
            baddies,
            chests,
        })
    }

    /// Apply a board modify (PLO_BOARDMODIFY style) to layer 0.
    /// Creates or extends BOARD runs as needed.
    pub fn apply_board_modify(&mut self, m: &BoardModify) {
        // For simplicity we only handle layer 0 and rebuild affected rows.
        for row in 0..m.height as i32 {
            let y = m.y as i32 + row;
            let row_tiles: Vec<Tile> = (0..m.width as usize)
                .filter_map(|col| {
                    let idx = (row as usize * m.width as usize + col) * 2;
                    if idx + 1 < m.tiles.len() {
                        Some([m.tiles[idx], m.tiles[idx + 1]])
                    } else {
                        None
                    }
                })
                .collect();

            // Remove any existing layer-0 run that overlaps this row segment
            self.boards.retain(|b| {
                !(b.layer == 0 && b.y == y && b.x < m.x as i32 + m.width as i32 && b.x + b.width > m.x as i32)
            });

            // Insert a new run for the modified segment
            if !row_tiles.is_empty() {
                self.boards.push(BoardRun {
                    x: m.x as i32,
                    y,
                    width: row_tiles.len() as i32,
                    layer: 0,
                    tiles: row_tiles,
                });
            }
        }
    }

    /// Get the tile at (x, y) on layer 0, if present.
    pub fn tile_at(&self, x: i32, y: i32) -> Option<Tile> {
        for run in &self.boards {
            if run.layer == 0 && run.y == y && x >= run.x && x < run.x + run.width {
                let idx = (x - run.x) as usize;
                return run.tiles.get(idx).copied();
            }
        }
        None
    }

    /// Tile id at (x, y), if present.
    pub fn tile_id_at(&self, x: i32, y: i32) -> Option<u16> {
        self.tile_at(x, y).map(tile_id)
    }

    /// True if the cell is a wall (or out of bounds) given a collision table.
    pub fn is_on_wall(&self, types: &TileTypes, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 || x > 63.0 || y > 63.0 {
            return true;
        }
        let tx = x.round() as i32;
        let ty = y.round() as i32;
        match self.tile_id_at(tx, ty) {
            Some(id) => types.is_wall(id),
            None => false,
        }
    }

    /// True if the cell is water.
    pub fn is_on_water(&self, types: &TileTypes, x: f32, y: f32) -> bool {
        let tx = x as i32;
        let ty = y as i32;
        match self.tile_id_at(tx, ty) {
            Some(id) => types.is_water(id),
            None => false,
        }
    }
}

fn parse_board(line: &str) -> Option<BoardRun> {
    // BOARD x y width layer tiledata
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 || parts[0] != "BOARD" {
        return None;
    }
    let x: i32 = parts[1].parse().ok()?;
    let y: i32 = parts[2].parse().ok()?;
    let width: i32 = parts[3].parse().ok()?;
    let layer: i32 = parts[4].parse().ok()?;
    let data = parts[5];

    let mut tiles = Vec::new();
    let bytes = data.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        tiles.push([bytes[i], bytes[i + 1]]);
        i += 2;
    }

    Some(BoardRun {
        x,
        y,
        width,
        layer,
        tiles,
    })
}

fn parse_npc(lines: &[String]) -> Option<Npc> {
    // NPC image x y
    // script...
    // NPCEND
    if lines.is_empty() {
        return None;
    }
    let header: Vec<&str> = lines[0].split_whitespace().collect();
    if header.len() < 4 || header[0] != "NPC" {
        return None;
    }
    let image = header[1].to_string();
    let x: f32 = header[2].parse().ok()?;
    let y: f32 = header[3].parse().ok()?;

    let script = lines[1..lines.len().saturating_sub(1)]
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    Some(Npc {
        image,
        x,
        y,
        script,
    })
}

fn parse_link(line: &str) -> Option<Link> {
    // LINK dest x y width height newx newy
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 8 || parts[0] != "LINK" {
        return None;
    }
    Some(Link {
        dest: parts[1].to_string(),
        x: parts[2].parse().ok()?,
        y: parts[3].parse().ok()?,
        width: parts[4].parse().ok()?,
        height: parts[5].parse().ok()?,
        new_x: parts[6].parse().ok()?,
        new_y: parts[7].parse().ok()?,
    })
}

fn parse_sign(lines: &[String]) -> Option<Sign> {
    // SIGN x y
    // text...
    // SIGNEND
    // GLEVNW01 text is already plain. Binary records use decode_sign_record.
    if lines.is_empty() {
        return None;
    }
    let header: Vec<&str> = lines[0].split_whitespace().collect();
    if header.len() < 3 || header[0] != "SIGN" {
        return None;
    }
    let x: i32 = header[1].parse().ok()?;
    let y: i32 = header[2].parse().ok()?;
    let text = lines[1..lines.len().saturating_sub(1)]
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Some(Sign { x, y, text })
}

fn parse_baddy(lines: &[String]) -> Option<Baddy> {
    // BADDY x y type
    // verses...
    // BADDYEND
    if lines.is_empty() {
        return None;
    }
    let header: Vec<&str> = lines[0].split_whitespace().collect();
    if header.len() < 4 || header[0] != "BADDY" {
        return None;
    }
    let x: f32 = header[1].parse().ok()?;
    let y: f32 = header[2].parse().ok()?;
    let kind = header[3].to_string();
    let verses = lines[1..lines.len().saturating_sub(1)]
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    Some(Baddy { x, y, kind, verses })
}

fn parse_chest(line: &str) -> Option<Chest> {
    // CHEST x y item signindex
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 || parts[0] != "CHEST" {
        return None;
    }
    Some(Chest {
        x: parts[1].parse().ok()?,
        y: parts[2].parse().ok()?,
        item: parts[3].to_string(),
        sign_index: parts[4].parse().ok()?,
    })
}

/// Binary sign alphabet: stored = char + 89.
/// Decode `(byte - 89) & 0x7F`, then subtract 6 if the result is A–Z.
/// The 7-byte token `v*eUWf` + 0x80 is a space.
/// Do not use on GLEVNW01 text or NPC scripts.
pub fn decode_sign_text(payload: &[u8]) -> String {
    const SPACE: &[u8] = &[0x76, 0x2a, 0x65, 0x55, 0x57, 0x66, 0x80];
    let mut text = String::new();
    let mut i = 0;
    while i < payload.len() {
        if i + SPACE.len() <= payload.len() && &payload[i..i + SPACE.len()] == SPACE {
            text.push(' ');
            i += SPACE.len();
            continue;
        }
        let mut value = (payload[i].wrapping_sub(89)) & 0x7f;
        if (0x41..=0x5a).contains(&value) {
            value -= 6;
        }
        i += 1;
        if value == 0x0a || value == 0x0d {
            text.push('\n');
        } else if value >= 0x20 && value != 0x7f {
            text.push(char::from(value));
        }
    }
    text.trim().to_string()
}

/// Full binary sign record: byte0 = x+0x20, byte1 = y+0x20, rest = alphabet.
pub fn decode_sign_record(line: &[u8]) -> Option<(i32, i32, String)> {
    if line.len() < 2 {
        return None;
    }
    let x = line[0] as i32 - 0x20;
    let y = line[1] as i32 - 0x20;
    let text = decode_sign_text(&line[2..]);
    Some((x, y, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        let src = r#"GLEVNW01
BOARD 0 0 4 0 IdIdIdAb
NPC - 10 20
if (created) { }
NPCEND
LINK town 0 63 64 1 30 1
SIGN 5 5
Hello
SIGNEND
BADDY 10 12 0
grrrr
BADDYEND
CHEST 8 8 greenrupee 0
"#;
        let lvl = Level::parse(src).unwrap();
        assert_eq!(lvl.boards.len(), 1);
        assert_eq!(lvl.boards[0].tiles.len(), 4);
        assert_eq!(lvl.npcs.len(), 1);
        assert_eq!(lvl.npcs[0].x, 10.0);
        assert_eq!(lvl.links.len(), 1);
        assert_eq!(lvl.links[0].dest, "town");
        assert_eq!(lvl.signs.len(), 1);
        assert_eq!(lvl.signs[0].text, "Hello");
        assert_eq!(lvl.baddies.len(), 1);
        assert_eq!(lvl.baddies[0].kind, "0");
        assert_eq!(lvl.chests.len(), 1);
        assert_eq!(lvl.chests[0].item, "greenrupee");
    }

    #[test]
    fn tile_id_roundtrip() {
        assert_eq!(tile_id([b'A', b'A']), 0);
        assert_eq!(tile_id([b'/', b'/']), 4095);
    }

    #[test]
    fn tiletypes_wall_water() {
        let mut text = String::from("# comment\n");
        for i in 0..4096 {
            let v = if i == 10 { 11 } else if i >= 100 { 20 } else { 0 };
            text.push_str(&format!("{v},"));
            if (i + 1) % 128 == 0 {
                text.push('\n');
            }
        }
        let tt = TileTypes::parse(&text).unwrap();
        assert!(tt.is_water(10));
        assert!(tt.is_wall(100));
        assert!(!tt.is_wall(0));
    }
}
