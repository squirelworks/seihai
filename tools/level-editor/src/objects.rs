//! Links, signs, chests, NPCs and baddies of an open level, as records of
//! lines. They are read with the client's level parser (`seihai::assets::Level`)
//! and edited by rewriting only their own lines, so the rest of the file
//! stays as it was.

use crate::{Document, EditError, SIZE};
use seihai::assets::Level;

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

    /// The kinds the editor can place, move and edit (baddies are only shown).
    pub const EDITABLE: [ObjectKind; 4] = [ObjectKind::Link, ObjectKind::Sign, ObjectKind::Chest, ObjectKind::Npc];
}

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
    /// The record could not be read by the client's parser (shown, not editable).
    pub unreadable: bool,
}

/// A change to one object.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectEdit {
    /// Move the marker's top-left corner to (x, y), in tiles. Links, signs and
    /// chests take whole tiles (rounded); NPCs keep the fraction.
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
    /// Up to the first [`SCRIPT_PANE_LINES`] lines of an NPC script. They
    /// replace that many lines at the top of the script; any lines after
    /// them are kept as they are.
    ScriptHead(Vec<String>),
}

/// How many script lines the NPC script pane shows and edits.
pub const SCRIPT_PANE_LINES: usize = 20;

/// The top of an NPC script, as the script pane shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptHead {
    /// The first lines of the script (at most [`SCRIPT_PANE_LINES`]).
    pub lines: Vec<String>,
    /// How many lines the whole script has.
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
        self.lines[r.first..=r.last].iter().map(|l| text(l)).collect()
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
            unreadable: false,
        };
        let head = lines.first().map(|l| l.split_whitespace().map(str::to_string).collect::<Vec<_>>()).unwrap_or_default();
        let found = level.and_then(|lv| match r.kind {
            ObjectKind::Link => lv.links.first().map(|l| {
                o.x = l.x as f32;
                o.y = l.y as f32;
                o.w = l.width as f32;
                o.h = l.height as f32;
                o.name = l.dest.clone();
                o.arrival = (head.get(6).cloned().unwrap_or_default(), head.get(7).cloned().unwrap_or_default());
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
        if found.is_none() {
            o.unreadable = true;
            // place it by its header numbers if there are any, so it can still be seen
            let at = if r.kind == ObjectKind::Link || r.kind == ObjectKind::Npc { 2 } else { 1 };
            o.x = head.get(at).and_then(|t| t.parse().ok()).unwrap_or(0.0);
            o.y = head.get(at + 1).and_then(|t| t.parse().ok()).unwrap_or(0.0);
        }
        o
    }

    /// Every link, sign, chest, NPC and baddy, in file order.
    pub fn objects(&self) -> Vec<Object> {
        self.records.iter().enumerate().map(|(i, r)| self.read_record(i, r)).collect()
    }

    /// One object by id.
    pub fn object(&self, id: usize) -> Option<Object> {
        self.records.get(id).map(|r| self.read_record(id, r))
    }

    /// The first [`SCRIPT_PANE_LINES`] lines of an NPC's script, read from
    /// the file's own lines (between the `NPC` line and `NPCEND`).
    pub fn script_head(&self, id: usize) -> Result<ScriptHead, EditError> {
        let r = *self.records.get(id).ok_or_else(|| bad(format!("there is no object {id}")))?;
        if r.kind != ObjectKind::Npc {
            return Err(bad(format!("a {} has no script", r.kind.name())));
        }
        if self.read_record(id, &r).unreadable || r.last == r.first || text(&self.lines[r.last]).trim() != "NPCEND" {
            return Err(bad("this NPC record could not be read, so it is left as it is"));
        }
        let body = &self.lines[r.first + 1..r.last];
        Ok(ScriptHead { lines: body.iter().take(SCRIPT_PANE_LINES).map(|l| text(l)).collect(), total: body.len() })
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
        let r = *self.records.get(id).ok_or_else(|| bad(format!("there is no object {id}")))?;
        if r.kind == ObjectKind::Baddy {
            return Err(bad("baddies are shown but not edited yet"));
        }
        let current = self.read_record(id, &r);
        if current.unreadable {
            return Err(bad(format!("this {} record could not be read, so it is left as it is", r.kind.name())));
        }
        if let ObjectEdit::ScriptHead(new_lines) = edit {
            return self.set_script_head(id, r, new_lines);
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
                let (x, y) = if r.kind == ObjectKind::Npc { (x, y) } else { (x.round(), y.round()) };
                let max_x = if r.kind == ObjectKind::Link { SIZE as f32 - w } else { SIZE as f32 - 1.0 };
                let max_y = if r.kind == ObjectKind::Link { SIZE as f32 - h } else { SIZE as f32 - 1.0 };
                if !(0.0..=max_x).contains(&x) || !(0.0..=max_y).contains(&y) {
                    return Err(bad(format!("({x}, {y}) is off the board for this {}", r.kind.name())));
                }
                head[xi] = num(x);
                head[yi] = num(y);
            }
            ObjectEdit::LinkDest(dest) => {
                let dest = dest.trim();
                if r.kind != ObjectKind::Link || dest.is_empty() || dest.contains(char::is_whitespace) {
                    return Err(bad("a link destination is one level name with no spaces"));
                }
                head[1] = dest.to_string();
            }
            ObjectEdit::LinkArrival(ax, ay) => {
                let (ax, ay) = (ax.trim(), ay.trim());
                let ok = |t: &str| !t.is_empty() && !t.contains(char::is_whitespace);
                if r.kind != ObjectKind::Link || !ok(ax) || !ok(ay) {
                    return Err(bad("an arrival is a number or playerx / playery (with an optional +n or -n)"));
                }
                head[6] = ax.to_string();
                head[7] = ay.to_string();
            }
            ObjectEdit::LinkSize(w, h) => {
                if r.kind != ObjectKind::Link || w < 1 || h < 1 || current.x as i32 + w > SIZE as i32 || current.y as i32 + h > SIZE as i32 {
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
                    return Err(bad("one line only; the full script editor is not there yet"));
                }
                let end = if r.kind == ObjectKind::Npc { "NPCEND" } else { "SIGNEND" };
                if text(&self.lines[r.last]).trim() != end || r.last == r.first {
                    return Err(bad(format!("this {} has no {end} line, so it is left as it is", r.kind.name())));
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
            ObjectEdit::ScriptHead(_) => unreachable!("handled above"),
        }
        let new = self.line(&head.join(" "), r.first);
        let check = Level::parse(&format!("GLEVNW01\n{}\n", String::from_utf8_lossy(strip_eol(&new)))).ok();
        if r.kind == ObjectKind::Link && check.map(|l| l.links.len()) != Some(1) {
            return Err(bad("the link would not read back; nothing was changed"));
        }
        self.splice(r.first, r.first, vec![new])
    }

    /// Replace the top of an NPC script: the lines the pane showed (up to
    /// [`SCRIPT_PANE_LINES`]) become `new_lines`; lines after them are kept.
    fn set_script_head(&mut self, id: usize, r: Record, new_lines: Vec<String>) -> Result<(), EditError> {
        let shown = self.script_head(id)?.lines.len();
        if new_lines.len() > SCRIPT_PANE_LINES {
            return Err(bad(format!("the script pane edits the first {SCRIPT_PANE_LINES} lines only; the full script editor is not there yet")));
        }
        if new_lines.iter().any(|l| l.contains(['\n', '\r'])) {
            return Err(bad("a script line cannot hold a line break"));
        }
        if new_lines.iter().any(|l| l.trim() == "NPCEND") {
            return Err(bad("a script line cannot be NPCEND; it would end the NPC early"));
        }
        // each new line keeps the line ending of the line it replaces (or the file's)
        let new: Vec<Vec<u8>> = new_lines.iter().enumerate().map(|(i, l)| self.line(l, r.first + 1 + i.min(shown.saturating_sub(1)))).collect();
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

    /// Add a link, sign, chest or NPC at (x, y) at the end of the file.
    /// Returns its id.
    pub fn add_object(&mut self, kind: ObjectKind, x: i32, y: i32) -> Result<usize, EditError> {
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
            ObjectKind::Baddy => return Err(bad("baddies are shown but not placed yet")),
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
        let r = *self.records.get(id).ok_or_else(|| bad(format!("there is no object {id}")))?;
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
