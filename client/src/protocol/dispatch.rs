//! Packet dispatcher.
//!
//! Splits a decompressed frame body on `\n` and routes each message by
//! its GCHAR id (wire byte - 0x20). The level-entry set is handled;
//! everything else is recorded as unhandled.

use super::extras::{
    parse_ghost_icon, parse_level_mod_time, parse_map_spec, parse_signature, parse_text_list, MapSpec,
};
use super::profile::IdSet;
use super::props::parse_player_props_in;
use super::version::ClientVersion;
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

/// Read a GINT4: four GCHARs, high 7 bits first.
pub fn read_gint4(b: &[u8]) -> Option<u32> {
    if b.len() < 4 {
        return None;
    }
    Some(
        b[..4]
            .iter()
            .fold(0u32, |acc, &c| (acc << 7) | (c.wrapping_sub(0x20) as u32 & 0x7F)),
    )
}

/// A weapon from `PLO_NPCWEAPONADD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weapon {
    pub name: String,
    /// NPCPROP_IMAGE (0), if sent.
    pub image: Option<String>,
    /// NPCPROP_SCRIPT (1): the GS1 client script, if sent.
    pub script: Option<String>,
}

/// Parse a `PLO_NPCWEAPONADD` payload: `GCHAR len` + name, then NPC props.
/// Only image (0, `GCHAR len`) and script (1, `GSHORT len`) are read;
/// parsing stops at any other prop.
pub fn parse_weapon_add(b: &[u8]) -> Option<Weapon> {
    let len = b.first()?.wrapping_sub(0x20) as usize;
    let name = latin1(b.get(1..1 + len)?);
    let mut w = Weapon { name, image: None, script: None };
    let mut i = 1 + len;
    while let Some(&id) = b.get(i) {
        match id.wrapping_sub(0x20) {
            0 => {
                let n = b.get(i + 1)?.wrapping_sub(0x20) as usize;
                w.image = Some(latin1(b.get(i + 2..i + 2 + n)?));
                i += 2 + n;
            }
            1 => {
                let n = read_gshort(b.get(i + 1..)?)? as usize;
                w.script = Some(latin1(b.get(i + 3..i + 3 + n)?));
                i += 3 + n;
            }
            _ => break,
        }
    }
    Some(w)
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
    /// Chat bubble text (CURCHAT, prop 12). Empty clears it.
    pub chat: Option<String>,
}

/// Where a chat line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatKind {
    /// A chat bubble in the level (CURCHAT).
    Say,
    /// `PLO_TOALL`.
    Toall,
    /// `PLO_PRIVATEMESSAGE`.
    Private,
    /// `PLO_RC_ADMINMESSAGE`, or a line from Seihai itself.
    Admin,
}

/// One line in the chat history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLine {
    pub kind: ChatKind,
    pub from: String,
    pub text: String,
}

