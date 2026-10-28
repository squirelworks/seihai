//! The status display, drawn from the version's own `state.png`.
//!
//! The rules are the 2.05 rev 2 client's own status routine (see
//! `documentation/hud.md`): every rectangle on the sheet, every screen
//! offset, the heart rows and the `showstats` bits. The 2.22 sheet is the
//! 2.05 sheet except for one small unused patch, and a 2.22 lab screenshot
//! puts everything in the same places, so 2.22 and 2.31 draw the same way
//! (inferred for the code, confirmed for the picture). Builds older than
//! 1.42 ship other button art; Seihai draws their sheets with the same
//! offsets (inferred).

use super::font::{glyph, CELL_W};
use super::throw::draw_sprite;
use super::{Framebuffer, Sheet};
use crate::assets::ObjectTables;
use crate::protocol::HudArt;

/// `showstats` bits the routine tests. Bits 0x100 (the player icons) and
/// 0x200 (the weapon icon list) belong to other parts of the screen.
pub mod stats {
    pub const BUTTONS: u16 = 0x01;
    pub const ICONS: u16 = 0x02;
    pub const RUPEES: u16 = 0x04;
    pub const BOMBS: u16 = 0x08;
    pub const ARROWS: u16 = 0x10;
    pub const HEARTS: u16 = 0x20;
    pub const AP: u16 = 0x40;
    pub const MP: u16 = 0x80;
    pub const ALL: u16 = 0x3ff;
}

/// The images the status display draws from.
#[derive(Clone, Copy)]
pub struct HudSheets<'a> {
    pub state: Sheet<'a>,
    /// `sprites.png`, for the glove icons.
    pub sprites: Option<Sheet<'a>>,
    /// The player's sword image (`sword1.png` by default), for the sword icon.
    pub sword: Option<Sheet<'a>>,
    /// The selected weapon's image, drawn beside the D button.
    pub item: Option<Sheet<'a>>,
}

/// What the status display shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudValues {
    /// Current power in half hearts.
    pub half_hearts: i32,
    /// Heart containers in half hearts.
    pub max_half_hearts: i32,
    pub rupees: i32,
    pub bombs: i32,
    pub arrows: i32,
    /// Alignment points, 0 to 100.
    pub ap: i32,
    /// Magic points; the bar is drawn only above 0.
    pub mp: i32,
    pub glove_power: i32,
    pub sword_power: i32,
    /// The client's weapons flag: off, it draws only the A button and its
    /// key (and the weapon keys do nothing).
    pub weapons: bool,
    /// The `showstats` bits (see [`stats`]).
    pub stats: u16,
    /// Key names on the action, attack and item buttons, then the small
    /// one above the item button (the client's key 9, Q by default).
    pub keys: [char; 4],
}

#[derive(Clone, Copy)]
struct Rect(i32, i32, i32, i32);

// Sheet rectangles (x, y, w, h), from the 2.05 routine.
const HEART_BIG: Rect = Rect(0, 0, 40, 44);
const HEART_SMALL: Rect = Rect(80, 0, 31, 33);
/// Rupee, bomb and arrow icons, one strip.
const ICONS: Rect = Rect(80, 33, 120, 16);
const BUTTON: Rect = Rect(202, 0, 22, 33);
/// 1.38 to 1.41 sheets (and 1.33 to 1.37 with the letters on them): three
/// buttons side by side, action, attack, item. [inferred from the sheets]
const BUTTONS_OLD: [Rect; 3] = [Rect(158, 0, 22, 30), Rect(180, 0, 22, 30), Rect(202, 0, 22, 30)];
const DIGIT_Y: i32 = 81;
const DIGIT_W: i32 = 14;
const DIGIT_H: i32 = 16;
const AP_FRAME: Rect = Rect(0, 97, 130, 22);
const AP_FILL_Y: i32 = 119;
const MP_FRAME: Rect = Rect(0, 49, 130, 22);
const MP_FILL_Y: i32 = 71;
const FILL_H: i32 = 10;

