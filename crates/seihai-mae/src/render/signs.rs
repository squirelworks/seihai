//! The sign panel: the scroll frame and the text, both cut from the
//! version's `letters.png` (256×224, 16 glyphs a row, each 16×32), the way
//! the 2.05 client draws a sign it is reading. See `documentation/signs.md`.

use super::tiles::{Framebuffer, Sheet};
use crate::keys::Action;

/// The glyph order of `letters.png`: a character's index in this string is
/// its glyph. The `#` places hold glyphs that have no plain character of
/// their own (the button pictures and the frame pieces).
pub const ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!?-.,#>()#####\"####':/~&### <####;\n";

/// Half the advance of glyphs 0 to 95, in pixels (the client doubles it).
const HALF_ADVANCE: [u8; 96] = [
    6, 6, 6, 6, 6, 6, 6, 6, 3, 6, 6, 6, 7, 6, 6, 6, 6, 6, 6, 7, 6, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, //
    6, 6, 3, 5, 6, 3, 7, 6, 6, 6, 6, 5, 6, 6, 6, 7, 7, 7, 7, 6, 6, 4, 6, 6, 6, 6, 6, 6, 6, 6, 3, 7, //
    6, 4, 4, 6, 6, 6, 6, 6, 6, 6, 8, 8, 5, 7, 7, 7, 7, 4, 3, 7, 8, 7, 8, 8, 8, 4, 6, 8, 8, 8, 8, 6,
];

/// `#` followed by one of these draws button pictures in place of text:
/// A, B, X, Y, the four directions, and so on (glyph lists from the 2.05
/// table). Any other letter after `#` draws glyph 86 and moves 16 pixels.
const BUTTON_CODES: &[(char, &[u8])] = &[
    ('A', &[91]),
    ('B', &[92]),
    ('X', &[93]),
    ('Y', &[94]),
    ('u', &[77]),
    ('d', &[78]),
    ('l', &[79]),
    ('r', &[80]),
    ('h', &[74, 75]),
    ('x', &[71]),
    ('y', &[72]),
    ('z', &[73]),
    ('4', &[87, 88]),
    ('.', &[67]),
];

/// Frame pieces: corners top-left, top-right, bottom-left, bottom-right,
/// then the top, bottom, left and right edges.
const FRAME: [u8; 8] = [96, 97, 98, 99, 100, 101, 102, 103];

/// The panel's size in 16-pixel cells, and the lines shown at a time.
pub const PANEL_COLS: i32 = 24;
pub const PANEL_ROWS: i32 = 9;
pub const LINES_PER_PAGE: usize = 3;

fn glyph(fb: &mut Framebuffer, sheet: &Sheet, g: u8, x: i32, y: i32) {
    let (sx, sy) = (i32::from(g % 16) * 16, i32::from(g / 16) * 32);
    fb.blit_sprite(sheet.px, sheet.w, sheet.bpp, sx, sy, 16, 32, x, y);
}

fn advance(g: u8) -> i32 {
    HALF_ADVANCE.get(usize::from(g)).map_or(16, |&w| i32::from(w) * 2)
}

/// The sheet's pixel at `(x, y)`, or None when it is transparent (the
/// client skips a fill whose colour is the sheet's transparent one).
fn sheet_colour(sheet: &Sheet, x: u32, y: u32) -> Option<[u8; 3]> {
    let i = (y as usize * sheet.w as usize + x as usize) * sheet.bpp;
    let p = sheet.px.get(i..i + sheet.bpp)?;
    let transparent = if sheet.bpp == 4 { p[3] == 0 } else { p == [255, 0, 255] };
    (!transparent).then(|| [p[0], p[1], p[2]])
}