/// Lines kept in the chat history.
pub const CHAT_HISTORY: usize = 200;

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
    /// Our own nickname (NICKNAME, prop 0, in `PLO_PLAYERPROPS`).
    pub nickname: Option<String>,
    /// Read prop 10 as the pre-2.1 bow slot.
    pub bow_prop10: bool,
    /// Server→client ids this client version handles. Others are counted
    /// in `ignored` and dropped, as that client would. `None` takes all.
    pub known_plo: Option<IdSet>,
    /// Ids the own-player prop parser accepts; the first other id ends
    /// the stream. `None` takes every id Seihai can decode.
    pub own_prop_ids: Option<IdSet>,
    /// Same for other players' props.
    pub other_prop_ids: Option<IdSet>,
    /// Messages dropped because this version has no handler for them.
    pub ignored: HashMap<u8, usize>,
    /// Text of `PLO_DISCMESSAGE` (16). The server closes after it.
    pub disconnect_message: Option<String>,
    /// Chat history, oldest first, at most [`CHAT_HISTORY`] lines.
    pub chat_log: Vec<ChatLine>,
    /// Chests the server reports as opened, by tile position. The game
    /// loop moves them into the session.
    pub opened_chests: Vec<(i32, i32)>,
    /// Server messages received, for the debug overlay.
    pub messages_seen: usize,
    /// Server flags from `PLO_FLAGSET` (28). A flag without `=` has an
    /// empty value.
    pub flags: HashMap<String, String>,
    /// Weapons the player holds, by name, in the order they arrived
    /// (`PLO_NPCWEAPONADD` 33, `PLO_NPCWEAPONDEL` 34).
    pub weapons: Vec<Weapon>,
    /// Built-in weapons from `PLO_DEFAULTWEAPON` (43), by id.
    pub default_weapons: Vec<u8>,
    /// Tiles of the current level from `PLO_BOARDPACKET` (101): 64×64
    /// tile ids, row by row.
    pub board: Option<Vec<u16>>,
    /// Server world time from `PLO_NEWWORLDTIME` (42).
    pub world_time: Option<u32>,
    /// `PLO_SIGNATURE` (25) value; gs2 servers send 73.
    pub signature: Option<u8>,
    /// `PLO_LEVELMODTIME` (39): Unix time of the level file, with the level
    /// it arrived for.
    pub level_mod_time: Option<(String, u32)>,
    /// `PLO_STARTMESSAGE` (41): the server's HTML welcome page. Empty on
    /// some servers.
    pub start_message: Option<String>,
    /// `PLO_HASNPCSERVER` (44) seen: the server runs NPC scripts, so the
    /// client leaves NPC props alone.
    pub has_npc_server: bool,
    /// `PLO_STAFFGUILDS` (47): guild tags that mark staff.
    pub staff_guilds: Vec<String>,
    /// `PLO_SETACTIVELEVEL` (156): the level the following chests, baddies
    /// and NPCs belong to.
    pub active_level: Option<String>,
    /// `PLO_GHOSTICON` (174).
    pub ghost_icon: bool,
    /// `PLO_RPGWINDOW` (179): the strings of the RPG window, in order
    /// (a welcome line and a credits line on the servers seen).
    pub rpg_window: Vec<String>,
    /// `PLO_STATUSLIST` (180): player status names (Online, Away, ...).
    pub status_list: Vec<String>,
    /// `PLO_BIGMAP` (171) and `PLO_MINIMAP` (172).
    pub bigmap: Option<MapSpec>,
    pub minimap: Option<MapSpec>,
    /// `PLO_ISLEADER` (10) seen.
    pub is_leader: bool,
    /// `PLO_LISTPROCESSES` (182) requests seen. Seihai never answers with
    /// a process list.
    pub process_list_requests: usize,
    pub unhandled: HashMap<u8, usize>,
    splitter: Splitter,
}

impl SessionState {
    /// A session that reads server traffic the way `version` does.
    pub fn for_version(version: ClientVersion) -> Self {
        let p = version.profile();
        Self {
            file_has_modtime: p.files.modtime,
            bow_prop10: p.props.bow_prop10,
            known_plo: Some(p.protocol.plo),
            own_prop_ids: Some(p.props.own),
            other_prop_ids: Some(p.props.other),
            ..Default::default()
        }
    }

    /// True if this client version would act on message `id`. A raw block
    /// (101 board, 102 file) belongs to the `PLO_RAWDATA` (100) before it.
    pub fn handles(&self, id: u8) -> bool {
        match &self.known_plo {
            None => true,
            Some(set) => set.contains(id) || (matches!(id, 101 | 102) && set.contains(100)),
        }
    }

