//! Links, signs, chests, NPCs and baddies of an open level, as records of
//! lines. They are read with the client's level parser (`seihai_core::assets::Level`)
//! and edited by rewriting only their own lines, so the rest of the file
//! stays as it was.

use crate::{Document, EditError, SIZE};
use seihai_core::assets::{Level, CHEST_ITEMS};

/// What a record is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Link,
    Sign,
    Chest,
    Npc,
    Baddy,
}

impl ObjectKind {
    pub fn name(self) -> &'static str {
        match self {
            ObjectKind::Link => "link",
            ObjectKind::Sign => "sign",
            ObjectKind::Chest => "chest",
            ObjectKind::Npc => "NPC",
            ObjectKind::Baddy => "baddy",
        }
    }

    /// The kinds the editor can place, move and edit.
    pub const EDITABLE: [ObjectKind; 5] = [
        ObjectKind::Link,
        ObjectKind::Sign,
        ObjectKind::Chest,
        ObjectKind::Npc,
        ObjectKind::Baddy,
    ];
}

/// The ten baddy types (0 to 9), as the classic editor lists them.
pub const BADDY_TYPE_NAMES: [&str; 10] = [
    "gray soldier",
    "blue soldier",
    "red soldier",
    "shooting soldier",
    "swamp soldier",
    "frog / hare",
    "octopus",
    "gold soldier",
    "lizardon",
    "dragon",
];

/// How many verse lines the baddy editor shows and writes.
pub const BADDY_VERSE_LINES: usize = 3;

/// Where a record sits in the file: its first and last line (inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Record {
    pub kind: ObjectKind,
    pub first: usize,
    pub last: usize,
}

/// One object as the client reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    /// Index in file order; changes when objects are added or deleted.
    pub id: usize,
    pub kind: ObjectKind,
    /// Position in tiles (the top-left corner of the marker).
    pub x: f32,
    pub y: f32,
    /// Marker size in tiles: a link's rectangle; 2 x 1 for a sign; 2 x 2 otherwise.
    pub w: f32,
    pub h: f32,
    /// Link: destination. NPC: image (`-` for none). Chest: item. Baddy: type. Sign: empty.
    pub name: String,
    /// Link: the arrival tokens as written (`playerx`, `30`, `playery+10`).
    pub arrival: (String, String),
    /// NPC: the first script line; sign: the first text line.
    pub first_line: String,
    /// Baddy: the verse lines between the header and `BADDYEND` (at most
    /// [`BADDY_VERSE_LINES`] are edited; any further lines are kept).
    pub verses: Vec<String>,
    /// Chest: the sign index on the `CHEST` line (`-1` for none).
    pub sign_index: i32,
    /// The record could not be read by the client's parser (shown, not editable).
    pub unreadable: bool,
}

/// A change to one object.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectEdit {
    /// Move the marker's top-left corner to (x, y), in tiles. Links, signs and
    /// chests and baddies take whole tiles (rounded); NPCs keep the fraction.
    Move(f32, f32),
    /// A link's destination level.
    LinkDest(String),
    /// A link's arrival x and y tokens: a number, or `playerx` / `playery`
    /// with an optional signed offset.
    LinkArrival(String, String),
    /// A link's rectangle size in tiles.
    LinkSize(i32, i32),
    /// The first line of an NPC script or of a sign's text.
    FirstLine(String),
    /// Every line of an NPC script (between `NPC` and `NPCEND`). Replaces
    /// the whole body; an empty list leaves only the end line.
    Script(Vec<String>),
    /// Every line of a sign's text (between `SIGN` and `SIGNEND`). Replaces
    /// the whole body; an empty list leaves only the end line.
    SignText(Vec<String>),
    /// A baddy's type (0 to 9, as written on the `BADDY` line).
    BaddyKind(String),
    /// Up to the first [`BADDY_VERSE_LINES`] verse lines of a baddy. They
    /// replace that many lines between the header and `BADDYEND`; any lines
    /// after them are kept. Types 4 to 6 (swamp, hare, octopus) have no
    /// verses in the classic editor; the pane still accepts an empty list.
    BaddyVerses(Vec<String>),
    /// A chest's item name (one of the 25 classic names, no spaces).
    ChestItem(String),
    /// A chest's sign index (`-1` for none).
    ChestSign(i32),
}

/// How many rows the script / sign text pane prefers to show at once.
pub const SCRIPT_PANE_LINES: usize = 20;

