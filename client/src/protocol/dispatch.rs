//! Packet dispatcher.
//!
//! Splits a decompressed frame body on `\n` and routes each message by
//! its GCHAR id (wire byte - 0x20). The level-entry set is handled;
//! everything else is recorded as unhandled.

use super::props::parse_player_props_with;
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
/// Each message is `GCHAR(id)` + payload + `\n`. `PLO_RAWDATA` (100) is
/// `GCHAR(100)` + `GINT3 n` + `\n`, and the next `n` bytes are one message
/// (id + payload) that may contain any byte. A trailing `\n` inside those
/// `n` bytes is part of the payload; `PLO_FILE` for 2.1+ clients ends in one.
///
/// This works on one body. [`SessionState::apply_body`] keeps a partial
/// message across bodies instead.
pub fn split_messages(body: &[u8]) -> Vec<Message> {
    let mut sp = Splitter::default();
    let mut out = sp.push(body);
    out.extend(sp.finish());
    out
}

/// Read a GINT3: three GCHARs, high 7 bits first.
pub fn read_gint3(b: &[u8]) -> Option<usize> {
    if b.len() < 3 {
        return None;
    }
    Some(
        ((b[0].wrapping_sub(0x20) as usize & 0x7F) << 14)
            | ((b[1].wrapping_sub(0x20) as usize & 0x7F) << 7)
            | (b[2].wrapping_sub(0x20) as usize & 0x7F),
    )
}

/// Message splitter that carries a partial message from one body to the
/// next.
#[derive(Debug, Default)]
pub struct Splitter {
    /// Bytes of an unfinished line or raw block.
    buf: Vec<u8>,
    /// Length of the raw block that starts at `buf[0]`, once its
    /// `PLO_RAWDATA` header has been read.
    raw_len: Option<usize>,
}

impl Splitter {
    /// Add a body and return every message it completes.
    pub fn push(&mut self, body: &[u8]) -> Vec<Message> {
        self.buf.extend_from_slice(body);
        let mut out = Vec::new();
        let mut i = 0;
        loop {
            if let Some(n) = self.raw_len {
                if self.buf.len() - i < n {
                    break;
                }
                if n > 0 {
                    out.push(Message {
                        id: self.buf[i].wrapping_sub(0x20),
                        payload: self.buf[i + 1..i + n].to_vec(),
                    });
                }
                i += n;
                self.raw_len = None;
                continue;
            }
            let Some(nl) = self.buf[i..].iter().position(|&b| b == b'\n') else {
                break;
            };
            let line = &self.buf[i..i + nl];
            i += nl + 1;
            let Some((&id, payload)) = line.split_first() else {
                continue;
            };
            let id = id.wrapping_sub(0x20);
            if id == 100 {
                if let Some(n) = read_gint3(payload) {
                    self.raw_len = Some(n);
                }
                continue;
            }
            out.push(Message {
                id,
                payload: payload.to_vec(),
            });
        }
        self.buf.drain(..i);
        out
    }

    /// Flush a final line that had no `\n`.
    pub fn finish(&mut self) -> Vec<Message> {
        let rest = std::mem::take(&mut self.buf);
        if self.raw_len.take().is_some() {
            // A cut-off raw block (a partial file) is not a message.
            return Vec::new();
        }
        match rest.split_first() {
            Some((&id, payload)) => vec![Message {
                id: id.wrapping_sub(0x20),
                payload: payload.to_vec(),
            }],
            None => Vec::new(),
        }
    }
}

