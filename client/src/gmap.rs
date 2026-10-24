//! GMAP overworlds: the grid of level names, positions on it, and the
//! edge crossings that move a player from one cell to the next.
//!
//! A gmap is a grid of ordinary 64×64-tile levels. A player is in one cell
//! and has a position local to that cell; the real client keeps the global
//! position (`local + 64 * cell`) on the side and, when the player's x or y
//! leaves `0..64`, changes the cell prop (`GMAPLEVELX` 43 or `GMAPLEVELY`
//! 44) and wraps the coordinate, all in one props message (lab, real 2.22
//! and 2.31 clients, 2026-10-24; Graal_analysis feature spec 06).
//!
//! This module has no I/O. The session side (`SessionState`) feeds it the
//! server's 49/6/186 messages; the walking side (`OfflineSession`) asks it
//! whether a step crosses an edge.
//!
//! Everything version-dependent comes from [`GmapRules`], which is read from
//! the version profile, never from the version number.

use crate::protocol::{
    encode_pos_classic, encode_pos_precise, ClientVersion, IdSet, TamperSender,
};
use std::collections::HashMap;

/// Tiles along one side of a level, and so of a gmap cell.
pub const CELL_TILES: i32 = 64;

/// `PLI_PLAYERPROPS` (2).
const PLI_PLAYERPROPS: u8 = 2;
/// `PLI_ADJACENTLEVEL` (35).
pub const PLI_ADJACENTLEVEL: u8 = 35;
/// Player props used at a crossing.
const PROP_X: u8 = 15;
const PROP_Y: u8 = 16;
const PROP_GMAPLEVELX: u8 = 43;
const PROP_GMAPLEVELY: u8 = 44;
const PROP_X2: u8 = 78;
const PROP_Y2: u8 = 79;

/// What a version profile says about gmaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmapRules {
    /// The build understands gmaps: its profile has `.gmap` levels,
    /// `PLO_PLAYERWARP2` (49) and the cell props 43 and 44 in the own-player
    /// prop set. A client without it never enters a gmap and sends no cell
    /// prop.
    pub enabled: bool,
    /// Positions go as X2/Y2 (78/79) instead of X/Y (15/16).
    pub precise_pos: bool,
    /// The build can send `PLI_ADJACENTLEVEL` (35).
    pub adjacent: bool,
}

impl GmapRules {
    /// Read the rules from `version`'s profile.
    pub fn for_version(version: ClientVersion) -> Self {
        let p = version.profile();
        let has = |s: &IdSet, id: u8| s.contains(id);
        Self {
            enabled: p.levels.gmap
                && has(&p.protocol.plo, 49)
                && has(&p.props.own, PROP_GMAPLEVELX)
                && has(&p.props.own, PROP_GMAPLEVELY),
            precise_pos: p.props.precise_pos,
            adjacent: has(&p.protocol.pli, PLI_ADJACENTLEVEL),
        }
    }
}

/// What the server sent for one level while the player was elsewhere: the
/// per-level lists the session keeps for the level it is currently filling.
#[derive(Debug, Default, Clone)]
pub struct LevelSlot {
    pub board: Option<Vec<u16>>,
    pub board_lines: Vec<String>,
    pub links: Vec<String>,
    pub signs: Vec<String>,
    pub chests: Vec<String>,
}

/// The session's view of the gmap the player is on, built from the server's
/// messages: `PLO_PLAYERWARP2` (49) names the gmap and the cell, the `.gmap`
/// file gives the grid, and level data arrives per level (the player's own
/// and the neighbours the client asked for with 35), each introduced by a
/// `PLO_LEVELNAME` (6) and closed by a 6 that names the gmap again.
#[derive(Debug, Default)]
pub struct GmapSession {
    /// The gmap file name from 49.
    pub name: String,
    /// The player's cell.
    pub cell: (i32, i32),
    /// The grid, once the file is known.
    pub map: Option<Gmap>,
    /// 64×64 tile ids of every level the server sent, by lower-case name.
    pub boards: HashMap<String, Vec<u16>>,
    /// The level the first 6 after a 49 named: the player's level until
    /// the grid says which cell is which.
    pub level_hint: Option<String>,
    /// A 49 arrived and its level name has not.
    pub awaiting_level: bool,
    /// Level data parked while another level is the one being filled.
    pub slots: HashMap<String, LevelSlot>,
}

