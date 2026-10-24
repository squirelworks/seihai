//! Server to client payload layouts for ids 49, 75, 150 to 189 and the
//! lower ids that went undecoded.
//!
//! Source: Graal_analysis `client-internals/decomp-findings-2026-10-24-b/`
//! (`plo_layouts.tsv`, read from the 2.17.1, 2.22 and 2.31 receive
//! dispatchers) and the 2026-10-24 lab captures. Which versions handle an id
//! is not decided here: [`super::profile::VersionProfile`]'s `plo` set gates
//! it before a payload reaches these parsers.
//!
//! Number forms: GCHAR is a byte minus 0x20, GSHORT two 7-bit groups, GINT3
//! three. Every parser returns `None` on a payload it cannot read and never
//! panics. Where the client's own reading is only partly known, the field
//! says so.

use super::dispatch::{latin1, read_gint3, read_gshort, split_quoted};

/// A GCHAR: the byte minus 0x20 (wrapping, as the client's subtraction).
fn gchar(b: u8) -> u8 {
    b.wrapping_sub(0x20)
}

/// `PLO_PLAYERWARP2` (49), 2.1 test builds and 2.17 on:
/// `GCHAR x*2, GCHAR y*2, GCHAR z, GCHAR gmapX, GCHAR gmapY`, then the
/// level name. The client reads `z` and does not use it. `gmapX`/`gmapY`
/// matter only when the name ends in `.gmap`; the client then clamps them to
/// the gmap's grid and swaps the name for the cell's level. The lab server
/// followed a gmap warp with `PLO_LEVELNAME` (6) for that cell's level.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerWarp2 {
    /// Tiles (the wire value is half-tile steps).
    pub x: f32,
    pub y: f32,
    /// The z byte as sent, unused by the client.
    pub z_raw: u8,
    pub gmap_x: u8,
    pub gmap_y: u8,
    /// A level file name, or a `.gmap` name.
    pub level: String,
}

impl PlayerWarp2 {
    pub fn is_gmap(&self) -> bool {
        self.level.to_ascii_lowercase().ends_with(".gmap")
    }
}

pub fn parse_player_warp2(payload: &[u8]) -> Option<PlayerWarp2> {
    let [x, y, z, gx, gy, name @ ..] = payload else {
        return None;
    };
    Some(PlayerWarp2 {
        x: f32::from(gchar(*x)) / 2.0,
        y: f32::from(gchar(*y)) / 2.0,
        z_raw: gchar(*z),
        gmap_x: gchar(*gx),
        gmap_y: gchar(*gy),
        level: latin1(name),
    })
}

/// `PLO_PROFILE` (75): repeated `GCHAR n` + `n` characters. The first field
/// is the account the profile belongs to; the rest are its lines. A field
/// that runs past the end of the payload ends the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCard {
    pub account: String,
    pub lines: Vec<String>,
}

pub fn parse_profile(payload: &[u8]) -> Option<ProfileCard> {
    let mut fields = Vec::new();
    let mut rest = payload;
    while let Some((&n, tail)) = rest.split_first() {
        let n = usize::from(gchar(n));
        let Some(field) = tail.get(..n) else { break };
        fields.push(latin1(field).trim().to_string());
        rest = &tail[n..];
    }
    let mut it = fields.into_iter();
    let account = it.next()?;
    Some(ProfileCard { account, lines: it.collect() })
}

/// `PLO_NPCDEL2` (150): `GCHAR n`, an `n`-character level name, `GINT3` npc id.
pub fn parse_npc_del2(payload: &[u8]) -> Option<(String, usize)> {
    let (&n, tail) = payload.split_first()?;
    let n = usize::from(gchar(n));
    let level = tail.get(..n)?;
    let id = read_gint3(tail.get(n..)?)?;
    Some((latin1(level), id))
}

