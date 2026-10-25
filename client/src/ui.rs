//! In-game overlays: chat window and input line, chat bubbles, nickname
//! tags, the player list and the debug overlay.
//!
//! Everything draws onto a finished scene with the 5×7 font, so the
//! overlays work the same offline and online.

use crate::chatlists::{ChatFilter, Identity, NameList};
use crate::protocol::{ChatKind, ChatLine, OtherPlayer};
use crate::render::font::{text_width, CELL_H, CELL_W};
use crate::render::{Framebuffer, TILE_PX};

/// Chat history lines shown at once.
pub const CHAT_LINES: usize = 6;

/// What the input line is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Chat bar: a bubble, or a toall with a `toall:` prefix.
    Chat,
    /// Private message to one player.
    Pm { id: u16, name: String },
}

/// What a finished chat-bar line should send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatSend {
    /// CURCHAT bubble.
    Say(String),
    /// `PLI_TOALL`.
    Toall(String),
}

/// Sort a chat-bar line the way 2.05 does: text that starts with
/// `toall:` (any case, after trimming) goes out as a toall.
pub fn classify_chat(line: &str) -> ChatSend {
    let t = line.trim();
    match t.get(..6) {
        Some(p) if p.eq_ignore_ascii_case("toall:") => ChatSend::Toall(t[6..].trim().to_string()),
        _ => ChatSend::Say(line.to_string()),
    }
}

/// Typed lines kept for Up/Down recall (1.39 keeps 100).
pub const RECALL_LINES: usize = 100;

/// The text being typed.
#[derive(Debug, Default)]
pub struct Input {
    pub mode: Option<InputMode>,
    pub text: String,
    /// Earlier submitted lines, oldest first.
    recall: Vec<String>,
    /// Index into `recall` while stepping through it.
    recall_pos: Option<usize>,
}

/// What a chat key press does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatKeyAction {
    None,
    /// Open the chat line.
    Open,
    /// Send what is typed and close the line.
    Submit,
}

/// The chat keys. Enter (either Enter key) or T opens the line when it is
/// closed, and Enter sends it when it is open. While it is open `T` is just a
/// letter. The real clients open the line on Enter and on T and send the
/// typed text as the CURCHAT player prop.
pub fn chat_key_action(open: bool, enter: bool, talk: bool) -> ChatKeyAction {
    match (open, enter, talk) {
        (true, true, _) => ChatKeyAction::Submit,
        (false, true, _) | (false, false, true) => ChatKeyAction::Open,
        _ => ChatKeyAction::None,
    }
}

impl Input {
    pub fn open(&mut self, mode: InputMode) {
        self.mode = Some(mode);
        self.text.clear();
        self.recall_pos = None;
    }

    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }

    /// Add typed characters. Control characters are dropped.
    pub fn type_char(&mut self, c: char) {
        if self.is_open() && !c.is_control() && self.text.len() < 200 {
            self.text.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.text.pop();
    }

    /// Close the line and return what was typed with its mode.
    pub fn submit(&mut self) -> Option<(InputMode, String)> {
        let mode = self.mode.take()?;
        let text = std::mem::take(&mut self.text);
        if !text.trim().is_empty() {
            self.recall.push(text.clone());
            if self.recall.len() > RECALL_LINES {
                self.recall.remove(0);
            }
        }
        self.recall_pos = None;
        Some((mode, text))
    }

    /// Up: put the previous submitted line in the input.
    pub fn recall_prev(&mut self) {
        if !self.is_open() || self.recall.is_empty() {
            return;
        }
        let i = match self.recall_pos {
            Some(i) => i.saturating_sub(1),
            None => self.recall.len() - 1,
        };
        self.recall_pos = Some(i);
        self.text = self.recall[i].clone();
    }

    /// Down: step forward again; past the newest line the input empties.
    pub fn recall_next(&mut self) {
        let Some(i) = self.recall_pos else { return };
        if i + 1 < self.recall.len() {
            self.recall_pos = Some(i + 1);
            self.text = self.recall[i + 1].clone();
        } else {
            self.recall_pos = None;
            self.text.clear();
        }
    }

    pub fn cancel(&mut self) {
        self.mode = None;
        self.text.clear();
    }
}

fn kind_colour(kind: ChatKind) -> [u8; 3] {
    match kind {
        ChatKind::Say => [0xFF, 0xFF, 0xFF],
        ChatKind::Toall => [0xFF, 0xE0, 0x70],
        ChatKind::Private => [0x90, 0xE0, 0xFF],
        ChatKind::Admin => [0xFF, 0x90, 0x90],
    }
}