impl GmapSession {
    pub fn new(name: String, cell: (i32, i32)) -> Self {
        Self { name, cell, awaiting_level: true, ..Default::default() }
    }

    /// Whether `name` is this gmap's file name (a 6 that names it means
    /// "the player's level again", not a level).
    pub fn is_name(&self, name: &str) -> bool {
        name.eq_ignore_ascii_case(&self.name)
    }

    /// The level at `cell` per the grid.
    pub fn level_at(&self, cell: (i32, i32)) -> Option<String> {
        self.map.as_ref()?.level_at(cell.0, cell.1).map(str::to_string)
    }

    /// The player's level: the grid's, else the one the server named first.
    pub fn current_level(&self) -> Option<String> {
        self.level_at(self.cell).or_else(|| self.level_hint.clone())
    }
}

/// A parsed `.gmap` file (`GRMAP001`), the reader keys only: the keys a
/// client needs to place levels. Generator keys (`GENERATED`, `HEIGHTMAP`,
/// ...) are skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gmap {
    /// Levels across and down.
    pub width: usize,
    pub height: usize,
    /// Level names, row by row, `width * height` of them. An empty name is
    /// a hole.
    pub names: Vec<String>,
    /// `MAPIMG` and `MINIMAPIMG`: overview image file names.
    pub map_img: Option<String>,
    pub minimap_img: Option<String>,
    /// `NOAUTOMAPPING` and `LOADFULLMAP` flags.
    pub no_auto_mapping: bool,
    pub load_full_map: bool,
    /// `LOADATSTART` names.
    pub load_at_start: Vec<String>,
}

/// Split one `LEVELNAMES` row: comma-separated, each name optionally in
/// double quotes. A trailing comma does not add a cell; `""` is a hole.
fn split_row(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_suffix(',').unwrap_or(line);
    line.split(',')
        .map(|f| f.trim().trim_matches('"').trim().to_string())
        .collect()
}

impl Gmap {
    /// Parse `GRMAP001` text. `None` if the signature line is missing or the
    /// grid has no cells.
    pub fn parse(text: &str) -> Option<Self> {
        let mut lines = text.lines().map(|l| l.trim_end_matches('\r'));
        if lines.by_ref().find(|l| !l.trim().is_empty())?.trim() != "GRMAP001" {
            return None;
        }
        let mut g = Gmap::default();
        let (mut declared_w, mut declared_h) = (None, None);
        let mut rows: Vec<Vec<String>> = Vec::new();
        while let Some(line) = lines.next() {
            let mut words = line.split_whitespace();
            let Some(key) = words.next() else { continue };
            match key {
                "WIDTH" => declared_w = words.next().and_then(|v| v.parse::<usize>().ok()),
                "HEIGHT" => declared_h = words.next().and_then(|v| v.parse::<usize>().ok()),
                "MAPIMG" => g.map_img = words.next().map(str::to_string),
                "MINIMAPIMG" => g.minimap_img = words.next().map(str::to_string),
                "NOAUTOMAPPING" => g.no_auto_mapping = true,
                "LOADFULLMAP" => g.load_full_map = true,
                "LEVELNAMES" => {
                    for row in lines.by_ref() {
                        if row.trim() == "LEVELNAMESEND" {
                            break;
                        }
                        if !row.trim().is_empty() {
                            rows.push(split_row(row));
                        }
                    }
                }
                "LOADATSTART" => {
                    for row in lines.by_ref() {
                        if row.trim() == "LOADATSTARTEND" {
                            break;
                        }
                        g.load_at_start.extend(split_row(row).into_iter().filter(|n| !n.is_empty()));
                    }
                }
                _ => {}
            }
        }
        // The declared size wins; without one, the rows decide.
        g.width = declared_w.unwrap_or_else(|| rows.iter().map(Vec::len).max().unwrap_or(0));
        g.height = declared_h.unwrap_or(rows.len());
        if g.width == 0 || g.height == 0 || g.width > 255 || g.height > 255 {
            return None;
        }
        g.names = vec![String::new(); g.width * g.height];
        for (y, row) in rows.iter().take(g.height).enumerate() {
            for (x, name) in row.iter().take(g.width).enumerate() {
                g.names[y * g.width + x] = name.clone();
            }
        }
        Some(g)
    }