/// Another player seen via `PLO_OTHERPLPROPS`.
#[derive(Debug, Clone)]
pub struct OtherPlayer {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub dir: u8,
    pub nick: Option<String>,
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
    /// Other players parsed from `PLO_OTHERPLPROPS`.
    pub others: Vec<OtherPlayer>,
    /// File names the server asked for or sent.
    pub file_requests: Vec<String>,
    /// Completed files: (name, bytes).
    pub incoming_files: Vec<(String, Vec<u8>)>,
    /// In-progress large file: (name, accumulated bytes).
    pub large_file: Option<(String, Vec<u8>)>,
    /// Size the server announced for the large file (`PLO_LARGEFILESIZE`).
    pub large_file_size: Option<u64>,
    /// `PLO_FILE` carries a 5-byte modtime for 2.1+ clients. 2.05 and
    /// older get the name and data only. Set this from the version.
    pub file_has_modtime: bool,
    /// Read prop 10 as the pre-2.1 bow slot (1.33–1.39 logins).
    pub bow_prop10: bool,
    /// Text of `PLO_DISCMESSAGE` (16). The server closes after it.
    pub disconnect_message: Option<String>,
    pub unhandled: HashMap<u8, usize>,
    splitter: Splitter,
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
                // PLO_OTHERPLPROPS — parse position and store.
                self.other_props.push((msg.id, msg.payload.clone()));
                if let Some(other) = parse_other_player(&msg.payload) {
                    if let Some(existing) = self.others.iter_mut().find(|o| o.id == other.id) {
                        *existing = other;
                    } else {
                        self.others.push(other);
                    }
                }
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
            56 => {
                // PLO_DELPLAYER — remove by id (first byte).
                if let Some(&id) = msg.payload.first() {
                    self.others.retain(|o| o.id != id as u16);
                }
            }
            16 => {
                // PLO_DISCMESSAGE: rest = text, `\r` = line break.
                let text = String::from_utf8_lossy(&msg.payload).replace('\r', "\n");
                self.disconnect_message = Some(text);
            }
            102 => {
                // PLO_FILE, always behind PLO_RAWDATA.
                let parsed = parse_file_packet(&msg.payload, self.file_has_modtime)
                    .or_else(|| parse_file_packet(&msg.payload, !self.file_has_modtime));
                if let Some((name, data)) = parsed {
                    self.file_requests.push(format!("FILE:{name}"));
                    match self.large_file {
                        Some((ref n, ref mut buf)) if n == &name => buf.extend_from_slice(&data),
                        _ => self.incoming_files.push((name, data)),
                    }
                }
            }
            68 => {
                // PLO_LARGEFILESTART: rest = name. PLO_FILE chunks follow.
                let name = String::from_utf8_lossy(&msg.payload).into_owned();
                self.large_file = Some((name, Vec::new()));
                self.large_file_size = None;
            }
            84 => {
                // PLO_LARGEFILESIZE: GINT5 size.
                self.large_file_size = read_gint5(&msg.payload);
            }
            69 => {
                // PLO_LARGEFILEEND: rest = name.
                if let Some((name, data)) = self.large_file.take() {
                    self.incoming_files.push((name, data));
                }
                self.large_file_size = None;
            }
            30 => {
                // PLO_FILESENDFAILED: rest = name. Before 2.1 the server
                // also uses it for "already up to date".
                let name = String::from_utf8_lossy(&msg.payload);
                self.file_requests.push(format!("FAIL:{name}"));
            }
            45 => {
                // PLO_FILEUPTODATE: rest = name (2.1+).
                let name = String::from_utf8_lossy(&msg.payload);
                self.file_requests.push(format!("UPTODATE:{name}"));
            }
            _ => {
                *self.unhandled.entry(msg.id).or_insert(0) += 1;
            }
        }
    }

    /// Apply every message in a decompressed body.
    ///
    /// A message cut off at the end of the body (a raw block or a line
    /// without its `\n`) is kept and finished by the next body.
    pub fn apply_body(&mut self, body: &[u8]) {
        for msg in self.splitter.push(body) {
            self.apply(&msg);
        }
    }

    /// Pick position and direction out of a `PLO_PLAYERPROPS` stream.
    ///
    /// X/Y are props 15/16 (GCHAR, half-tiles); direction is SPRITE
    /// (17) mod 4.
    fn apply_player_props(&mut self, payload: &[u8]) {
        for prop in parse_player_props_with(payload, self.bow_prop10) {
            match (prop.id, prop.gchar()) {
                (15, Some(v)) => self.player_x = Some(v as f32 / 2.0),
                (16, Some(v)) => self.player_y = Some(v as f32 / 2.0),
                (17, Some(v)) => self.player_dir = Some(v % 4),
                _ => {}
            }
        }
    }

    /// Write every completed file into `dir` and clear the list.
    /// Returns the names written.
    pub fn flush_files(&mut self, dir: &std::path::Path) -> Vec<String> {
        let mut written = Vec::new();
        for (name, data) in self.incoming_files.drain(..) {
            let path = dir.join(&name);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if std::fs::write(&path, &data).is_ok() {
                written.push(name);
            }
        }
        written
    }
}

