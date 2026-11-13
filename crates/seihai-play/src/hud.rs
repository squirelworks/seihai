//! The status display: hearts, gralats, bombs, arrows, the AP bar and the
//! three item slots, drawn from the build's own `state.png` onto the frame.
//!
//! The rules are the 2.05 rev 2 status routine that `seihai-mae` draws
//! (`documentation/hud.md`), measured from the view's top-left, with the
//! hearts measured from its right edge. A lab run of the real 6.03r7 Linux
//! and 6.00r2 Windows clients (squirelworks/seihai#444, 1024 x 768, no
//! server GUI scripts) put every part on the same pixels: the slots at x
//! 15, 80 and 145 from y 30, the sword icon at (105, 31), the AP bar at
//! (15, 65), the gralat, bomb and arrow icons from (274, 30) with their
//! counts at y 48 (bombs and arrows two digits, zero-padded), and the
//! hearts from 198 px left of the right edge at y 24, 45 px apart. So the
//! 6.x default status display is this routine on the 6.x window, and
//! Issho draws it.
//!
//! Left out, as the 6.x lab run showed nothing of them: the key letters
//! on the slots, the small item-list key and the sword and glove power
//! numbers. The selected weapon's image waits for weapons (#445).

use crate::render::Frame;
use seihai_core::assets::{builtin, read_png_rgba, AssetResolver};
use seihai_core::client::OwnStats;
use seihai_draw::{draw_sprite, Image};

/// A decoded RGBA picture.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl Picture {
    fn view(&self) -> Image<'_> {
        Image::rgba(&self.rgba, self.w, self.h)
    }

    /// `name` from the asset folders (in `dir`), else the built-in file.
    fn load(resolver: Option<&AssetResolver>, dir: &str, name: &str) -> Option<Picture> {
        let from_disk = resolver
            .and_then(|r| r.image_in(dir, name))
            .and_then(|p| std::fs::read(p).ok());
        let rel = if dir.is_empty() {
            name.to_string()
        } else {
            format!("{dir}/{name}")
        };
        let data = from_disk.or_else(|| builtin::bytes(&rel).map(<[u8]>::to_vec))?;
        let (w, h, rgba) = read_png_rgba(&data).ok()?;
        Some(Picture { w, h, rgba })
    }
}

/// The pictures the status display draws from.
#[derive(Debug, Clone, Default)]
pub struct HudArt {
    /// `state.png`: hearts, icons, digits, slots and bars. Without it
    /// nothing is drawn.
    pub state: Option<Picture>,
    /// `sprites.png`, for the glove icons.
    pub sprites: Option<Picture>,
    /// `sword1.png`, for the sword icon beside the second slot.
    pub sword: Option<Picture>,
}

impl HudArt {
    /// The build's own pictures from the asset folders, else the built-in
    /// ones.
    pub fn load(resolver: Option<&AssetResolver>) -> HudArt {
        HudArt {
            state: Picture::load(resolver, "", "state.png"),
            sprites: Picture::load(resolver, "", "sprites.png"),
            sword: Picture::load(resolver, "swords", "sword1.png"),
        }
    }
}

/// What the status display shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudValues {
    /// Power in half hearts.
    pub half_hearts: i32,
    /// Heart containers in half hearts.
    pub max_half_hearts: i32,
    pub rupees: i32,
    pub bombs: i32,
    pub arrows: i32,
    /// AP, 0 to 100.
    pub ap: i32,
    pub glove_power: i32,
    /// Draw the sword icon (the player has a sword).
    pub sword: bool,
}

impl HudValues {
    /// A new player's counters, before the server sends any: three hearts,
    /// no gralats, five bombs, ten arrows, AP 50, glove 1, a sword. The
    /// 6.x lab run's world showed these.
    pub const NEW_PLAYER: HudValues = HudValues {
        half_hearts: 6,
        max_half_hearts: 6,
        rupees: 0,
        bombs: 5,
        arrows: 10,
        ap: 50,
        glove_power: 1,
        sword: true,
    };

    /// The counters the server sent, with a new player's for the rest.
    pub fn of(own: &OwnStats) -> HudValues {
        let d = HudValues::NEW_PLAYER;
        let max_half_hearts = own
            .max_power
            .map_or(d.max_half_hearts, |m| i32::from(m) * 2);
        HudValues {
            half_hearts: own
                .cur_power
                .map_or(max_half_hearts, i32::from)
                .min(max_half_hearts),
            max_half_hearts,
            rupees: own
                .rupees
                .map_or(d.rupees, |r| r.min(i32::MAX as u32) as i32),
            bombs: own.bombs.map_or(d.bombs, i32::from),
            arrows: own.arrows.map_or(d.arrows, i32::from),
            ap: own.alignment.map_or(d.ap, i32::from),
            glove_power: own.glove_power.map_or(d.glove_power, i32::from),
            sword: d.sword,
        }
    }
}

