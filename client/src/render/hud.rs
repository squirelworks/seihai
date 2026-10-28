//! The status display drawn from a version's own `state.png`.
//!
//! Every client from 1.33 to 2.05 ships a 224-pixel-wide `state.png` with
//! the same places for hearts, the item counters and the bars; what
//! differs per build is the item buttons, the digits and the AP bar (see
//! [`HudArt`]). Sheet rectangles were measured on the shipped sheets, and
//! screen positions on a 2.05 screenshot at 640×480. Older builds are
//! drawn with the same screen positions. [inferred]

use super::font::{glyph, CELL_W};
use super::Framebuffer;
use crate::protocol::HudArt;

/// An RGBA `state.png`.
pub struct HudSheet<'a> {
    pub px: &'a [u8],
    pub w: u32,
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
    /// Alignment points 0–100, for builds with the AP bar.
    pub ap: i32,
    /// Letters on the item buttons (action, attack, item).
    pub keys: [char; 3],
}

#[derive(Clone, Copy)]
struct Rect(i32, i32, i32, i32);

const HEART_FULL: Rect = Rect(0, 0, 40, 44);
const HEART_HALF: Rect = Rect(40, 0, 30, 44);
const SMALL_FULL: Rect = Rect(80, 0, 33, 33);
const SMALL_HALF: Rect = Rect(113, 0, 24, 33);
const RUPEE: Rect = Rect(80, 33, 16, 16);
const BOMB: Rect = Rect(136, 33, 16, 16);
const ARROW: Rect = Rect(185, 33, 15, 16);
/// 1.42 and later: one button shape, letters drawn on it.
const BUTTON: Rect = Rect(202, 0, 22, 30);
/// Before 1.42: three buttons side by side (action, attack, item).
const BUTTONS: [Rect; 3] = [Rect(158, 0, 22, 30), Rect(180, 0, 22, 30), Rect(202, 0, 22, 30)];
const AP_FRAME: Rect = Rect(0, 97, 130, 22);
const AP_FILL: Rect = Rect(0, 119, 100, 10);
/// Digits 0–9 on the sheet from 1.38, 14 pixels apart.
const DIGIT_Y: i32 = 81;
const DIGIT_W: i32 = 14;
const DIGIT_H: i32 = 16;

// Screen positions (2.05 at 640×480).
const BUTTON_AT: [(i32, i32); 3] = [(15, 30), (80, 30), (145, 30)];
const BAR_AT: (i32, i32) = (15, 65);
/// Inside of the bar frame, where the fill goes.
const BAR_INNER: (i32, i32) = (22, 6);
const RUPEE_AT: (i32, i32) = (274, 30);
const BOMB_AT: (i32, i32) = (330, 31);
const ARROW_AT: (i32, i32) = (379, 30);
const COUNT_Y: i32 = 48;
const DIGIT_ADVANCE: i32 = 12;
const HEARTS_AT: (i32, i32) = (442, 24);
const HEART_STEP: i32 = 45;
/// More containers than this switch to the small hearts.
const BIG_HEARTS: i32 = 4;
const SMALL_STEP: (i32, i32) = (28, 30);
const SMALL_PER_ROW: i32 = 6;

impl HudSheet<'_> {
    fn h(&self) -> i32 {
        (self.px.len() / 4 / self.w.max(1) as usize) as i32
    }

    fn has(&self, r: Rect) -> bool {
        r.0 + r.2 <= self.w as i32 && r.1 + r.3 <= self.h()
    }

    fn blit(&self, fb: &mut Framebuffer, r: Rect, x: i32, y: i32) {
        if self.has(r) {
            fb.blit_sprite(self.px, self.w, 4, r.0, r.1, r.2, r.3, x, y);
        }
    }

    fn blit_cols(&self, fb: &mut Framebuffer, r: Rect, cols: i32, x: i32, y: i32) {
        if self.has(r) {
            fb.blit_sprite(self.px, self.w, 4, r.0, r.1, cols.clamp(0, r.2), r.3, x, y);
        }
    }
}

