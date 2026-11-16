//! The Seihai level editor: open a `GLEVNW01` text level or a binary
//! `GR-V1` / `Z3-V1` level, look at its tiles, paint a rectangle, flood-fill
//! a connected region, save. The window (`gui`) is a tile palette and
//! click-to-paint on top of the same `Document`.
//!
//! The rules come from the 2.22 editor notes (see `documentation/level-editor.md`):
//! the board is one 64 x 64 grid of 12-bit tile ids; a `BOARD` row is placed by
//! `x`, `y` and `w` and its layer field is not read; rows that do not fit are
//! skipped without an error; a character outside the base64 alphabet ends a
//! row.
//!
//! **Round trip.** A level that was not changed is written back byte for
//! byte (text lines as they were; binary the original file). After a paint,
//! text `BOARD` rows are replaced by the canonical 2.22 writer shape, and
//! binary levels are rebuilt with seihai-core's `encode_binary` (`GR-V1.03`
//! or `GR-V1.05` for Graal, `Z3-V1.04` for Zelda). The write profile follows
//! the path extension: `.nw` text, `.zelda` Zelda binary, everything else
//! Graal binary.
//!
//! **Layers.** The 2.22 reader does not read a row's layer field, so `tile`
//! and `row` give that view: every row on one board, in file order. The
//! editor also keeps layer 0 and layer 1 apart (`layer_tile`,
//! `paint_layer`), the way Gonstruct does: layer 0 starts as tile 0
//! everywhere, layer 1 starts empty. After a change, layer 0 is written as
//! the 64 canonical rows and layer 1 as one `BOARD x y w 1 data` row per run
//! of filled cells; rows of other layers are written back as they were.
//! Binary `GR-V1.05` keeps layers above 1 from the core reader.
//!
//! **Objects.** Links, signs, chests, baddies and NPCs are records of lines
//! (`objects`) on text levels, read with the client's level parser. Moving
//! one, or changing a link's destination and arrival, a baddy's type and
//! verses, or the body of an NPC script or a sign, rewrites only that
//! record's lines (`edit_object`). Object edits of binary levels are not
//! supported yet; their records are kept and written back after a paint.
//!
//! **Profiles.** The user picks a [`VersionProfile`] (2.22 text, GR-V1.03,
//! GR-V1.05, Z3-V1.04, …). Open follows the file magic (`GLEVNW01`, `GR-V1`,
//! `Z3-V1`). Save follows the path extension (`.nw` text, `.zelda` Zelda,
//! otherwise Graal binary).
//!
//! **Selection.** A tile rectangle on the active layer can be copied, cut,
//! deleted or pasted. Copy and cut lift the tiles into a block the window
//! can float under the mouse; dropping the float writes them. An unchanged
//! level still round-trips byte for byte: only a write that changes a cell
//! marks the file dirty.

pub mod binary;
pub mod gui;
pub mod profile;

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

pub mod objects;
pub use binary::LevelProfile;
pub use objects::{
    Object, ObjectEdit, ObjectKind, ScriptHead, BADDY_TYPE_NAMES, BADDY_VERSE_LINES,
    SCRIPT_PANE_LINES,
};
pub use profile::{IoStatus, Version, VersionProfile};

use seihai_core::assets::Level;

/// Board edge in tiles.
pub const SIZE: usize = 64;
/// Tile ids the text format can hold (two base64 characters).
pub const TILE_LIMIT: u16 = 4096;
/// Layers the editor shows and edits (0 and 1).
pub const LAYERS: usize = 2;

