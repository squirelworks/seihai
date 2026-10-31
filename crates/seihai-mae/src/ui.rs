//! In-game overlays: chat window and input line, chat bubbles, nickname
//! tags, the player list and the debug overlay.
//!
//! Everything draws onto a finished scene with the 5×7 font, so the
//! overlays work the same offline and online.

use crate::chatlists::{ChatFilter, Identity, NameList};
use crate::protocol::{ChatKind, ChatLine, OtherPlayer};
use crate::render::font::{text_width, CELL_H, CELL_W};
use crate::render::chattext::{draw_chat_text, draw_nick_text, ChatFont};
use crate::render::{Framebuffer, TILE_PX};

/// Chat history lines shown at once.
pub const CHAT_LINES: usize = 6;

/// How long an overlay line stays fully visible, in milliseconds.
///
/// **Assumed, not from the real client.** The 2.05 client has no on-screen
/// chat log to take this from (see `documentation/chat-log.md`).
pub const OVERLAY_VISIBLE_MS: u64 = 8000;
/// How long the fade lasts after that. **Assumed.**
pub const OVERLAY_FADE_MS: u64 = 2000;
/// How many alpha steps the fade has (the line is gone after the last).
/// **Assumed.**
pub const OVERLAY_FADE_STEPS: u64 = 4;

/// The opacity (0..=255) of a chat-overlay line `age_ms` old: full for
/// [`OVERLAY_VISIBLE_MS`], then [`OVERLAY_FADE_STEPS`] equal steps down to
/// nothing over [`OVERLAY_FADE_MS`] (204, 153, 102, 51 with the assumed
/// values), then gone.
pub fn overlay_alpha(age_ms: u64) -> u8 {
    if age_ms < OVERLAY_VISIBLE_MS {
        return 255;
    }
    let into = age_ms - OVERLAY_VISIBLE_MS;
    if into >= OVERLAY_FADE_MS {
        return 0;
    }
    let step = into * OVERLAY_FADE_STEPS / OVERLAY_FADE_MS;
    (255 * (OVERLAY_FADE_STEPS - step) / (OVERLAY_FADE_STEPS + 1)) as u8
}

/// The lines the overlay shows at chat-clock time `now_ms`, oldest first,
/// each with its opacity: the newest [`CHAT_LINES`] lines that have not
/// faded away. A new line pushes the older ones up (the newest is drawn
/// lowest).
pub fn overlay_lines(log: &[ChatLine], now_ms: u64) -> Vec<(&ChatLine, u8)> {
    let mut v: Vec<(&ChatLine, u8)> = log
        .iter()
        .rev()
        .map(|l| (l, overlay_alpha(now_ms.saturating_sub(l.born_ms))))
        .take_while(|(_, a)| *a > 0)
        .take(CHAT_LINES)
        .collect();
    v.reverse();
    v
}

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
    /// Exactly `showname`: shows our own nickname for 60 frames. Not sent.
    ShowName,
}

/// Sort a chat-bar line the way 2.05 does: text that starts with
/// `toall:` (any case, after trimming) goes out as a toall.
pub fn classify_chat(line: &str) -> ChatSend {
    let t = line.trim();
    if t.eq_ignore_ascii_case("showname") {
        return ChatSend::ShowName;
    }
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

/// What the quit prompt does with one frame's keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitKey {
    /// Y or Enter: quit.
    Quit,
    /// N or Escape: close the prompt and play on.
    Cancel,
    /// S: leave this server (or offline game) and go back to the server list.
    ServerList,
    /// Nothing for the prompt.
    None,
}

/// The quit prompt's answer. `was_open` is whether the prompt was up before
/// this frame: the Escape that opens it is still "pressed" when the prompt
/// reads its keys in the same frame, and must not close it again (that left
/// Escape doing nothing at all from 2026-10-23 until it was fixed).
pub fn quit_key(was_open: bool, yes: bool, no: bool, escape: bool) -> QuitKey {
    quit_key_with_list(was_open, yes, no, escape, false)
}