    /// Apply one message to the session.
    pub fn apply(&mut self, msg: &Message) {
        self.messages_seen += 1;
        if !self.handles(msg.id) {
            *self.ignored.entry(msg.id).or_insert(0) += 1;
            return;
        }
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
                // PLO_LEVELCHEST: c opened, c x, c y [, c item, c sign].
                if let Ok(s) = String::from_utf8(msg.payload.clone()) {
                    self.chests.push(s);
                }
                if let [o, x, y, ..] = msg.payload[..] {
                    if o.wrapping_sub(0x20) == 1 {
                        let pos = (x.wrapping_sub(0x20) as i32, y.wrapping_sub(0x20) as i32);
                        self.opened_chests.push(pos);
                    }
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
                    self.enter_level(s);
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
                if let Some(u) = parse_other_player(&msg.payload, self.bow_prop10, self.other_prop_ids.as_ref()) {
                    self.apply_other(u);
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
                    let name = latin1(level);
                    // Pre-2.1 clients get no PLO_LEVELNAME on a warp; the
                    // level name here is the only one they see.
                    if !name.is_empty() {
                        self.enter_level(name.clone());
                    }
                    self.warps.push(name);
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
                // PLO_DELPLAYER: GSHORT id.
                if let Some(id) = read_gshort(&msg.payload) {
                    self.others.retain(|o| o.id != id);
                }
            }
            13 => {
                // PLO_TOALL: GSHORT from, GSTRING message.
                if let Some(from) = read_gshort(&msg.payload) {
                    // GSTRING; fall back to the rest of the line if the
                    // length runs past it.
                    let rest = msg.payload.get(2..).unwrap_or(&[]);
                    let text = match rest.split_first() {
                        Some((&n, t)) => t.get(..n.wrapping_sub(0x20) as usize).unwrap_or(t),
                        None => rest,
                    };
                    let who = self.name_of(from);
                    self.push_chat(ChatKind::Toall, who, latin1(text));
                }
            }
            37 => {
                // PLO_PRIVATEMESSAGE: GSHORT from, rest = "type","message".
                if let Some(from) = read_gshort(&msg.payload) {
                    let rest = latin1(msg.payload.get(2..).unwrap_or(&[]));
                    let fields = split_quoted(&rest);
                    let text = fields.last().cloned().unwrap_or_default();
                    let who = self.name_of(from);
                    self.push_chat(ChatKind::Private, who, text);
                }
            }
            35 => {
                // PLO_RC_ADMINMESSAGE: "<from>:\xa7<text>".
                let rest = latin1(&msg.payload);
                let (who, text) = match rest.split_once('\u{a7}') {
                    Some((w, t)) => (w.trim_end_matches(':').to_string(), t.to_string()),
                    None => ("Server".to_string(), rest),
                };
                self.push_chat(ChatKind::Admin, who, text);
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
            101 => {
                // PLO_BOARDPACKET: 64×64 little-endian u16 tile ids,
                // always behind PLO_RAWDATA.
                if msg.payload.len() >= 8192 {
                    self.board = Some(
                        msg.payload[..8192]
                            .chunks_exact(2)
                            .map(|p| u16::from_le_bytes([p[0], p[1]]) & 0x0FFF)
                            .collect(),
                    );
                }
            }
            28 => {
                // PLO_FLAGSET: rest = `name` or `name=value`.
                let text = latin1(&msg.payload);
                let (k, v) = text.split_once('=').unwrap_or((text.as_str(), ""));
                self.flags.insert(k.to_string(), v.to_string());
            }
            33 => {
                // PLO_NPCWEAPONADD: replaces a weapon of the same name.
                if let Some(w) = parse_weapon_add(&msg.payload) {
                    match self.weapons.iter_mut().find(|o| o.name == w.name) {
                        Some(old) => *old = w,
                        None => self.weapons.push(w),
                    }
                }
            }
            34 => {
                // PLO_NPCWEAPONDEL: rest = name.
                let name = latin1(&msg.payload);
                self.weapons.retain(|w| w.name != name);
            }
            42 => {
                // PLO_NEWWORLDTIME: GINT4.
                self.world_time = read_gint4(&msg.payload);
            }
            43 => {
                // PLO_DEFAULTWEAPON: GCHAR id.
                if let Some(&c) = msg.payload.first() {
                    let id = c.wrapping_sub(0x20);
                    if !self.default_weapons.contains(&id) {
                        self.default_weapons.push(id);
                    }
                }
            }
            45 => {
                // PLO_FILEUPTODATE: rest = name (2.1+).
                let name = String::from_utf8_lossy(&msg.payload);
                self.file_requests.push(format!("UPTODATE:{name}"));
            }
            10 => self.is_leader = true,
            25 => self.signature = parse_signature(&msg.payload),
            39 => {
                if let (Some(t), Some(level)) = (parse_level_mod_time(&msg.payload), &self.level_name) {
                    self.level_mod_time = Some((level.clone(), t));
                }
            }
            41 => self.start_message = Some(latin1(&msg.payload)),
            44 => self.has_npc_server = true,
            47 => self.staff_guilds = parse_text_list(&msg.payload),
            156 => {
                let name = latin1(&msg.payload);
                if !name.is_empty() {
                    self.active_level = Some(name);
                }
            }
            168 => {
                // PLO_UNKNOWN168: empty; login servers send it. Nothing to do.
            }
            171 => self.bigmap = parse_map_spec(&msg.payload),
            172 => self.minimap = parse_map_spec(&msg.payload),
            174 => self.ghost_icon = parse_ghost_icon(&msg.payload),
            179 => self.rpg_window = parse_text_list(&msg.payload),
            180 => self.status_list = parse_text_list(&msg.payload),
            182 => self.process_list_requests += 1,
            _ => {
                *self.unhandled.entry(msg.id).or_insert(0) += 1;
            }
        }
    }

    /// Switch to level `name`. When it differs from the current level,
    /// drop the old level's board, links, signs and chests: the server
    /// sends the new ones after the name.
    fn enter_level(&mut self, name: String) {
        if self.level_name.as_deref() != Some(name.as_str()) {
            self.board = None;
            self.board_lines.clear();
            self.links.clear();
            self.signs.clear();
            self.chests.clear();
        }
        self.level_name = Some(name);
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

    /// Add a line to the chat history.
    pub fn push_chat(&mut self, kind: ChatKind, from: String, text: String) {
        self.chat_log.push(ChatLine { kind, from, text });
        if self.chat_log.len() > CHAT_HISTORY {
            let extra = self.chat_log.len() - CHAT_HISTORY;
            self.chat_log.drain(..extra);
        }
    }

    /// Nickname for a player id, or the id itself.
    fn name_of(&self, id: u16) -> String {
        self.others
            .iter()
            .find(|o| o.id == id)
            .and_then(|o| o.nick.clone())
            .unwrap_or_else(|| format!("#{id}"))
    }

    fn apply_other(&mut self, u: OtherUpdate) {
        if u.left {
            self.others.retain(|o| o.id != u.id);
            return;
        }
        let i = match self.others.iter().position(|o| o.id == u.id) {
            Some(i) => i,
            None => {
                self.others.push(OtherPlayer {
                    id: u.id,
                    x: 0.0,
                    y: 0.0,
                    dir: 2,
                    nick: None,
                    chat: None,
                });
                self.others.len() - 1
            }
        };
        let o = &mut self.others[i];
        if let Some(v) = u.x {
            o.x = v;
        }
        if let Some(v) = u.y {
            o.y = v;
        }
        if let Some(v) = u.dir {
            o.dir = v;
        }
        if u.nick.is_some() {
            o.nick = u.nick;
        }
        if let Some(text) = u.chat {
            o.chat = Some(text.clone());
            if !text.is_empty() {
                let who = self.name_of(u.id);
                self.push_chat(ChatKind::Say, who, text);
            }
        }
    }

    /// Pick position and direction out of a `PLO_PLAYERPROPS` stream.
    ///
    /// X/Y are props 15/16 (GCHAR, half-tiles); direction is SPRITE
    /// (17) mod 4.
    fn apply_player_props(&mut self, payload: &[u8]) {
        for prop in parse_player_props_in(payload, self.bow_prop10, self.own_prop_ids.as_ref()) {
            match (prop.id, prop.gchar()) {
                (15, Some(v)) => self.player_x = Some(v as f32 / 2.0),
                (16, Some(v)) => self.player_y = Some(v as f32 / 2.0),
                (17, Some(v)) => self.player_dir = Some(v % 4),
                (0, _) => self.nickname = Some(latin1(prop.raw.get(1..).unwrap_or(&[]))),
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

/// Read a GSHORT: two GCHARs, high 7 bits first.
pub fn read_gshort(b: &[u8]) -> Option<u16> {
    let (hi, lo) = (*b.first()?, *b.get(1)?);
    Some(((hi.wrapping_sub(0x20) as u16 & 0x7F) << 7) | (lo.wrapping_sub(0x20) as u16 & 0x7F))
}

/// The props one `PLO_OTHERPLPROPS` carried. The server sends only the
/// props that changed, so anything absent keeps its old value.
#[derive(Debug, Default)]
struct OtherUpdate {
    id: u16,
    x: Option<f32>,
    y: Option<f32>,
    dir: Option<u8>,
    nick: Option<String>,
    chat: Option<String>,
    /// PCONNECTED (51) or JOINLEAVELVL (50) = 0: the player left.
    left: bool,
}

/// Parse one `PLO_OTHERPLPROPS` payload: `GSHORT id` + prop stream.
fn parse_other_player(payload: &[u8], bow_prop10: bool, accepted: Option<&IdSet>) -> Option<OtherUpdate> {
    let id = read_gshort(payload)?;
    let mut u = OtherUpdate { id, ..Default::default() };
    let text = |raw: &[u8]| latin1(raw.get(1..).unwrap_or(&[]));
    for prop in parse_player_props_in(&payload[2..], bow_prop10, accepted) {
        match prop.id {
            0 => u.nick = Some(text(&prop.raw)),
            12 => u.chat = Some(text(&prop.raw)),
            15 => u.x = prop.gchar().map(|v| v as f32 / 2.0),
            16 => u.y = prop.gchar().map(|v| v as f32 / 2.0),
            17 => u.dir = prop.gchar().map(|v| v % 4),
            50 => u.left |= prop.gchar() == Some(0),
            51 => u.left = true,
            // X2/Y2: precise position, GSHORT.
            78 => u.x = read_gshort(&prop.raw).map(super::props::decode_pos_precise),
            79 => u.y = read_gshort(&prop.raw).map(super::props::decode_pos_precise),
            _ => {}
        }
    }
    Some(u)
}

/// Graal text is single-byte (Windows code page); map each byte to the
/// same code point, which is exact for Latin-1.
pub fn latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

/// Split a `"a","b",c` list, as GServer's gtokenize writes it.
pub(super) fn split_quoted(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() || s.ends_with(',') {
        out.push(cur);
    }
    out
}

/// Read a GINT5: five GCHARs, high 7 bits first.
pub(super) fn read_gint5(b: &[u8]) -> Option<u64> {
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
    fn board_packet_behind_rawdata() {
        let n = 1 + 8192 + 1;
        let mut body = vec![
            100 + 0x20,
            ((n >> 14) & 0x7F) as u8 + 0x20,
            ((n >> 7) & 0x7F) as u8 + 0x20,
            (n & 0x7F) as u8 + 0x20,
            b'\n',
            101 + 0x20,
        ];
        for i in 0..4096u16 {
            body.extend_from_slice(&(i % 512).to_le_bytes());
        }
        body.push(b'\n');
        let mut state = SessionState::default();
        state.apply_body(&body);
        let board = state.board.expect("board");
        assert_eq!(board.len(), 4096);
        assert_eq!((board[0], board[1], board[600]), (0, 1, 88));
        assert!(state.unhandled.is_empty());
    }

    #[test]
    fn warp_to_another_level_resets_level_lists() {
        let mut state = SessionState::default();
        state.apply_body(b"&onlinestartlocal.nw\n!new1.nw 26 32 12 4 20 12.5\n");
        assert_eq!(state.links.len(), 1);
        // A pre-2.1 warp: PLO_PLAYERWARP carries the only level name.
        state.apply_body(b".HYnew1.nw\n");
        assert_eq!(state.level_name.as_deref(), Some("new1.nw"));
        assert_eq!((state.player_x, state.player_y), (Some(20.0), Some(28.5)));
        assert!(state.links.is_empty());
        // The same name again (2.1+ sends both) keeps what followed it.
        state.apply_body(b"!a 0 0 1 1 1 1\n&new1.nw\n");
        assert_eq!(state.links.len(), 1);
    }

    #[test]
    fn flags_weapons_and_world_time() {
        let mut state = SessionState::default();
        // PLO_FLAGSET (28) with and without a value.
        state.apply_body(b"<gr.fileerror=none\n<quest1\n");
        assert_eq!(state.flags.get("gr.fileerror").map(String::as_str), Some("none"));
        assert_eq!(state.flags.get("quest1").map(String::as_str), Some(""));
        // PLO_NPCWEAPONADD (33): name, NPCPROP_IMAGE, NPCPROP_SCRIPT.
        let mut add = vec![b'A', 0x20 + 3];
        add.extend_from_slice(b"Bow");
        add.extend_from_slice(&[0x20, 0x20 + 7]);
        add.extend_from_slice(b"bow.png");
        add.push(0x21);
        add.extend_from_slice(&gs(2));
        add.extend_from_slice(b"//\n");
        state.apply_body(&add);
        assert_eq!(
            state.weapons,
            vec![Weapon { name: "Bow".into(), image: Some("bow.png".into()), script: Some("//".into()) }]
        );
        // PLO_DEFAULTWEAPON (43) and PLO_NEWWORLDTIME (42).
        state.apply_body(b"K!\nK!\n");
        assert_eq!(state.default_weapons, vec![1]);
        let mut t = vec![b'J'];
        t.extend_from_slice(&[0x20, 0x20, 0x21, 0x20 + 5]);
        t.push(b'\n');
        state.apply_body(&t);
        assert_eq!(state.world_time, Some(128 + 5));
        // PLO_NPCWEAPONDEL (34).
        state.apply_body(b"BBow\n");
        assert!(state.weapons.is_empty());
        assert!(state.unhandled.is_empty());
    }

    fn gs(v: u16) -> [u8; 2] {
        [((v >> 7) as u8) + 0x20, (v & 0x7F) as u8 + 0x20]
    }

    /// `PLO_OTHERPLPROPS` for player `id` with a prop stream.
    fn other(id: u16, props: &[u8]) -> Vec<u8> {
        [&[8 + 0x20][..], &gs(id), props, b"\n"].concat()
    }

    #[test]
    fn other_player_updates_merge() {
        let mut st = SessionState::default();
        // Nick "bob" (prop 0), x 30 (15 = 60), y 12 (16 = 24), dir 3.
        let first = other(200, &[0x20, 0x23, b'b', b'o', b'b', 15 + 0x20, 60 + 0x20, 16 + 0x20, 24 + 0x20, 17 + 0x20, 3 + 0x20]);
        st.apply_body(&first);
        // Later: only the chat bubble (prop 12).
        st.apply_body(&other(200, &[12 + 0x20, 0x22, b'h', b'i']));
        let o = &st.others[0];
        assert_eq!((o.id, o.x, o.y, o.dir), (200, 30.0, 12.0, 3));
        assert_eq!(o.nick.as_deref(), Some("bob"));
        assert_eq!(o.chat.as_deref(), Some("hi"));
        assert_eq!(st.chat_log.last().map(|l| (l.from.as_str(), l.text.as_str())), Some(("bob", "hi")));

        // PCONNECTED (51) with no value: gone.
        st.apply_body(&other(200, &[51 + 0x20]));
        assert!(st.others.is_empty());
    }

    #[test]
    fn own_nickname_from_player_props() {
        let mut st = SessionState::default();
        st.apply_body(b") 'default\n");
        assert_eq!(st.nickname.as_deref(), Some("default"));
    }

    #[test]
    fn delplayer_reads_gshort() {
        let mut st = SessionState::default();
        st.apply_body(&other(300, &[15 + 0x20, 0x30]));
        st.apply_body(&[&[56 + 0x20][..], &gs(300), b"\n"].concat());
        assert!(st.others.is_empty());
    }

    #[test]
    fn toall_pm_and_admin_lines() {
        let mut st = SessionState::default();
        st.apply_body(&other(5, &[0x20, 0x21, b'z']));
        st.apply_body(&[&[13 + 0x20][..], &gs(5), b"%hello\n"].concat());
        st.apply_body(&[&[37 + 0x20][..], &gs(5), b"\"\",\"Private message:\",\"psst, \"\"x\"\"\"\n"].concat());
        st.apply_body(b"CServer message:\xa7Shutting down\n");
        let got: Vec<_> = st.chat_log.iter().map(|l| (l.kind, l.from.as_str(), l.text.as_str())).collect();
        assert_eq!(
            got,
            vec![
                (ChatKind::Toall, "z", "hello"),
                (ChatKind::Private, "z", "psst, \"x\""),
                (ChatKind::Admin, "Server message", "Shutting down"),
            ]
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

    /// 1.39 has no handler for world time (42); 2.05 does.
    #[test]
    fn version_drops_unknown_ids() {
        let body = [&[42 + 0x20][..], b"    \n", &[6 + 0x20][..], b"start.nw\n"].concat();
        let mut old = SessionState::for_version(ClientVersion::V139);
        old.apply_body(&body);
        assert_eq!(old.ignored.get(&42), Some(&1));
        assert_eq!(old.level_name.as_deref(), Some("start.nw"));
        let mut new = SessionState::for_version(ClientVersion::V205);
        new.apply_body(&body);
        assert!(new.ignored.is_empty());
    }

    /// A file inside a raw block reaches a version whose switch has 100
    /// but not 102.
    #[test]
    fn raw_block_ids_follow_rawdata() {
        let st = SessionState::for_version(ClientVersion::V139);
        assert!(st.handles(101) && st.handles(102));
        assert!(!st.handles(169));
    }

    fn msg(id: u8, payload: &[u8]) -> Vec<u8> {
        [&[id + 0x20][..], payload, b"\n"].concat()
    }

    /// The messages every public 2.22 gserver sent around the level (ids
    /// 10, 25, 39, 41, 44, 47, 156, 168, 174, 179, 180, 182), shaped like
    /// the captured ones, plus the big-map and mini-map specs.
    #[test]
    fn public_login_extras_are_decoded() {
        let mut st = SessionState::for_version(ClientVersion::V222);
        st.apply_body(&msg(6, b"start.nw"));
        let mut body = Vec::new();
        for (id, p) in [
            (25u8, &b"i"[..]),
            (44, b""),
            (168, b""),
            (171, b"bigmap2.txt,bigmap.png,0,0"),
            (172, b"minimap.txt,map2.png,0,0"),
            (10, b""),
            (174, b" "),
            (156, b"start.nw"),
            (179, b"\"Welcome to X.\",\"Credits, with a comma.\""),
            (41, b"<html><body>hi</body></html>"),
            (47, b"\"Server\",\"Manager\",\"GP\""),
            (180, b"Online,Away,DND"),
            (182, b""),
        ] {
            body.extend(msg(id, p));
        }
        // LEVELMODTIME: GINT5 1_781_284_436.
        let t: u64 = 1_781_284_436;
        let g5: Vec<u8> = (0..5).rev().map(|i| (((t >> (7 * i)) & 0x7F) as u8) + 0x20).collect();
        body.extend(msg(39, &g5));
        st.apply_body(&body);

        assert!(st.unhandled.is_empty(), "{:?}", st.unhandled);
        assert!(st.ignored.is_empty(), "{:?}", st.ignored);
        assert_eq!(st.signature, Some(73));
        assert!(st.has_npc_server && st.is_leader && !st.ghost_icon);
        assert_eq!(st.bigmap.as_ref().map(|m| m.png.as_str()), Some("bigmap.png"));
        assert_eq!(st.minimap.as_ref().map(|m| m.txt.as_str()), Some("minimap.txt"));
        assert_eq!(st.active_level.as_deref(), Some("start.nw"));
        assert_eq!(st.rpg_window, vec!["Welcome to X.", "Credits, with a comma."]);
        assert_eq!(st.start_message.as_deref(), Some("<html><body>hi</body></html>"));
        assert_eq!(st.staff_guilds, vec!["Server", "Manager", "GP"]);
        assert_eq!(st.status_list, vec!["Online", "Away", "DND"]);
        assert_eq!(st.process_list_requests, 1);
        assert_eq!(st.level_mod_time, Some(("start.nw".to_string(), 1_781_284_436)));
    }

    /// Ghost icon on, and the map specs are ignored by a version without
    /// handlers for them (2.05 has no 171 or 172).
    #[test]
    fn extras_follow_the_version() {
        let mut old = SessionState::for_version(ClientVersion::V205);
        old.apply_body(&msg(171, b"a.txt,b.png,1,2"));
        assert!(old.bigmap.is_none());
        assert_eq!(old.ignored.get(&171), Some(&1));
        let mut new = SessionState::for_version(ClientVersion::V222);
        new.apply_body(&[msg(171, b"a.txt,b.png,1,2"), msg(174, b"!")].concat());
        assert_eq!(new.bigmap.as_ref().map(|m| (m.x, m.y)), Some((1, 2)));
        assert!(new.ghost_icon);
    }

    /// No id and no payload shape may panic any version's session, and a
    /// bad payload must leave the state as it was.
    #[test]
    fn no_message_panics() {
        let payloads: [&[u8]; 6] = [b"", b" ", b"\xff", b"\x00\x00\x00", b"\"unterminated,", &[0x7f; 40]];
        for v in ClientVersion::all_versions() {
            for id in 0..=255u8 {
                for p in payloads {
                    let mut st = SessionState::for_version(*v);
                    st.apply(&Message { id, payload: p.to_vec() });
                }
            }
        }
        let mut st = SessionState::default();
        st.apply_body(&msg(171, b"not a map spec"));
        st.apply_body(&msg(39, b"abc"));
        assert!(st.bigmap.is_none() && st.level_mod_time.is_none());
    }

    /// Another player's stream stops at MAXPOWER (1) for 2.05.
    #[test]
    fn other_player_props_per_version() {
        let mut st = SessionState::for_version(ClientVersion::V205);
        st.apply_body(&other(7, &[15 + 0x20, 0x20 + 20, 1 + 0x20, 0x23, 16 + 0x20, 0x20 + 8]));
        assert_eq!(st.others[0].x, 10.0);
        assert_eq!(st.others[0].y, 0.0);
        let mut all = SessionState::default();
        all.apply_body(&other(7, &[15 + 0x20, 0x20 + 20, 1 + 0x20, 0x23, 16 + 0x20, 0x20 + 8]));
        assert_eq!(all.others[0].y, 4.0);
    }
}