// Screen offsets from the view's top-left.
const BASE: (i32, i32) = (15, 15);
const BUTTON_STEP: i32 = 65;
const ICONS_AT: (i32, i32) = (274, 30);
const COUNT_X: i32 = 280;
const COUNT_Y: i32 = 48;
const DIGIT_ADVANCE: i32 = 12;
const BOMBS_X: i32 = COUNT_X + 46;
const ARROWS_X: i32 = COUNT_X + 94;
/// Hearts start this far from the right edge.
const HEARTS_FROM_RIGHT: i32 = 198;
const HEARTS_Y: i32 = 24;
const BIG_STEP: i32 = 45;
const SMALL_STEP: i32 = 35;
/// Small hearts: a row starts at each of these heart numbers; row `k + 1`
/// moves `round((k + 1) * 17.5) - 35` pixels sideways and 20 down.
const SMALL_ROWS: [i32; 5] = [5, 11, 16, 22, 27];
const SMALL_ROW_DY: i32 = 20;
/// Power up to this many hearts is drawn with the big hearts.
const BIG_MAX_HEARTS: i32 = 4;
/// Glove icons (sprites 145 for power 2, 325 drawn over it for power 3).
const GLOVE_SPRITES: [u16; 2] = [145, 325];
const SWORD_SPRITE: u16 = 17;

/// Built-in rectangles for the icon sprites, from the 2.22 `arrays.dat`,
/// for clients without one.
fn icon_rect(id: u16) -> Option<(i32, i32, i32, i32)> {
    match id {
        17 => Some((94, 36, 28, 28)),
        145 => Some((28, 212, 28, 32)),
        325 => Some((0, 212, 28, 32)),
        _ => None,
    }
}

fn rows(s: &Sheet) -> i32 {
    (s.px.len() / (s.w as usize * s.bpp).max(1)) as i32
}

fn has(s: &Sheet, r: Rect) -> bool {
    r.0 >= 0 && r.1 >= 0 && r.0 + r.2 <= s.w as i32 && r.1 + r.3 <= rows(s)
}

fn blit(fb: &mut Framebuffer, s: &Sheet, r: Rect, x: i32, y: i32) {
    if has(s, r) {
        fb.blit_sprite(s.px, s.w, s.bpp, r.0, r.1, r.2, r.3, x, y);
    }
}

/// True when the sheet has the three-button art of 1.38 to 1.41: the first
/// of the three is opaque where the 1.42 and later sheets are empty.
fn three_buttons(s: &Sheet) -> bool {
    let (x, y) = (169usize, 15usize);
    if !has(s, Rect(x as i32, y as i32, 1, 1)) {
        return false;
    }
    let i = (y * s.w as usize + x) * s.bpp;
    match s.bpp {
        4 => s.px[i + 3] != 0,
        _ => s.px[i..i + 3] != [255, 0, 255],
    }
}

fn sprite(fb: &mut Framebuffer, sheet: Sheet, tables: Option<&ObjectTables>, id: u16, x: i32, y: i32) {
    if !draw_sprite(fb, sheet, tables, id, x, y) {
        if let Some((sx, sy, w, h)) = icon_rect(id) {
            if has(&sheet, Rect(sx, sy, w, h)) {
                fb.blit_sprite(sheet.px, sheet.w, sheet.bpp, sx, sy, w, h, x, y);
            }
        }
    }
}

/// Draw the status display for a build with `art`.
pub fn draw_hud(fb: &mut Framebuffer, art: HudArt, sheets: &HudSheets, tables: Option<&ObjectTables>, v: &HudValues) {
    let s = &sheets.state;
    let on = |bit: u16| v.stats & bit != 0;
    let (bx, by) = BASE;

    if on(stats::ICONS) {
        blit(fb, s, ICONS, ICONS_AT.0, ICONS_AT.1);
    }

    if on(stats::HEARTS) {
        draw_hearts(fb, s, v);
    }

    let digits = has(s, Rect(0, DIGIT_Y, 10 * DIGIT_W, DIGIT_H));
    if on(stats::RUPEES) {
        // Up to seven digits, no leading zeros, centred on x 280.
        let text = v.rupees.clamp(0, 9_999_999).to_string();
        let x = COUNT_X - (text.len() as i32 * DIGIT_ADVANCE >> 1);
        draw_number(fb, s, digits, &text, x);
    }
    if on(stats::BOMBS) {
        draw_number(fb, s, digits, &format!("{:02}", v.bombs.clamp(0, 99)), BOMBS_X);
    }
    if on(stats::ARROWS) {
        draw_number(fb, s, digits, &format!("{:02}", v.arrows.clamp(0, 99)), ARROWS_X);
    }

    if on(stats::BUTTONS) {
        let old = matches!(art, HudArt::Letters | HudArt::LettersChat) || three_buttons(s);
        let lettered = matches!(art, HudArt::Letters | HudArt::LettersChat);
        let shown = if v.weapons { 3 } else { 1 };
        for i in 0..shown {
            let x = bx + i as i32 * BUTTON_STEP;
            blit(fb, s, if old { BUTTONS_OLD[i] } else { BUTTON }, x, by + 15);
            if !lettered {
                draw_big_letter(fb, v.keys[i], x + 2, by + 17 + 4);
            }
        }
        if v.weapons {
            let small = v.keys[3].to_ascii_uppercase().to_string();
            fb.draw_text_outlined(bx + 132, by + 11, &small, [0xFF, 0xFF, 0xFF]);
            if let Some(sp) = sheets.sprites {
                for (n, &id) in GLOVE_SPRITES.iter().enumerate() {
                    if v.glove_power >= 2 + n as i32 {
                        sprite(fb, sp, tables, id, bx + 25, by + 14);
                    }
                }
            }
            if v.glove_power >= 2 {
                draw_small_number(fb, v.glove_power, bx + 45, by + 18);
            }
            if let Some(sw) = sheets.sword {
                if v.sword_power > 0 {
                    sprite(fb, sw, tables, SWORD_SPRITE, bx + 90, by + 16);
                    draw_small_number(fb, v.sword_power, bx + 110, by + 18);
                }
            }
            if let Some(item) = &sheets.item {
                let h = rows(item);
                blit(fb, item, Rect(0, 0, item.w as i32, h), bx + 155, by + 30 - (h >> 1));
            }
        }
    }

    if on(stats::AP) {
        draw_bar(fb, s, AP_FRAME, AP_FILL_Y, v.ap.clamp(0, 100), bx, by + 50);
    }
    if on(stats::MP) && v.weapons && v.mp > 0 {
        draw_bar(fb, s, MP_FRAME, MP_FILL_Y, v.mp.clamp(0, 100), bx, by + 75);
    }
}