/// The scroll frame, `cols` by `rows` cells with its top-left at `(x, y)`.
/// The sides are 32-pixel pieces; the inside is filled with the colours at
/// (254, 223) and, 8 pixels further in, (255, 223) of the sheet.
pub fn draw_frame(fb: &mut Framebuffer, sheet: &Sheet, x: i32, y: i32, rows: i32, cols: i32) {
    let (right, bottom) = (x + (cols - 1) * 16, y + (rows - 2) * 16);
    for i in 1..cols - 1 {
        glyph(fb, sheet, FRAME[4], x + i * 16, y);
    }
    for i in 1..=(rows - 3) / 2 {
        glyph(fb, sheet, FRAME[6], x, y + i * 32);
        glyph(fb, sheet, FRAME[7], right, y + i * 32);
    }
    for i in 1..cols - 1 {
        glyph(fb, sheet, FRAME[5], x + i * 16, bottom);
    }
    glyph(fb, sheet, FRAME[0], x, y);
    glyph(fb, sheet, FRAME[1], right, y);
    glyph(fb, sheet, FRAME[2], x, bottom);
    glyph(fb, sheet, FRAME[3], right, bottom);
    if let Some(c) = sheet_colour(sheet, 254, 223) {
        fb.fill_rect(x + 16, y + 24, right - x - 16, bottom - y - 24, c);
    }
    if let Some(c) = sheet_colour(sheet, 255, 223) {
        fb.fill_rect(x + 24, y + 24, right - x - 32, bottom - y - 32, c);
    }
}

/// One line of sign text from `(x, y)`. Characters outside the alphabet are
/// left out (the client draws them in its system font).
pub fn draw_line(fb: &mut Framebuffer, sheet: &Sheet, line: &str, x: i32, y: i32) {
    let mut cx = x;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '#' {
            let Some(code) = chars.next() else { break };
            match BUTTON_CODES.iter().find(|(k, _)| *k == code) {
                Some((_, glyphs)) => {
                    for &g in *glyphs {
                        glyph(fb, sheet, g, cx, y);
                        cx += advance(g);
                    }
                }
                None => {
                    glyph(fb, sheet, 86, cx, y);
                    cx += 16;
                }
            }
            continue;
        }
        if let Some(g) = ALPHABET.find(c).filter(|_| c.is_ascii()) {
            let g = g as u8;
            glyph(fb, sheet, g, cx, y);
            cx += advance(g);
        }
    }
}

/// The panel's top-left for a view `view_w` pixels wide. It sits in the
/// lower part of the view unless the player is far enough down it, then
/// higher up. `player_row` is the player's row in the view, in tiles.
pub fn panel_origin(view_w: i32, player_row: f32) -> (i32, i32) {
    let (limit, high, low) = match view_w {
        800 => (18.0, 160, 320),
        1024 => (22.0, 192, 448),
        _ => (14.0, 128, 256),
    };
    ((view_w - PANEL_COLS * 16) / 2, if player_row > limit { high } else { low })
}

/// Draw the sign panel with the page of `text` that starts at line `first`.
pub fn draw_sign(fb: &mut Framebuffer, sheet: &Sheet, text: &str, first: usize, player_row: f32) {
    let (x, y) = panel_origin(fb.width as i32, player_row);
    draw_frame(fb, sheet, x, y, PANEL_ROWS, PANEL_COLS);
    for (i, line) in text.split('\n').skip(first).take(LINES_PER_PAGE).enumerate() {
        draw_line(fb, sheet, line, x + 16, y + 16 + i as i32 * 32);
    }
}

/// The width in pixels of `line` as [`draw_line`] draws it (letters only;
/// `#` codes are not counted).
pub fn line_width(line: &str) -> i32 {
    line.chars().filter_map(|c| ALPHABET.find(c).filter(|_| c.is_ascii())).map(|g| advance(g as u8)).sum()
}

/// The death scroll: a one-line sign panel, 7 by 5 cells, reading "End".
/// The 2.00 and 2.22 clients show it while the player is dead, under the
/// player, on a black board (lab, 2026-10-30; the size is read off the
/// 2.22 screenshot, one line of the sign panel).
pub const END_TEXT: &str = "End";
pub const END_COLS: i32 = 7;
pub const END_ROWS: i32 = PANEL_ROWS - 4;

/// Top-left of the death scroll for a player whose 48x48 box starts at
/// `player_box`: centred on the box, its top at the box's bottom.
pub fn end_scroll_origin(player_box: (i32, i32)) -> (i32, i32) {
    (player_box.0 + 24 - END_COLS * 8, player_box.1 + 48)
}