/// `PLO_MOVE` (165), `PLO_SHOWIMGNPC` (166) and `PLO_MOVE2` (189, 2.31): a
/// `GINT3` npc id, then the rest (move data or an NPC prop list). The client
/// requires an id above 0 for 165. 189 is the same shape and the client
/// calls the move routine with an extra flag.
pub fn parse_npc_id_rest(payload: &[u8]) -> Option<(usize, &[u8])> {
    Some((read_gint3(payload)?, payload.get(3..)?))
}

/// `PLO_SHOOT` (175): `GSHORT` player id, then the shot data. The client
/// uses it only when the player is in the active level.
pub fn parse_shoot(payload: &[u8]) -> Option<(u16, &[u8])> {
    Some((read_gshort(payload)?, payload.get(2..)?))
}

/// A GCHAR flag where above 0 means on (`PLO_GHOSTMODE` 170,
/// `PLO_GHOSTICON` 174). Only the first byte is read.
pub fn parse_flag(payload: &[u8]) -> Option<bool> {
    payload.first().map(|&b| i32::from(b) - 0x20 > 0)
}

/// `PLO_UNKNOWN181` (181): `GSHORT` player id, `GCHAR` toggle (above 0 is
/// on). The client needs at least 3 bytes.
pub fn parse_player_toggle(payload: &[u8]) -> Option<(u16, bool)> {
    Some((read_gshort(payload)?, parse_flag(payload.get(2..)?)?))
}

/// Which mode `PLO_KINGDOMSMODE` (176) or `PLO_NEWWORLDMODE` (177) switches
/// the client main form to. Neither has a payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    Kingdoms,
    NewWorld,
}

/// `PLO_SERVERWARP` (178): comma text with at least 4 fields; fields 1, 2
/// and 3 are used and field 0 is ignored. They go to the client's
/// connect-to-other-server routine; which is the server name and what the
/// other two are is **not resolved**, so they are kept in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerWarp {
    pub field1: String,
    pub field2: String,
    pub field3: String,
}

pub fn parse_server_warp(payload: &[u8]) -> Option<ServerWarp> {
    let f = split_quoted(&latin1(payload));
    let [_, a, b, c, ..] = &f[..] else { return None };
    Some(ServerWarp { field1: a.clone(), field2: b.clone(), field3: c.clone() })
}

/// `PLO_TAKESCREENSHOT` (184): a file name; the client accepts only the
/// extension `.png` (compared as written here) and saves under that base
/// name in its screenshots folder. Seihai never takes the screenshot.
pub fn parse_screenshot_name(payload: &[u8]) -> Option<String> {
    let name = latin1(payload);
    let name = name.trim_end_matches(['\r', '\n']);
    (name.len() > 4 && name.ends_with(".png")).then(|| name.to_string())
}

/// `PLO_BOARDHEIGHTS` (185), 2.17 on: `GCHAR mapX, mapY, blockX, blockY,
/// w-1, h-1`, then `w*h` cells of two raw bytes, row-major. A cell is
/// `(b0 - 0x20 - 50) + (b1 - 0x20) / 128` (a height with a fraction). `mapX`
/// and `mapY` pick the gmap cell and must lie inside the gmap's grid (the
/// client checks that, Seihai has no gmap yet). The client's own length
/// check is only `w*h + 6` bytes although cells take two each; here all
/// `2*w*h` bytes must be present.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardHeights {
    pub map_x: u8,
    pub map_y: u8,
    pub block_x: u8,
    pub block_y: u8,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<f32>,
}

pub fn parse_board_heights(payload: &[u8]) -> Option<BoardHeights> {
    let [mx, my, bx, by, w, h, cells @ ..] = payload else {
        return None;
    };
    let (width, height) = (usize::from(gchar(*w)) + 1, usize::from(gchar(*h)) + 1);
    let cells = cells.get(..2 * width * height)?;
    Some(BoardHeights {
        map_x: gchar(*mx),
        map_y: gchar(*my),
        block_x: gchar(*bx),
        block_y: gchar(*by),
        width,
        height,
        cells: cells
            .chunks_exact(2)
            .map(|c| (f32::from(gchar(c[0])) - 50.0) + f32::from(gchar(c[1])) / 128.0)
            .collect(),
    })
}