fn draw_bar(fb: &mut Framebuffer, s: &Sheet, frame: Rect, fill_y: i32, value: i32, x: i32, y: i32) {
    if !has(s, frame) {
        return;
    }
    blit(fb, s, frame, x, y);
    blit(fb, s, Rect(0, fill_y, value, FILL_H), x + 22, y + 6);
}

fn draw_number(fb: &mut Framebuffer, s: &Sheet, sheet_digits: bool, text: &str, x: i32) {
    if sheet_digits {
        for (i, c) in text.bytes().enumerate() {
            let d = (c - b'0') as i32;
            blit(fb, s, Rect(d * DIGIT_W, DIGIT_Y, DIGIT_W, DIGIT_H), x + i as i32 * DIGIT_ADVANCE, COUNT_Y);
        }
    } else {
        // 1.33 to 1.37 keep their digits in sprites.png; the font stands in.
        fb.draw_text_outlined(x, COUNT_Y + 4, text, [0xFF, 0xFF, 0xFF]);
    }
}

fn draw_hearts(fb: &mut Framebuffer, s: &Sheet, v: &HudValues) {
    let containers = (v.max_half_hearts.max(0) + 1) / 2;
    let power = v.half_hearts.clamp(0, containers * 2);
    let x0 = fb.width as i32 - HEARTS_FROM_RIGHT;
    // The client compares power in hearts (a float) with the heart number:
    // heart i is full when power - 1 >= i, half when i < power.
    let big = power <= BIG_MAX_HEARTS * 2;
    let mut y0 = HEARTS_Y;
    if !big && power < (BIG_MAX_HEARTS + 2) * 2 {
        y0 += 10;
    }
    for i in 0..containers {
        let half = if power - 2 >= i * 2 {
            0
        } else if i * 2 < power {
            1
        } else {
            break;
        };
        if big {
            let r = Rect(HEART_BIG.0 + half * HEART_BIG.2, 0, HEART_BIG.2, HEART_BIG.3);
            blit(fb, s, r, x0 + i * BIG_STEP, HEARTS_Y);
        } else {
            let (mut x, mut y) = (x0 + i * SMALL_STEP, y0);
            for k in (0..SMALL_ROWS.len()).rev() {
                if SMALL_ROWS[k] <= i {
                    let shift = ((k as f32 + 1.0) * 17.5 + 0.5) as i32;
                    x = x0 + (i - SMALL_ROWS[k]) * SMALL_STEP + shift - SMALL_STEP;
                    y = y0 + (k as i32 + 1) * SMALL_ROW_DY;
                    break;
                }
            }
            let r = Rect(HEART_SMALL.0 + half * HEART_SMALL.2, 0, HEART_SMALL.2, HEART_SMALL.3);
            blit(fb, s, r, x, y);
        }
    }
}

/// A sword or glove power beside its icon. The client prints it in its
/// world-text font; Seihai uses its own. [position from the routine, font
/// inferred]
fn draw_small_number(fb: &mut Framebuffer, n: i32, x: i32, y: i32) {
    let text = n.to_string();
    fb.draw_text_outlined(x - text.len() as i32 * CELL_W / 2, y, &text, [0xFF, 0xFF, 0xFF]);
}

