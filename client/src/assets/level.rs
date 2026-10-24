//! Level file parser (.nw / GLEVNW01 format).

use std::fs;
use std::path::Path;

/// A single tile (2-character code, e.g. "Id").
pub type Tile = [u8; 2];

/// Encode a 12-bit tile id (0–4095) as its 2-char base64 code.
pub fn tile_code(id: u16) -> Tile {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    [B64[((id >> 6) & 0x3F) as usize], B64[(id & 0x3F) as usize]]
}

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
    /// Visibility flag. `hide`/`show` commands toggle this.
    pub visible: bool,
}

/// A level link (LINK record).
#[derive(Debug, Clone)]
pub struct Link {
    pub dest: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// Arrival position in the destination level. `None` means the
    /// link says `playerx` / `playery`: keep the player's coordinate.
    pub new_x: Option<f32>,
    pub new_y: Option<f32>,
}

impl Link {
    /// True if tile `(x, y)` is inside the link's rectangle.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
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

/// Tile type byte for water (`isOnWater`).
pub const TYPE_WATER: u8 = 11;
/// Tile type byte for lava. Walkable, like every type below 20.
pub const TYPE_LAVA: u8 = 12;
/// Type bytes from here up are walls (`isOnWall`).
pub const TYPE_WALL_MIN: u8 = 20;
/// Type given to tiles that `arrays.dat` does not list as passable.
const TYPE_HARD_WALL: u8 = 22;

/// What a tile does to a walking player.
///
/// The 2.05 client and GServer only ever ask two questions of the type
/// byte: `>= 20` is a wall and `== 11` is water. Lava (12) is told apart
/// because the client has its own `lava` animation, but it does not
/// block. Every other type below 20 is plain floor for movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Open,
    Water,
    Lava,
    Blocking,
}

