//! The Seihai level editor: open a `GLEVNW01` text level, look at its tiles,
//! paint a rectangle, save. The window (`gui`) is a tile palette and
//! click-to-paint on top of the same `Document`.
//!
//! The rules come from the 2.22 editor notes (see `documentation/level-editor.md`):
//! the board is one 64 x 64 grid of 12-bit tile ids; a `BOARD` row is placed by
//! `x`, `y` and `w` and its layer field is not read; rows that do not fit are
//! skipped without an error; a character outside the base64 alphabet ends a
//! row.
//!
//! **Round trip.** The file is kept as its original lines. A level that was
//! not changed is written back byte for byte. After a change, the `BOARD`
//! rows are replaced by 64 canonical rows (`BOARD 0 y 64 0 <128 characters>`,
//! the 2.22 writer's shape) at the place of the first old `BOARD` row, and
//! every other line (links, signs, chests, baddies, NPC scripts, `HEIGHTS`,
//! comments) is written back as it was.
//!
//! **Layers.** The 2.22 reader does not read a row's layer field, so `tile`
//! and `row` give that view: every row on one board, in file order. The
//! editor also keeps layer 0 and layer 1 apart (`layer_tile`,
//! `paint_layer`), the way Gonstruct does: layer 0 starts as tile 0
//! everywhere, layer 1 starts empty. After a change, layer 0 is written as
//! the 64 canonical rows and layer 1 as one `BOARD x y w 1 data` row per run
//! of filled cells; rows of other layers are written back as they were.
//!
//! **Objects.** Links, signs, chests, baddies and NPCs are records of lines
//! (`objects`), read with the client's level parser. Moving one, or changing
//! a link's destination and arrival or the first line of an NPC script or a
//! sign, rewrites only that record's lines (`edit_object`).

pub mod gui;

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

pub mod objects;
pub use objects::{Object, ObjectEdit, ObjectKind, ScriptHead, SCRIPT_PANE_LINES};

/// Board edge in tiles.
pub const SIZE: usize = 64;
/// Tile ids the text format can hold (two base64 characters).
pub const TILE_LIMIT: u16 = 4096;
/// Layers the editor shows and edits (0 and 1).
pub const LAYERS: usize = 2;

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
    /// Not a text level.
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
            EditError::Io(m) | EditError::NotText(m) | EditError::Bounds(m) | EditError::Version(m) => {
                f.write_str(m)
            }
            EditError::TileRange(id) => write!(
                f,
                "tile id {id} is out of range: text levels hold ids 0 to {} (two base64 characters)",
                TILE_LIMIT - 1
            ),
        }
    }
}

impl std::error::Error for EditError {}

/// The editor version being imitated. 2.22 is the reference; the others are
/// deltas that are not implemented unless they change nothing the editor does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// The 2.22 client's built-in editor.
    V222,
    /// 2.17.1: no difference from 2.22 was found in the notes, so it is the same profile.
    V217,
}

impl Version {
    /// Versions the editor accepts, in the order they are listed.
    pub const SUPPORTED: [Version; 2] = [Version::V222, Version::V217];

    pub fn name(self) -> &'static str {
        match self {
            Version::V222 => "2.22",
            Version::V217 => "2.17.1",
        }
    }

    /// Parse a `--version` value. Anything but 2.22 and 2.17(.1) is a clear "not yet".
    pub fn parse(text: &str) -> Result<Version, EditError> {
        let t = text.trim().trim_start_matches('v');
        match t {
            "2.22" => return Ok(Version::V222),
            "2.17" | "2.17.1" => return Ok(Version::V217),
            _ => {}
        }
        let nums: Vec<u32> = t.split('.').map_while(|p| p.parse().ok()).collect();
        let why = match (nums.first(), nums.get(1)) {
            (Some(2), Some(5)) if t == "2.05" || t == "2.5" => {
                "2.05 differs from 2.22 (no HEIGHTS block, no predefined NPCs, a *.graal-only open filter)"
            }
            (Some(2), Some(31)) | (Some(2), Some(3)) => {
                "2.31 has no built-in editor (the standalone GraalEditor replaced it)"
            }
            (Some(2), Some(m)) if *m < 5 => "this build has no text level format at all",
            (Some(1), _) | (Some(0), _) => "this build has no text level format",
            (Some(2), Some(_)) => "this build is not covered by the editor notes",
            _ => return Err(EditError::Version(format!("unknown version \"{text}\"; supported: 2.22, 2.17.1"))),
        };
        Err(EditError::Version(format!(
            "version {text} is not yet supported: {why}. Supported: 2.22, 2.17.1"
        )))
    }
}

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
    /// The object records, in file order.
    records: Vec<objects::Record>,
    /// Lines were changed by an object edit (the board rows are not rewritten for that).
    edited: bool,
    summary: Summary,
    eol: &'static [u8],
    dirty: bool,
}

