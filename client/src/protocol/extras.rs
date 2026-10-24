//! Server messages that every public 2.22 gserver sends at login and that
//! earlier Seihai counted as "unhandled".
//!
//! What is decoded here is backed by two things: the layouts in gs2lib's
//! `IEnums.h` and Graal_analysis `protocol/packet-ids.md`, and the payload
//! shapes seen in the 2026-10-24 public 2.22 captures (four gservers).
//! Anything that is not backed by both is left out; see the list at the end
//! of `documentation/protocol.md`.
//!
//! Every parser returns `None` (or an empty list) on a payload it does not
//! understand and never panics.

use super::dispatch::{latin1, read_gint5, split_quoted};

/// A map picture spec from `PLO_BIGMAP` (171) or `PLO_MINIMAP` (172):
/// `txt,png,x,y` as one comma-separated line.
///
/// Seen: `bigmap2.txt,bigmap.png,0,0` and `minimap.txt,map2.png,0,0`. The
/// text file is the map's level grid and the PNG its picture; what `x` and
/// `y` position is not established (both were 0 in every capture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapSpec {
    pub txt: String,
    pub png: String,
    pub x: i32,
    pub y: i32,
}

/// Parse `txt,png,x,y`. Needs exactly four fields with numeric x and y.
pub fn parse_map_spec(payload: &[u8]) -> Option<MapSpec> {
    let text = latin1(payload);
    let parts: Vec<&str> = text.trim_end_matches(['\r', '\n']).split(',').collect();
    let [txt, png, x, y] = parts[..] else {
        return None;
    };
    Some(MapSpec {
        txt: txt.trim().to_string(),
        png: png.trim().to_string(),
        x: x.trim().parse().ok()?,
        y: y.trim().parse().ok()?,
    })
}

/// `PLO_SIGNATURE` (25): one GCHAR. gs2 servers send 73 (`i`); GServer-RS
/// sent 41 (`I`). Its meaning is not established.
pub fn parse_signature(payload: &[u8]) -> Option<u8> {
    payload.first().map(|c| c.wrapping_sub(0x20))
}

/// `PLO_LEVELMODTIME` (39): GINT5 Unix time of the level file. All four
/// captured values decode to dates in 2025 and 2026.
pub fn parse_level_mod_time(payload: &[u8]) -> Option<u32> {
    u32::try_from(read_gint5(payload)?).ok()
}

/// `PLO_GHOSTICON` (174): GCHAR, 1 shows the ghost icon. The captures sent
/// `0x20` (0) from three servers and a raw `0x00` from GServer-RS; both mean
/// off.
pub fn parse_ghost_icon(payload: &[u8]) -> bool {
    payload.first().map(|c| c.wrapping_sub(0x20)) == Some(1)
}

/// A comma-separated list as GServer writes it, with optional double quotes
/// around items (`"a","b, c"` or `a,b`). Used by `PLO_STAFFGUILDS` (47),
/// `PLO_STATUSLIST` (180) and `PLO_RPGWINDOW` (179).
pub fn parse_text_list(payload: &[u8]) -> Vec<String> {
    split_quoted(&latin1(payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_spec_as_seen() {
        let m = parse_map_spec(b"bigmap2.txt,bigmap.png,0,0").unwrap();
        assert_eq!((m.txt.as_str(), m.png.as_str(), m.x, m.y), ("bigmap2.txt", "bigmap.png", 0, 0));
        // The sample in gs2lib's header has non-zero x and y.
        let m = parse_map_spec(b"zodiacminimap.txt,zodiacworldminimap3.png,10,10").unwrap();
        assert_eq!((m.x, m.y), (10, 10));
        assert_eq!(parse_map_spec(b"a.txt,b.png,1"), None);
        assert_eq!(parse_map_spec(b"a.txt,b.png,x,y"), None);
        assert_eq!(parse_map_spec(b""), None);
    }

    #[test]
    fn signature_and_ghost_icon() {
        assert_eq!(parse_signature(b"i"), Some(73));
        assert_eq!(parse_signature(b"I"), Some(41));
        assert_eq!(parse_signature(b""), None);
        assert!(!parse_ghost_icon(b" "));
        assert!(!parse_ghost_icon(b"\x00"));
        assert!(parse_ghost_icon(b"!"));
        assert!(!parse_ghost_icon(b""));
    }

    #[test]
    fn level_mod_time_is_gint5_unix_time() {
        // 1_781_284_436 (2026-06-12) as GINT5: 7-bit groups, high first, +0x20.
        let v: u64 = 1_781_284_436;
        let b: Vec<u8> = (0..5).rev().map(|i| (((v >> (7 * i)) & 0x7F) as u8) + 0x20).collect();
        assert_eq!(parse_level_mod_time(&b), Some(1_781_284_436));
        assert_eq!(parse_level_mod_time(&b[..4]), None);
    }

    #[test]
    fn text_lists_quoted_and_plain() {
        assert_eq!(
            parse_text_list(b"\"Welcome to X.\",\"Programmed by A, B, and C.\""),
            vec!["Welcome to X.", "Programmed by A, B, and C."]
        );
        assert_eq!(
            parse_text_list(b"Online,Away,DND,Eating,Hiding,No PMs,RPing,Sparring,PKing").len(),
            9
        );
        assert_eq!(parse_text_list(b"Server,Staff,Manager"), vec!["Server", "Staff", "Manager"]);
        assert!(parse_text_list(b"").is_empty());
    }
}