/// The body of an NPC script or a sign, as the text pane shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptHead {
    /// Every line between the header and the end marker.
    pub lines: Vec<String>,
    /// How many lines the body has (same as `lines.len()`).
    pub total: usize,
}

fn strip_eol(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn eol_of(line: &[u8]) -> &[u8] {
    &line[strip_eol(line).len()..]
}

fn text(line: &[u8]) -> String {
    String::from_utf8_lossy(strip_eol(line)).into_owned()
}

fn tokens(line: &[u8]) -> Vec<String> {
    text(line).split_whitespace().map(str::to_string).collect()
}

/// A coordinate as the level files write it: no fraction for whole numbers.
fn num(v: f32) -> String {
    format!("{v}")
}

fn bad(msg: impl Into<String>) -> EditError {
    EditError::Bounds(msg.into())
}

impl Document {
    /// The lines of one record, without line endings.
    fn record_lines(&self, r: &Record) -> Vec<String> {
        self.lines[r.first..=r.last]
            .iter()
            .map(|l| text(l))
            .collect()
    }

    /// Read one record with the client's parser.
    fn read_record(&self, id: usize, r: &Record) -> Object {
        let lines = self.record_lines(r);
        let level = Level::parse(&format!("GLEVNW01\n{}\n", lines.join("\n"))).ok();
        let mut o = Object {
            id,
            kind: r.kind,
            x: 0.0,
            y: 0.0,
            w: 2.0,
            h: 2.0,
            name: String::new(),
            arrival: (String::new(), String::new()),
            first_line: String::new(),
            verses: Vec::new(),
            sign_index: -1,
            unreadable: false,
        };
        let head = lines
            .first()
            .map(|l| l.split_whitespace().map(str::to_string).collect::<Vec<_>>())
            .unwrap_or_default();
        let found = level.and_then(|lv| match r.kind {
            ObjectKind::Link => lv.links.first().map(|l| {
                o.x = l.x as f32;
                o.y = l.y as f32;
                o.w = l.width as f32;
                o.h = l.height as f32;
                o.name = l.dest.clone();
                o.arrival = (
                    head.get(6).cloned().unwrap_or_default(),
                    head.get(7).cloned().unwrap_or_default(),
                );
            }),
            ObjectKind::Sign => lv.signs.first().map(|s| {
                o.x = s.x as f32;
                o.y = s.y as f32;
                o.w = 2.0;
                o.h = 1.0;
                o.first_line = s.text.lines().next().unwrap_or("").to_string();
            }),
            ObjectKind::Chest => lv.chests.first().map(|c| {
                o.x = c.x as f32;
                o.y = c.y as f32;
                o.name = c.item.clone();
                o.sign_index = c.sign_index;
            }),
            ObjectKind::Npc => lv.npcs.first().map(|n| {
                o.x = n.x;
                o.y = n.y;
                o.name = n.image.clone();
                o.first_line = n.script.lines().next().unwrap_or("").to_string();
            }),
            ObjectKind::Baddy => lv.baddies.first().map(|b| {
                o.x = b.x;
                o.y = b.y;
                o.name = b.kind.clone();
            }),
        });
        if r.kind == ObjectKind::Baddy && found.is_some() && r.last > r.first {
            // read verse lines from the file so empty trailing lines are kept
            let end = text(&self.lines[r.last]).trim() == "BADDYEND";
            let last_body = if end { lines.len() - 1 } else { lines.len() };
            o.verses = lines[1..last_body]
                .iter()
                .take(BADDY_VERSE_LINES)
                .cloned()
                .collect();
        }
        if found.is_none() {
            o.unreadable = true;
            // place it by its header numbers if there are any, so it can still be seen
            let at = if r.kind == ObjectKind::Link || r.kind == ObjectKind::Npc {
                2
            } else {
                1
            };
            o.x = head.get(at).and_then(|t| t.parse().ok()).unwrap_or(0.0);
            o.y = head.get(at + 1).and_then(|t| t.parse().ok()).unwrap_or(0.0);
        }
        o
    }

    /// Every link, sign, chest, NPC and baddy, in file order.
    pub fn objects(&self) -> Vec<Object> {
        if let Some(level) = self.binary_level() {
            return objects_from_level(level);
        }
        self.records
            .iter()
            .enumerate()
            .map(|(i, r)| self.read_record(i, r))
            .collect()
    }

    /// One object by id.
    pub fn object(&self, id: usize) -> Option<Object> {
        self.objects().into_iter().find(|o| o.id == id)
    }

    fn require_text_objects(&self) -> Result<(), EditError> {
        if self.is_binary() {
            return Err(bad(
                "object editing of binary levels is not yet supported; paint and save work",
            ));
        }
        Ok(())
    }

    /// Every line of an NPC's script, read from the file's own lines
    /// (between the `NPC` line and `NPCEND`).
    pub fn script_head(&self, id: usize) -> Result<ScriptHead, EditError> {
        self.require_text_objects()?;
        self.text_body(id, ObjectKind::Npc)
    }

    /// Every line of a sign's text, read from the file's own lines
    /// (between the `SIGN` line and `SIGNEND`).
    pub fn sign_text(&self, id: usize) -> Result<ScriptHead, EditError> {
        self.require_text_objects()?;
        self.text_body(id, ObjectKind::Sign)
    }

    fn text_body(&self, id: usize, kind: ObjectKind) -> Result<ScriptHead, EditError> {
        let r = *self
            .records
            .get(id)
            .ok_or_else(|| bad(format!("there is no object {id}")))?;
        if r.kind != kind {
            return Err(bad(match kind {
                ObjectKind::Npc => format!("a {} has no script", r.kind.name()),
                ObjectKind::Sign => format!("a {} has no sign text", r.kind.name()),
                _ => format!("a {} has no text body", r.kind.name()),
            }));
        }
        let end = match kind {
            ObjectKind::Npc => "NPCEND",
            ObjectKind::Sign => "SIGNEND",
            _ => unreachable!(),
        };
        if self.read_record(id, &r).unreadable
            || r.last == r.first
            || text(&self.lines[r.last]).trim() != end
        {
            return Err(bad(format!(
                "this {} record could not be read, so it is left as it is",
                kind.name()
            )));
        }
        let body = &self.lines[r.first + 1..r.last];
        Ok(ScriptHead {
            lines: body.iter().map(|l| text(l)).collect(),
            total: body.len(),
        })
    }

    /// Replace the lines `first..=last` with `new` and read the file again,
    /// so record and row positions stay right.
    fn splice(&mut self, first: usize, last: usize, new: Vec<Vec<u8>>) -> Result<(), EditError> {
        let removed = last + 1 - first;
        let added = new.len();
        self.lines.splice(first..=last, new);
        for b in &mut self.board_lines {
            if *b > last {
                *b = *b + added - removed;
            }
        }
        // records are re-found by the parse below; the board (painted or not) is kept by to_bytes
        let bytes = self.to_bytes();
        let mut again = Document::parse(&bytes)?;
        again.edited = true;
        *self = again;
        Ok(())
    }

    fn line(&self, content: &str, like: usize) -> Vec<u8> {
        let mut l = content.as_bytes().to_vec();
        let eol = eol_of(&self.lines[like]);
        l.extend_from_slice(if eol.is_empty() { self.eol } else { eol });
        l
    }

    /// Change one object. Only its own lines are rewritten.
    pub fn edit_object(&mut self, id: usize, edit: ObjectEdit) -> Result<(), EditError> {
        self.require_text_objects()?;
        let r = *self
            .records
            .get(id)
            .ok_or_else(|| bad(format!("there is no object {id}")))?;
        let current = self.read_record(id, &r);
        if current.unreadable {
            return Err(bad(format!(
                "this {} record could not be read, so it is left as it is",
                r.kind.name()
            )));
        }
        match &edit {
            ObjectEdit::Script(new_lines) => {
                return self.set_text_body(id, r, ObjectKind::Npc, new_lines.clone());
            }
            ObjectEdit::SignText(new_lines) => {
                return self.set_text_body(id, r, ObjectKind::Sign, new_lines.clone());
            }
            ObjectEdit::BaddyVerses(new_lines) => {
                return self.set_baddy_verses(id, r, new_lines.clone());
            }
            _ => {}
        }
        let mut head = tokens(&self.lines[r.first]);
        // header positions of x and y
        let (xi, yi) = match r.kind {
            ObjectKind::Link | ObjectKind::Npc => (2, 3),
            _ => (1, 2),
        };
        match edit {
            ObjectEdit::Move(x, y) => {
                let (w, h) = (current.w, current.h);
                let (x, y) = if r.kind == ObjectKind::Npc {
                    (x, y)
                } else {
                    (x.round(), y.round())
                };
                let max_x = if r.kind == ObjectKind::Link {
                    SIZE as f32 - w
                } else {
                    SIZE as f32 - 1.0
                };
                let max_y = if r.kind == ObjectKind::Link {
                    SIZE as f32 - h
                } else {
                    SIZE as f32 - 1.0
                };
                if !(0.0..=max_x).contains(&x) || !(0.0..=max_y).contains(&y) {
                    return Err(bad(format!(
                        "({x}, {y}) is off the board for this {}",
                        r.kind.name()
                    )));
                }
                head[xi] = num(x);
                head[yi] = num(y);
            }
            ObjectEdit::LinkDest(dest) => {
                let dest = dest.trim();
                if r.kind != ObjectKind::Link
                    || dest.is_empty()
                    || dest.contains(char::is_whitespace)
                {
                    return Err(bad("a link destination is one level name with no spaces"));
                }
                head[1] = dest.to_string();
            }
            ObjectEdit::LinkArrival(ax, ay) => {
                let (ax, ay) = (ax.trim(), ay.trim());
                let ok = |t: &str| !t.is_empty() && !t.contains(char::is_whitespace);
                if r.kind != ObjectKind::Link || !ok(ax) || !ok(ay) {
                    return Err(bad(
                        "an arrival is a number or playerx / playery (with an optional +n or -n)",
                    ));
                }
                head[6] = ax.to_string();
                head[7] = ay.to_string();
            }
            ObjectEdit::LinkSize(w, h) => {
                if r.kind != ObjectKind::Link
                    || w < 1
                    || h < 1
                    || current.x as i32 + w > SIZE as i32
                    || current.y as i32 + h > SIZE as i32
                {
                    return Err(bad("a link is at least 1 x 1 and stays on the board"));
                }
                head[4] = w.to_string();
                head[5] = h.to_string();
            }
            ObjectEdit::FirstLine(line) => {
                if !matches!(r.kind, ObjectKind::Npc | ObjectKind::Sign) {
                    return Err(bad("only NPCs and signs have text lines"));
                }
                if line.contains(['\n', '\r']) {
                    return Err(bad(
                        "one line only; use the full text pane for more than one line",
                    ));
                }
                let end = if r.kind == ObjectKind::Npc {
                    "NPCEND"
                } else {
                    "SIGNEND"
                };
                if text(&self.lines[r.last]).trim() != end || r.last == r.first {
                    return Err(bad(format!(
                        "this {} has no {end} line, so it is left as it is",
                        r.kind.name()
                    )));
                }
                let new = self.line(&line, r.first);
                return if r.first + 1 < r.last {
                    self.splice(r.first + 1, r.first + 1, vec![new])
                } else {
                    // no body yet: the line goes before the end line
                    let end_line = self.lines[r.last].clone();
                    self.splice(r.last, r.last, vec![new, end_line])
                };
            }
            ObjectEdit::BaddyKind(kind) => {
                let kind = kind.trim();
                let n: u32 = kind
                    .parse()
                    .map_err(|_| bad("a baddy type is a number 0 to 9"))?;
                if r.kind != ObjectKind::Baddy || n > 9 || kind.contains(char::is_whitespace) {
                    return Err(bad("a baddy type is a number 0 to 9"));
                }
                if head.len() < 4 {
                    return Err(bad(
                        "this baddy header is incomplete, so it is left as it is",
                    ));
                }
                head[3] = kind.to_string();
            }
            ObjectEdit::ChestItem(item) => {
                let item = item.trim();
                if r.kind != ObjectKind::Chest
                    || item.is_empty()
                    || item.contains(char::is_whitespace)
                    || !CHEST_ITEMS.contains(&item)
                {
                    return Err(bad(
                        "a chest item is one of the 25 classic names (for example greenrupee)",
                    ));
                }
                if head.len() < 5 {
                    return Err(bad("this chest line is incomplete, so it is left as it is"));
                }
                head[3] = item.to_string();
            }
            ObjectEdit::ChestSign(sign) => {
                if r.kind != ObjectKind::Chest || sign < -1 {
                    return Err(bad(
                        "a chest sign index is -1 for none, or 0 and up for a sign",
                    ));
                }
                if head.len() < 5 {
                    return Err(bad("this chest line is incomplete, so it is left as it is"));
                }
                head[4] = sign.to_string();
            }
            ObjectEdit::Script(_) | ObjectEdit::SignText(_) | ObjectEdit::BaddyVerses(_) => {
                unreachable!("handled above")
            }
        }
        let new = self.line(&head.join(" "), r.first);
        let check = Level::parse(&format!(
            "GLEVNW01\n{}\n",
            String::from_utf8_lossy(strip_eol(&new))
        ))
        .ok();
        if r.kind == ObjectKind::Link && check.map(|l| l.links.len()) != Some(1) {
            return Err(bad("the link would not read back; nothing was changed"));
        }
        self.splice(r.first, r.first, vec![new])
    }

    /// Replace the top of a baddy's verses: the lines the pane showed (up to
    /// [`BADDY_VERSE_LINES`]) become `new_lines`; lines after them are kept.
    fn set_baddy_verses(
        &mut self,
        id: usize,
        r: Record,
        new_lines: Vec<String>,
    ) -> Result<(), EditError> {
        if r.kind != ObjectKind::Baddy {
            return Err(bad(format!("a {} has no verses", r.kind.name())));
        }
        let current = self.read_record(id, &r);
        if current.unreadable || text(&self.lines[r.last]).trim() != "BADDYEND" || r.last == r.first
        {
            return Err(bad(
                "this baddy record could not be read, so it is left as it is",
            ));
        }
        if new_lines.len() > BADDY_VERSE_LINES {
            return Err(bad(format!(
                "a baddy has at most {BADDY_VERSE_LINES} verse lines"
            )));
        }
        if new_lines.iter().any(|l| l.contains(['\n', '\r'])) {
            return Err(bad("a verse line cannot hold a line break"));
        }
        if new_lines.iter().any(|l| l.trim() == "BADDYEND") {
            return Err(bad(
                "a verse line cannot be BADDYEND; it would end the baddy early",
            ));
        }
        let shown = current.verses.len();
        let new: Vec<Vec<u8>> = new_lines
            .iter()
            .enumerate()
            .map(|(i, l)| self.line(l, r.first + 1 + i.min(shown.saturating_sub(1))))
            .collect();
        if shown == 0 {
            if new.is_empty() {
                return Ok(());
            }
            let mut with_end = new;
            with_end.push(self.lines[r.last].clone());
            return self.splice(r.last, r.last, with_end);
        }
        let (first, last) = (r.first + 1, r.first + shown);
        self.splice(first, last, new)
    }

    /// Replace the whole body of an NPC script or a sign's text. Each new
    /// line keeps the line ending of the line it replaces (or the file's).
    fn set_text_body(
        &mut self,
        id: usize,
        r: Record,
        kind: ObjectKind,
        new_lines: Vec<String>,
    ) -> Result<(), EditError> {
        if r.kind != kind {
            return Err(bad(match kind {
                ObjectKind::Npc => format!("a {} has no script", r.kind.name()),
                ObjectKind::Sign => format!("a {} has no sign text", r.kind.name()),
                _ => format!("a {} has no text body", r.kind.name()),
            }));
        }
        let end = match kind {
            ObjectKind::Npc => "NPCEND",
            ObjectKind::Sign => "SIGNEND",
            _ => unreachable!(),
        };
        let body = self.text_body(id, kind)?;
        let old = body.lines.len();
        if new_lines.iter().any(|l| l.contains(['\n', '\r'])) {
            return Err(bad("a text line cannot hold a line break"));
        }
        if new_lines.iter().any(|l| l.trim() == end) {
            return Err(bad(format!(
                "a text line cannot be {end}; it would end the {} early",
                kind.name()
            )));
        }
        // each new line keeps the line ending of the line it replaces (or the file's)
        let new: Vec<Vec<u8>> = new_lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let like = if old > 0 {
                    r.first + 1 + i.min(old - 1)
                } else {
                    r.first
                };
                self.line(l, like)
            })
            .collect();
        if old == 0 {
            if new.is_empty() {
                return Ok(());
            }
            let mut with_end = new;
            with_end.push(self.lines[r.last].clone());
            return self.splice(r.last, r.last, with_end);
        }
        // replace every body line; the end marker stays
        self.splice(r.first + 1, r.last - 1, new)
    }

    /// Add a link, sign, chest, NPC or baddy at (x, y) at the end of the file.
    /// Returns its id.
    pub fn add_object(&mut self, kind: ObjectKind, x: i32, y: i32) -> Result<usize, EditError> {
        self.require_text_objects()?;
        if !(0..SIZE as i32).contains(&x) || !(0..SIZE as i32).contains(&y) {
            return Err(bad(format!("({x}, {y}) is off the board")));
        }
        let content: Vec<String> = match kind {
            ObjectKind::Link => {
                let (w, h) = ((SIZE as i32 - x).min(2), (SIZE as i32 - y).min(2));
                vec![format!("LINK level.nw {x} {y} {w} {h} playerx playery")]
            }
            ObjectKind::Sign => vec![format!("SIGN {x} {y}"), String::new(), "SIGNEND".into()],
            ObjectKind::Chest => vec![format!("CHEST {x} {y} greenrupee -1")],
            ObjectKind::Npc => vec![format!("NPC - {x} {y}"), String::new(), "NPCEND".into()],
            ObjectKind::Baddy => vec![
                format!("BADDY {x} {y} 0"),
                String::new(),
                String::new(),
                String::new(),
                "BADDYEND".into(),
            ],
        };
        let last = self.lines.len() - 1;
        let mut tail = self.lines[last].clone();
        if !tail.ends_with(b"\n") {
            tail.extend_from_slice(self.eol);
        }
        let mut new = vec![tail];
        for c in content {
            let mut l = c.into_bytes();
            l.extend_from_slice(self.eol);
            new.push(l);
        }
        let before = self.records.len();
        self.splice(last, last, new)?;
        if self.records.len() != before + 1 {
            return Err(bad("the new record did not read back"));
        }
        Ok(self.records.len() - 1)
    }

    /// Remove one object's lines.
    pub fn delete_object(&mut self, id: usize) -> Result<(), EditError> {
        self.require_text_objects()?;
        let r = *self
            .records
            .get(id)
            .ok_or_else(|| bad(format!("there is no object {id}")))?;
        if r.first == 0 {
            return Err(bad("the header is not an object"));
        }
        self.lines.drain(r.first..=r.last);
        let removed = r.last + 1 - r.first;
        for b in &mut self.board_lines {
            if *b > r.last {
                *b -= removed;
            }
        }
        let bytes = self.to_bytes();
        let mut again = Document::parse(&bytes)?;
        again.edited = true;
        *self = again;
        Ok(())
    }
}

