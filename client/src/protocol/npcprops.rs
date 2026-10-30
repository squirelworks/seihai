//! `PLO_NPCPROPS` (3) and `PLI_NPCPROPS` (3): the props of a server NPC.
//!
//! Both ways the payload is `GINT3 id`, then props as `GCHAR prop id` and
//! a value. The prop ids and value shapes are GServer-v2's (`TNPC.h`); the
//! 2.05 behaviour was checked in the lab (`documentation/ledges-push-pull.md`).
//! Only image, script, position and the two flag bytes are kept; the other
//! props of the pre-2.2 table are skipped by their length, and reading stops
//! at a prop whose length is not known.

use super::dispatch::{latin1, read_gint3, read_gshort, script_text};

/// `NPCPROP_X` (2) and `NPCPROP_Y` (3): `GCHAR` in half tiles.
pub const NPCPROP_X: u8 = 2;
pub const NPCPROP_Y: u8 = 3;
/// `NPCPROP_BLOCKFLAGS` (14), see [`BLOCK_DONTBLOCK`] and the others.
pub const NPCPROP_BLOCKFLAGS: u8 = 14;

/// Block flag bits, as the 2.05 client reports them after the NPC's script
/// ran: `dontblock` 1, `canbepulled` 4, `canbepushed` 8.
pub const BLOCK_DONTBLOCK: u8 = 1;
pub const BLOCK_PULLABLE: u8 = 4;
pub const BLOCK_PUSHABLE: u8 = 8;

/// What one `PLO_NPCPROPS` said about an NPC. The server sends a whole
/// record when the NPC first shows, and only what changed after that.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NpcProps {
    pub image: Option<String>,
    /// The GS1 client script, `§` turned into line breaks.
    pub script: Option<String>,
    /// Position in tiles.
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub vis_flags: Option<u8>,
    pub block_flags: Option<u8>,
}

impl NpcProps {
    /// Take every prop `other` carries.
    pub fn merge(&mut self, other: &NpcProps) {
        let o = other.clone();
        self.image = o.image.or(self.image.take());
        self.script = o.script.or(self.script.take());
        self.x = o.x.or(self.x);
        self.y = o.y.or(self.y);
        self.vis_flags = o.vis_flags.or(self.vis_flags);
        self.block_flags = o.block_flags.or(self.block_flags);
    }
}

/// Parse a `PLO_NPCPROPS` payload into the NPC id and its props. `bow_prop12`
/// reads prop 12 as the pre-2.1 bow (power, or `GCHAR 10 + len` and the
/// image) instead of a GANI name.
pub fn parse_npc_props(b: &[u8], bow_prop12: bool) -> Option<(u32, NpcProps)> {
    let id = read_gint3(b)? as u32;
    let mut p = NpcProps::default();
    let mut i = 3;
    let gc = |i: usize| b.get(i).map(|c| c.wrapping_sub(0x20) as usize);
    // A GCHAR length, then that many bytes.
    let text = |i: usize| -> Option<(String, usize)> {
        let n = gc(i)?;
        Some((latin1(b.get(i + 1..i + 1 + n)?), 1 + n))
    };
    while let Some(prop) = gc(i) {
        i += 1;
        match prop {
            0 => {
                let (s, n) = text(i)?;
                p.image = Some(s);
                i += n;
            }
            1 => {
                let n = read_gshort(b.get(i..)?)? as usize;
                p.script = Some(script_text(b.get(i + 2..i + 2 + n)?));
                i += 2 + n;
            }
            2 => {
                p.x = Some(gc(i)? as f32 / 2.0);
                i += 1;
            }
            3 => {
                p.y = Some(gc(i)? as f32 / 2.0);
                i += 1;
            }
            13 => {
                p.vis_flags = Some(gc(i)? as u8);
                i += 1;
            }
            14 => {
                p.block_flags = Some(gc(i)? as u8);
                i += 1;
            }
            // Power, arrows, bombs, glove and bomb power, sprite, saves,
            // alignment, gmap level x / y: one byte.
            4 | 6..=9 | 18 | 23..=33 | 41 | 42 => i += 1,
            // Rupees and id: GINT3.
            5 | 17 => i += 3,
            // Hurt dx / dy: two bytes. Colours: five. Image part: six.
            16 => i += 2,
            19 => i += 5,
            34 => i += 6,
            // Sword: power 0..=4, or 30 + power then the image.
            10 => {
                let sp = gc(i)?;
                i += 1;
                if sp > 4 {
                    i += text(i)?.1;
                }
            }
            // Shield: power 0..=3, or 10 + power then the image.
            11 => {
                let sp = gc(i)?;
                i += 1;
                if sp > 3 {
                    i += text(i)?.1;
                }
            }
            12 if bow_prop12 => {
                let sp = gc(i)?;
                i += 1 + sp.saturating_sub(10);
            }
            // GANI, message, nickname, horse, body, GANI attributes 1 to 5.
            12 | 15 | 20 | 21 | 35..=40 => i += text(i)?.1,
            // Head: an index under 100, else 100 + the name's length.
            22 => {
                let len = gc(i)?;
                i += 1 + len.saturating_sub(100);
            }
            _ => break,
        }
        if i > b.len() {
            return None;
        }
    }
    Some((id, p))
}