/// A rectangle of board cells (top-left and size), clipped to the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl TileRect {
    /// A non-empty rectangle that stays on the 64 x 64 board.
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Result<TileRect, EditError> {
        if w == 0 || h == 0 {
            return Err(EditError::Bounds(format!(
                "the rectangle is empty ({w} x {h}); width and height must be at least 1"
            )));
        }
        if x >= SIZE || y >= SIZE || x + w > SIZE || y + h > SIZE {
            return Err(EditError::Bounds(format!(
                "the rectangle x {x}, y {y}, {w} x {h} leaves the {SIZE} x {SIZE} board (x + width and y + height must be at most {SIZE})"
            )));
        }
        Ok(TileRect { x, y, w, h })
    }

    /// The axis-aligned rectangle covering the two corner cells (order free).
    pub fn from_corners(a: (usize, usize), b: (usize, usize)) -> Result<TileRect, EditError> {
        let x0 = a.0.min(b.0);
        let y0 = a.1.min(b.1);
        let x1 = a.0.max(b.0);
        let y1 = a.1.max(b.1);
        TileRect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1)
    }
}

/// A rectangular block of tiles lifted from one layer. `None` is an empty
/// layer-1 cell; layer 0 always stores `Some`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileBlock {
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<Option<u16>>,
}

impl TileBlock {
    fn at(&self, x: usize, y: usize) -> Option<u16> {
        self.tiles.get(y * self.w + x).copied().flatten()
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_value(c: u8) -> Option<u16> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as u16),
        b'a'..=b'z' => Some((c - b'a') as u16 + 26),
        b'0'..=b'9' => Some((c - b'0') as u16 + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// The two-character code of a tile id (id < 4096).
pub fn tile_code(id: u16) -> [u8; 2] {
    [B64[((id >> 6) & 63) as usize], B64[(id & 63) as usize]]
}

/// What can go wrong; every message says what to do or what is not there yet.
#[derive(Debug, PartialEq, Eq)]
pub enum EditError {
    /// The file could not be read or written.
    Io(String),
    /// Not a recognised level.
    NotText(String),
    /// A rectangle that leaves the 64 x 64 board, or is empty.
    Bounds(String),
    /// A tile id of 4096 or more.
    TileRange(u32),
    /// A version the editor does not handle (yet).
    Version(String),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::Io(m)
            | EditError::NotText(m)
            | EditError::Bounds(m)
            | EditError::Version(m) => f.write_str(m),
            EditError::TileRange(id) => write!(
                f,
                "tile id {id} is out of range: levels hold ids 0 to {} (12-bit)",
                TILE_LIMIT - 1
            ),
        }
    }
}

impl std::error::Error for EditError {}

/// Counts of what a level holds, for `list`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    /// BOARD rows that were placed on the board.
    pub rows_placed: usize,
    /// BOARD rows that were skipped (out of range, too short, not numbers).
    pub rows_skipped: usize,
    /// Placed rows whose layer field is not 0 (the 2.22 reader ignores it).
    pub rows_other_layer: usize,
    pub links: usize,
    pub signs: usize,
    pub chests: usize,
    pub baddies: usize,
    pub npcs: usize,
    /// A HEIGHTS block is present; the editor leaves it as it is.
    pub heights: bool,
    /// Placed rows with a layer other than 0 and 1 (kept as they are).
    pub rows_other_layers_kept: usize,
}

/// An opened level.
#[derive(Debug, Clone)]
pub struct Document {
    /// The file as lines, each with its line terminator (the last may have none).
    /// Empty for a binary level.
    lines: Vec<Vec<u8>>,
    /// Indexes of the BOARD lines in `lines`.
    board_lines: Vec<usize>,
    /// The board as 2.22 reads it (all rows, layer not read), index = y * 64 + x.
    board: Vec<u16>,
    /// Layer 0 (starts as tile 0) and layer 1 (starts empty), index = y * 64 + x.
    layer0: Vec<u16>,
    layer1: Vec<Option<u16>>,
    /// Placed BOARD rows of other layers, without their line ending, in file order.
    other_rows: Vec<Vec<u8>>,
    /// The object records, in file order (text only).
    records: Vec<objects::Record>,
    /// Lines were changed by an object edit (the board rows are not rewritten for that).
    edited: bool,
    summary: Summary,
    eol: &'static [u8],
    dirty: bool,
    /// How the file was opened.
    profile: LevelProfile,
    /// Original file bytes (byte-identical unchanged save).
    raw: Vec<u8>,
    /// Binary only: objects and layers above 1 from the core reader.
    binary_level: Option<Level>,
}