/// Markers from a binary level's core `Level` (read-only in the editor for now).
fn objects_from_level(level: &Level) -> Vec<Object> {
    let mut out = Vec::new();
    let mut id = 0usize;
    for l in &level.links {
        out.push(Object {
            id,
            kind: ObjectKind::Link,
            x: l.x as f32,
            y: l.y as f32,
            w: l.width as f32,
            h: l.height as f32,
            name: l.dest.clone(),
            arrival: (
                l.new_x
                    .map(|v| format!("{v}"))
                    .unwrap_or_else(|| "playerx".into()),
                l.new_y
                    .map(|v| format!("{v}"))
                    .unwrap_or_else(|| "playery".into()),
            ),
            first_line: String::new(),
            unreadable: false,
        });
        id += 1;
    }
    for s in &level.signs {
        out.push(Object {
            id,
            kind: ObjectKind::Sign,
            x: s.x as f32,
            y: s.y as f32,
            w: 2.0,
            h: 1.0,
            name: String::new(),
            arrival: (String::new(), String::new()),
            first_line: s.text.lines().next().unwrap_or("").to_string(),
            unreadable: false,
        });
        id += 1;
    }
    for c in &level.chests {
        out.push(Object {
            id,
            kind: ObjectKind::Chest,
            x: c.x as f32,
            y: c.y as f32,
            w: 2.0,
            h: 2.0,
            name: c.item.clone(),
            arrival: (String::new(), String::new()),
            first_line: String::new(),
            unreadable: false,
        });
        id += 1;
    }
    for n in &level.npcs {
        out.push(Object {
            id,
            kind: ObjectKind::Npc,
            x: n.x,
            y: n.y,
            w: 2.0,
            h: 2.0,
            name: n.image.clone(),
            arrival: (String::new(), String::new()),
            first_line: n.script.lines().next().unwrap_or("").to_string(),
            unreadable: false,
        });
        id += 1;
    }
    for b in &level.baddies {
        out.push(Object {
            id,
            kind: ObjectKind::Baddy,
            x: b.x,
            y: b.y,
            w: 2.0,
            h: 2.0,
            name: b.kind.clone(),
            arrival: (String::new(), String::new()),
            first_line: String::new(),
            unreadable: false,
        });
        id += 1;
    }
    out
}