/// A key name on a button. The client uses 26 pt Helvetica, white with a
/// black shadow one pixel down and right; Seihai doubles its own font.
fn draw_big_letter(fb: &mut Framebuffer, c: char, x: i32, y: i32) {
    let g = glyph(c.to_ascii_uppercase());
    for (ox, oy, rgb) in [(1, 1, [0, 0, 0]), (0, 0, [0xFF, 0xFF, 0xFF])] {
        for (row, bits) in g.iter().enumerate() {
            for col in 0..5 {
                if bits & (0x10 >> col) != 0 {
                    fb.fill_rect(x + ox + col * 2, y + oy + row as i32 * 2, 2, 2, rgb);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white(w: u32, h: u32) -> Vec<u8> {
        vec![0xFF; (w * h * 4) as usize]
    }

    fn lit(fb: &Framebuffer, x: i32, y: i32) -> bool {
        let i = (y as usize * fb.width as usize + x as usize) * 3;
        fb.pixels[i..i + 3] != [0, 0, 0]
    }

    fn values(half_hearts: i32, max: i32) -> HudValues {
        HudValues {
            half_hearts,
            max_half_hearts: max,
            rupees: 0,
            bombs: 5,
            arrows: 10,
            ap: 50,
            mp: 0,
            glove_power: 1,
            sword_power: 1,
            weapons: true,
            stats: stats::ALL,
            keys: ['A', 'S', 'D', 'Q'],
        }
    }

    fn draw(v: &HudValues, sheet_h: u32) -> Framebuffer {
        let px = white(224, sheet_h);
        let sheets = HudSheets { state: Sheet { px: &px, w: 224, bpp: 4 }, sprites: None, sword: None, item: None };
        let mut fb = Framebuffer::new(640, 480);
        draw_hud(&mut fb, HudArt::BlankButtons, &sheets, None, v);
        fb
    }

    #[test]
    fn big_hearts_full_and_half_where_2_05_draws_them() {
        // 2.5 hearts of 3: two full, one half (30 of its 40 columns are
        // not the empty right side; the half heart is the left 40 of the
        // half-heart cell, which the white sheet fills).
        let fb = draw(&values(5, 6), 145);
        assert!(lit(&fb, 442 + 20, 40));
        assert!(lit(&fb, 487 + 20, 40));
        assert!(lit(&fb, 532 + 20, 40));
        assert!(!lit(&fb, 577 + 20, 40));
    }

    #[test]
    fn more_than_four_hearts_switch_to_small_rows() {
        // 6 hearts: row 0 holds hearts 0..4 at y 24, heart 5 starts row 1,
        // 17 pixels left of the first column and 20 down.
        let fb = draw(&values(12, 12), 145);
        assert!(lit(&fb, 442 + 4 * 35 + 15, 24 + 16));
        assert!(lit(&fb, 442 - 17 + 15, 44 + 16));
        assert!(!lit(&fb, 442 + 33, 40), "small hearts leave a gap a big heart would fill");
    }

    #[test]
    fn weapons_off_draws_only_the_action_button() {
        let mut v = values(6, 6);
        v.weapons = false;
        let fb = draw(&v, 145);
        assert!(lit(&fb, 15 + 20, 30 + 16));
        assert!(!lit(&fb, 80 + 20, 30 + 16));
        let fb = draw(&values(6, 6), 145);
        assert!(lit(&fb, 80 + 20, 30 + 16));
        assert!(lit(&fb, 145 + 20, 30 + 16));
    }

    #[test]
    fn stats_bits_hide_parts() {
        let mut v = values(6, 6);
        v.stats = stats::ALL & !stats::HEARTS & !stats::AP;
        let fb = draw(&v, 145);
        assert!(!lit(&fb, 442 + 20, 40));
        assert!(!lit(&fb, 15 + 60, 65 + 2));
        assert!(lit(&fb, 274 + 8, 38), "icons stay");
    }

    #[test]
    fn rupees_centre_on_x_280() {
        let mut v = values(6, 6);
        v.rupees = 123;
        let fb = draw(&v, 145);
        // Three digits, 36 wide: 262 to 274+14.
        assert!(lit(&fb, 263, COUNT_Y + 8));
        assert!(!lit(&fb, 260, COUNT_Y + 8));
    }

    #[test]
    fn no_ap_bar_on_a_short_sheet() {
        let fb = draw(&values(6, 6), 97);
        assert!(!lit(&fb, 15 + 60, 65 + 2));
    }
}