#[derive(Clone, Copy)]
struct Rect(i32, i32, i32, i32);

// Sheet rectangles (x, y, w, h), as in `documentation/hud.md`.
const HEART_BIG: Rect = Rect(0, 0, 40, 44);
const HEART_SMALL: Rect = Rect(80, 0, 31, 33);
/// Gralat, bomb and arrow icons, one strip.
const ICONS: Rect = Rect(80, 33, 120, 16);
const SLOT: Rect = Rect(202, 0, 22, 33);
const DIGIT_Y: i32 = 81;
const DIGIT_W: i32 = 14;
const DIGIT_H: i32 = 16;
const AP_FRAME: Rect = Rect(0, 97, 130, 22);
const AP_FILL_Y: i32 = 119;
const FILL_H: i32 = 10;

// Screen offsets from the view's top-left.
const BASE: (i32, i32) = (15, 15);
const SLOT_STEP: i32 = 65;
const ICONS_AT: (i32, i32) = (274, 30);
const COUNT_X: i32 = 280;
const COUNT_Y: i32 = 48;
const DIGIT_ADVANCE: i32 = 12;
const BOMBS_X: i32 = COUNT_X + 46;
const ARROWS_X: i32 = COUNT_X + 94;
/// The hearts start this far from the right edge.
pub const HEARTS_FROM_RIGHT: i32 = 198;
const HEARTS_Y: i32 = 24;
const BIG_STEP: i32 = 45;
const SMALL_STEP: i32 = 35;
/// Small hearts: a row starts at each of these heart numbers; row `k + 1`
/// moves `round((k + 1) * 17.5) - 35` px sideways and 20 down.
const SMALL_ROWS: [i32; 5] = [5, 11, 16, 22, 27];
const SMALL_ROW_DY: i32 = 20;
/// Power up to this many hearts is drawn with the big hearts.
const BIG_MAX_HEARTS: i32 = 4;

/// Icon rectangles on `sprites.png` and the sword image (the 2.22
/// `arrays.dat` places): the sword (17) and the gloves (145 for power 2,
/// 325 drawn over it for power 3).
const SWORD_ICON: Rect = Rect(94, 36, 28, 28);
const GLOVE_ICONS: [Rect; 2] = [Rect(28, 212, 28, 32), Rect(0, 212, 28, 32)];

fn has(p: &Picture, r: Rect) -> bool {
    r.0 >= 0 && r.1 >= 0 && r.2 > 0 && r.3 > 0 && r.0 + r.2 <= p.w as i32 && r.1 + r.3 <= p.h as i32
}

fn blit(f: &mut Frame, p: &Picture, r: Rect, x: i32, y: i32) {
    if has(p, r) {
        draw_sprite(f, p.view(), (r.0, r.1, r.2, r.3), x, y);
    }
}

/// Draw the status display over `f`.
pub fn draw(f: &mut Frame, art: &HudArt, v: &HudValues) {
    let Some(s) = art.state.as_ref() else {
        return;
    };
    let (bx, by) = BASE;
    blit(f, s, ICONS, ICONS_AT.0, ICONS_AT.1);
    draw_hearts(f, s, v);
    // Up to seven digits, no leading zeros, centred on x 280.
    let text = v.rupees.clamp(0, 9_999_999).to_string();
    draw_number(
        f,
        s,
        &text,
        COUNT_X - ((text.len() as i32 * DIGIT_ADVANCE) >> 1),
    );
    draw_number(f, s, &format!("{:02}", v.bombs.clamp(0, 99)), BOMBS_X);
    draw_number(f, s, &format!("{:02}", v.arrows.clamp(0, 99)), ARROWS_X);
    for i in 0..3 {
        blit(f, s, SLOT, bx + i * SLOT_STEP, by + 15);
    }
    if let Some(sp) = art.sprites.as_ref() {
        for (n, &r) in GLOVE_ICONS.iter().enumerate() {
            if v.glove_power >= 2 + n as i32 {
                blit(f, sp, r, bx + 25, by + 14);
            }
        }
    }
    if let (Some(sw), true) = (art.sword.as_ref(), v.sword) {
        blit(f, sw, SWORD_ICON, bx + 90, by + 16);
    }
    if has(s, AP_FRAME) {
        blit(f, s, AP_FRAME, bx, by + 50);
        let fill = Rect(0, AP_FILL_Y, v.ap.clamp(0, 100), FILL_H);
        blit(f, s, fill, bx + 22, by + 56);
    }
}

fn draw_number(f: &mut Frame, s: &Picture, text: &str, x: i32) {
    for (i, c) in text.bytes().enumerate() {
        let d = i32::from(c - b'0');
        let r = Rect(d * DIGIT_W, DIGIT_Y, DIGIT_W, DIGIT_H);
        blit(f, s, r, x + i as i32 * DIGIT_ADVANCE, COUNT_Y);
    }
}