/// 4096-byte collision table: one type byte per tile id.
/// Wall >= 20, water == 11, lava == 12.
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
            // index,x,y,type[,name]; longer lines are flat lists
            if (4..=5).contains(&parts.len()) && parts[0].parse::<u16>().is_ok() {
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

    /// Load a Graal `arrays.dat`. See [`Self::parse_arrays`].
    pub fn load_arrays<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        Self::parse_arrays(&data)
    }

    /// Parse a Graal `arrays.dat`.
    ///
    /// Format (big-endian, from the Zelda Online `makeArrays` lineage):
    /// a run of length-prefixed int arrays. The ones used here:
    ///
    /// - array 0: the passable-tile list (`nowall`). Listed tiles are
    ///   type 0; every tile not listed is a hard wall (22).
    /// - array 1: the liquid list. Water and lava tiles are in it, and
    ///   many of the water tiles are *not* in `nowall`, so the first
    ///   version of this loader turned water into wall.
    /// - array 3: the lava list. A liquid tile that is also here is lava
    ///   (12, walkable); one that is not is water (11, swimmable).
    ///
    /// Arrays 2 and 4 to 7 do not change movement and are skipped. Any
    /// array that is cut short or implausibly long ends the scan, so a
    /// file with only the first array still loads.
    pub fn parse_arrays(data: &[u8]) -> Result<Self, String> {
        if data.len() < 4 {
            return Err("arrays.dat too short".into());
        }
        let mut arrays: Vec<Vec<usize>> = Vec::new();
        let mut off = 0usize;
        while arrays.len() < 4 && off + 4 <= data.len() {
            let count = u32::from_be_bytes(data[off..off + 4].try_into().unwrap()) as usize;
            let end = off.saturating_add(4).saturating_add(count.saturating_mul(4));
            if end > data.len() || (!arrays.is_empty() && count > 4096) {
                if arrays.is_empty() {
                    return Err(format!("arrays.dat truncated: need {count} ints"));
                }
                break;
            }
            arrays.push(
                data[off + 4..end]
                    .chunks_exact(4)
                    .map(|c| u32::from_be_bytes(c.try_into().unwrap()) as usize)
                    .collect(),
            );
            off = end;
        }
        let mut types = [TYPE_HARD_WALL; 4096];
        if let Some(nowall) = arrays.first() {
            for &id in nowall.iter().filter(|&&id| id < 4096) {
                types[id] = 0;
            }
        }
        let lava: std::collections::HashSet<usize> =
            arrays.get(3).map(|a| a.iter().copied().collect()).unwrap_or_default();
        if let Some(liquid) = arrays.get(1) {
            for &id in liquid.iter().filter(|&&id| id < 4096) {
                types[id] = if lava.contains(&id) { TYPE_LAVA } else { TYPE_WATER };
            }
        }
        Ok(TileTypes { types })
    }

    /// The raw type byte for `id`, if the id is on the table.
    pub fn raw(&self, id: u16) -> Option<u8> {
        self.types.get(id as usize).copied()
    }

    /// What the tile does to a walking player. An id off the table is a
    /// wall, as in the client.
    pub fn kind(&self, id: u16) -> TileKind {
        match self.raw(id) {
            None => TileKind::Blocking,
            Some(t) if t >= TYPE_WALL_MIN => TileKind::Blocking,
            Some(TYPE_WATER) => TileKind::Water,
            Some(TYPE_LAVA) => TileKind::Lava,
            Some(_) => TileKind::Open,
        }
    }

    pub fn is_wall(&self, id: u16) -> bool {
        self.kind(id) == TileKind::Blocking
    }

    pub fn is_water(&self, id: u16) -> bool {
        self.kind(id) == TileKind::Water
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

    fn arrays_blob(arrays: &[&[u32]]) -> Vec<u8> {
        let mut data = Vec::new();
        for a in arrays {
            data.extend_from_slice(&(a.len() as u32).to_be_bytes());
            for v in *a {
                data.extend_from_slice(&v.to_be_bytes());
            }
        }
        data
    }

    #[test]
    fn arrays_liquid_list_makes_water_and_lava() {
        // 5 is water and also passable; 6 is water and not passable;
        // 7 is lava (liquid list and lava list).
        let data = arrays_blob(&[&[0, 5], &[5, 6, 7], &[9], &[7]]);
        let t = TileTypes::parse_arrays(&data).unwrap();
        assert_eq!(t.kind(0), TileKind::Open);
        assert_eq!(t.kind(5), TileKind::Water);
        assert_eq!(t.kind(6), TileKind::Water, "water must not be a wall");
        assert_eq!(t.kind(7), TileKind::Lava);
        assert_eq!(t.kind(1), TileKind::Blocking);
        assert!(t.is_water(6) && !t.is_wall(6) && !t.is_wall(7));
    }

    #[test]
    fn arrays_tolerates_trailing_data_and_one_array() {
        let mut data = arrays_blob(&[&[0], &[4]]);
        // junk that looks like a huge count
        data.extend_from_slice(&0x0001_0004u32.to_be_bytes());
        data.extend_from_slice(&[1, 2, 3]);
        let t = TileTypes::parse_arrays(&data).unwrap();
        assert_eq!(t.kind(4), TileKind::Water);
        assert!(TileTypes::parse_arrays(&arrays_blob(&[&[0]])).is_ok());
        assert!(TileTypes::parse_arrays(&[0, 0, 1]).is_err());
        assert!(TileTypes::parse_arrays(&[0, 0, 0, 9, 0, 0]).is_err());
    }

    #[test]
    fn kind_follows_the_type_byte() {
        let mut t = TileTypes { types: [0; 4096] };
        t.types[1] = TYPE_WATER;
        t.types[2] = TYPE_LAVA;
        t.types[3] = 19;
        t.types[4] = 20;
        t.types[5] = 22;
        t.types[6] = 3; // beds, chairs and the rest of 1..19 do not block
        assert_eq!(
            [0u16, 1, 2, 3, 4, 5, 6].map(|id| t.kind(id)),
            [
                TileKind::Open,
                TileKind::Water,
                TileKind::Lava,
                TileKind::Open,
                TileKind::Blocking,
                TileKind::Blocking,
                TileKind::Open
            ]
        );
        assert_eq!(t.kind(4096), TileKind::Blocking);
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


use crate::protocol::layouts::BoardModify2;
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
    /// Build a level from a 64×64 board of tile ids, row by row, as the
    /// server sends it in `PLO_BOARDPACKET`. Links, signs and chests
    /// arrive in their own packets and start empty here.
    pub fn from_board(tiles: &[u16]) -> Self {
        let boards = tiles
            .chunks(64)
            .take(64)
            .enumerate()
            .map(|(y, row)| BoardRun {
                x: 0,
                y: y as i32,
                width: row.len() as i32,
                layer: 0,
                tiles: row.iter().map(|&t| tile_code(t)).collect(),
            })
            .collect();
        Level {
            boards,
            npcs: Vec::new(),
            links: Vec::new(),
            signs: Vec::new(),
            baddies: Vec::new(),
            chests: Vec::new(),
        }
    }

    /// Add the server's `PLO_LEVELLINK` lines (`dest x y w h newx newy`).
    /// Lines that do not parse are skipped.
    pub fn add_server_links(&mut self, lines: &[String]) {
        self.links
            .extend(lines.iter().filter_map(|l| parse_link(&format!("LINK {l}"))));
    }

    /// Load a .nw or .graal file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let data = fs::read(path).map_err(|e| e.to_string())?;
        if data.len() >= 8 && &data[0..8] == b"GLEVNW01" {
            let text = String::from_utf8_lossy(&data);
            return Self::parse(&text);
        }
        if data.len() >= 8 && (data[0..8].starts_with(b"GR-V1") || data[0..8].starts_with(b"Z3-V1")) {
            return Self::parse_binary(&data);
        }
        Err(format!("unrecognised level header in {}", path.display()))
    }

    /// Parse a binary .graal (GR-V1) bitstream into a Level.
    fn parse_binary(data: &[u8]) -> Result<Self, String> {
        if data.len() < 8 {
            return Err("binary level too short".into());
        }
        let magic = &data[0..8];
        // 13 bits for GR-V1.02 to 1.04 (the 4096-tile sheet needs the top
        // bit free); 12 for GR-V1.00/1.01 and Z3-V1.03/04; 11 for Z3-V1.02.
        let codebits = if matches!(magic, b"GR-V1.02" | b"GR-V1.03" | b"GR-V1.04") {
            13
        } else if magic == b"Z3-V1.02" {
            11
        } else {
            12
        };
        let mut tiles = vec![0u16; 4096];
        let mut boardindex = 0usize;
        let mut bitsread = 0u32;
        let mut buffer = 0u32;
        let mut count = 1u32;
        let mut firstcode = -1i32;
        let mut twicerepeat = false;
        let mut pos = 8usize;

        while boardindex < 4096 && pos < data.len() {
            while bitsread < codebits && pos < data.len() {
                buffer |= (data[pos] as u32) << bitsread;
                bitsread += 8;
                pos += 1;
            }
            if bitsread < codebits {
                break;
            }
            let mask = (1u32 << codebits) - 1;
            let code = buffer & mask;
            buffer >>= codebits;
            bitsread -= codebits;

            if code & (1 << (codebits - 1)) != 0 {
                twicerepeat = (code & 0x100) != 0;
                count = code & 0xFF;
            } else if count == 1 {
                tiles[boardindex] = code as u16;
                boardindex += 1;
            } else if twicerepeat {
                if firstcode < 0 {
                    firstcode = code as i32;
                } else {
                    for _ in 1..=count {
                        if boardindex >= 4095 {
                            break;
                        }
                        tiles[boardindex] = firstcode as u16;
                        tiles[boardindex + 1] = code as u16;
                        boardindex += 2;
                    }
                    firstcode = -1;
                    twicerepeat = false;
                    count = 1;
                }
            } else {
                for _ in 1..=count {
                    if boardindex >= 4096 {
                        break;
                    }
                    tiles[boardindex] = code as u16;
                    boardindex += 1;
                }
                count = 1;
            }
        }

        // Convert the flat tile array into row runs for layer 0.
        let mut boards = Vec::new();
        for y in 0..64 {
            let mut run_start = 0;
            let mut run_tiles = Vec::new();
            for x in 0..64 {
                let t = tiles[y * 64 + x];
                if t != 0 || !run_tiles.is_empty() {
                    if run_tiles.is_empty() {
                        run_start = x;
                    }
                    run_tiles.push(tile_code(t));
                }
                if (t == 0 || x == 63) && !run_tiles.is_empty() {
                    boards.push(BoardRun {
                        x: run_start as i32,
                        y: y as i32,
                        width: run_tiles.len() as i32,
                        layer: 0,
                        tiles: run_tiles.clone(),
                    });
                    run_tiles.clear();
                }
            }
        }

        // `pos` is the byte after the last code, so the sections start there.
        let mut level = Level {
            boards,
            npcs: Vec::new(),
            links: Vec::new(),
            signs: Vec::new(),
            baddies: Vec::new(),
            chests: Vec::new(),
        };
        parse_binary_sections(magic, &data[pos.min(data.len())..], &mut level);
        Ok(level)
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

    /// Place a stamp onto the board at (x, y).
    /// Each stamp row becomes a new layer-0 BOARD run.
    /// Collision is decided later by the type table.
    pub fn place_stamp(&mut self, stamp: &crate::assets::Stamp, x: i32, y: i32) {
        for row in 0..stamp.height {
            let mut tiles = Vec::with_capacity(stamp.width);
            for col in 0..stamp.width {
                if let Some(t) = stamp.tile_at(col, row) {
                    tiles.push(t);
                }
            }
            if !tiles.is_empty() {
                self.boards.push(BoardRun {
                    x,
                    y: y + row as i32,
                    width: tiles.len() as i32,
                    layer: 0,
                    tiles,
                });
            }
        }
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

    /// Layer 0 as a 64×64 board of tile ids, row by row. A tile the level
    /// does not define is 0.
    pub fn layer0_board(&self) -> Vec<u16> {
        let mut board = vec![0u16; 64 * 64];
        for run in self.boards.iter().filter(|r| r.layer == 0) {
            if !(0..64).contains(&run.y) {
                continue;
            }
            for (i, t) in run.tiles.iter().enumerate() {
                let x = run.x + i as i32;
                if (0..64).contains(&x) {
                    board[run.y as usize * 64 + x as usize] = tile_id(*t);
                }
            }
        }
        board
    }

    /// Replace layer 0 with a 64×64 board of tile ids (the other layers,
    /// links, signs and chests stay).
    pub fn set_layer0_board(&mut self, board: &[u16]) {
        self.boards.retain(|r| r.layer != 0);
        let rows = board.chunks(64).take(64).enumerate().map(|(y, row)| BoardRun {
            x: 0,
            y: y as i32,
            width: row.len() as i32,
            layer: 0,
            tiles: row.iter().map(|&t| tile_code(t)).collect(),
        });
        let mut runs: Vec<BoardRun> = rows.collect();
        runs.append(&mut self.boards);
        self.boards = runs;
    }

    /// Apply a `PLO_BOARDMODIFY2` block to layer 0 with its clipping rules
    /// (see [`BoardModify2::apply_to`]). Returns the tiles written.
    pub fn apply_board_modify2(&mut self, m: &BoardModify2) -> usize {
        let mut board = self.layer0_board();
        let n = m.apply_to(&mut board);
        if n > 0 {
            self.set_layer0_board(&board);
        }
        n
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

    /// The kind of the tile under the point `(x, y)` (in tiles), or
    /// `Blocking` off the board. A point is in the tile it falls in:
    /// the original client truncates its test point to a tile (`int
    /// testx = ...`), so tile 63 runs from 63.0 up to 64.0. A tile the
    /// level does not define is open floor.
    pub fn kind_at(&self, types: &TileTypes, x: f32, y: f32) -> TileKind {
        if x < 0.0 || y < 0.0 || x >= 64.0 || y >= 64.0 {
            return TileKind::Blocking;
        }
        match self.tile_id_at(x as i32, y as i32) {
            Some(id) => types.kind(id),
            None => TileKind::Open,
        }
    }

    /// True if the point is a wall (or off the board).
    pub fn is_on_wall(&self, types: &TileTypes, x: f32, y: f32) -> bool {
        self.kind_at(types, x, y) == TileKind::Blocking
    }

    /// True if the point is in water.
    pub fn is_on_water(&self, types: &TileTypes, x: f32, y: f32) -> bool {
        self.kind_at(types, x, y) == TileKind::Water
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
        visible: true,
    })
}

/// Names of the chest item indexes 0 to 24 (`LevelItemType`).
pub const CHEST_ITEMS: [&str; 25] = [
    "greenrupee", "bluerupee", "redrupee", "bombs", "darts", "heart", "glove1", "bow", "bomb", "shield", "sword",
    "fullheart", "superbomb", "battleaxe", "goldensword", "mirrorshield", "glove2", "lizardshield", "lizardsword",
    "goldrupee", "fireball", "fireblast", "nukeshot", "joltbomb", "spinattack",
];

/// Bytes to text, one char per byte (the files are Latin-1 era).
fn latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

/// Line reader over the bytes after a binary level's board.
struct Sections<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Sections<'a> {
    fn at_end(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// The next line without its `\n` (and without a trailing `\r`).
    fn line(&mut self) -> Option<&'a [u8]> {
        if self.at_end() {
            return None;
        }
        let rest = &self.data[self.pos..];
        let (line, used) = match rest.iter().position(|&b| b == b'\n') {
            Some(i) => (&rest[..i], i + 1),
            None => (rest, rest.len()),
        };
        self.pos += used;
        Some(line.strip_suffix(b"\r").unwrap_or(line))
    }
}

/// Read the sections after the board of a GR-V1 or Z3-V1 file: links,
/// baddies, NPCs, chests, signs, in that order, and skip a GR-V1.04
/// `HEIGHTS` block. Layout from Graal_analysis `formats/level-binary.md`
/// and `level-binary-client.md` (2.05, 2.31). A section that is cut short
/// leaves what was read; nothing here fails the whole level.
fn parse_binary_sections(magic: &[u8], tail: &[u8], level: &mut Level) {
    let graal = magic.starts_with(b"GR-V1");
    // GR-V1.00 and later have NPCs; GR-V1.01 and later have chests; Z3-V1
    // has neither, and its baddies have a verse line from Z3-V1.04.
    let has_npcs = graal;
    let has_chests = graal && magic != b"GR-V1.00";
    let baddy_verse = graal || magic == b"Z3-V1.04";
    let mut s = Sections { data: tail, pos: 0 };

    // Links: lines up to "#" or an empty line.
    while let Some(l) = s.line() {
        if l.is_empty() || l == b"#" {
            break;
        }
        if let Some(link) = parse_binary_link(&latin1(l)) {
            level.links.push(link);
        }
    }

    // Baddies: 3 raw bytes x, y, type, then a verse line. The record ends at
    // the first `\n` after those 3 bytes, which may themselves hold 0x0A.
    // A first byte of 0x80 or more (FF FF FF) ends the section.
    loop {
        if s.pos + 3 > s.data.len() {
            s.pos = s.data.len();
            break;
        }
        let rec = &s.data[s.pos..];
        let end = rec[3..].iter().position(|&b| b == b'\n').map(|i| i + 3);
        let stop = rec[0] >= 0x80;
        let (verse, used) = match (baddy_verse, end) {
            (true, Some(e)) => (&rec[3..e], e + 1),
            (true, None) => (&rec[3..], rec.len()),
            (false, _) => (&rec[3..3], 3),
        };
        let (x, y, kind) = (rec[0], rec[1], rec[2]);
        s.pos += used;
        if stop {
            // Without a verse line the terminator's own line end is next.
            if !baddy_verse && s.data.get(s.pos) == Some(&b'\n') {
                s.pos += 1;
            }
            break;
        }
        level.baddies.push(Baddy {
            x: x as f32,
            y: y as f32,
            kind: kind.to_string(),
            verses: latin1(verse).replace('\\', "\n"),
        });
    }

    // NPCs: `x+32, y+32, image#script`, the script's line breaks as 0xA7.
    if has_npcs {
        while let Some(l) = s.line() {
            if l.is_empty() || l == b"#" {
                break;
            }
            if l.len() < 2 {
                continue;
            }
            let body = &l[2..];
            let (image, script) = match body.iter().position(|&b| b == b'#') {
                Some(i) => (&body[..i], &body[i + 1..]),
                None => (body, &body[..0]),
            };
            level.npcs.push(Npc {
                image: latin1(image),
                x: l[0] as f32 - 32.0,
                y: l[1] as f32 - 32.0,
                script: latin1(script).replace('\u{a7}', "\n"),
                visible: true,
            });
        }
    }

    // Chests: four GCHARs, `x y item signindex`.
    if has_chests {
        while let Some(l) = s.line() {
            if l.is_empty() || l == b"#" {
                break;
            }
            if l.len() < 4 {
                continue;
            }
            let item = l[2] as i32 - 32;
            level.chests.push(Chest {
                x: l[0] as i32 - 32,
                y: l[1] as i32 - 32,
                item: usize::try_from(item)
                    .ok()
                    .and_then(|i| CHEST_ITEMS.get(i))
                    .map_or_else(|| item.to_string(), |n| n.to_string()),
                sign_index: l[3] as i32 - 32,
            });
        }
    }

    // Signs: to an empty line (2.31) or the end (2.05). The sign alphabet is
    // the one used on the wire.
    while let Some(l) = s.line() {
        if l.is_empty() {
            break;
        }
        if let Some((x, y, text)) = decode_sign_record(l) {
            level.signs.push(Sign { x, y, text });
        }
    }
    // A GR-V1.04 `HEIGHTS` block may follow; it is read past and not kept.
}

/// A binary level's link line, `dest x y w h newx newy`. Spaces in the
/// level name are allowed: every token before the last six is the name.
fn parse_binary_link(line: &str) -> Option<Link> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 7 {
        return None;
    }
    let n = parts.len();
    Some(Link {
        dest: parts[..n - 6].join(" "),
        x: parts[n - 6].parse().ok()?,
        y: parts[n - 5].parse().ok()?,
        width: parts[n - 4].parse().ok()?,
        height: parts[n - 3].parse().ok()?,
        new_x: link_coord(parts[n - 2], "playerx")?,
        new_y: link_coord(parts[n - 1], "playery")?,
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
        new_x: link_coord(parts[6], "playerx")?,
        new_y: link_coord(parts[7], "playery")?,
    })
}