    /// The level at cell `(cx, cy)`, or `None` outside the grid or on a hole.
    pub fn level_at(&self, cx: i32, cy: i32) -> Option<&str> {
        if cx < 0 || cy < 0 || cx as usize >= self.width || cy as usize >= self.height {
            return None;
        }
        let n = self.names[cy as usize * self.width + cx as usize].as_str();
        (!n.is_empty()).then_some(n)
    }

    /// The cell a level is in (case-insensitive; the first match).
    pub fn cell_of(&self, level: &str) -> Option<(i32, i32)> {
        let i = self.names.iter().position(|n| !n.is_empty() && n.eq_ignore_ascii_case(level))?;
        Some(((i % self.width) as i32, (i / self.width) as i32))
    }

    /// The cells around `(cx, cy)` (up to eight) that hold a level.
    pub fn neighbours(&self, (cx, cy): (i32, i32)) -> Vec<(i32, i32)> {
        let mut v = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && self.level_at(cx + dx, cy + dy).is_some() {
                    v.push((cx + dx, cy + dy));
                }
            }
        }
        v
    }

    /// Where a position that may have left its cell really is. `local` is
    /// the position in tiles relative to `cell`, and may be outside
    /// `0..64` after a step.
    pub fn resolve(&self, cell: (i32, i32), local: (f32, f32)) -> Resolved {
        let (nx, ny, lx, ly) = wrap(cell, local);
        let to = (nx, ny);
        if to == cell {
            return Resolved::Inside;
        }
        if self.level_at(nx, ny).is_none() {
            return Resolved::NoLevel;
        }
        Resolved::Crossed(Crossing { from: cell, to, local_x: lx, local_y: ly })
    }
}

/// Wrap a cell-relative position into its own cell.
fn wrap(cell: (i32, i32), (x, y): (f32, f32)) -> (i32, i32, f32, f32) {
    let side = CELL_TILES as f32;
    let (kx, ky) = ((x / side).floor(), (y / side).floor());
    (cell.0 + kx as i32, cell.1 + ky as i32, x - kx * side, y - ky * side)
}

/// The global position (tiles) of a cell-local one.
pub fn to_global(cell: (i32, i32), (x, y): (f32, f32)) -> (f32, f32) {
    (x + (cell.0 * CELL_TILES) as f32, y + (cell.1 * CELL_TILES) as f32)
}

/// The cell and cell-local position of a global one.
pub fn to_local((gx, gy): (f32, f32)) -> ((i32, i32), (f32, f32)) {
    let (cx, cy, lx, ly) = wrap((0, 0), (gx, gy));
    ((cx, cy), (lx, ly))
}

/// What [`Gmap::resolve`] found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolved {
    /// Still in the same cell.
    Inside,
    /// The position is past the edge of the grid or in a hole: there is
    /// nowhere to go.
    NoLevel,
    /// In another cell.
    Crossed(Crossing),
}

/// A move from one cell to another.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crossing {
    pub from: (i32, i32),
    pub to: (i32, i32),
    /// The wrapped position in the new cell, tiles.
    pub local_x: f32,
    pub local_y: f32,
}

impl Crossing {
    /// The cell x changed (prop 43 goes out).
    pub fn x_changed(&self) -> bool {
        self.from.0 != self.to.0
    }

    /// The cell y changed (prop 44 goes out).
    pub fn y_changed(&self) -> bool {
        self.from.1 != self.to.1
    }
}

fn gshort(v: u16) -> [u8; 2] {
    [((v >> 7) as u8 & 0x7F) + 0x20, (v as u8 & 0x7F) + 0x20]
}