/// [`quit_key`] with the S key, which goes back to the server list.
pub fn quit_key_with_list(was_open: bool, yes: bool, no: bool, escape: bool, list: bool) -> QuitKey {
    if yes {
        QuitKey::Quit
    } else if list {
        QuitKey::ServerList
    } else if no || (was_open && escape) {
        QuitKey::Cancel
    } else {
        QuitKey::None
    }
}

#[cfg(test)]
mod quit_key_tests {
    use super::*;

    /// The Escape that opens the prompt leaves it open.
    #[test]
    fn the_opening_escape_does_not_close_it() {
        assert_eq!(quit_key(false, false, false, true), QuitKey::None);
    }

    /// A second Escape, N, Y or Enter answer it.
    #[test]
    fn later_keys_answer_it() {
        assert_eq!(quit_key(true, false, false, true), QuitKey::Cancel);
        assert_eq!(quit_key(true, false, true, false), QuitKey::Cancel);
        assert_eq!(quit_key(true, true, false, false), QuitKey::Quit);
        assert_eq!(quit_key(true, false, false, false), QuitKey::None);
    }

    /// S answers it with the server list.
    #[test]
    fn s_goes_back_to_the_server_list() {
        assert_eq!(quit_key_with_list(true, false, false, false, true), QuitKey::ServerList);
        assert_eq!(quit_key_with_list(true, false, false, false, false), QuitKey::None);
        // Y still wins if both are down.
        assert_eq!(quit_key_with_list(true, true, false, false, true), QuitKey::Quit);
    }
}

/// The text of the quit prompt.
pub const QUIT_PROMPT: &str = "Quit Seihai?  Y / Enter = yes,  N / Esc = no,  S = server list";

/// Offline, a link to a level that is not on disk is walked into on every
/// step the player takes inside it. This says so once per link: again only
/// after the player has stepped out of every link, or walked into another
/// missing one.
#[derive(Debug, Default)]
pub struct MissingLevelNotice {
    shown: Option<String>,
}

impl MissingLevelNotice {
    pub fn new() -> Self {
        Self::default()
    }

    /// The player is in a link to `dest`, which has no local file. Returns
    /// the line to show, or `None` when it was already shown.
    pub fn missing(&mut self, dest: &str) -> Option<String> {
        if self.shown.as_deref().is_some_and(|s| s.eq_ignore_ascii_case(dest)) {
            return None;
        }
        self.shown = Some(dest.to_string());
        Some(format!("{dest} is not here offline (not in your levels or the download cache)"))
    }

    /// The player is in no link (or went through one): the next missing
    /// level is told again.
    pub fn clear(&mut self) {
        self.shown = None;
    }
}

#[cfg(test)]
mod missing_level_tests {
    use super::*;

    #[test]
    fn standing_in_the_link_tells_it_once() {
        let mut n = MissingLevelNotice::new();
        assert!(n.missing("level5.nw").unwrap().contains("level5.nw"));
        for _ in 0..50 {
            assert_eq!(n.missing("level5.nw"), None);
        }
        assert_eq!(n.missing("LEVEL5.NW"), None);
    }

