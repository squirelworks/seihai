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
/// 2.31). The real 2.17.1 F5 opens a position window and never loads levels, so
/// its online text for the older F5 level loader is not known: that line is
/// inferred.
pub fn editor_key(has_editor: bool, label: &str, online: bool, is_f4: bool) -> EditorKey {
    if !has_editor {
        // 2.3 moved the editor and level loading to GraalEditor.
        return EditorKey::Refuse(format!("{label} has no built-in editor"));
    }
    if online {
        return EditorKey::Refuse(if is_f4 {
            "You cannot open the editor in the online mode.".into()
        } else {
            // Inferred wording, not from a real client.
            "You are online and this feature is not available".into()
        });
    }
    EditorKey::Run
}

#[cfg(test)]
mod editor_key_tests {
    use super::*;

    #[test]
    fn online_refuses_both_keys() {
        assert_eq!(
            editor_key(true, "2.17", true, true),
            EditorKey::Refuse("You cannot open the editor in the online mode.".into())
        );
        assert_eq!(
            editor_key(true, "2.17", true, false),
            EditorKey::Refuse("You are online and this feature is not available".into())
        );
    }

    #[test]
    fn offline_runs_and_no_editor_refuses() {
        assert_eq!(editor_key(true, "2.17", false, true), EditorKey::Run);
        assert_eq!(editor_key(true, "2.17", false, false), EditorKey::Run);
        for online in [false, true] {
            assert_eq!(
                editor_key(false, "2.3", online, false),
                EditorKey::Refuse("2.3 has no built-in editor".into())
            );
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