/// The `PLI_NPCPROPS` (3) message: `GINT3 id`, then `(prop id, GCHAR value)`
/// pairs, and the newline. The 2.05 client sends the props of a server NPC
/// that its client-side script changed: X and Y (half tiles) after a push,
/// a pull or `x = …`, BLOCKFLAGS after `canbepushed` and the like.
pub fn npc_props_message(id: u32, props: &[(u8, u8)]) -> Vec<u8> {
    let mut m = vec![3 + 0x20, ((id >> 14) & 0x7F) as u8 + 0x20, ((id >> 7) & 0x7F) as u8 + 0x20, (id & 0x7F) as u8 + 0x20];
    for &(prop, v) in props {
        m.push(prop + 0x20);
        m.push(v.min(223) + 0x20);
    }
    m.push(b'\n');
    m
}

/// A position in tiles as the `GCHAR` half tiles of `NPCPROP_X` / `Y`.
pub fn half_tiles(v: f32) -> u8 {
    (v * 2.0).round().clamp(0.0, 223.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gc(n: usize) -> u8 {
        n as u8 + 0x20
    }

    /// The record the lab stub sent the 2.05 client: id 1001, image, script
    /// (with `§` breaks), x 30, y 25.
    fn lab_record() -> Vec<u8> {
        let mut p = vec![gc(0), gc(7), gc(105)];
        p.extend([gc(0), gc(12)]);
        p.extend_from_slice(b"labblock.gif");
        let script = b"if (created) { canbepushed; }\xa7";
        p.extend([gc(1), gc(0), gc(script.len())]);
        p.extend_from_slice(script);
        p.extend([gc(2), gc(60), gc(3), gc(50)]);
        p
    }

    #[test]
    fn a_full_record_gives_id_image_script_and_position() {
        let (id, p) = parse_npc_props(&lab_record(), true).unwrap();
        assert_eq!(id, 1001);
        assert_eq!(p.image.as_deref(), Some("labblock.gif"));
        assert_eq!(p.script.as_deref(), Some("if (created) { canbepushed; }\n"));
        assert_eq!((p.x, p.y), (Some(30.0), Some(25.0)));
        assert_eq!(p.block_flags, None);
    }

    #[test]
    fn skipped_props_keep_the_reader_in_step() {
        // Sword with an image, shield by power, rupees, colours, a message,
        // a head by name, then the position.
        let mut p = vec![gc(0), gc(0), gc(5)];
        p.extend([gc(10), gc(31), gc(3)]);
        p.extend_from_slice(b"s.p");
        p.extend([gc(11), gc(2), gc(5), gc(0), gc(1), gc(2), gc(19), 1, 2, 3, 4, 5]);
        p.extend([gc(15), gc(2)]);
        p.extend_from_slice(b"hi");
        p.extend([gc(22), gc(103)]);
        p.extend_from_slice(b"h.p");
        p.extend([gc(14), gc(12), gc(2), gc(10), gc(3), gc(11)]);
        let (id, p) = parse_npc_props(&p, true).unwrap();
        assert_eq!(id, 5);
        assert_eq!((p.x, p.y, p.block_flags), (Some(5.0), Some(5.5), Some(12)));
    }

    #[test]
    fn an_unknown_prop_ends_the_record_and_a_short_one_is_dropped() {
        let p = [gc(0), gc(0), gc(1), gc(2), gc(8), gc(70), gc(1), gc(3), gc(9)];
        assert_eq!(parse_npc_props(&p, true).unwrap().1.x, Some(4.0));
        assert_eq!(parse_npc_props(&p[..2], true), None);
        assert_eq!(parse_npc_props(&[gc(0), gc(0), gc(1), gc(0), gc(9), b'a'], true), None);
    }

    #[test]
    fn prop_12_is_the_bow_before_2_1_and_a_gani_after() {
        let p = [gc(0), gc(0), gc(1), gc(12), gc(12), b'b', b'w', gc(2), gc(4)];
        assert_eq!(parse_npc_props(&p, true).unwrap().1.x, Some(2.0));
        let p = [gc(0), gc(0), gc(1), gc(12), gc(4), b'i', b'd', b'l', b'e', gc(2), gc(4)];
        assert_eq!(parse_npc_props(&p, false).unwrap().1.x, Some(2.0));
    }

    #[test]
    fn the_message_matches_the_2_05_client() {
        // Lab capture: after a push up the 2.05 client sent `3`, id 1001,
        // Y = 48 half tiles; after `canbepushed; canbepulled;` BLOCKFLAGS 12.
        assert_eq!(npc_props_message(1001, &[(NPCPROP_Y, 48)]), b"# '\x89#P\n");
        assert_eq!(npc_props_message(1001, &[(NPCPROP_BLOCKFLAGS, 12)]), b"# '\x89.,\n");
    }

    #[test]
    fn merge_keeps_what_an_update_leaves_out() {
        let (_, mut full) = parse_npc_props(&lab_record(), true).unwrap();
        full.merge(&NpcProps { y: Some(24.0), ..NpcProps::default() });
        assert_eq!((full.x, full.y), (Some(30.0), Some(24.0)));
        assert!(full.image.is_some() && full.script.is_some());
    }
}