/// Draw the death scroll under the player.
pub fn draw_end_scroll(fb: &mut Framebuffer, sheet: &Sheet, player_box: (i32, i32)) {
    let (x, y) = end_scroll_origin(player_box);
    draw_frame(fb, sheet, x, y, END_ROWS, END_COLS);
    draw_line(fb, sheet, END_TEXT, x + (END_COLS * 16 - line_width(END_TEXT)) / 2, y + 16);
}

/// The page after the one starting at `first`, or None when the sign ends
/// there and reading stops.
pub fn next_page(text: &str, first: usize) -> Option<usize> {
    let lines = text.split('\n').count();
    (lines >= first + LINES_PER_PAGE).then_some(first + LINES_PER_PAGE)
}

/// The page before, or None on the first page.
pub fn previous_page(first: usize) -> Option<usize> {
    (first >= LINES_PER_PAGE).then(|| first - LINES_PER_PAGE)
}

/// What a key does to a sign being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignKey {
    /// Up: the page before.
    Back,
    /// The map key: stop reading at once.
    Close,
    /// Any other game key (the arrows but Up, D, S, A, the item list and
    /// pause): the next page, or stop on the last.
    Next,
}

/// The 2.05 sign keys. While a sign is open the game keys work it and
/// nothing else: the player does not walk, swing, lift or use an item.
/// `pressed` says whether an action's key went down this frame; `held` that
/// some game key (switch-to-chat aside) was down the frame before, in which
/// case nothing counts, so a key held from before the sign (the arrow that
/// walked into it) or a second key pressed while one is held does nothing.
pub fn sign_key(pressed: impl Fn(Action) -> bool, held: bool) -> Option<SignKey> {
    if held {
        return None;
    }
    if pressed(Action::Map) {
        return Some(SignKey::Close);
    }
    if pressed(Action::Up) {
        return Some(SignKey::Back);
    }
    Action::ALL
        .iter()
        .any(|&a| !matches!(a, Action::Up | Action::Map | Action::SwitchToChat) && pressed(a))
        .then_some(SignKey::Next)
}

/// The sign keys from one frame to the next.
///
/// minifb reports a key as down (`is_key_down`) one frame before it reports
/// it as pressed (`is_key_pressed`): `update` ages the key times before it
/// reads the new key events. Taking the presses from one and "a key was down
/// last frame" from the other made every press look held, so A, S and D did
/// nothing while a sign was open. Both now come from the same `down`, which
/// the caller makes "down or pressed this frame".
///
/// After reading ends the game keys stay away from the player until every
/// one is let go, so the S that closes a sign does not also swing the sword
/// a frame later, and the Up that pushed into a sign does not open it again
/// straight away.
#[derive(Debug, Default, Clone, Copy)]
pub struct SignKeys {
    /// Some game key (switch-to-chat aside) was down last frame.
    held: bool,
    /// Reading ended and a game key has stayed down since.
    after: bool,
}