/// The props message a client sends at a crossing: `PLI_PLAYERPROPS` (2)
/// with, for each axis that changed, the cell prop (43 for x, 44 for y)
/// followed by the wrapped coordinate of that axis: X2/Y2 (78/79) when the
/// profile uses precise positions, X/Y (15/16) otherwise.
///
/// Real 2.31 messages, from the lab: `43 = 1, X2 = 0` going east, `43 = 0,
/// X2 = 63.5` going west, and the same with 44 and Y2 for south and north.
/// Nothing else (no direction, no other axis) is in the message. A step
/// that changes both axes at once was never seen; this writes the pairs
/// one after the other.
pub fn crossing_props_message(rules: &GmapRules, c: &Crossing) -> Vec<u8> {
    let mut m = vec![PLI_PLAYERPROPS + 0x20];
    let coord = |m: &mut Vec<u8>, classic: u8, precise: u8, v: f32| {
        if rules.precise_pos {
            m.push(precise + 0x20);
            m.extend_from_slice(&gshort(encode_pos_precise(v)));
        } else {
            m.push(classic + 0x20);
            m.push(encode_pos_classic(v).wrapping_add(0x20));
        }
    };
    if c.x_changed() {
        m.push(PROP_GMAPLEVELX + 0x20);
        m.push(c.to.0 as u8 + 0x20);
        coord(&mut m, PROP_X, PROP_X2, c.local_x);
    }
    if c.y_changed() {
        m.push(PROP_GMAPLEVELY + 0x20);
        m.push(c.to.1 as u8 + 0x20);
        coord(&mut m, PROP_Y, PROP_Y2, c.local_y);
    }
    m.push(b'\n');
    m
}

/// The messages of the frame a client sends the moment it crosses an edge
/// (all before the server's answer, lab 2026-10-24):
///
/// 1. `PLI_ENTERLEVEL` (46) when the build sends it (2.3, 2.31);
/// 2. the tamper check (39) when `tamper` is given and the build re-sends it
///    on level changes (2.22);
/// 3. the props message ([`crossing_props_message`]).
///
/// An empty list for a profile without gmaps.
pub fn crossing_frame(
    version: ClientVersion,
    c: &Crossing,
    tamper: Option<&mut TamperSender>,
) -> Vec<Vec<u8>> {
    let rules = GmapRules::for_version(version);
    if !rules.enabled {
        return Vec::new();
    }
    let mut msgs = Vec::new();
    if version.sends_enter_level() {
        msgs.push(crate::net::enter_level_message());
    }
    if let Some(t) = tamper {
        if t.level_changed() {
            msgs.extend(t.message());
        }
    }
    msgs.push(crossing_props_message(&rules, c));
    msgs
}

/// `PLI_ADJACENTLEVEL` (35): `GINT5` mod time of the cached copy (0 when
/// there is none) and the level name. The server answers with that level's
/// data, then names the gmap again.
pub fn adjacent_level_message(mod_time: u32, level: &str) -> Vec<u8> {
    let mut m = vec![PLI_ADJACENTLEVEL + 0x20];
    for shift in [28, 21, 14, 7, 0] {
        m.push(((mod_time >> shift) as u8 & 0x7F) + 0x20);
    }
    m.extend(level.chars().map(|c| if (c as u32) < 256 { c as u8 } else { b'?' }));
    m.push(b'\n');
    m
}