fn draw_hearts(f: &mut Frame, s: &Picture, v: &HudValues) {
    let containers = (v.max_half_hearts.max(0) + 1) / 2;
    let power = v.half_hearts.clamp(0, containers * 2);
    let x0 = f.width as i32 - HEARTS_FROM_RIGHT;
    // Heart i is full when power - 1 >= i (in hearts), half when i < power.
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
            let r = Rect(
                HEART_BIG.0 + half * HEART_BIG.2,
                0,
                HEART_BIG.2,
                HEART_BIG.3,
            );
            blit(f, s, r, x0 + i * BIG_STEP, HEARTS_Y);
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
            let r = Rect(
                HEART_SMALL.0 + half * HEART_SMALL.2,
                0,
                HEART_SMALL.2,
                HEART_SMALL.3,
            );
            blit(f, s, r, x, y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An opaque white sheet `h` rows high.
    fn white(h: u32) -> HudArt {
        let p = |w: u32, h: u32| Picture {
            w,
            h,
            rgba: vec![0xFF; (w * h * 4) as usize],
        };
        HudArt {
            state: Some(p(224, h)),
            sprites: Some(p(256, 256)),
            sword: Some(p(128, 128)),
        }
    }

    fn lit(f: &Frame, x: i32, y: i32) -> bool {
        f.pixel(x as u32, y as u32)
            .is_some_and(|p| p[..3] != [0, 0, 0])
    }

    fn frame(v: &HudValues, art: &HudArt) -> Frame {
        let mut f = Frame::filled(1024, 768, [0, 0, 0, 255]);
        draw(&mut f, art, v);
        f
    }

    /// The 6.x lab run's measurements at 1024 x 768: three hearts at x
    /// 826, 871 and 916 from y 24; slots at 15, 80 and 145 from y 30;
    /// the sword at 105; the AP bar at (15, 65); the icons from 274.
    #[test]
    fn the_6x_default_lands_where_the_lab_measured_it() {
        let f = frame(&HudValues::NEW_PLAYER, &white(145));
        for x in [826, 871, 916] {
            assert!(lit(&f, x + 20, 24 + 20), "heart at {x}");
        }
        assert!(!lit(&f, 961 + 20, 24 + 20), "no fourth heart");
        for x in [15, 80, 145] {
            assert!(lit(&f, x + 10, 30 + 15), "slot at {x}");
        }
        assert!(lit(&f, 105 + 10, 31 + 10), "sword icon");
        assert!(lit(&f, 15 + 60, 65 + 2), "AP bar");
        assert!(lit(&f, 274 + 8, 30 + 8), "icons");
        // Bombs "05" and arrows "10": two digits each, 326 and 374.
        assert!(lit(&f, 326 + 2, COUNT_Y + 8) && lit(&f, 338 + 13, COUNT_Y + 8));
        assert!(lit(&f, 374 + 2, COUNT_Y + 8));
    }

    #[test]
    fn the_hearts_follow_the_right_edge() {
        let mut f = Frame::filled(640, 480, [0, 0, 0, 255]);
        draw(&mut f, &white(145), &HudValues::NEW_PLAYER);
        assert!(lit(&f, 442 + 20, 44));
    }

    #[test]
    fn half_a_heart_and_small_rows() {
        let mut v = HudValues::NEW_PLAYER;
        v.half_hearts = 5;
        let f = frame(&v, &white(145));
        assert!(lit(&f, 916 + 5, 44), "the third heart is half");
        v.half_hearts = 12;
        v.max_half_hearts = 12;
        let f = frame(&v, &white(145));
        // Six hearts: small, heart 5 starts row 1, 17 px left and 20 down.
        assert!(lit(&f, 826 - 17 + 15, 44 + 16));
    }

    #[test]
    fn no_state_png_draws_nothing_and_a_short_sheet_has_no_ap_bar() {
        let f = frame(&HudValues::NEW_PLAYER, &HudArt::default());
        assert!(f.rgba.chunks(4).all(|p| p[..3] == [0, 0, 0]));
        let f = frame(&HudValues::NEW_PLAYER, &white(97));
        assert!(!lit(&f, 15 + 60, 65 + 2));
    }

    #[test]
    fn the_servers_counters_win_over_the_new_player_ones() {
        let own = OwnStats {
            max_power: Some(5),
            cur_power: Some(7),
            rupees: Some(123),
            bombs: Some(9),
            ..Default::default()
        };
        let v = HudValues::of(&own);
        assert_eq!((v.max_half_hearts, v.half_hearts), (10, 7));
        assert_eq!((v.rupees, v.bombs, v.arrows, v.ap), (123, 9, 10, 50));
        // Power above the containers is cut to them.
        let v = HudValues::of(&OwnStats {
            max_power: Some(2),
            cur_power: Some(9),
            ..Default::default()
        });
        assert_eq!(v.half_hearts, 4);
    }

    #[test]
    fn the_built_in_art_loads() {
        let art = HudArt::load(None);
        assert!(art.state.is_some(), "built-in state.png");
    }
}
