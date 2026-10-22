//! Packet dispatcher.
//!
//! Splits a decompressed frame body on `\n` and routes each message by
//! its GCHAR id (wire byte - 0x20). The level-entry set is handled;
//! everything else is recorded as unhandled.

use super::props::parse_player_props;
use std::collections::HashMap;

/// A single logical message after frame decompression.
#[derive(Debug, Clone)]
pub struct Message {
    /// Logical id (wire byte - 0x20).
    pub id: u8,
    /// Payload bytes (excluding the id and the trailing newline).
    pub payload: Vec<u8>,
}

/// Split a decompressed body into messages.
///
/// Each message is `GCHAR(id)` + payload + `\n`.
pub fn split_messages(body: &[u8]) -> Vec<Message> {
    let mut out = Vec::new();
    for line in body.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let id = line[0].wrapping_sub(0x20);
        out.push(Message {
            id,
            payload: line[1..].to_vec(),
        });
    }
    out
}

/// Minimal session state built from the level-entry packets.
#[derive(Debug, Default)]
pub struct SessionState {
    pub level_name: Option<String>,
    pub board_lines: Vec<String>,
    pub links: Vec<String>,
    pub chests: Vec<String>,
    pub signs: Vec<String>,
    pub board_modifies: Vec<String>,
    pub baddies: Vec<String>,
    pub npcs: Vec<String>,
    pub items: Vec<String>,
    pub warps: Vec<String>,
    pub player_props: Vec<u8>,
    /// Player position extracted from `PLO_PLAYERPROPS` (classic scale).
    pub player_x: Option<f32>,
    pub player_y: Option<f32>,
    pub player_dir: Option<u8>,
    pub other_props: Vec<(u8, Vec<u8>)>,
    pub unhandled: HashMap<u8, usize>,
}

impl SessionState {
    /// Apply one message to the session.
    pub fn apply(&mut self, msg: &Message) {
        match msg.id {
            0 => {
                // PLO_LEVELBOARD — rest is a board line
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.board_lines.push(s);
                }
            }
            1 => {
                // PLO_LEVELLINK
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.links.push(s);
                }
            }
            4 => {
                // PLO_LEVELCHEST
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.chests.push(s);
                }
            }
            5 => {
                // PLO_LEVELSIGN
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.signs.push(s);
                }
            }
            6 => {
                // PLO_LEVELNAME
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.level_name = Some(s);
                }
            }
            7 => {
                // PLO_BOARDMODIFY
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.board_modifies.push(s);
                }
            }
            8 => {
                // PLO_OTHERPLPROPS
                self.other_props.push((msg.id, msg.payload.clone()));
            }
            9 => {
                // PLO_PLAYERPROPS — extract classic position if present.
                self.player_props = msg.payload.clone();
                self.apply_player_props(&msg.payload);
            }
            2 => {
                // PLO_BADDYPROPS
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.baddies.push(s);
                }
            }
            3 => {
                // PLO_NPCPROPS
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.npcs.push(s);
                }
            }
            14 => {
                // PLO_PLAYERWARP: c x*2, c y*2, rest = level name
                if let [x, y, level @ ..] = msg.payload.as_slice() {
                    self.player_x = Some(x.wrapping_sub(0x20) as f32 / 2.0);
                    self.player_y = Some(y.wrapping_sub(0x20) as f32 / 2.0);
                    self.warps.push(String::from_utf8_lossy(level).into_owned());
                }
            }
            22 => {
                // PLO_ITEMADD
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.items.push(s);
                }
            }
            23 => {
                // PLO_ITEMDEL — record as a removal marker
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.items.push(format!("DEL:{s}"));
                }
            }
            _ => {
                *self.unhandled.entry(msg.id).or_insert(0) += 1;
            }
        }
    }

    /// Apply every message in a decompressed body.
    pub fn apply_body(&mut self, body: &[u8]) {
        for msg in split_messages(body) {
            self.apply(&msg);
        }
    }

    /// Pick position and direction out of a `PLO_PLAYERPROPS` stream.
    ///
    /// X/Y are props 15/16 (GCHAR, half-tiles); direction is SPRITE
    /// (17) mod 4.
    fn apply_player_props(&mut self, payload: &[u8]) {
        for prop in parse_player_props(payload) {
            match (prop.id, prop.gchar()) {
                (15, Some(v)) => self.player_x = Some(v as f32 / 2.0),
                (16, Some(v)) => self.player_y = Some(v as f32 / 2.0),
                (17, Some(v)) => self.player_dir = Some(v % 4),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_route() {
        // wire: id 6 (LEVELNAME) = 0x26, payload "start", newline
        // wire: id 0 (LEVELBOARD) = 0x20, payload "BOARD 0 ...", newline
        let body = b"&start\n BOARD 0\n";
        let msgs = split_messages(body);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 6);
        assert_eq!(msgs[1].id, 0);

        let mut state = SessionState::default();
        state.apply_body(body);
        assert_eq!(state.level_name.as_deref(), Some("start"));
        assert_eq!(state.board_lines.len(), 1);
    }

    #[test]
    fn player_warp_sets_position() {
        // id 14, x = 30.5 (61), y = 12 (24), level "start.nw"
        let body = [&[0x2e, 0x20 + 61, 0x20 + 24][..], b"start.nw\n"].concat();
        let mut state = SessionState::default();
        state.apply_body(&body);
        assert_eq!(state.player_x, Some(30.5));
        assert_eq!(state.player_y, Some(12.0));
        assert_eq!(state.warps, vec!["start.nw".to_string()]);
    }
}