/// A LINK arrival coordinate: a number, or `playerx` / `playery`
/// (`Some(None)`). Anything else rejects the link.
fn link_coord(s: &str, keep: &str) -> Option<Option<f32>> {
    if s.eq_ignore_ascii_case(keep) {
        Some(None)
    } else {
        s.parse().ok().map(Some)
    }
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
    fn link_keeps_player_coordinate() {
        let lvl = Level::parse("GLEVNW01\nLINK new1.nw 0 10 1 4 62 playery\n").unwrap();
        assert_eq!(lvl.links.len(), 1);
        assert_eq!((lvl.links[0].new_x, lvl.links[0].new_y), (Some(62.0), None));
        // The server's PLO_LEVELLINK form has no LINK keyword.
        let mut board = Level::from_board(&[0; 4096]);
        board.add_server_links(&["new1.nw 33 28 2 6 20 12.5".to_string(), "bad".to_string()]);
        assert_eq!(board.links.len(), 1);
        assert_eq!((board.links[0].x, board.links[0].new_y), (33, Some(12.5)));
    }

    #[test]
    fn board_ids_round_trip() {
        let tiles: Vec<u16> = (0..4096).collect();
        let lvl = Level::from_board(&tiles);
        assert_eq!(tile_code(3 * 64 + 29), *b"Dd");
        for (x, y) in [(0, 0), (63, 0), (5, 9), (63, 63)] {
            assert_eq!(lvl.tile_id_at(x, y), Some((y * 64 + x) as u16));
        }
    }

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
        assert_eq!((lvl.links[0].new_x, lvl.links[0].new_y), (Some(30.0), Some(1.0)));
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

    #[test]
    fn layer0_board_round_trips_and_keeps_other_layers() {
        let mut board = vec![0u16; 64 * 64];
        board[0] = 7;
        board[64 * 63 + 63] = 4095;
        let mut level = Level::from_board(&board);
        level.boards.push(BoardRun { x: 0, y: 0, width: 1, layer: 1, tiles: vec![tile_code(9)] });
        assert_eq!(level.layer0_board(), board);
        board[5] = 12;
        level.set_layer0_board(&board);
        assert_eq!(level.layer0_board(), board);
        assert_eq!(level.boards.iter().filter(|r| r.layer == 1).count(), 1);
        assert_eq!(level.tile_id_at(5, 0), Some(12));
    }

    #[test]
    fn board_modify2_clips_at_column_63_on_a_level() {
        let mut level = Level::from_board(&vec![1u16; 64 * 64]);
        let m = BoardModify2 { map_x: 0, map_y: 0, x: 62, y: 3, width: 4, height: 2, tiles: vec![9; 8], trailer: vec![] };
        assert_eq!(level.apply_board_modify2(&m), 4);
        assert_eq!(level.tile_id_at(61, 3), Some(1));
        assert_eq!(level.tile_id_at(62, 3), Some(9));
        assert_eq!(level.tile_id_at(63, 4), Some(9));
        assert_eq!(level.tile_id_at(0, 4), Some(1), "no wrap to the next row");
        assert_eq!(level.tile_id_at(62, 5), Some(1));
    }
    /// A GR-V1 board writer after the 2.05 rules (`level-binary-client.md`):
    /// a single run for equal neighbours (up to 64), a double run for an
    /// `A B A B` pattern, plain tiles otherwise; codes packed LSB first.
    fn encode_board(tiles: &[u16], bits: u32) -> Vec<u8> {
        let flag = 1u32 << (bits - 1);
        let mut codes: Vec<u32> = Vec::new();
        let mut i = 0;
        while i < tiles.len() {
            if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
                let mut n = 1;
                while i + n < tiles.len() && tiles[i + n] == tiles[i] && n < 64 {
                    n += 1;
                }
                codes.push(flag | n as u32);
                codes.push(tiles[i] as u32);
                i += n;
            } else if i + 3 < tiles.len() && tiles[i] == tiles[i + 2] && tiles[i + 1] == tiles[i + 3] {
                let mut n = 1;
                while i + 2 * n + 1 < tiles.len() && tiles[i + 2 * n] == tiles[i] && tiles[i + 2 * n + 1] == tiles[i + 1] && n < 64 {
                    n += 1;
                }
                codes.push(flag | 0x100 | n as u32);
                codes.push(tiles[i] as u32);
                codes.push(tiles[i + 1] as u32);
                i += 2 * n;
            } else {
                codes.push(tiles[i] as u32);
                i += 1;
            }
        }
        let (mut out, mut buf, mut have) = (Vec::new(), 0u64, 0u32);
        for c in codes {
            buf |= (c as u64) << have;
            have += bits;
            while have >= 8 {
                out.push(buf as u8);
                buf >>= 8;
                have -= 8;
            }
        }
        if have > 0 {
            out.push(buf as u8);
        }
        out
    }

    fn sample_board() -> Vec<u16> {
        let mut t = vec![0u16; 4096];
        for (i, v) in t.iter_mut().enumerate() {
            *v = match i % 64 {
                0..=9 => 2,
                10..=19 => if i % 2 == 0 { 7 } else { 9 },
                20..=21 => 1500,
                _ => (i % 5) as u16 * 3,
            };
        }
        t[63] = 4095;
        t[4095] = 2047;
        t
    }

    fn binary_level(magic: &[u8; 8], bits: u32, tail: &[u8]) -> Vec<u8> {
        let mut d = magic.to_vec();
        d.extend(encode_board(&sample_board(), bits));
        d.extend_from_slice(tail);
        d
    }

    fn board_of(l: &Level) -> Vec<u16> {
        l.layer0_board()
    }

    #[test]
    fn graal_boards_decode_at_every_code_width() {
        let want = sample_board();
        for (magic, bits) in [
            (b"GR-V1.00", 12u32),
            (b"GR-V1.01", 12),
            (b"GR-V1.02", 13),
            (b"GR-V1.03", 13),
            (b"GR-V1.04", 13),
        ] {
            let mut b = want.clone();
            if bits == 12 {
                // 12-bit files cannot hold tiles of 2048 or more.
                for t in b.iter_mut() {
                    *t &= 0x7FF;
                }
            }
            let mut d = magic.to_vec();
            d.extend(encode_board(&b, bits));
            d.extend_from_slice(b"\n");
            let l = Level::parse_binary(&d).unwrap();
            assert_eq!(board_of(&l), b, "{}", String::from_utf8_lossy(magic));
        }
    }

    #[test]
    fn graal_sections_are_read_in_order() {
        let mut tail: Vec<u8> = Vec::new();
        // Links, with a space in a level name.
        tail.extend_from_slice(b"house 1.graal 3 4 2 1 10 11\nnew1.nw 0 10 1 4 62 playery\n#\n");
        // Baddies: x, y, type, verses; a 0x0A inside the 3 raw bytes; then FF FF FF.
        tail.extend_from_slice(&[5, 6, 2]);
        tail.extend_from_slice(b"saw you\\ouch\\grr\n");
        tail.extend_from_slice(&[10, 7, 0]);
        tail.extend_from_slice(b"a\\b\\c\n");
        tail.extend_from_slice(&[0xFF, 0xFF, 0xFF, b'\n']);
        // NPCs: x+32, y+32, image#script with 0xA7 for line breaks.
        tail.extend_from_slice(&[32 + 30, 32 + 28]);
        tail.extend_from_slice(b"door.png#if (playerenters) {\xA7  say hi;\xA7}\n");
        tail.extend_from_slice(&[32 + 1, 32 + 2]);
        tail.extend_from_slice(b"#message x;\n");
        tail.extend_from_slice(b"#\n");
        // Chests: x, y, item, sign index, each +32.
        tail.extend_from_slice(&[32 + 7, 32 + 8, 32 + 1, 32 + 0, b'\n']);
        tail.extend_from_slice(&[32 + 9, 32 + 9, 32 + 99, 32 + 3, b'\n']);
        tail.extend_from_slice(b"#\n");
        // A sign: x+32, y+32, then text in the sign alphabet (a lower-case letter is stored as char + 89).
        tail.extend_from_slice(&[32 + 12, 32 + 13, b'a' + 89, b'\n']);
        let d = binary_level(b"GR-V1.03", 13, &tail);
        let l = Level::parse_binary(&d).unwrap();
        assert_eq!(l.links.len(), 2);
        assert_eq!((l.links[0].dest.as_str(), l.links[0].x, l.links[0].y, l.links[0].width, l.links[0].height), ("house 1.graal", 3, 4, 2, 1));
        assert_eq!((l.links[0].new_x, l.links[0].new_y), (Some(10.0), Some(11.0)));
        assert_eq!((l.links[1].new_x, l.links[1].new_y), (Some(62.0), None));
        assert_eq!(l.baddies.len(), 2);
        assert_eq!((l.baddies[0].x, l.baddies[0].y, l.baddies[0].kind.as_str()), (5.0, 6.0, "2"));
        assert_eq!(l.baddies[0].verses, "saw you\nouch\ngrr");
        assert_eq!((l.baddies[1].x, l.baddies[1].y), (10.0, 7.0));
        assert_eq!(l.npcs.len(), 2);
        assert_eq!((l.npcs[0].image.as_str(), l.npcs[0].x, l.npcs[0].y), ("door.png", 30.0, 28.0));
        assert_eq!(l.npcs[0].script, "if (playerenters) {\n  say hi;\n}");
        assert_eq!(l.npcs[1].image, "");
        assert_eq!((l.npcs[1].x, l.npcs[1].y), (1.0, 2.0));
        assert_eq!(l.chests.len(), 2);
        assert_eq!((l.chests[0].x, l.chests[0].y, l.chests[0].item.as_str(), l.chests[0].sign_index), (7, 8, "bluerupee", 0));
        // An item index with no name stays a number.
        assert_eq!(l.chests[1].item, "99");
        assert_eq!(l.signs.len(), 1);
        assert_eq!((l.signs[0].x, l.signs[0].y, l.signs[0].text.as_str()), (12, 13, "a"));
        // The board is untouched by the tail.
        assert_eq!(board_of(&l), sample_board());
    }

    #[test]
    fn graal_00_has_no_chest_section_and_heights_are_skipped() {
        // GR-V1.00: links, baddies, NPCs, then the signs directly.
        let mut tail: Vec<u8> = Vec::new();
        tail.extend_from_slice(b"#\n");
        tail.extend_from_slice(&[0xFF, 0xFF, 0xFF, b'\n']);
        tail.extend_from_slice(b"#\n");
        tail.extend_from_slice(&[32 + 1, 32 + 1, b'a' + 89, b'\n']);
        let mut b = sample_board();
        for t in b.iter_mut() {
            *t &= 0x7FF;
        }
        let mut d = b"GR-V1.00".to_vec();
        d.extend(encode_board(&b, 12));
        d.extend_from_slice(&tail);
        let l = Level::parse_binary(&d).unwrap();
        assert!(l.chests.is_empty());
        assert_eq!(l.signs.len(), 1);

        // GR-V1.04: a HEIGHTS block after the signs is not read as a sign.
        let mut tail: Vec<u8> = Vec::new();
        tail.extend_from_slice(b"#\n");
        tail.extend_from_slice(&[0xFF, 0xFF, 0xFF, b'\n']);
        tail.extend_from_slice(b"#\n#\n");
        tail.extend_from_slice(&[32 + 2, 32 + 2, b'a' + 89, b'\n']);
        tail.extend_from_slice(b"\nHEIGHTS\n0,0,0,0,0,0,0,0,0\nHEIGHTSEND\n");
        let d = binary_level(b"GR-V1.04", 13, &tail);
        let l = Level::parse_binary(&d).unwrap();
        assert_eq!(l.signs.len(), 1);
    }

    #[test]
    fn graal_tail_that_is_cut_short_keeps_the_board() {
        for cut in [0usize, 1, 5, 30] {
            let tail = b"dest.nw 1 2 3 4 5 6\n#\n\x05\x06\x02verse\n".to_vec();
            let d = binary_level(b"GR-V1.03", 13, &tail[..cut.min(tail.len())]);
            let l = Level::parse_binary(&d).unwrap();
            assert_eq!(board_of(&l), sample_board(), "cut {cut}");
        }
        // No tail at all.
        let l = Level::parse_binary(&binary_level(b"GR-V1.03", 13, b"")).unwrap();
        assert!(l.links.is_empty() && l.npcs.is_empty() && l.signs.is_empty());
    }

    #[test]
    fn load_reads_a_graal_file_from_disk() {
        let dir = std::env::temp_dir().join(format!("seihai-graal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("t.graal");
        std::fs::write(&p, binary_level(b"GR-V1.03", 13, b"a.graal 0 0 1 1 5 5\n#\n")).unwrap();
        let l = Level::load(&p).unwrap();
        assert_eq!(l.links.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

}