/// Parse one `PLO_OTHERPLPROPS` payload into an `OtherPlayer`.
///
/// The first two bytes are the player id (GCHAR pair or raw). The rest
/// is a prop stream like `PLO_PLAYERPROPS`.
fn parse_other_player(payload: &[u8]) -> Option<OtherPlayer> {
    if payload.len() < 4 {
        return None;
    }
    let id = payload[0] as u16;
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut dir = 2u8;
    let mut nick = None;
    for prop in parse_player_props_with(&payload[2..], false) {
        match prop.id {
            0 => {
                // GSTRING nickname.
                if prop.raw.len() > 1 {
                    nick = String::from_utf8(prop.raw[1..].to_vec()).ok();
                }
            }
            15 => {
                if let Some(v) = prop.gchar() {
                    x = v as f32 / 2.0;
                }
            }
            16 => {
                if let Some(v) = prop.gchar() {
                    y = v as f32 / 2.0;
                }
            }
            17 => {
                if let Some(v) = prop.gchar() {
                    dir = v % 4;
                }
            }
            _ => {}
        }
    }
    Some(OtherPlayer {
        id,
        x,
        y,
        dir,
        nick,
    })
}

/// Read a GINT5: five GCHARs, high 7 bits first.
fn read_gint5(b: &[u8]) -> Option<u64> {
    if b.len() < 5 {
        return None;
    }
    Some(
        b[..5]
            .iter()
            .fold(0u64, |acc, &c| (acc << 7) | (c.wrapping_sub(0x20) as u64 & 0x7F)),
    )
}