/// Fit `text` into `max_px` pixels, cutting it with `..` if needed.
fn fit(text: &str, max_px: i32) -> String {
    let max = (max_px / CELL_W).max(2) as usize;
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut s: String = text.chars().take(max - 2).collect();
        s.push_str("..");
        s
    }
}

/// Draw the chat window at the bottom. Always shows a bar.
///
/// The last [`CHAT_LINES`] lines appear above a permanent input line.
pub fn draw_chat(fb: &mut Framebuffer, log: &[ChatLine], scroll: usize, input: &Input) {
    draw_chat_lines(fb, log, scroll, input, CHAT_LINES)
}

/// [`draw_chat`] with `lines` history lines; 0 shows only the input line.
pub fn draw_chat_lines(fb: &mut Framebuffer, log: &[ChatLine], scroll: usize, input: &Input, lines: usize) {
    let w = fb.width as i32;
    let line_h = CELL_H + 1;
    let input_h = line_h + 4;
    let end = log.len().saturating_sub(scroll);
    let start = end.saturating_sub(lines);
    let shown = &log[start..end];
    let h = shown.len() as i32 * line_h + input_h;
    let top = fb.height as i32 - h - 2;
    fb.shade_rect(0, top - 2, w, h + 4, 140);
    for (i, l) in shown.iter().enumerate() {
        let text = match l.kind {
            ChatKind::Say => format!("{}: {}", l.from, l.text),
            ChatKind::Toall => format!("{} (toall): {}", l.from, l.text),
            ChatKind::Private => format!("{} (PM): {}", l.from, l.text),
            ChatKind::Admin => format!("{}: {}", l.from, l.text),
        };
        fb.draw_text(4, top + i as i32 * line_h, &fit(&text, w - 8), kind_colour(l.kind));
    }
    if scroll > 0 {
        let more = format!("^{scroll}");
        fb.draw_text(w - 4 - text_width(&more), top, &more, [0xA0, 0xA0, 0xA0]);
    }
    let y = top + shown.len() as i32 * line_h + 1;
    fb.fill_rect(0, y - 1, w, 1, [0x60, 0x60, 0x60]);
    let line = if let Some(mode) = &input.mode {
        let prompt = match mode {
            InputMode::Chat => String::from("> "),
            InputMode::Pm { name, .. } => format!("PM {name}> "),
        };
        format!("{prompt}{}_", input.text)
    } else {
        String::from("> ")
    };
    let max = ((w - 8) / CELL_W) as usize;
    let n = line.chars().count();
    let line: String = line.chars().skip(n.saturating_sub(max)).collect();
    fb.draw_text(4, y + 1, &line, [0xFF, 0xFF, 0xFF]);
}

/// Screen position of a level point, given the camera origin in pixels.
fn to_screen(x: f32, y: f32, cam: (i32, i32)) -> (i32, i32) {
    (
        (x * TILE_PX as f32) as i32 - cam.0,
        (y * TILE_PX as f32) as i32 - cam.1,
    )
}

/// Draw a chat bubble centred over a 48×48 player box at `(dx, dy)`.
pub fn draw_bubble(fb: &mut Framebuffer, text: &str, dx: i32, dy: i32) {
    if text.is_empty() {
        return;
    }
    let text = fit(text, 240);
    let tw = text_width(&text);
    let x = dx + 24 - tw / 2;
    let y = dy - CELL_H - 2;
    fb.shade_rect(x - 2, y - 2, tw + 3, CELL_H + 3, 90);
    fb.draw_text(x, y, &text, [0xFF, 0xFF, 0xFF]);
}

/// Nickname tags and chat bubbles for other players.
pub fn draw_other_labels(fb: &mut Framebuffer, others: &[OtherPlayer], cam: (i32, i32)) {
    for o in others {
        let (dx, dy) = to_screen(o.x, o.y, cam);
        if let Some(nick) = &o.nick {
            fb.draw_nick(nick, dx, dy);
        }
        if let Some(chat) = &o.chat {
            draw_bubble(fb, chat, dx, dy);
        }
    }
}