/// `PLO_BOARDMODIFY2` (186), 2.22 on: `GCHAR mapX, mapY, x, y, w, h`, then
/// `w*h` GSHORT tiles row-major (the gmap cell's board is replaced from
/// `x`,`y`), then a trailer the client does not read. It needs at least
/// `6 + 2*w*h` bytes. What the trailer holds is unknown (none was seen).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardModify2 {
    pub map_x: u8,
    pub map_y: u8,
    pub x: u8,
    pub y: u8,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<u16>,
    pub trailer: Vec<u8>,
}

pub fn parse_board_modify2(payload: &[u8]) -> Option<BoardModify2> {
    let [mx, my, x, y, w, h, rest @ ..] = payload else {
        return None;
    };
    let (width, height) = (usize::from(gchar(*w)), usize::from(gchar(*h)));
    let body = rest.get(..2 * width * height)?;
    Some(BoardModify2 {
        map_x: gchar(*mx),
        map_y: gchar(*my),
        x: gchar(*x),
        y: gchar(*y),
        width,
        height,
        tiles: body.chunks_exact(2).map(|c| read_gshort(c).unwrap_or(0)).collect(),
        trailer: rest[2 * width * height..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gc(v: u8) -> u8 {
        v + 0x20
    }

    fn gs(v: u16) -> [u8; 2] {
        [gc((v >> 7) as u8 & 0x7F), gc(v as u8 & 0x7F)]
    }

    #[test]
    fn player_warp2_halves_the_position() {
        let mut p = vec![gc(2 * 30 + 1), gc(2 * 12), gc(50), gc(1), gc(0)];
        p.extend_from_slice(b"gm_ne.nw");
        let w = parse_player_warp2(&p).unwrap();
        assert_eq!((w.x, w.y), (30.5, 12.0));
        assert_eq!((w.gmap_x, w.gmap_y, w.z_raw), (1, 0, 50));
        assert_eq!(w.level, "gm_ne.nw");
        assert!(!w.is_gmap());
        p.truncate(5);
        p.extend_from_slice(b"gm2x2.gmap");
        assert!(parse_player_warp2(&p).unwrap().is_gmap());
        assert_eq!(parse_player_warp2(&p[..4]), None);
        // An empty name is still a warp (the client then keeps its level).
        assert_eq!(parse_player_warp2(&p[..5]).unwrap().level, "");
    }

    #[test]
    fn profile_fields() {
        let mut p = Vec::new();
        for f in ["acct", "  Name: X ", "line two"] {
            p.push(gc(f.len() as u8));
            p.extend_from_slice(f.as_bytes());
        }
        let c = parse_profile(&p).unwrap();
        assert_eq!(c.account, "acct");
        assert_eq!(c.lines, vec!["Name: X", "line two"]);
        // A field cut short ends the list.
        p.truncate(p.len() - 3);
        assert_eq!(parse_profile(&p).unwrap().lines, vec!["Name: X"]);
        assert_eq!(parse_profile(b""), None);
    }

    #[test]
    fn npc_ids() {
        let mut p = vec![gc(5)];
        p.extend_from_slice(b"a.nwx");
        p.extend_from_slice(&[gc(0), gc(1), gc(5)]);
        assert_eq!(parse_npc_del2(&p), Some(("a.nwx".to_string(), 133)));
        assert_eq!(parse_npc_del2(&p[..8]), None);
        let m = [gc(0), gc(0), gc(7), b'a', b'b'];
        assert_eq!(parse_npc_id_rest(&m), Some((7, &b"ab"[..])));
        assert_eq!(parse_npc_id_rest(&m[..2]), None);
        let s = [gs(300)[0], gs(300)[1], b'z'];
        assert_eq!(parse_shoot(&s), Some((300, &b"z"[..])));
    }

    #[test]
    fn flags_and_toggles() {
        assert_eq!(parse_flag(b" "), Some(false));
        assert_eq!(parse_flag(b"!"), Some(true));
        assert_eq!(parse_flag(b"\x00"), Some(false));
        assert_eq!(parse_flag(b""), None);
        let t = [gs(9)[0], gs(9)[1], gc(1)];
        assert_eq!(parse_player_toggle(&t), Some((9, true)));
        assert_eq!(parse_player_toggle(&t[..2]), None);
    }

    #[test]
    fn server_warp_uses_fields_1_to_3() {
        let w = parse_server_warp(b"x,Main Server,127.0.0.1,14900").unwrap();
        assert_eq!((w.field1.as_str(), w.field2.as_str(), w.field3.as_str()), ("Main Server", "127.0.0.1", "14900"));
        assert_eq!(parse_server_warp(b"a,b,c"), None);
        assert!(parse_server_warp(b"\"a,b\",\"c, d\",e,f,g").is_some());
    }

    #[test]
    fn screenshot_name_needs_png() {
        assert_eq!(parse_screenshot_name(b"shot.png"), Some("shot.png".to_string()));
        assert_eq!(parse_screenshot_name(b"shot.jpg"), None);
        assert_eq!(parse_screenshot_name(b".png"), None);
        assert_eq!(parse_screenshot_name(b""), None);
    }

    #[test]
    fn board_heights_cells() {
        // 2x1 block at (3,4) of gmap cell (1,0); cells 50 -> 0.0, 53.5 -> 3.5.
        let mut p = vec![gc(1), gc(0), gc(3), gc(4), gc(1), gc(0)];
        p.extend_from_slice(&[gc(50), gc(0), gc(53), gc(64)]);
        let h = parse_board_heights(&p).unwrap();
        assert_eq!((h.map_x, h.map_y, h.block_x, h.block_y, h.width, h.height), (1, 0, 3, 4, 2, 1));
        assert_eq!(h.cells, vec![0.0, 3.5]);
        // Cells must be present in full.
        assert_eq!(parse_board_heights(&p[..p.len() - 1]), None);
        assert_eq!(parse_board_heights(&p[..5]), None);
    }

    #[test]
    fn board_modify2_tiles_and_trailer() {
        let mut p = vec![gc(0), gc(1), gc(10), gc(11), gc(2), gc(1)];
        p.extend_from_slice(&gs(0x2a5));
        p.extend_from_slice(&gs(7));
        let m = parse_board_modify2(&p).unwrap();
        assert_eq!((m.map_x, m.map_y, m.x, m.y, m.width, m.height), (0, 1, 10, 11, 2, 1));
        assert_eq!(m.tiles, vec![0x2a5, 7]);
        assert!(m.trailer.is_empty());
        p.extend_from_slice(b"tail");
        assert_eq!(parse_board_modify2(&p).unwrap().trailer, b"tail");
        // Needs 6 + 2*w*h bytes.
        assert_eq!(parse_board_modify2(&p[..9]), None);
        // Zero-sized rectangles are fine.
        assert!(parse_board_modify2(&[gc(0); 6]).is_some());
    }

    /// No payload may panic any parser.
    #[test]
    fn parsers_never_panic() {
        let mut samples: Vec<Vec<u8>> = vec![vec![], vec![0], vec![0xff], vec![0x20; 3], vec![0x7f; 40]];
        for n in 0..14usize {
            samples.push((0..n as u8).map(|i| i.wrapping_mul(37).wrapping_add(0x1b)).collect());
            samples.push(vec![0xff; n]);
            samples.push(vec![0x20; n]);
        }
        for s in &samples {
            let _ = parse_player_warp2(s);
            let _ = parse_profile(s);
            let _ = parse_npc_del2(s);
            let _ = parse_npc_id_rest(s);
            let _ = parse_shoot(s);
            let _ = parse_flag(s);
            let _ = parse_player_toggle(s);
            let _ = parse_server_warp(s);
            let _ = parse_screenshot_name(s);
            let _ = parse_board_heights(s);
            let _ = parse_board_modify2(s);
        }
        // Largest rectangles a GCHAR can ask for with nothing behind them.
        assert_eq!(parse_board_heights(&[0xff; 6]), None);
        assert_eq!(parse_board_modify2(&[0xff; 6]), None);
    }
}