/// Draw the status display for `art` from `sheet`.
pub fn draw_hud(fb: &mut Framebuffer, sheet: &HudSheet, art: HudArt, ap_bar: bool, v: &HudValues) {
    // Item buttons.
    for (i, &(x, y)) in BUTTON_AT.iter().enumerate() {
        match art {
            HudArt::OneButton => sheet.blit(fb, BUTTON, x, y),
            _ => sheet.blit(fb, BUTTONS[i], x, y),
        }
        let lettered = matches!(art, HudArt::Letters | HudArt::LettersChat);
        if !lettered {
            draw_big_letter(fb, v.keys[i], x + 4, y + 7);
        }
    }

    if ap_bar {
        let (x, y) = BAR_AT;
        sheet.blit(fb, AP_FRAME, x, y);
        fb.fill_rect(x + BAR_INNER.0, y + BAR_INNER.1, AP_FILL.2, AP_FILL.3, [0x50, 0x08, 0x08]);
        sheet.blit_cols(fb, AP_FILL, v.ap, x + BAR_INNER.0, y + BAR_INNER.1);
    }

    let digits_on_sheet = matches!(art, HudArt::BlankButtons | HudArt::OneButton);
    for (icon, at, n, pad) in [
        (RUPEE, RUPEE_AT, v.rupees, 1),
        (BOMB, BOMB_AT, v.bombs, 2),
        (ARROW, ARROW_AT, v.arrows, 2),
    ] {
        sheet.blit(fb, icon, at.0, at.1);
        let text = format!("{:0pad$}", n.max(0), pad = pad);
        let centre = at.0 + icon.2 / 2;
        if digits_on_sheet {
            let width = text.len() as i32 * DIGIT_ADVANCE;
            let mut x = centre - width / 2;
            for c in text.bytes() {
                let d = (c - b'0') as i32;
                sheet.blit(fb, Rect(d * DIGIT_W, DIGIT_Y, DIGIT_W, DIGIT_H), x, COUNT_Y);
                x += DIGIT_ADVANCE;
            }
        } else {
            let x = centre - text.len() as i32 * CELL_W / 2;
            fb.draw_text_outlined(x, COUNT_Y + 4, &text, [0xFF, 0xFF, 0xFF]);
        }
    }

    let containers = (v.max_half_hearts.max(1) + 1) / 2;
    let filled = v.half_hearts.clamp(0, containers * 2);
    for i in 0..containers {
        let left = filled - i * 2;
        if left <= 0 {
            break;
        }
        let (full, half, x, y) = if containers <= BIG_HEARTS {
            (HEART_FULL, HEART_HALF, HEARTS_AT.0 + i * HEART_STEP, HEARTS_AT.1)
        } else {
            let (col, row) = (i % SMALL_PER_ROW, i / SMALL_PER_ROW);
            (
                SMALL_FULL,
                SMALL_HALF,
                HEARTS_AT.0 + col * SMALL_STEP.0,
                HEARTS_AT.1 + row * SMALL_STEP.1,
            )
        };
        sheet.blit(fb, if left >= 2 { full } else { half }, x, y);
    }
}

/// A button letter at twice the font size, white with a dark edge.
fn draw_big_letter(fb: &mut Framebuffer, c: char, x: i32, y: i32) {
    let g = glyph(c.to_ascii_uppercase());
    for (ox, oy, rgb) in [(1, 1, [0x40, 0, 0]), (0, 0, [0xFF, 0xFF, 0xFF])] {
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

    fn sheet(h: u32) -> Vec<u8> {
        vec![0xFF; 224 * h as usize * 4]
    }

    fn lit(fb: &Framebuffer, x: i32, y: i32) -> bool {
        let i = (y as usize * fb.width as usize + x as usize) * 3;
        fb.pixels[i..i + 3] != [0, 0, 0]
    }

    fn values() -> HudValues {
        HudValues {
            half_hearts: 5,
            max_half_hearts: 6,
            rupees: 0,
            bombs: 5,
            arrows: 10,
            ap: 50,
            keys: ['A', 'S', 'D'],
        }
    }

    #[test]
    fn hearts_and_counters_land_where_2_05_draws_them() {
        let px = sheet(145);
        let s = HudSheet { px: &px, w: 224 };
        let mut fb = Framebuffer::new(640, 480);
        draw_hud(&mut fb, &s, HudArt::OneButton, true, &values());
        // Two full hearts and a half one.
        assert!(lit(&fb, 442 + 20, 40));
        assert!(lit(&fb, 487 + 20, 40));
        assert!(lit(&fb, 532 + 10, 40));
        assert!(!lit(&fb, 532 + 35, 40));
        assert!(lit(&fb, 274 + 8, 38));
        assert!(lit(&fb, 15 + 10, 65 + 2));
    }

    #[test]
    fn no_ap_bar_before_1_41() {
        let px = sheet(97);
        let s = HudSheet { px: &px, w: 224 };
        let mut fb = Framebuffer::new(640, 480);
        draw_hud(&mut fb, &s, HudArt::BlankButtons, false, &values());
        assert!(!lit(&fb, 15 + 60, 65 + 10));
    }
}