/// The rows of the player list: a title, ourselves, then each other
/// player with `[buddy]`, `[ignored]` and `[muted]` marks from the lists.
/// A buddy mark is also the online marker: only players in the level
/// are listed.
pub fn player_list_rows(me: &str, others: &[OtherPlayer], buddy: &NameList, filter: &ChatFilter) -> Vec<String> {
    let mut rows = vec![format!("Players ({})", others.len() + 1), format!("  {me} (you)")];
    for o in others {
        let name = o.nick.clone().unwrap_or_default();
        let who = Identity { account: o.account.clone(), nick: o.nick.clone() };
        let mut row = format!("  {name} #{} {:.1},{:.1}", o.id, o.x, o.y);
        if who.account.as_deref().or(who.nick.as_deref()).is_some_and(|k| buddy.contains(k)) {
            row.push_str(" [buddy]");
        }
        if filter.ignored(&who) {
            row.push_str(" [ignored]");
        }
        if filter.muted(&who) {
            row.push_str(" [muted]");
        }
        rows.push(row);
    }
    rows
}

/// Player list window in the top-right corner.
pub fn draw_player_list(fb: &mut Framebuffer, me: &str, others: &[OtherPlayer], buddy: &NameList, filter: &ChatFilter) {
    let line_h = CELL_H + 2;
    let rows = player_list_rows(me, others, buddy, filter);
    let w = rows.iter().map(|r| text_width(r)).max().unwrap_or(0).min(fb.width as i32 - 16) + 8;
    let x = fb.width as i32 - w - 4;
    let y = 72;
    fb.shade_rect(x, y, w, rows.len() as i32 * line_h + 6, 90);
    for (i, r) in rows.iter().enumerate() {
        let colour = if i == 0 {
            [0xFF, 0xE0, 0x70]
        } else if r.contains("[ignored]") || r.contains("[muted]") {
            [0xFF, 0x70, 0x70]
        } else if r.contains("[buddy]") {
            [0x90, 0xFF, 0x90]
        } else {
            [0xFF, 0xFF, 0xFF]
        };
        fb.draw_text(x + 4, y + 4 + i as i32 * line_h, &fit(r, w - 8), colour);
    }
}

/// Numbers for the debug overlay.
#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub fps: f32,
    pub rupees: i32,
    pub bombs: i32,
    pub darts: i32,
    pub x: f32,
    pub y: f32,
    pub dir: u8,
    pub level: String,
    pub frames: usize,
    pub messages: usize,
    pub others: usize,
    pub online: bool,
}

/// Debug overlay in the top-left corner, under the HUD bar.
pub fn draw_debug(fb: &mut Framebuffer, d: &DebugInfo) {
    let rows = [
        format!("fps {:.0}", d.fps),
        format!("pos {:.1},{:.1} dir {}", d.x, d.y, d.dir),
        format!("level {}", if d.level.is_empty() { "-" } else { &d.level }),
        if d.online {
            format!("frames {} msgs {}", d.frames, d.messages)
        } else {
            String::from("offline")
        },
        format!("players {}", d.others + 1),
        format!("rupees {} bombs {} arrows {}", d.rupees, d.bombs, d.darts),
    ];
    let w = rows.iter().map(|r| text_width(r)).max().unwrap_or(0) + 8;
    let line_h = CELL_H + 2;
    fb.shade_rect(4, 20, w, rows.len() as i32 * line_h + 6, 90);
    for (i, r) in rows.iter().enumerate() {
        fb.draw_text(8, 24 + i as i32 * line_h, r, [0xB0, 0xFF, 0xB0]);
    }
}