/// The `PLI_ADJACENTLEVEL` messages for the cells around `cell`: one per
/// neighbouring level that has no entry in `have` (lower-case names), mod
/// time 0 as on a cold cache. Empty when the profile cannot send 35.
pub fn adjacent_requests(
    rules: &GmapRules,
    map: &Gmap,
    cell: (i32, i32),
    have: &HashMap<String, u32>,
) -> Vec<Vec<u8>> {
    if !rules.enabled || !rules.adjacent {
        return Vec::new();
    }
    map.neighbours(cell)
        .into_iter()
        .filter_map(|(x, y)| map.level_at(x, y))
        .filter(|n| !have.contains_key(&n.to_ascii_lowercase()))
        .map(|n| adjacent_level_message(0, n))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_BY_TWO: &str = "GRMAP001\nWIDTH 2\nHEIGHT 2\nLEVELNAMES\n\"gm_nw.nw\",\"gm_ne.nw\",\n\"gm_sw.nw\",\"gm_se.nw\",\nLEVELNAMESEND\nMAPIMG over.png\n";

    fn map() -> Gmap {
        Gmap::parse(TWO_BY_TWO).unwrap()
    }

    #[test]
    fn parses_a_two_by_two_file() {
        let g = map();
        assert_eq!((g.width, g.height), (2, 2));
        assert_eq!(g.level_at(0, 0), Some("gm_nw.nw"));
        assert_eq!(g.level_at(1, 0), Some("gm_ne.nw"));
        assert_eq!(g.level_at(0, 1), Some("gm_sw.nw"));
        assert_eq!(g.level_at(1, 1), Some("gm_se.nw"));
        assert_eq!(g.map_img.as_deref(), Some("over.png"));
        assert_eq!(g.level_at(2, 0), None);
        assert_eq!(g.level_at(-1, 0), None);
    }

    #[test]
    fn size_comes_from_the_rows_without_width_and_height() {
        let g = Gmap::parse("GRMAP001\nLEVELNAMES\na.nw,b.nw,c.nw\nd.nw,e.nw,f.nw\nLEVELNAMESEND\n").unwrap();
        assert_eq!((g.width, g.height), (3, 2));
        assert_eq!(g.level_at(2, 1), Some("f.nw"));
    }

    #[test]
    fn holes_short_rows_and_crlf() {
        let t = "GRMAP001\r\nWIDTH 3\r\nHEIGHT 3\r\nLEVELNAMES\r\n\"a.nw\",\"\",\"c.nw\"\r\n\"d.nw\"\r\nLEVELNAMESEND\r\n";
        let g = Gmap::parse(t).unwrap();
        assert_eq!(g.level_at(0, 0), Some("a.nw"));
        assert_eq!(g.level_at(1, 0), None, "a quoted empty name is a hole");
        assert_eq!(g.level_at(2, 0), Some("c.nw"));
        assert_eq!(g.level_at(1, 1), None, "a short row leaves holes");
        assert_eq!(g.level_at(0, 2), None, "a missing row too");
    }

    #[test]
    fn flags_images_and_load_list() {
        let t = "GRMAP001\nWIDTH 1\nHEIGHT 1\nMINIMAPIMG mini.png\nNOAUTOMAPPING\nLOADFULLMAP\nLOADATSTART\n\"a.nw\",\"b.nw\"\nLOADATSTARTEND\nLEVELNAMES\n\"a.nw\"\nLEVELNAMESEND\n";
        let g = Gmap::parse(t).unwrap();
        assert!(g.no_auto_mapping && g.load_full_map);
        assert_eq!(g.minimap_img.as_deref(), Some("mini.png"));
        assert_eq!(g.load_at_start, vec!["a.nw", "b.nw"]);
    }

    #[test]
    fn bad_signature_or_empty_grid_is_none() {
        assert!(Gmap::parse("GLEVNW01\nWIDTH 1\n").is_none());
        assert!(Gmap::parse("GRMAP001\nWIDTH 2\nHEIGHT 2\n").is_some(), "declared size, all holes");
        assert!(Gmap::parse("GRMAP001\n").is_none());
        assert!(Gmap::parse("").is_none());
    }

    #[test]
    fn cell_lookup_is_case_insensitive() {
        let g = map();
        assert_eq!(g.cell_of("GM_SE.NW"), Some((1, 1)));
        assert_eq!(g.cell_of("nope.nw"), None);
        assert_eq!(g.neighbours((0, 0)), vec![(1, 0), (0, 1), (1, 1)]);
        assert_eq!(g.neighbours((1, 1)).len(), 3);
    }

    #[test]
    fn global_and_local_round_trip() {
        assert_eq!(to_global((1, 0), (0.0, 30.5)), (64.0, 30.5));
        assert_eq!(to_local((64.0, 30.5)), ((1, 0), (0.0, 30.5)));
        assert_eq!(to_local((63.5, 94.5)), ((0, 1), (63.5, 30.5)));
        assert_eq!(to_local((-0.5, 0.0)), ((-1, 0), (63.5, 0.0)));
        for cell in [(0, 0), (1, 0), (3, 2)] {
            for local in [(0.0, 0.0), (63.5, 12.0), (30.5, 63.5)] {
                assert_eq!(to_local(to_global(cell, local)), (cell, local));
            }
        }
    }

    #[test]
    fn crossing_wraps_like_the_lab_captures() {
        let g = map();
        // East: global x reaches 64.0.
        assert_eq!(g.resolve((0, 0), (63.5, 30.5)), Resolved::Inside);
        let Resolved::Crossed(c) = g.resolve((0, 0), (64.0, 30.5)) else { panic!() };
        assert_eq!((c.to, c.local_x, c.local_y), ((1, 0), 0.0, 30.5));
        assert!(c.x_changed() && !c.y_changed());
        // West: from cell 1 at local 0.0, one half-step back.
        let Resolved::Crossed(c) = g.resolve((1, 0), (-0.5, 30.5)) else { panic!() };
        assert_eq!((c.to, c.local_x), ((0, 0), 63.5));
        // South and north.
        let Resolved::Crossed(c) = g.resolve((0, 0), (10.0, 64.0)) else { panic!() };
        assert_eq!((c.to, c.local_y), ((0, 1), 0.0));
        assert!(c.y_changed() && !c.x_changed());
        let Resolved::Crossed(c) = g.resolve((0, 1), (10.0, -0.5)) else { panic!() };
        assert_eq!((c.to, c.local_y), ((0, 0), 63.5));
    }

    #[test]
    fn nothing_beyond_the_grid_or_into_a_hole() {
        let g = map();
        assert_eq!(g.resolve((0, 0), (-0.5, 5.0)), Resolved::NoLevel);
        assert_eq!(g.resolve((1, 1), (64.0, 5.0)), Resolved::NoLevel);
        let holey = Gmap::parse("GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"\"\nLEVELNAMESEND\n").unwrap();
        assert_eq!(holey.resolve((0, 0), (64.0, 5.0)), Resolved::NoLevel);
    }

    #[test]
    fn diagonal_crossing_changes_both_axes() {
        let g = map();
        let Resolved::Crossed(c) = g.resolve((0, 0), (64.0, 64.0)) else { panic!() };
        assert_eq!(c.to, (1, 1));
        assert!(c.x_changed() && c.y_changed());
    }

    #[test]
    fn rules_come_from_the_profile() {
        use ClientVersion::*;
        // 2.05 has no 49 handler and no cell props; 2.22 and 2.31 do.
        assert!(!GmapRules::for_version(V205).enabled);
        let r222 = GmapRules::for_version(V222);
        let r231 = GmapRules::for_version(V231);
        assert!(r222.enabled && r231.enabled);
        assert!(!r222.precise_pos && r231.precise_pos);
        assert!(r222.adjacent && r231.adjacent);
        for v in [V138, V139, V200] {
            assert!(!GmapRules::for_version(v).enabled, "{v:?}");
        }
    }

    fn cross(from: (i32, i32), to: (i32, i32), lx: f32, ly: f32) -> Crossing {
        Crossing { from, to, local_x: lx, local_y: ly }
    }

    /// The four props messages a real 2.31 sent in the lab (the id byte is
    /// the PLAYERPROPS id): `43 = 1, X2 = 0`, `43 = 0, X2 = 63.5`,
    /// `44 = 1, Y2 = 0`, `44 = 0, Y2 = 63.5`.
    #[test]
    fn props_message_matches_the_real_231_bytes() {
        let r = GmapRules::for_version(ClientVersion::V231);
        let east = crossing_props_message(&r, &cross((0, 0), (1, 0), 0.0, 30.5));
        assert_eq!(east, [&[0x22, 0x4b, 0x21, 0x6e, 0x20, 0x20][..], b"\n"].concat());
        let west = crossing_props_message(&r, &cross((1, 0), (0, 0), 63.5, 30.5));
        assert_eq!(west, [&[0x22, 0x4b, 0x20, 0x6e, 0x2f, 0x90][..], b"\n"].concat());
        let south = crossing_props_message(&r, &cross((0, 0), (0, 1), 10.0, 0.0));
        assert_eq!(south, [&[0x22, 0x4c, 0x21, 0x6f, 0x20, 0x20][..], b"\n"].concat());
        let north = crossing_props_message(&r, &cross((0, 1), (0, 0), 10.0, 63.5));
        assert_eq!(north, [&[0x22, 0x4c, 0x20, 0x6f, 0x2f, 0x90][..], b"\n"].concat());
    }

    /// A profile without precise positions writes the classic X/Y (x2 as a
    /// GCHAR), the cell prop first.
    #[test]
    fn props_message_uses_classic_coordinates_for_222() {
        let r = GmapRules::for_version(ClientVersion::V222);
        let east = crossing_props_message(&r, &cross((0, 0), (1, 0), 0.0, 30.5));
        assert_eq!(east, [&[0x22, 0x4b, 0x21, 0x2f, 0x20][..], b"\n"].concat());
        let west = crossing_props_message(&r, &cross((1, 0), (0, 0), 63.5, 30.5));
        assert_eq!(west, [&[0x22, 0x4b, 0x20, 0x2f, 127 + 0x20][..], b"\n"].concat());
        let south = crossing_props_message(&r, &cross((0, 0), (0, 1), 10.0, 0.5));
        assert_eq!(south, [&[0x22, 0x4c, 0x21, 0x30, 0x21][..], b"\n"].concat());
    }

    #[test]
    fn diagonal_props_message_writes_the_pairs_in_order() {
        let r = GmapRules::for_version(ClientVersion::V231);
        let m = crossing_props_message(&r, &cross((0, 0), (1, 1), 0.0, 0.0));
        let ids: Vec<u8> = crate::protocol::parse_player_props(&m[1..m.len() - 1]).iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![43, 78, 44, 79]);
    }

    /// The messages the props message parses back to what was crossed.
    #[test]
    fn props_message_round_trips_through_the_prop_parser() {
        use crate::protocol::{decode_pos_precise, parse_player_props_in};
        let v = ClientVersion::V231;
        let r = GmapRules::for_version(v);
        let own = v.profile().props.own;
        for (to, lx, ly) in [((1, 0), 0.0, 30.5), ((0, 1), 12.0, 0.0), ((0, 0), 63.5, 1.5)] {
            let from = if to == (0, 0) { (1, 0) } else { (0, 0) };
            let m = crossing_props_message(&r, &cross(from, to, lx, ly));
            let props = parse_player_props_in(&m[1..m.len() - 1], false, Some(&own));
            assert_eq!(props.len(), 2, "{to:?}");
            let (cell_prop, coord) = (&props[0], &props[1]);
            let cell = cell_prop.gchar().unwrap() as i32;
            assert_eq!(if cell_prop.id == 43 { cell } else { to.0 }, to.0);
            assert_eq!(if cell_prop.id == 44 { cell } else { to.1 }, to.1);
            let raw = crate::protocol::read_gshort(&coord.raw).unwrap();
            let want = if cell_prop.id == 43 { lx } else { ly };
            assert_eq!(decode_pos_precise(raw), want);
        }
    }

    #[test]
    fn crossing_frame_order_follows_the_profile() {
        let c = cross((0, 0), (1, 0), 0.0, 30.5);
        let ids = |f: &[Vec<u8>]| -> Vec<u8> { f.iter().map(|m| m[0] - 0x20).collect() };
        // 2.31: 46 then props. No tamper check at a crossing.
        let mut t231 = TamperSender::for_version(ClientVersion::V231);
        let f = crossing_frame(ClientVersion::V231, &c, t231.as_mut());
        assert_eq!(ids(&f), vec![46, 2]);
        assert_eq!(f[0], crate::net::enter_level_message());
        // 2.22: the tamper check, then props; without the opt-in just props.
        let mut t222 = TamperSender::for_version(ClientVersion::V222);
        let f = crossing_frame(ClientVersion::V222, &c, t222.as_mut());
        assert_eq!(ids(&f), vec![39, 2]);
        let f = crossing_frame(ClientVersion::V222, &c, None);
        assert_eq!(ids(&f), vec![2]);
        // A profile without gmaps sends nothing.
        assert!(crossing_frame(ClientVersion::V205, &c, None).is_empty());
    }

    #[test]
    fn adjacent_level_message_layout() {
        let m = adjacent_level_message(0, "gm_ne.nw");
        assert_eq!(m, [&[0x43, 0x20, 0x20, 0x20, 0x20, 0x20][..], b"gm_ne.nw\n"].concat());
        // A mod time round-trips through the GINT5 reader.
        let m = adjacent_level_message(1_781_284_436, "a.nw");
        let t = m[1..6].iter().fold(0u64, |a, &c| (a << 7) | (c - 0x20) as u64);
        assert_eq!(t, 1_781_284_436);
    }

    #[test]
    fn adjacent_requests_skip_cached_levels_and_follow_the_profile() {
        let g = map();
        let r = GmapRules::for_version(ClientVersion::V222);
        let none = HashMap::new();
        let all = adjacent_requests(&r, &g, (0, 0), &none);
        assert_eq!(all.len(), 3);
        assert!(all[0].ends_with(b"gm_ne.nw\n"));
        let mut have = HashMap::new();
        have.insert("gm_ne.nw".to_string(), 5);
        assert_eq!(adjacent_requests(&r, &g, (0, 0), &have).len(), 2);
        let off = GmapRules { adjacent: false, ..r };
        assert!(adjacent_requests(&off, &g, (0, 0), &none).is_empty());
        let disabled = GmapRules { enabled: false, ..r };
        assert!(adjacent_requests(&disabled, &g, (0, 0), &none).is_empty());
    }
}