    #[test]
    fn stepping_out_or_another_link_tells_it_again() {
        let mut n = MissingLevelNotice::new();
        assert!(n.missing("level5.nw").is_some());
        assert!(n.missing("house1newbie.nw").is_some());
        assert!(n.missing("level5.nw").is_some());
        n.clear();
        assert!(n.missing("level5.nw").is_some());
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
        fb.draw_text(4, top + i as i32 * line_h, &fit(&chat_line_text(l), w - 8), kind_colour(l.kind));
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

fn chat_line_text(l: &ChatLine) -> String {
    match l.kind {
        ChatKind::Say => format!("{}: {}", l.from, l.text),
        ChatKind::Toall => format!("{} (toall): {}", l.from, l.text),
        ChatKind::Private => format!("{} (PM): {}", l.from, l.text),
        ChatKind::Admin => format!("{}: {}", l.from, l.text),
    }
}

/// The chat area at the bottom as the game shows it: the input bar always,
/// and the history as a fading overlay (lines older than
/// [`OVERLAY_VISIBLE_MS`] fade out over [`OVERLAY_FADE_MS`]). While the
/// input line is open, or the history is scrolled back (`scroll > 0`), the
/// full window is drawn instead and nothing fades, so older lines can be
/// read.
pub fn draw_chat_overlay(fb: &mut Framebuffer, log: &[ChatLine], now_ms: u64, scroll: usize, input: &Input) {
    if input.is_open() || scroll > 0 {
        return draw_chat(fb, log, scroll, input);
    }
    // The bar alone (no history lines) ...
    draw_chat_lines(fb, &[], 0, input, 0);
    // ... and the live lines above it, newest lowest.
    let w = fb.width as i32;
    let line_h = CELL_H + 1;
    let bar_top = fb.height as i32 - (line_h + 4) - 2 - 2;
    let lines = overlay_lines(log, now_ms);
    let n = lines.len() as i32;
    for (i, (l, alpha)) in lines.into_iter().enumerate() {
        let y = bar_top - 2 - (n - i as i32) * line_h;
        // The window's own shade is 140/256; fade it with the text.
        let keep = 256 - (116 * u16::from(alpha) / 255);
        fb.shade_rect(0, y - 1, w, line_h, keep);
        fb.draw_text_alpha(4, y, &fit(&chat_line_text(l), w - 8), kind_colour(l.kind), alpha);
    }
}

/// What typing a chat line does on our side: the line goes to the history
/// (the overlay shows it), becomes our own bubble, and starts the bubble's
/// clock held until the player moves. An empty line clears the bubble
/// without a history entry. Returns whether a line was recorded.
pub fn own_say(state: &mut crate::protocol::SessionState, who: &str, text: &str, bubble: &mut String, timer: &mut crate::chatlists::BubbleTimer) -> bool {
    *bubble = text.to_string();
    if text.is_empty() {
        *timer = crate::chatlists::BubbleTimer::default();
        return false;
    }
    *timer = crate::chatlists::BubbleTimer::start_held();
    state.push_chat(ChatKind::Say, who.to_string(), text.to_string());
    true
}

/// What a script setting the player's chat (`setplayerprop #c,...`) does on
/// our side: the text becomes our bubble with a running clock, with no
/// history entry, as the real client shows it. Returns whether it should go
/// to the server as CURCHAT: only a change does, so a script that sets the
/// same text every second does not send it every second.
pub fn script_say(text: &str, bubble: &mut String, timer: &mut crate::chatlists::BubbleTimer) -> bool {
    let changed = bubble != text;
    *bubble = text.to_string();
    *timer = if text.is_empty() { crate::chatlists::BubbleTimer::default() } else { crate::chatlists::BubbleTimer::start() };
    changed
}

/// Screen position of a level point, given the camera origin in pixels.
fn to_screen(x: f32, y: f32, cam: (i32, i32)) -> (i32, i32) {
    (
        (x * TILE_PX as f32) as i32 - cam.0,
        (y * TILE_PX as f32) as i32 - cam.1,
    )
}

/// Draw a player's chat text over the 48×48 player box at `(dx, dy)`, the
/// way the version's own client does (see [`crate::render::chattext`]).
pub fn draw_bubble(fb: &mut Framebuffer, font: Option<&ChatFont>, text: &str, dx: i32, dy: i32, carrying: bool) {
    draw_chat_text(fb, font, text, dx, dy, carrying);
}

/// Nicknames and chat text for other players. Their nickname is always up,
/// in the colour of their AP (50, white, until the server sends one).
pub fn draw_other_labels(fb: &mut Framebuffer, font: Option<&ChatFont>, others: &[OtherPlayer], cam: (i32, i32)) {
    for o in others {
        let (dx, dy) = to_screen(o.x, o.y, cam);
        if let Some(nick) = &o.nick {
            let label = crate::offline::name_label(nick, o.status);
            draw_nick_text(fb, font, &label, o.ap.unwrap_or(50), dx, dy);
        }
        if let Some(chat) = &o.chat {
            draw_bubble(fb, font, chat, dx, dy, o.carry.is_some());
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

/// What F4 / F5 do, given whether the build has the built-in editor and
/// whether we are connected to a server.
#[derive(Debug, PartialEq, Eq)]
pub enum EditorKey {
    /// Run the key (F4 editor stub, F5 level picker).
    Run,
    /// Show this line in chat instead, and do nothing else.
    Refuse(String),
}

/// Decide F4 (`is_f4`) or F5. Online the editor is refused with the real
/// client's own text (`MainWindow-Error7`, seen in 2.00, 2.17.1, 2.22 and
/// 2.31). F5 is Seihai's level window, online or offline and in every
/// build: it names the server's level, opens it (or a level file) offline,
/// and goes back to the server list. (The real 2.17.1 F5 opened a position
/// window instead.)
pub fn editor_key(has_editor: bool, label: &str, online: bool, is_f4: bool) -> EditorKey {
    if !is_f4 {
        return EditorKey::Run;
    }
    if !has_editor {
        // 2.3 moved the editor and level loading to GraalEditor.
        return EditorKey::Refuse(format!("{label} has no built-in editor"));
    }
    if online {
        return EditorKey::Refuse("You cannot open the editor in the online mode.".into());
    }
    EditorKey::Run
}

/// How long a character key is held before it repeats.
pub const REPEAT_DELAY: std::time::Duration = std::time::Duration::from_millis(500);
/// Time between repeats once a key repeats (30 a second).
pub const REPEAT_RATE: std::time::Duration = std::time::Duration::from_millis(33);

/// Repeats a held character key in the chat line where the window system
/// does not (a Wayland window gets no repeat of its own, and a Mac with
/// press-and-hold shows its accent menu instead). Where the system does
/// repeat (Windows, X11), its repeats are seen and this adds none.
///
/// `K` is the key type; the caller says which key went down with each
/// character and whether that key is still down.
#[derive(Debug, Default)]
pub struct CharRepeat<K> {
    held: Option<Held<K>>,
}

#[derive(Debug)]
struct Held<K> {
    ch: char,
    key: K,
    since: std::time::Duration,
    last: std::time::Duration,
    /// The system sent a repeat of its own: add none.
    system: bool,
}

impl<K: PartialEq + Copy> CharRepeat<K> {
    pub fn new() -> Self {
        Self { held: None }
    }

    /// One frame at time `now` (from any fixed start). `typed` is what the
    /// system sent this frame, `pressed` the key that went down this frame
    /// (if any) and `down(key)` whether a key is still held. Returns the
    /// characters to type: `typed`, plus any repeats due.
    pub fn frame(&mut self, now: std::time::Duration, typed: &[char], pressed: Option<K>, down: impl Fn(K) -> bool) -> Vec<char> {
        let mut out = typed.to_vec();
        for &c in typed {
            match (&mut self.held, pressed) {
                // A new press: this key is the one to repeat.
                (_, Some(k)) => self.held = Some(Held { ch: c, key: k, since: now, last: now, system: false }),
                // The same character with no new press: the system repeats.
                (Some(h), None) if h.ch == c && down(h.key) => {
                    h.system = true;
                    h.last = now;
                }
                _ => self.held = None,
            }
        }
        if self.held.as_ref().is_some_and(|h| !down(h.key)) {
            self.held = None;
        }
        if let Some(h) = self.held.as_mut().filter(|h| !h.system && typed.is_empty()) {
            if now >= h.since + REPEAT_DELAY {
                // The first repeat comes at the delay, then one per rate.
                let mut next = if h.last < h.since + REPEAT_DELAY { h.since + REPEAT_DELAY } else { h.last + REPEAT_RATE };
                while next <= now {
                    out.push(h.ch);
                    h.last = next;
                    next += REPEAT_RATE;
                }
            }
        }
        out
    }

    /// Forget the held key (the chat line closed).
    pub fn clear(&mut self) {
        self.held = None;
    }
}

#[cfg(test)]
mod char_repeat_tests {
    use super::*;
    use std::time::Duration;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_held_key_repeats_after_the_delay() {
        let mut r = CharRepeat::new();
        let down = |_: u8| true;
        assert_eq!(r.frame(ms(0), &['a'], Some(1u8), down), ['a']);
        assert!(r.frame(ms(400), &[], None, down).is_empty());
        assert_eq!(r.frame(ms(500), &[], None, down), ['a']);
        assert!(r.frame(ms(520), &[], None, down).is_empty());
        // 533 and 566 are due by 570.
        assert_eq!(r.frame(ms(570), &[], None, down), ['a', 'a']);
    }

    #[test]
    fn letting_go_stops_it() {
        let mut r = CharRepeat::new();
        r.frame(ms(0), &['y'], Some(7u8), |_| true);
        assert!(r.frame(ms(600), &[], None, |_| false).is_empty());
        assert!(r.frame(ms(900), &[], None, |_| true).is_empty(), "nothing held any more");
    }

    #[test]
    fn the_systems_own_repeat_is_not_doubled() {
        let mut r = CharRepeat::new();
        let down = |_: u8| true;
        r.frame(ms(0), &['g'], Some(1u8), down);
        assert_eq!(r.frame(ms(500), &['g'], None, down), ['g']);
        assert!(r.frame(ms(520), &[], None, down).is_empty());
        assert!(r.frame(ms(700), &[], None, down).is_empty());
        assert_eq!(r.frame(ms(733), &['g'], None, down), ['g']);
    }

    #[test]
    fn a_new_key_takes_over() {
        let mut r = CharRepeat::new();
        let down = |_: u8| true;
        r.frame(ms(0), &['a'], Some(1u8), down);
        r.frame(ms(300), &['y'], Some(2u8), down);
        assert!(r.frame(ms(700), &[], None, down).is_empty());
        assert_eq!(r.frame(ms(800), &[], None, down), ['y']);
    }
}

#[cfg(test)]
mod editor_key_tests {
    use super::*;

    #[test]
    fn online_refuses_the_editor() {
        assert_eq!(
            editor_key(true, "2.17", true, true),
            EditorKey::Refuse("You cannot open the editor in the online mode.".into())
        );
        assert_eq!(editor_key(false, "2.3", true, true), EditorKey::Refuse("2.3 has no built-in editor".into()));
    }

    #[test]
    fn offline_runs_and_no_editor_refuses() {
        assert_eq!(editor_key(true, "2.17", false, true), EditorKey::Run);
        assert_eq!(editor_key(false, "2.3", false, true), EditorKey::Refuse("2.3 has no built-in editor".into()));
    }

    /// F5 opens the level window online and offline, editor or not.
    #[test]
    fn f5_always_opens_the_level_window() {
        for has_editor in [false, true] {
            for online in [false, true] {
                assert_eq!(editor_key(has_editor, "2.22", online, false), EditorKey::Run);
            }
        }
    }
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
            carry: None,
            looks: Default::default(),
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
        assert_eq!(classify_chat(" ShowName "), ChatSend::ShowName);
        assert_eq!(classify_chat("showname please"), ChatSend::Say("showname please".into()));
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
            born_ms: 0,
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

    fn line(kind: ChatKind, from: &str, text: &str, born_ms: u64) -> ChatLine {
        ChatLine { kind, from: from.into(), text: text.into(), born_ms }
    }

    fn lit(fb: &Framebuffer, rgb: [u8; 3]) -> bool {
        fb.pixels.chunks(3).any(|p| p == rgb)
    }

    #[test]
    fn overlay_alpha_is_full_then_steps_down_then_gone() {
        assert_eq!(overlay_alpha(0), 255);
        assert_eq!(overlay_alpha(OVERLAY_VISIBLE_MS - 1), 255);
        // Four equal steps across the fade.
        let step = OVERLAY_FADE_MS / OVERLAY_FADE_STEPS;
        let a: Vec<u8> = (0..OVERLAY_FADE_STEPS).map(|k| overlay_alpha(OVERLAY_VISIBLE_MS + k * step)).collect();
        assert_eq!(a, [204, 153, 102, 51]);
        assert_eq!(overlay_alpha(OVERLAY_VISIBLE_MS + OVERLAY_FADE_MS - 1), 51);
        assert_eq!(overlay_alpha(OVERLAY_VISIBLE_MS + OVERLAY_FADE_MS), 0);
        assert_eq!(overlay_alpha(10_000_000), 0);
    }

    #[test]
    fn overlay_keeps_newest_lines_and_drops_faded_ones() {
        let log: Vec<ChatLine> = (0..10).map(|i| line(ChatKind::Say, "a", &format!("m{i}"), i * 100)).collect();
        // All fresh: only the newest CHAT_LINES, oldest first.
        let v = overlay_lines(&log, 1000);
        assert_eq!(v.len(), CHAT_LINES);
        assert_eq!(v[0].0.text, "m4");
        assert_eq!(v[CHAT_LINES - 1].0.text, "m9");
        assert!(v.iter().all(|(_, a)| *a == 255));
        // Late: the oldest have faded away first, the newest linger.
        let late = OVERLAY_VISIBLE_MS + OVERLAY_FADE_MS + 450;
        let v = overlay_lines(&log, late);
        assert_eq!(v.iter().map(|(l, _)| l.text.as_str()).collect::<Vec<_>>(), ["m5", "m6", "m7", "m8", "m9"]);
        assert!(v[0].1 <= v[4].1, "older lines are never brighter");
        // Long after: nothing.
        assert!(overlay_lines(&log, 1_000_000).is_empty());
        assert!(overlay_lines(&[], 0).is_empty());
    }

    #[test]
    fn a_new_line_pushes_older_ones_up() {
        let mut log = vec![line(ChatKind::Toall, "a", "one", 0)];
        let mut fb = Framebuffer::new(320, 120);
        draw_chat_overlay(&mut fb, &log, 0, 0, &Input::default());
        let rows = |fb: &Framebuffer| -> Vec<i32> {
            (0..fb.height as i32).filter(|&y| (0..fb.width as i32).any(|x| {
                let i = (y as usize * fb.width as usize + x as usize) * 3;
                fb.pixels[i..i + 3] == kind_colour(ChatKind::Toall)
            })).collect()
        };
        let before = rows(&fb);
        assert!(!before.is_empty(), "the line is drawn");
        log.push(line(ChatKind::Toall, "a", "two", 10));
        let mut fb2 = Framebuffer::new(320, 120);
        draw_chat_overlay(&mut fb2, &log, 10, 0, &Input::default());
        let after = rows(&fb2);
        assert!(after.iter().min() < before.iter().min(), "the older line moved up");
        assert_eq!(after.iter().max(), before.iter().max(), "the newest sits where the last one did");
    }

    #[test]
    fn overlay_line_fades_out_of_the_picture() {
        let log = vec![line(ChatKind::Toall, "a", "hi", 0)];
        let say = kind_colour(ChatKind::Toall);
        let draw_at = |now: u64| {
            let mut fb = Framebuffer::new(200, 80);
            draw_chat_overlay(&mut fb, &log, now, 0, &Input::default());
            fb
        };
        assert!(lit(&draw_at(0), say), "fresh: full colour");
        // Mid-fade the text is dimmer than the full colour but still there.
        let mid = draw_at(OVERLAY_VISIBLE_MS + 1);
        assert!(!lit(&mid, say));
        let bare = draw_at(OVERLAY_VISIBLE_MS + OVERLAY_FADE_MS);
        let none = draw_at(0);
        assert_ne!(mid.pixels, bare.pixels, "mid-fade differs from gone");
        assert_ne!(none.pixels, bare.pixels);
        // Gone: the same picture as an empty log (just the input bar).
        let mut empty = Framebuffer::new(200, 80);
        draw_chat_overlay(&mut empty, &[], 0, 0, &Input::default());
        assert_eq!(bare.pixels, empty.pixels);
    }

    #[test]
    fn open_input_or_scrollback_shows_the_whole_window_without_fading() {
        let log = vec![line(ChatKind::Toall, "a", "old", 0), line(ChatKind::Toall, "a", "newer", 1)];
        let say = kind_colour(ChatKind::Toall);
        let late = OVERLAY_VISIBLE_MS + OVERLAY_FADE_MS + 1;
        let mut input = Input::default();
        input.open(InputMode::Chat);
        let mut fb = Framebuffer::new(200, 80);
        draw_chat_overlay(&mut fb, &log, late, 0, &input);
        assert!(lit(&fb, say), "the open input shows old lines at full colour");
        let mut fb = Framebuffer::new(200, 80);
        draw_chat_overlay(&mut fb, &log, late, 1, &Input::default());
        assert!(lit(&fb, say), "scrolled back shows them too");
    }

    #[test]
    fn script_chat_is_the_bubble_and_is_sent_once_per_change() {
        let mut bubble = String::new();
        let mut timer = crate::chatlists::BubbleTimer::default();
        assert!(script_say("px 30 py 31", &mut bubble, &mut timer));
        assert_eq!(bubble, "px 30 py 31");
        // The clock runs: a script line is not held like a typed one.
        assert!(timer.active() && !timer.is_held());
        // The same text a second later is not sent again.
        assert!(!script_say("px 30 py 31", &mut bubble, &mut timer));
        assert!(script_say("px 32 py 31", &mut bubble, &mut timer));
        assert!(script_say("", &mut bubble, &mut timer));
        assert!(bubble.is_empty() && !timer.active());
    }

    #[test]
    fn typed_line_reaches_history_overlay_and_own_bubble() {
        let mut st = crate::protocol::SessionState::default();
        let mut bubble = String::new();
        let mut timer = crate::chatlists::BubbleTimer::default();
        // Some time has passed in the session.
        st.tick_bubbles(5000);
        assert!(own_say(&mut st, "me", "hello there", &mut bubble, &mut timer));
        // 1. history
        assert_eq!(st.chat_log.len(), 1);
        assert_eq!(st.chat_log[0].from, "me");
        assert_eq!(st.chat_log[0].text, "hello there");
        assert_eq!(st.chat_log[0].born_ms, 5000);
        // 2. overlay
        let mut fb = Framebuffer::new(320, 120);
        draw_chat_overlay(&mut fb, &st.chat_log, st.chat_clock_ms, 0, &Input::default());
        let mut bar = Framebuffer::new(320, 120);
        draw_chat_overlay(&mut bar, &[], st.chat_clock_ms, 0, &Input::default());
        assert_ne!(fb.pixels, bar.pixels, "more than the bare input bar is drawn");
        // 3. own bubble, held while the player stands still
        assert_eq!(bubble, "hello there");
        assert!(timer.is_held());
        assert!(!timer.tick(60_000));
        assert!(!bubble.is_empty());
        // Empty line: no history entry, bubble cleared.
        assert!(!own_say(&mut st, "me", "", &mut bubble, &mut timer));
        assert_eq!(st.chat_log.len(), 1);
        assert!(bubble.is_empty());
    }

    #[test]
    fn other_kinds_of_chat_go_to_the_overlay_too() {
        let log = vec![
            line(ChatKind::Toall, "t", "to everyone", 0),
            line(ChatKind::Private, "p", "psst", 0),
            line(ChatKind::Admin, "Seihai", "note", 0),
        ];
        let mut fb = Framebuffer::new(320, 120);
        draw_chat_overlay(&mut fb, &log, 0, 0, &Input::default());
        for k in [ChatKind::Toall, ChatKind::Private, ChatKind::Admin] {
            assert!(lit(&fb, kind_colour(k)), "{k:?} line drawn");
        }
    }
}