/// Parse a `PLO_FILE` payload (the bytes after the id).
///
/// - 2.1+: `GINT5 modtime` + `GCHAR name_len` + name + data + `\n`
/// - before 2.1 (2.05): `GCHAR name_len` + name + data, with no `\n`
fn parse_file_packet(payload: &[u8], has_modtime: bool) -> Option<(String, Vec<u8>)> {
    let p = if has_modtime { payload.get(5..)? } else { payload };
    let (&len, rest) = p.split_first()?;
    let name_len = len.wrapping_sub(0x20) as usize;
    if rest.len() < name_len {
        return None;
    }
    // GServer decides by its own version table whether the modtime is
    // there, so a name that is not a plain file name means the other
    // layout.
    let name_bytes = &rest[..name_len];
    let plain = |b: &u8| b.is_ascii_graphic() || *b == b' ';
    if name_len == 0 || !name_bytes.iter().all(plain) || !name_bytes.contains(&b'.') {
        return None;
    }
    let name = String::from_utf8_lossy(name_bytes).into_owned();
    let mut data = &rest[name_len..];
    if has_modtime {
        data = data.strip_suffix(b"\n").unwrap_or(data);
    }
    Some((name, data.to_vec()))
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

    /// GServer's 2.05 file reply: `PLO_RAWDATA` + GINT3 length + `\n`,
    /// then `PLO_FILE` + GCHAR name length + name + data, with no `\n`.
    fn raw_file_205(name: &str, data: &[u8]) -> Vec<u8> {
        let n = 1 + 1 + name.len() + data.len();
        let mut out = vec![100 + 0x20];
        out.extend([((n >> 14) & 0x7F) as u8 + 0x20, ((n >> 7) & 0x7F) as u8 + 0x20, (n & 0x7F) as u8 + 0x20]);
        out.push(b'\n');
        out.push(102 + 0x20);
        out.push(name.len() as u8 + 0x20);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        out
    }

    #[test]
    fn gint3_matches_lab_board_header() {
        // Lab 2.22 board packet: RAWDATA length 8194 = 1 + 64*64*2 + 1.
        assert_eq!(read_gint3(b" `\""), Some(8194));
    }

    #[test]
    fn rawdata_file_205() {
        // Data holds a \n and ends without one; the next message follows.
        let data = b"GLEVNW01\nBOARD 0 0 64 0 aa";
        let mut body = raw_file_205("new1.nw", data);
        body.extend_from_slice(b"&new1.nw\n");
        let msgs = split_messages(&body);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 102);
        assert_eq!(msgs[1].id, 6);

        let mut state = SessionState::default();
        state.apply_body(&body);
        assert_eq!(state.incoming_files, vec![("new1.nw".to_string(), data.to_vec())]);
        assert_eq!(state.level_name.as_deref(), Some("new1.nw"));
    }

    #[test]
    fn rawdata_file_with_modtime_split_across_frames() {
        // 2.1+ layout: GINT5 modtime, GCHAR name length, name, data, \n.
        let name = "a.png";
        let data: Vec<u8> = (0..=255u8).collect();
        let mut file = vec![102 + 0x20];
        file.extend_from_slice(b"  !\"#");
        file.push(name.len() as u8 + 0x20);
        file.extend_from_slice(name.as_bytes());
        file.extend_from_slice(&data);
        file.push(b'\n');
        let n = file.len();
        let mut body = vec![100 + 0x20, ((n >> 14) as u8) + 0x20, ((n >> 7) & 0x7F) as u8 + 0x20, (n & 0x7F) as u8 + 0x20, b'\n'];
        body.extend_from_slice(&file);
        body.extend_from_slice(b"&a\n");

        let mut state = SessionState::default();
        state.file_has_modtime = true;
        let (one, two) = body.split_at(40);
        state.apply_body(one);
        assert!(state.incoming_files.is_empty());
        state.apply_body(two);
        assert_eq!(state.incoming_files, vec![(name.to_string(), data)]);
        assert_eq!(state.level_name.as_deref(), Some("a"));
    }

    #[test]
    fn file_layout_follows_the_server() {
        // A 2.1+ reply while the client expected the 2.05 layout.
        let name = "new1.nw";
        let mut file = vec![102 + 0x20];
        file.extend_from_slice(b"  !\"#");
        file.push(name.len() as u8 + 0x20);
        file.extend_from_slice(name.as_bytes());
        file.extend_from_slice(b"GLEVNW01\n");
        file.push(b'\n');
        let n = file.len();
        let mut body = vec![100 + 0x20, 0x20, (n >> 7) as u8 + 0x20, (n & 0x7F) as u8 + 0x20, b'\n'];
        body.extend_from_slice(&file);
        let mut st = SessionState::default();
        st.apply_body(&body);
        assert_eq!(st.incoming_files, vec![(name.to_string(), b"GLEVNW01\n".to_vec())]);
    }

    #[test]
    fn large_file_chunks_join() {
        let mut body = b"dbig.nw\n".to_vec();
        body.extend_from_slice(&raw_file_205("big.nw", b"one,"));
        body.extend_from_slice(&raw_file_205("big.nw", b"two"));
        body.extend_from_slice(b"ebig.nw\n");
        let mut state = SessionState::default();
        state.apply_body(&body);
        assert_eq!(state.incoming_files, vec![("big.nw".to_string(), b"one,two".to_vec())]);
    }

    #[test]
    fn disconnect_message_is_kept() {
        let mut state = SessionState::default();
        state.apply_body(b"0Your client version is not allowed.\rAllowed: 2.22\n");
        assert_eq!(
            state.disconnect_message.as_deref(),
            Some("Your client version is not allowed.\nAllowed: 2.22")
        );
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
