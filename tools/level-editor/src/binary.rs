//! Binary level open and save (`GR-V1` / `Z3-V1`) through seihai-core.
//!
//! The editor keeps the file's own bytes when nothing was changed. After a
//! paint it rebuilds with [`encode_binary`]: `.graal` is `GR-V1.03` for one
//! tile layer and `GR-V1.05` when there are more; `.zelda` is `Z3-V1.04`.
//! Layers above 1 and every object record stay as the core reader left them.

use crate::{Document, EditError, Summary, SIZE};
use seihai_core::assets::{encode_binary, tile_code, tile_id, BinaryLevel, BoardRun, Level};

/// On-disk empty marker for layers above 0 in `GR-V1.05`.
const UPPER_EMPTY: u16 = 4095;

/// How a level file is stored: text, Graal binary, or Zelda binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelProfile {
    /// `GLEVNW01` text (`.nw`).
    Text,
    /// `GR-V1.*` binary (`.graal` and other non-`.nw` / non-`.zelda` names).
    Graal,
    /// `Z3-V1.*` binary (`.zelda`).
    Zelda,
}

impl LevelProfile {
    /// Pick the write profile from a path's extension.
    pub fn from_path(path: &std::path::Path) -> LevelProfile {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("nw") => LevelProfile::Text,
            Some("zelda") => LevelProfile::Zelda,
            _ => LevelProfile::Graal,
        }
    }

    /// Profile from the file magic, if it is a known level header.
    pub fn from_magic(data: &[u8]) -> Option<LevelProfile> {
        if data.len() < 8 {
            return None;
        }
        let magic = &data[..8];
        if magic == b"GLEVNW01" {
            Some(LevelProfile::Text)
        } else if magic.starts_with(b"Z3-V1") {
            Some(LevelProfile::Zelda)
        } else if magic.starts_with(b"GR-V1") {
            Some(LevelProfile::Graal)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LevelProfile::Text => "GLEVNW01 text",
            LevelProfile::Graal => "GR-V1 binary",
            LevelProfile::Zelda => "Z3-V1 binary",
        }
    }

    pub(crate) fn binary_format(self) -> Option<BinaryLevel> {
        match self {
            LevelProfile::Text => None,
            LevelProfile::Graal => Some(BinaryLevel::Graal),
            LevelProfile::Zelda => Some(BinaryLevel::Zelda),
        }
    }
}

pub(crate) fn parse_binary(data: &[u8]) -> Result<Document, EditError> {
    let profile = LevelProfile::from_magic(data)
        .ok_or_else(|| EditError::NotText("not a GR-V1 or Z3-V1 binary level".into()))?;
    let level = Level::from_bytes(data)
        .map_err(|e| EditError::NotText(format!("cannot read binary level: {e}")))?;

    let layer0 = level.layer0_board();
    let mut layer1 = vec![None; SIZE * SIZE];
    let mut layer1_filled = 0usize;
    let mut max_other = 0i32;
    for run in &level.boards {
        if run.layer == 1 {
            if !(0..64).contains(&run.y) {
                continue;
            }
            for (i, t) in run.tiles.iter().enumerate() {
                let x = run.x + i as i32;
                if !(0..64).contains(&x) {
                    continue;
                }
                let id = tile_id(*t);
                let at = run.y as usize * SIZE + x as usize;
                if id == UPPER_EMPTY {
                    layer1[at] = None;
                } else {
                    layer1[at] = Some(id);
                    layer1_filled += 1;
                }
            }
        } else if run.layer > 1 {
            max_other = max_other.max(run.layer);
        }
    }

    let mut board = layer0.clone();
    for (b, t) in board.iter_mut().zip(&layer1) {
        if let Some(t) = t {
            *b = *t;
        }
    }
    for run in level.boards.iter().filter(|r| r.layer > 1) {
        if !(0..64).contains(&run.y) {
            continue;
        }
        for (i, t) in run.tiles.iter().enumerate() {
            let x = run.x + i as i32;
            if !(0..64).contains(&x) {
                continue;
            }
            let id = tile_id(*t);
            if id != UPPER_EMPTY {
                board[run.y as usize * SIZE + x as usize] = id;
            }
        }
    }

    let rows_other_layers_kept = if max_other > 1 {
        max_other as usize - 1
    } else {
        0
    };

    let summary = Summary {
        rows_placed: SIZE,
        rows_skipped: 0,
        rows_other_layer: usize::from(layer1_filled > 0 || max_other > 1),
        links: level.links.len(),
        signs: level.signs.len(),
        chests: level.chests.len(),
        baddies: level.baddies.len(),
        npcs: level.npcs.len(),
        heights: false,
        rows_other_layers_kept,
    };

    Ok(Document::from_binary(
        data.to_vec(),
        profile,
        level,
        board,
        layer0,
        layer1,
        summary,
    ))
}

pub(crate) fn encode_binary_doc(doc: &Document, format: BinaryLevel) -> Vec<u8> {
    let mut level = doc.binary_level_for_write();
    level.boards.retain(|r| r.layer >= 2);

    for y in 0..SIZE {
        let tiles: Vec<_> = (0..SIZE)
            .map(|x| tile_code(doc.layer0_tiles()[y * SIZE + x]))
            .collect();
        level.boards.push(BoardRun {
            x: 0,
            y: y as i32,
            width: SIZE as i32,
            layer: 0,
            tiles,
        });
    }

    if doc.layer1_count() > 0 {
        for y in 0..SIZE {
            let tiles: Vec<_> = (0..SIZE)
                .map(|x| tile_code(doc.layer1_tiles()[y * SIZE + x].unwrap_or(UPPER_EMPTY)))
                .collect();
            level.boards.push(BoardRun {
                x: 0,
                y: y as i32,
                width: SIZE as i32,
                layer: 1,
                tiles,
            });
        }
    }

    encode_binary(&level, format).bytes
}

fn empty_level() -> Level {
    Level {
        boards: Vec::new(),
        npcs: Vec::new(),
        links: Vec::new(),
        signs: Vec::new(),
        baddies: Vec::new(),
        chests: Vec::new(),
    }
}

impl Document {
    pub(crate) fn binary_level_for_write(&self) -> Level {
        if let Some(level) = self.binary_level() {
            return level.clone();
        }
        let bytes = self.text_snapshot_bytes();
        Level::from_bytes(&bytes).unwrap_or_else(|_| empty_level())
    }
}