fn strip_eol(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn parse_num(tok: &[u8]) -> Option<i64> {
    std::str::from_utf8(tok).ok()?.parse().ok()
}

impl Document {
    /// Open a level from its bytes: `GLEVNW01` text, or `GR-V1` / `Z3-V1` binary.
    pub fn parse(data: &[u8]) -> Result<Document, EditError> {
        if data.len() >= 8 && (data.starts_with(b"GR-V1") || data.starts_with(b"Z3-V1")) {
            return binary::parse_binary(data);
        }
        Self::parse_text(data)
    }

    pub(crate) fn from_binary(
        raw: Vec<u8>,
        profile: LevelProfile,
        level: Level,
        board: Vec<u16>,
        layer0: Vec<u16>,
        layer1: Vec<Option<u16>>,
        summary: Summary,
    ) -> Document {
        Document {
            lines: Vec::new(),
            board_lines: Vec::new(),
            board,
            layer0,
            layer1,
            other_rows: Vec::new(),
            records: Vec::new(),
            edited: false,
            summary,
            eol: b"\n",
            dirty: false,
            profile,
            raw,
            binary_level: Some(level),
        }
    }

    pub(crate) fn binary_level(&self) -> Option<&Level> {
        self.binary_level.as_ref()
    }

    pub(crate) fn layer0_tiles(&self) -> &[u16] {
        &self.layer0
    }

    pub(crate) fn layer1_tiles(&self) -> &[Option<u16>] {
        &self.layer1
    }

    /// Text bytes for converting objects when writing binary from a text level.
    pub(crate) fn text_snapshot_bytes(&self) -> Vec<u8> {
        if self.profile != LevelProfile::Text {
            return self.raw.clone();
        }
        if self.dirty {
            self.encode_text()
        } else {
            self.lines.concat()
        }
    }

    fn parse_text(data: &[u8]) -> Result<Document, EditError> {
        let mut lines: Vec<Vec<u8>> = Vec::new();
        let mut start = 0;
        for (i, &b) in data.iter().enumerate() {
            if b == b'\n' {
                lines.push(data[start..=i].to_vec());
                start = i + 1;
            }
        }
        if start < data.len() {
            lines.push(data[start..].to_vec());
        }
        let header_ok = lines
            .first()
            .map(|l| strip_eol(l).trim_ascii() == b"GLEVNW01")
            .unwrap_or(false);
        if !header_ok {
            return Err(EditError::NotText(
                "not a GLEVNW01 text level (the first line must be GLEVNW01) or a GR-V1 / Z3-V1 binary level".into(),
            ));
        }
        let eol: &'static [u8] = if lines[0].ends_with(b"\r\n") {
            b"\r\n"
        } else {
            b"\n"
        };

        let mut doc = Document {
            lines,
            board_lines: Vec::new(),
            board: vec![0; SIZE * SIZE],
            layer0: vec![0; SIZE * SIZE],
            layer1: vec![None; SIZE * SIZE],
            other_rows: Vec::new(),
            records: Vec::new(),
            edited: false,
            summary: Summary::default(),
            eol,
            dirty: false,
            profile: LevelProfile::Text,
            raw: data.to_vec(),
            binary_level: None,
        };
        let mut i = 1;
        while i < doc.lines.len() {
            let raw = strip_eol(&doc.lines[i]).to_vec();
            let toks: Vec<&[u8]> = raw
                .split(|&b| b == b' ')
                .filter(|t| !t.is_empty())
                .collect();
            let key = toks.first().copied().unwrap_or(b"");
            let block_end: Option<&[u8]> = match key {
                b"NPC" => {
                    doc.summary.npcs += 1;
                    Some(b"NPCEND")
                }
                b"SIGN" => {
                    doc.summary.signs += 1;
                    Some(b"SIGNEND")
                }
                b"BADDY" => {
                    doc.summary.baddies += 1;
                    Some(b"BADDYEND")
                }
                b"HEIGHTS" => {
                    doc.summary.heights = true;
                    Some(b"HEIGHTSEND")
                }
                _ => None,
            };
            if let Some(end) = block_end {
                let first = i;
                i += 1;
                while i < doc.lines.len() && strip_eol(&doc.lines[i]).trim_ascii() != end {
                    i += 1;
                }
                let kind = match key {
                    b"NPC" => Some(ObjectKind::Npc),
                    b"SIGN" => Some(ObjectKind::Sign),
                    b"BADDY" => Some(ObjectKind::Baddy),
                    _ => None,
                };
                if let Some(kind) = kind {
                    doc.records.push(objects::Record {
                        kind,
                        first,
                        last: i.min(doc.lines.len() - 1),
                    });
                }
                i += 1;
                continue;
            }
            match key {
                b"BOARD" => {
                    doc.board_lines.push(i);
                    doc.place_row(&toks, &raw);
                }
                b"LINK" => {
                    doc.summary.links += 1;
                    doc.records.push(objects::Record {
                        kind: ObjectKind::Link,
                        first: i,
                        last: i,
                    });
                }
                b"CHEST" => {
                    doc.summary.chests += 1;
                    doc.records.push(objects::Record {
                        kind: ObjectKind::Chest,
                        first: i,
                        last: i,
                    });
                }
                _ => {}
            }
            i += 1;
        }
        Ok(doc)
    }

    /// Place one `BOARD x y w layer data` row by the 2.22 reader's rules.
    fn place_row(&mut self, toks: &[&[u8]], raw: &[u8]) {
        let (Some(x), Some(y), Some(w)) = (
            toks.get(1).and_then(|t| parse_num(t)),
            toks.get(2).and_then(|t| parse_num(t)),
            toks.get(3).and_then(|t| parse_num(t)),
        ) else {
            self.summary.rows_skipped += 1;
            return;
        };
        let layer = toks.get(4).and_then(|t| parse_num(t)).unwrap_or(0);
        let data = toks.get(5).copied().unwrap_or(b"");
        let fits = (0..64).contains(&x) && (0..64).contains(&y) && w >= 1 && x + w <= 64;
        if !fits || (data.len() as i64) < 2 * w {
            self.summary.rows_skipped += 1;
            return;
        }
        self.summary.rows_placed += 1;
        if layer != 0 {
            self.summary.rows_other_layer += 1;
        }
        if layer != 0 && layer != 1 {
            self.summary.rows_other_layers_kept += 1;
            self.other_rows.push(raw.to_vec());
        }
        for k in 0..w as usize {
            let (Some(hi), Some(lo)) = (b64_value(data[2 * k]), b64_value(data[2 * k + 1])) else {
                break;
            };
            let at = y as usize * SIZE + x as usize + k;
            let t = (hi << 6) | lo;
            self.board[at] = t;
            match layer {
                0 => self.layer0[at] = t,
                1 => self.layer1[at] = Some(t),
                _ => {}
            }
        }
    }

    /// Read a level file with the default (2.22 text) profile.
    pub fn open(path: &Path) -> Result<Document, EditError> {
        Document::open_with_profile(path, VersionProfile::DEFAULT)
    }

    /// Read a level file under `profile`. Every listed profile is ready;
    /// open still follows the file magic (`GLEVNW01` / `GR-V1` / `Z3-V1`).
    pub fn open_with_profile(path: &Path, profile: VersionProfile) -> Result<Document, EditError> {
        profile.require_io_ready()?;
        let data = fs::read(path)
            .map_err(|e| EditError::Io(format!("cannot read {}: {e}", path.display())))?;
        Document::parse(&data)
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
    }

    /// How this level was opened (text, Graal binary, or Zelda binary).
    pub fn profile(&self) -> LevelProfile {
        self.profile
    }

    /// True when the open file is `GR-V1` or `Z3-V1`.
    pub fn is_binary(&self) -> bool {
        self.profile != LevelProfile::Text
    }

    /// Whether a paint or an object edit changed the level since it was opened.
    pub fn is_modified(&self) -> bool {
        self.dirty || self.edited
    }

    /// Tile of `layer` (0 or 1) at (x, y). `None` outside the board, for
    /// another layer, and for an empty layer-1 cell.
    pub fn layer_tile(&self, layer: usize, x: usize, y: usize) -> Option<u16> {
        if x >= SIZE || y >= SIZE {
            return None;
        }
        match layer {
            0 => Some(self.layer0[y * SIZE + x]),
            1 => self.layer1[y * SIZE + x],
            _ => None,
        }
    }

    /// Filled cells of layer 1.
    pub fn layer1_count(&self) -> usize {
        self.layer1.iter().filter(|t| t.is_some()).count()
    }

    /// Tile id at (x, y), or `None` outside the board.
    pub fn tile(&self, x: usize, y: usize) -> Option<u16> {
        (x < SIZE && y < SIZE).then(|| self.board[y * SIZE + x])
    }

    /// Row `y` of the board (64 ids).
    pub fn row(&self, y: usize) -> Option<&[u16]> {
        (y < SIZE).then(|| &self.board[y * SIZE..(y + 1) * SIZE])
    }

    /// Distinct tile ids with their counts, most used first (ties by id).
    pub fn histogram(&self) -> Vec<(u16, usize)> {
        let mut counts = std::collections::BTreeMap::new();
        for &t in &self.board {
            *counts.entry(t).or_insert(0usize) += 1;
        }
        let mut v: Vec<(u16, usize)> = counts.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    /// Paint a `w` x `h` rectangle with its top-left corner at (x, y) with one
    /// tile id. Checks the bounds and the id first and changes nothing when
    /// they fail. Returns how many tiles changed value.
    pub fn paint(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        tile: u32,
    ) -> Result<usize, EditError> {
        self.paint_layer(0, x, y, w, h, Some(tile))
    }

    /// Paint a rectangle of `layer` (0 or 1) with one tile id, or empty it
    /// (`None`, layer 1 only). The same checks as `paint`. Returns how many
    /// cells of that layer changed.
    pub fn paint_layer(
        &mut self,
        layer: usize,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        tile: Option<u32>,
    ) -> Result<usize, EditError> {
        if layer >= LAYERS {
            return Err(EditError::Bounds(format!(
                "layer {layer} is not editable; the editor edits layers 0 and 1"
            )));
        }
        if layer == 0 && tile.is_none() {
            return Err(EditError::Bounds(
                "layer 0 has no empty cells; paint a tile instead".into(),
            ));
        }
        if w == 0 || h == 0 {
            return Err(EditError::Bounds(format!(
                "the rectangle is empty ({w} x {h}); width and height must be at least 1"
            )));
        }
        if x >= SIZE || y >= SIZE || x + w > SIZE || y + h > SIZE {
            return Err(EditError::Bounds(format!(
                "the rectangle x {x}, y {y}, {w} x {h} leaves the {SIZE} x {SIZE} board (x + width and y + height must be at most {SIZE})"
            )));
        }
        if let Some(t) = tile {
            if t >= TILE_LIMIT as u32 {
                return Err(EditError::TileRange(t));
            }
        }
        let tile = tile.map(|t| t as u16);
        let mut changed = 0;
        for yy in y..y + h {
            for xx in x..x + w {
                let at = yy * SIZE + xx;
                if layer == 0 {
                    let t = tile.expect("checked above");
                    if self.layer0[at] != t {
                        self.layer0[at] = t;
                        changed += 1;
                    }
                } else if self.layer1[at] != tile {
                    self.layer1[at] = tile;
                    changed += 1;
                }
            }
        }
        if changed > 0 {
            self.dirty = true;
            self.compose();
        }
        Ok(changed)
    }

    /// Flood-fill the 4-connected region of layer 0 that shares the tile at
    /// (x, y) with one tile id. Same checks as `paint`. Returns how many
    /// tiles changed value.
    pub fn flood_fill(&mut self, x: usize, y: usize, tile: u32) -> Result<usize, EditError> {
        self.flood_fill_layer(0, x, y, Some(tile))
    }

    /// Flood-fill the 4-connected region of `layer` (0 or 1) that matches the
    /// cell at (x, y) with one tile id, or empty it (`None`, layer 1 only).
    /// Neighbours share a side; diagonals do not join. Filling with the value
    /// already at the seed changes nothing and leaves the file as it was.
    /// Returns how many cells of that layer changed.
    pub fn flood_fill_layer(
        &mut self,
        layer: usize,
        x: usize,
        y: usize,
        tile: Option<u32>,
    ) -> Result<usize, EditError> {
        if layer >= LAYERS {
            return Err(EditError::Bounds(format!(
                "layer {layer} is not editable; the editor edits layers 0 and 1"
            )));
        }
        if layer == 0 && tile.is_none() {
            return Err(EditError::Bounds(
                "layer 0 has no empty cells; fill with a tile instead".into(),
            ));
        }
        if x >= SIZE || y >= SIZE {
            return Err(EditError::Bounds(format!(
                "the cell ({x}, {y}) is outside the {SIZE} x {SIZE} board"
            )));
        }
        if let Some(t) = tile {
            if t >= TILE_LIMIT as u32 {
                return Err(EditError::TileRange(t));
            }
        }
        let fill = tile.map(|t| t as u16);
        let seed = match layer {
            0 => Some(self.layer0[y * SIZE + x]),
            _ => self.layer1[y * SIZE + x],
        };
        if seed == fill {
            return Ok(0);
        }
        let mut stack = vec![(x, y)];
        let mut changed = 0;
        while let Some((cx, cy)) = stack.pop() {
            let at = cy * SIZE + cx;
            let here = match layer {
                0 => Some(self.layer0[at]),
                _ => self.layer1[at],
            };
            if here != seed {
                continue;
            }
            if layer == 0 {
                self.layer0[at] = fill.expect("checked above");
            } else {
                self.layer1[at] = fill;
            }
            changed += 1;
            if cx > 0 {
                stack.push((cx - 1, cy));
            }
            if cx + 1 < SIZE {
                stack.push((cx + 1, cy));
            }
            if cy > 0 {
                stack.push((cx, cy - 1));
            }
            if cy + 1 < SIZE {
                stack.push((cx, cy + 1));
            }
        }
        if changed > 0 {
            self.dirty = true;
            self.compose();
        }
        Ok(changed)
    }

    /// Copy a rectangle of `layer` (0 or 1) into a block. Empty layer-1 cells
    /// are `None`; layer 0 is always filled.
    pub fn read_layer_block(&self, layer: usize, rect: TileRect) -> Result<TileBlock, EditError> {
        if layer >= LAYERS {
            return Err(EditError::Bounds(format!(
                "layer {layer} is not editable; the editor edits layers 0 and 1"
            )));
        }
        let mut tiles = Vec::with_capacity(rect.w * rect.h);
        for yy in rect.y..rect.y + rect.h {
            for xx in rect.x..rect.x + rect.w {
                tiles.push(self.layer_tile(layer, xx, yy));
            }
        }
        Ok(TileBlock {
            w: rect.w,
            h: rect.h,
            tiles,
        })
    }

    /// Write a block onto `layer` with its top-left at (x, y). Cells that fall
    /// off the board are left out. On layer 1, `None` clears the cell; on
    /// layer 0, `None` is skipped. Returns how many cells changed.
    pub fn write_layer_block(
        &mut self,
        layer: usize,
        x: usize,
        y: usize,
        block: &TileBlock,
    ) -> Result<usize, EditError> {
        if layer >= LAYERS {
            return Err(EditError::Bounds(format!(
                "layer {layer} is not editable; the editor edits layers 0 and 1"
            )));
        }
        if block.tiles.len() != block.w * block.h {
            return Err(EditError::Bounds(format!(
                "tile block is {} cells but size is {} x {}",
                block.tiles.len(),
                block.w,
                block.h
            )));
        }
        let mut changed = 0;
        for sy in 0..block.h {
            for sx in 0..block.w {
                let (bx, by) = (x + sx, y + sy);
                if bx >= SIZE || by >= SIZE {
                    continue;
                }
                let tile = block.at(sx, sy);
                if layer == 0 {
                    let Some(t) = tile else { continue };
                    changed += self.paint_layer(0, bx, by, 1, 1, Some(t as u32))?;
                } else {
                    changed += self.paint_layer(1, bx, by, 1, 1, tile.map(|t| t as u32))?;
                }
            }
        }
        Ok(changed)
    }

    /// Clear a rectangle of `layer`: tile 0 on layer 0, empty on layer 1.
    pub fn clear_layer_rect(&mut self, layer: usize, rect: TileRect) -> Result<usize, EditError> {
        match layer {
            0 => self.paint_layer(0, rect.x, rect.y, rect.w, rect.h, Some(0)),
            1 => self.paint_layer(1, rect.x, rect.y, rect.w, rect.h, None),
            _ => Err(EditError::Bounds(format!(
                "layer {layer} is not editable; the editor edits layers 0 and 1"
            ))),
        }
    }
    /// The 2.22 view of the board as the editor will write it: layer 0, then
    /// the layer 1 runs, then the rows of other layers (all placed, in that order).
    fn compose(&mut self) {
        self.board.copy_from_slice(&self.layer0);
        for (b, t) in self.board.iter_mut().zip(&self.layer1) {
            if let Some(t) = t {
                *b = *t;
            }
        }
        if self.is_binary() {
            if let Some(level) = &self.binary_level {
                for run in level.boards.iter().filter(|r| r.layer > 1) {
                    if !(0..64).contains(&run.y) {
                        continue;
                    }
                    for (i, t) in run.tiles.iter().enumerate() {
                        let x = run.x + i as i32;
                        if !(0..64).contains(&x) {
                            continue;
                        }
                        let id = seihai_core::assets::tile_id(*t);
                        if id != 4095 {
                            self.board[run.y as usize * SIZE + x as usize] = id;
                        }
                    }
                }
            }
            return;
        }
        for raw in &self.other_rows {
            let toks: Vec<&[u8]> = raw
                .split(|&b| b == b' ')
                .filter(|t| !t.is_empty())
                .collect();
            let (Some(x), Some(y), Some(w)) = (
                toks.get(1).and_then(|t| parse_num(t)),
                toks.get(2).and_then(|t| parse_num(t)),
                toks.get(3).and_then(|t| parse_num(t)),
            ) else {
                continue;
            };
            let data = toks.get(5).copied().unwrap_or(b"");
            for k in 0..w as usize {
                let (Some(hi), Some(lo)) = (
                    data.get(2 * k).and_then(|&c| b64_value(c)),
                    data.get(2 * k + 1).and_then(|&c| b64_value(c)),
                ) else {
                    break;
                };
                self.board[y as usize * SIZE + x as usize + k] = (hi << 6) | lo;
            }
        }
    }

    fn encode_text(&self) -> Vec<u8> {
        let mut canonical = Vec::new();
        for y in 0..SIZE {
            canonical.extend_from_slice(format!("BOARD 0 {y} {SIZE} 0 ").as_bytes());
            for x in 0..SIZE {
                canonical.extend_from_slice(&tile_code(self.layer0[y * SIZE + x]));
            }
            canonical.extend_from_slice(self.eol);
        }
        for y in 0..SIZE {
            let mut x = 0;
            while x < SIZE {
                if self.layer1[y * SIZE + x].is_none() {
                    x += 1;
                    continue;
                }
                let start = x;
                let mut data = Vec::new();
                while x < SIZE {
                    let Some(t) = self.layer1[y * SIZE + x] else {
                        break;
                    };
                    data.extend_from_slice(&tile_code(t));
                    x += 1;
                }
                canonical
                    .extend_from_slice(format!("BOARD {start} {y} {} 1 ", x - start).as_bytes());
                canonical.extend_from_slice(&data);
                canonical.extend_from_slice(self.eol);
            }
        }
        for raw in &self.other_rows {
            canonical.extend_from_slice(raw);
            canonical.extend_from_slice(self.eol);
        }
        let first = self.board_lines.first().copied();
        let mut out = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if self.board_lines.contains(&i) {
                if Some(i) == first {
                    out.extend_from_slice(&canonical);
                }
                continue;
            }
            out.extend_from_slice(line);
            if first.is_none() && i == 0 {
                if !line.ends_with(b"\n") {
                    out.extend_from_slice(self.eol);
                }
                out.extend_from_slice(&canonical);
            }
        }
        out
    }

    /// Bytes for this document's open profile. Unchanged files return the
    /// original bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes_for_profile(self.profile)
    }

    /// Bytes for a destination path: extension picks text / Graal / Zelda.
    /// An unchanged level written back to the same profile stays byte identical.
    pub fn bytes_for_path(&self, path: &Path) -> Vec<u8> {
        self.bytes_for_profile(LevelProfile::from_path(path))
    }

    fn bytes_for_profile(&self, dest: LevelProfile) -> Vec<u8> {
        let unchanged = !self.dirty && !self.edited;
        if unchanged && dest == self.profile {
            return if self.profile == LevelProfile::Text {
                self.lines.concat()
            } else {
                self.raw.clone()
            };
        }
        match dest {
            LevelProfile::Text => {
                if self.profile == LevelProfile::Text {
                    if self.dirty {
                        self.encode_text()
                    } else {
                        self.lines.concat()
                    }
                } else {
                    // Binary → text: rebuild as GLEVNW01 from the editor boards.
                    self.encode_text_from_boards()
                }
            }
            LevelProfile::Graal | LevelProfile::Zelda => {
                let format = dest.binary_format().expect("binary profile");
                binary::encode_binary_doc(self, format)
            }
        }
    }

    /// Minimal text rewrite when converting a binary level to `.nw`.
    fn encode_text_from_boards(&self) -> Vec<u8> {
        let mut out = b"GLEVNW01\n".to_vec();
        for y in 0..SIZE {
            out.extend_from_slice(format!("BOARD 0 {y} {SIZE} 0 ").as_bytes());
            for x in 0..SIZE {
                out.extend_from_slice(&tile_code(self.layer0[y * SIZE + x]));
            }
            out.push(b'\n');
        }
        for y in 0..SIZE {
            let mut x = 0;
            while x < SIZE {
                if self.layer1[y * SIZE + x].is_none() {
                    x += 1;
                    continue;
                }
                let start = x;
                let mut data = Vec::new();
                while x < SIZE {
                    let Some(t) = self.layer1[y * SIZE + x] else {
                        break;
                    };
                    data.extend_from_slice(&tile_code(t));
                    x += 1;
                }
                out.extend_from_slice(format!("BOARD {start} {y} {} 1 ", x - start).as_bytes());
                out.extend_from_slice(&data);
                out.push(b'\n');
            }
        }
        out
    }

    /// Write the level to `path` through a temporary file in the same folder
    /// and a rename, so a crash cannot leave a half-written level (2.22
    /// deleted the old file first; its successors fixed that). The path
    /// extension picks the write profile.
    pub fn save(&self, path: &Path) -> Result<(), EditError> {
        write_atomic(path, &self.bytes_for_path(path))
    }
}

/// Write `bytes` to `path` via a temporary sibling file and a rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EditError> {
    let io = |e: std::io::Error| EditError::Io(format!("cannot write {}: {e}", path.display()));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "level".into());
    let tmp = path.with_file_name(format!(".{name}.tmp{}", std::process::id()));
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(io(e));
    }
    Ok(())
}