/// Reading panel for a sign or a chest note, in the upper middle.
pub fn draw_panel(fb: &mut Framebuffer, title: &str, text: &str) {
    let w = (fb.width as i32 - 80).min(400);
    let max_chars = ((w - 16) / CELL_W) as usize;
    let mut lines: Vec<String> = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        for word in para.split(' ') {
            if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > max_chars {
                lines.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
        lines.push(cur);
    }
    lines.truncate(12);
    let line_h = CELL_H + 2;
    let h = (lines.len() as i32 + 2) * line_h + 10;
    let x = (fb.width as i32 - w) / 2;
    let y = 60;
    fb.shade_rect(x, y, w, h, 60);
    fb.fill_rect(x, y, w, 1, [0xC0, 0xC0, 0xC0]);
    fb.fill_rect(x, y + h - 1, w, 1, [0xC0, 0xC0, 0xC0]);
    fb.draw_text(x + 8, y + 6, title, [0xFF, 0xE0, 0x70]);
    for (i, l) in lines.iter().enumerate() {
        fb.draw_text(x + 8, y + 6 + (i as i32 + 1) * line_h, &fit(l, w - 16), [0xFF, 0xFF, 0xFF]);
    }
    let hint = "A / Space to close";
    fb.draw_text(
        x + w - 8 - text_width(hint),
        y + h - 4 - CELL_H,
        hint,
        [0xA0, 0xA0, 0xA0],
    );
}

/// The other player nearest to `(x, y)`, for F2.
pub fn nearest(others: &[OtherPlayer], x: f32, y: f32) -> Option<&OtherPlayer> {
    others.iter().min_by(|a, b| {
        let da = (a.x - x).powi(2) + (a.y - y).powi(2);
        let db = (b.x - x).powi(2) + (b.y - y).powi(2);
        da.total_cmp(&db)
    })
}

#[cfg(test)]
mod chat_key_tests {
    use super::*;

    #[test]
    fn enter_and_t_open_and_enter_sends() {
        assert_eq!(chat_key_action(false, true, false), ChatKeyAction::Open);
        assert_eq!(chat_key_action(false, false, true), ChatKeyAction::Open);
        assert_eq!(chat_key_action(false, false, false), ChatKeyAction::None);
        assert_eq!(chat_key_action(true, true, false), ChatKeyAction::Submit);
        // T while typing is a letter, not a key action.
        assert_eq!(chat_key_action(true, false, true), ChatKeyAction::None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_list_marks() {
        let mk = |id: u16, nick: &str, account: Option<&str>| OtherPlayer {
            id,
            x: 1.0,
            y: 2.0,
            dir: 2,
            nick: Some(nick.into()),
            chat: None,
            chat_timer: Default::default(),
            account: account.map(Into::into),
            cell: None,
            sprite: None,
            status: None,
            ap: None,
            sword_power: None,
        };
        let others = vec![mk(2, "Pal", Some("pal_acct")), mk(3, "Jerk", Some("jerk_acct")), mk(4, "Loud", None), mk(5, "Plain", None)];
        let mut buddy = NameList::default();
        buddy.add("PAL_ACCT");
        let mut f = ChatFilter::default();
        f.ignore.add("jerk_acct");
        f.mute.add("loud");
        let rows = player_list_rows("me", &others, &buddy, &f);
        assert_eq!(rows[0], "Players (5)");
        assert!(rows[2].ends_with(" [buddy]"), "{}", rows[2]);
        assert!(rows[3].ends_with(" [ignored]"), "{}", rows[3]);
        assert!(rows[4].ends_with(" [muted]"), "{}", rows[4]);
        assert!(!rows[5].contains('['), "{}", rows[5]);
    }

    #[test]
    fn toall_prefix() {
        assert_eq!(classify_chat("  ToAll: hi all "), ChatSend::Toall("hi all".into()));
        assert_eq!(classify_chat("hello"), ChatSend::Say("hello".into()));
        assert_eq!(classify_chat("toal"), ChatSend::Say("toal".into()));
    }

    #[test]
    fn input_line() {
        let mut i = Input::default();
        i.type_char('x');
        assert!(i.text.is_empty(), "typing needs an open line");
        i.open(InputMode::Chat);
        for c in "hi\u{8}!".chars() {
            i.type_char(c);
        }
        i.backspace();
        assert_eq!(i.submit(), Some((InputMode::Chat, "hi".into())));
        assert!(!i.is_open());
    }

    #[test]
    fn chat_window_draws_text() {
        let mut fb = Framebuffer::new(320, 120);
        let log = vec![ChatLine {
            kind: ChatKind::Say,
            from: "a".into(),
            text: "hello".into(),
        }];
        let mut input = Input::default();
        input.open(InputMode::Chat);
        draw_chat(&mut fb, &log, 0, &input);
        assert!(fb.pixels.chunks(3).any(|p| p == [0xFF, 0xFF, 0xFF]));
    }

    #[test]
    fn recall_steps_through_sent_lines() {
        let mut input = Input::default();
        for line in ["one", "two"] {
            input.open(InputMode::Chat);
            input.text = line.into();
            input.submit();
        }
        input.open(InputMode::Chat);
        input.recall_prev();
        assert_eq!(input.text, "two");
        input.recall_prev();
        input.recall_prev();
        assert_eq!(input.text, "one");
        input.recall_next();
        assert_eq!(input.text, "two");
        input.recall_next();
        assert_eq!(input.text, "");
    }
}