fn strip_eol(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn parse_num(tok: &[u8]) -> Option<i64> {
    std::str::from_utf8(tok).ok()?.parse().ok()
}

impl Document {
    /// Open a level from its bytes. Only `GLEVNW01` text is read.
    pub fn parse(data: &[u8]) -> Result<Document, EditError> {
        if data.starts_with(b"GR-V1") || data.starts_with(b"Z3-V1") {
            return Err(EditError::NotText(
                "binary levels (GR-V1, Z3-V1) are not yet supported by the editor; only GLEVNW01 text levels (.nw) open".into(),
            ));
        }
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
        let header_ok = lines.first().map(|l| strip_eol(l).trim_ascii() == b"GLEVNW01").unwrap_or(false);
        if !header_ok {
            return Err(EditError::NotText("not a GLEVNW01 text level (the first line must be GLEVNW01)".into()));
        }
        let eol: &'static [u8] = if lines[0].ends_with(b"\r\n") { b"\r\n" } else { b"\n" };

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
        };
        let mut i = 1;
        while i < doc.lines.len() {
            let raw = strip_eol(&doc.lines[i]).to_vec();
            let toks: Vec<&[u8]> = raw.split(|&b| b == b' ').filter(|t| !t.is_empty()).collect();
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
                // A block swallows its lines up to the terminator (or the end of the file).
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
                    doc.records.push(objects::Record { kind, first, last: i.min(doc.lines.len() - 1) });
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
                    doc.records.push(objects::Record { kind: ObjectKind::Link, first: i, last: i });
                }
                b"CHEST" => {
                    doc.summary.chests += 1;
                    doc.records.push(objects::Record { kind: ObjectKind::Chest, first: i, last: i });
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
                break; // an invalid character ends the row; tiles already placed stay
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

    /// Read a level file.
    pub fn open(path: &Path) -> Result<Document, EditError> {
        let data = fs::read(path).map_err(|e| EditError::Io(format!("cannot read {}: {e}", path.display())))?;
        Document::parse(&data)
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
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
    pub fn paint(&mut self, x: usize, y: usize, w: usize, h: usize, tile: u32) -> Result<usize, EditError> {
        self.paint_layer(0, x, y, w, h, Some(tile))
    }

    /// Paint a rectangle of `layer` (0 or 1) with one tile id, or empty it
    /// (`None`, layer 1 only). The same checks as `paint`. Returns how many
    /// cells of that layer changed.
    pub fn paint_layer(&mut self, layer: usize, x: usize, y: usize, w: usize, h: usize, tile: Option<u32>) -> Result<usize, EditError> {
        if layer >= LAYERS {
            return Err(EditError::Bounds(format!("layer {layer} is not editable; the editor edits layers 0 and 1")));
        }
        if layer == 0 && tile.is_none() {
            return Err(EditError::Bounds("layer 0 has no empty cells; paint a tile instead".into()));
        }
        if w == 0 || h == 0 {
            return Err(EditError::Bounds(format!("the rectangle is empty ({w} x {h}); width and height must be at least 1")));
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

    /// The 2.22 view of the board as the editor will write it: layer 0, then
    /// the layer 1 runs, then the rows of other layers (all placed, in that order).
    fn compose(&mut self) {
        self.board.copy_from_slice(&self.layer0);
        for (b, t) in self.board.iter_mut().zip(&self.layer1) {
            if let Some(t) = t {
                *b = *t;
            }
        }
        for raw in &self.other_rows {
            let toks: Vec<&[u8]> = raw.split(|&b| b == b' ').filter(|t| !t.is_empty()).collect();
            let (Some(x), Some(y), Some(w)) = (
                toks.get(1).and_then(|t| parse_num(t)),
                toks.get(2).and_then(|t| parse_num(t)),
                toks.get(3).and_then(|t| parse_num(t)),
            ) else {
                continue;
            };
            let data = toks.get(5).copied().unwrap_or(b"");
            for k in 0..w as usize {
                let (Some(hi), Some(lo)) = (data.get(2 * k).and_then(|&c| b64_value(c)), data.get(2 * k + 1).and_then(|&c| b64_value(c))) else {
                    break;
                };
                self.board[y as usize * SIZE + x as usize + k] = (hi << 6) | lo;
            }
        }
    }

    /// The level as bytes: the original bytes when nothing was painted.
    pub fn to_bytes(&self) -> Vec<u8> {
        if !self.dirty {
            return self.lines.concat();
        }
        let mut canonical = Vec::new();
        for y in 0..SIZE {
            canonical.extend_from_slice(format!("BOARD 0 {y} {SIZE} 0 ").as_bytes());
            for x in 0..SIZE {
                canonical.extend_from_slice(&tile_code(self.layer0[y * SIZE + x]));
            }
            canonical.extend_from_slice(self.eol);
        }
        // layer 1: one row per run of filled cells, empty cells left out (as Gonstruct writes it)
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
                    let Some(t) = self.layer1[y * SIZE + x] else { break };
                    data.extend_from_slice(&tile_code(t));
                    x += 1;
                }
                canonical.extend_from_slice(format!("BOARD {start} {y} {} 1 ", x - start).as_bytes());
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
                // no BOARD row in the file: the board goes right after the header
                if !line.ends_with(b"\n") {
                    out.extend_from_slice(self.eol);
                }
                out.extend_from_slice(&canonical);
            }
        }
        out
    }

    /// Write the level to `path` through a temporary file in the same folder
    /// and a rename, so a crash cannot leave a half-written level (2.22
    /// deleted the old file first; its successors fixed that).
    pub fn save(&self, path: &Path) -> Result<(), EditError> {
        write_atomic(path, &self.to_bytes())
    }
}

/// Write `bytes` to `path` via a temporary sibling file and a rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EditError> {
    let io = |e: std::io::Error| EditError::Io(format!("cannot write {}: {e}", path.display()));
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "level".into());
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