impl SignKeys {
    /// One frame. `reading`: a sign is open. `active`: its keys go to it
    /// (not to the chat bar or the quit prompt). Returns what the sign does,
    /// and whether the game keys are kept from the player this frame.
    pub fn frame(&mut self, reading: bool, active: bool, down: impl Fn(Action) -> bool) -> (Option<SignKey>, bool) {
        let key = if reading && active { sign_key(&down, self.held) } else { None };
        let any = Action::ALL.iter().any(|&a| a != Action::SwitchToChat && down(a));
        if reading {
            self.after = true;
        } else if !any {
            self.after = false;
        }
        self.held = any;
        (key, reading || self.after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_keys_follow_2_05() {
        let only = |k: Action| move |a: Action| a == k;
        assert_eq!(sign_key(only(Action::Map), false), Some(SignKey::Close));
        assert_eq!(sign_key(only(Action::Up), false), Some(SignKey::Back));
        for k in [Action::Left, Action::Down, Action::Right, Action::Item, Action::Attack, Action::Act, Action::ItemList, Action::Pause] {
            assert_eq!(sign_key(only(k), false), Some(SignKey::Next), "{k:?}");
        }
        assert_eq!(sign_key(only(Action::SwitchToChat), false), None);
        assert_eq!(sign_key(|_| false, false), None);
        assert_eq!(sign_key(only(Action::Attack), true), None, "a key was still down");
    }

    /// A key is down a frame before minifb calls it pressed; the caller
    /// passes "down or pressed", and the press counts once.
    #[test]
    fn a_press_counts_on_the_frame_it_goes_down() {
        let only = |k: Action| move |a: Action| a == k;
        let none = |_: Action| false;
        let mut k = SignKeys::default();
        assert_eq!(k.frame(true, true, only(Action::Act)), (Some(SignKey::Next), true));
        assert_eq!(k.frame(true, true, only(Action::Act)), (None, true), "pressed a frame late");
        assert_eq!(k.frame(true, true, none), (None, true));
        assert_eq!(k.frame(true, true, only(Action::Attack)), (Some(SignKey::Next), true));
        assert_eq!(k.frame(true, true, none), (None, true));
        assert_eq!(k.frame(true, true, only(Action::Item)), (Some(SignKey::Next), true));
        assert_eq!(k.frame(true, true, none), (None, true));
        assert_eq!(k.frame(true, true, only(Action::Map)), (Some(SignKey::Close), true));
    }

    #[test]
    fn the_arrow_that_walked_in_waits_for_release() {
        let only = |k: Action| move |a: Action| a == k;
        let mut k = SignKeys::default();
        assert_eq!(k.frame(false, true, only(Action::Up)), (None, false), "walking");
        assert_eq!(k.frame(true, true, only(Action::Up)), (None, true), "the sign opened");
        assert_eq!(k.frame(true, true, |_| false), (None, true));
        assert_eq!(k.frame(true, true, only(Action::Up)), (Some(SignKey::Back), true));
    }

    #[test]
    fn the_key_that_ends_reading_stays_off_the_player_until_let_go() {
        let only = |k: Action| move |a: Action| a == k;
        let mut k = SignKeys::default();
        k.frame(true, true, |_| false);
        assert_eq!(k.frame(true, true, only(Action::Attack)), (Some(SignKey::Next), true));
        // The caller ended reading on that S; it is still down or pressed.
        assert_eq!(k.frame(false, true, only(Action::Attack)), (None, true), "no swing");
        assert_eq!(k.frame(false, true, only(Action::Attack)), (None, true));
        assert_eq!(k.frame(false, true, |_| false), (None, false));
        assert_eq!(k.frame(false, true, only(Action::Attack)), (None, false), "S swings again");
    }

    #[test]
    fn keys_wait_while_the_chat_bar_or_quit_prompt_has_them() {
        let only = |k: Action| move |a: Action| a == k;
        let mut k = SignKeys::default();
        assert_eq!(k.frame(true, false, only(Action::Act)), (None, true));
        assert_eq!(k.frame(true, true, |_| false), (None, true));
        assert_eq!(k.frame(true, true, only(Action::Act)), (Some(SignKey::Next), true));
    }

    /// A 256×224 RGBA sheet where every glyph is opaque in its own colour.
    fn sheet() -> Vec<u8> {
        let mut px = vec![0u8; 256 * 224 * 4];
        for y in 0..224 {
            for x in 0..256 {
                let g = (y / 32) * 16 + x / 16;
                let i = (y * 256 + x) * 4;
                px[i..i + 4].copy_from_slice(&[g as u8, 1, 2, 255]);
            }
        }
        px
    }

    fn at(fb: &Framebuffer, x: i32, y: i32) -> [u8; 3] {
        let i = (y as usize * fb.width as usize + x as usize) * 3;
        [fb.pixels[i], fb.pixels[i + 1], fb.pixels[i + 2]]
    }

    #[test]
    fn end_scroll_sits_under_the_player_with_the_word_centred() {
        assert_eq!(line_width("End"), 12 + 12 + 12);
        assert_eq!(end_scroll_origin((100, 50)), (100 + 24 - 56, 98));
        let px = sheet();
        let sh = Sheet { px: &px, w: 256, bpp: 4 };
        let mut fb = Framebuffer::new(320, 240);
        draw_end_scroll(&mut fb, &sh, (100, 50));
        // The top-left frame corner (glyph 96) at the origin, and the E
        // (glyph 4) one cell down, centred: (112 - 36) / 2 = 38 in.
        assert_eq!(at(&fb, 68, 98)[0], 96);
        assert_eq!(at(&fb, 68 + 38, 98 + 16)[0], 4);
        assert_eq!(at(&fb, 0, 0), [0, 0, 0], "nothing outside the scroll");
    }

    #[test]
    fn alphabet_places_letters_digits_and_space() {
        assert_eq!(ALPHABET.chars().count(), 97);
        assert_eq!(ALPHABET.find('a'), Some(26));
        assert_eq!(ALPHABET.find('0'), Some(52));
        assert_eq!(ALPHABET.find(' '), Some(89));
        assert_eq!(advance(8), 6, "I is narrow");
        assert_eq!(advance(89), 8, "space");
    }

    #[test]
    fn text_advances_by_the_glyph_widths_and_buttons_draw_their_pictures() {
        let px = sheet();
        let sh = Sheet { px: &px, w: 256, bpp: 4 };
        let mut fb = Framebuffer::new(200, 40);
        draw_line(&mut fb, &sh, "Ii#A", 0, 0);
        assert_eq!(at(&fb, 0, 0)[0], 8, "I is glyph 8");
        assert_eq!(at(&fb, 6, 0)[0], 34, "i follows 6 pixels on");
        // i advances 6, so the A button starts at 12, over the end of i.
        assert_eq!(at(&fb, 12, 0)[0], 91);
        assert_eq!(at(&fb, 27, 0)[0], 91);
        assert_eq!(at(&fb, 28, 0), [0, 0, 0]);
    }

    #[test]
    fn frame_corners_edges_and_fill() {
        let mut px = sheet();
        // Outer and inner fill colours.
        for (x, c) in [(254usize, [10u8, 20, 30]), (255, [40, 50, 60])] {
            let i = (223 * 256 + x) * 4;
            px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
        let sh = Sheet { px: &px, w: 256, bpp: 4 };
        let mut fb = Framebuffer::new(640, 480);
        draw_sign(&mut fb, &sh, "", 0, 5.0);
        let (x, y) = (128, 256);
        assert_eq!(at(&fb, x, y)[0], 96, "top-left corner");
        assert_eq!(at(&fb, x + 23 * 16, y)[0], 97, "top-right corner");
        assert_eq!(at(&fb, x + 16, y)[0], 100, "top edge");
        assert_eq!(at(&fb, x, y + 32)[0], 102, "left edge");
        assert_eq!(at(&fb, x, y + 7 * 16)[0], 98, "bottom-left corner");
        assert_eq!(at(&fb, x + 17, y + 25), [10, 20, 30], "outer fill");
        assert_eq!(at(&fb, x + 30, y + 30), [40, 50, 60], "inner fill");
        assert_eq!(at(&fb, x + 23 * 16 - 1, y + 25), [10, 20, 30], "fill ends at the right side");
        assert_eq!(at(&fb, x + 23 * 16, y + 40)[0], 103, "right edge");
    }

    #[test]
    fn panel_moves_up_when_the_player_is_low_in_the_view() {
        assert_eq!(panel_origin(640, 10.0), (128, 256));
        assert_eq!(panel_origin(640, 20.0), (128, 128));
        assert_eq!(panel_origin(800, 20.0), (208, 160));
    }

    #[test]
    fn pages_go_three_lines_at_a_time() {
        let text = "1\n2\n3\n4\n5";
        assert_eq!(next_page(text, 0), Some(3));
        assert_eq!(next_page(text, 3), None);
        assert_eq!(previous_page(3), Some(0));
        assert_eq!(previous_page(0), None);
        // Three lines exactly: the client turns to an empty page first.
        assert_eq!(next_page("a\nb\nc", 0), Some(3));
    }
}
