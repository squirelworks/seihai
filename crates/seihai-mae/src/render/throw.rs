//! Drawing carried and thrown objects and their break effects.
//!
//! Every image is a rectangle of the sprite sheet (`sprites.png`) cut out
//! with the sprite table ([`crate::throwing::sprite_rect`]) and drawn with
//! its transparent pixels skipped, so only the object's own pixels reach
//! the screen (the surrounding board tiles are not part of it). Bombs and
//! their explosions are cut from `bomb1.png` and `bomb2.png` instead
//! ([`crate::bombs`]).

use super::tiles::{Framebuffer, Sheet, TILE_PX};
use crate::bombs::{self as b, BombSheet};
use crate::offline::OfflineSession;
use crate::throwing as t;

/// `bomb1.png` and `bomb2.png`, where loaded. Without one, what it holds
/// is not drawn.
#[derive(Debug, Clone, Copy, Default)]
pub struct BombSheets<'a> {
    pub one: Option<Sheet<'a>>,
    pub two: Option<Sheet<'a>>,
}

impl<'a> BombSheets<'a> {
    fn get(&self, which: BombSheet) -> Option<Sheet<'a>> {
        match which {
            BombSheet::One => self.one,
            BombSheet::Two => self.two,
        }
    }

    /// The sheet sprite `id` of a carried, thrown or landed object is cut
    /// from: a bomb sheet for a bomb, else `sprites`.
    fn for_sprite(&self, sprites: Sheet<'a>, id: u16) -> Option<Sheet<'a>> {
        match b::sheet_of_sprite(id) {
            Some(which) => self.get(which),
            None => Some(sprites),
        }
    }
}

/// Draw sprite `id` with its top-left at screen `(sx, sy)`. A sprite whose
/// rectangle is not inside the sheet (a sheet from before these sprites
/// existed) draws nothing. Returns whether pixels were offered.
pub fn draw_sprite(fb: &mut Framebuffer, sheet: Sheet<'_>, tables: Option<&crate::assets::ObjectTables>, id: u16, sx: i32, sy: i32) -> bool {
    let Some((x, y, w, h)) = t::sprite_rect(tables, id) else { return false };
    let rows = sheet.px.len() / (sheet.w as usize * sheet.bpp).max(1);
    if x < 0 || y < 0 || x + w > sheet.w as i32 || y + h > rows as i32 {
        return false;
    }
    fb.blit_sprite(sheet.px, sheet.w, sheet.bpp, x, y, w, h, sx, sy);
    true
}

fn px(v: f32) -> i32 {
    (v * TILE_PX as f32).round() as i32
}

/// The carried object above the head, then everything in flight, then the
/// break effects and what stayed on the ground, from the camera `(cam_x,
/// cam_y)` in pixels.
pub fn draw_throw_layer(fb: &mut Framebuffer, session: &OfflineSession, sprites: Sheet<'_>, bombs: BombSheets<'_>, cam_x: i32, cam_y: i32) {
    let tables = session.object_tables.as_ref();
    for g in &session.grounded {
        if let Some(sheet) = bombs.for_sprite(sprites, g.sprite) {
            draw_sprite(fb, sheet, tables, g.sprite, px(g.x) - cam_x, px(g.y) - cam_y);
        }
    }
    for bomb in &session.placed_bombs {
        // The shadow, then the bomb centred on (x + 1, y + 1).
        draw_sprite(fb, sprites, tables, t::SHADOW_SPRITE, px(bomb.x + 0.25) - cam_x, px(bomb.y + 1.25) - cam_y);
        let Some(sheet) = bombs.get(b::sheet_of_bomb(bomb.power)) else { continue };
        let Some((_, _, w, h)) = t::sprite_rect(tables, bomb.sprite) else { continue };
        draw_sprite(fb, sheet, tables, bomb.sprite, px(bomb.x + 1.0) - w / 2 - cam_x, px(bomb.y + 1.0) - h / 2 - cam_y);
    }
    for o in &session.thrown {
        draw_sprite(fb, sprites, tables, t::SHADOW_SPRITE, px(o.x + 0.25) - cam_x, px(o.ground + 1.25) - cam_y);
    }
    if let Some(c) = &session.carried {
        if let Some(id) = t::carry_sprite(c.kind) {
            // Hidden (2.05 on): the object sits where the player stands.
            let (x, y) = if session.figure_hidden() {
                t::hide_origin(session.player_x, session.player_y)
            } else {
                t::carry_origin(session.player_x, session.player_y)
            };
            if let Some(sheet) = bombs.for_sprite(sprites, id) {
                draw_sprite(fb, sheet, tables, id, px(x) - cam_x, px(y) - cam_y);
            }
        }
    }
    for o in &session.thrown {
        if let Some(sheet) = o.sprite.and_then(|id| bombs.for_sprite(sprites, id)) {
            draw_sprite(fb, sheet, tables, o.sprite.unwrap_or_default(), px(o.x) - cam_x, px(o.y) - cam_y);
        }
    }
    for p in &session.blast {
        if let Some(sheet) = bombs.get(b::sheet_of_piece(p.power)) {
            draw_sprite(fb, sheet, tables, p.sprite, px(p.x) - cam_x, px(p.y) - cam_y);
        }
    }
    for a in &session.arrows {
        draw_sprite(fb, sprites, tables, a.sprite, px(a.x) - cam_x, px(a.y) - cam_y);
    }
    for l in &session.leaps {
        if l.frame < 0 {
            continue;
        }
        let frames = t::leap_frames(tables, l.kind);
        let Some(frame) = frames.get(l.frame as usize) else { continue };
        for &(id, dx, dy) in frame {
            draw_sprite(fb, sprites, tables, id, px(l.x) + i32::from(dx) - cam_x, px(l.y) + i32::from(dy) - cam_y);
        }
    }
}

/// The items on the floor ([`crate::items`]), as 2.05 draws them: the
/// shadow (sprite 42) at `(x + 0.5, y + 1 + h / 2 - 0.5)` for an item `h`
/// tiles high, then the item centred on `(x + 1, y + 1)`, raised by its
/// bounce. On its blink's off ticks nothing is drawn.
pub fn draw_floor_items(fb: &mut Framebuffer, session: &OfflineSession, sprites: Sheet<'_>, cam_x: i32, cam_y: i32) {
    let tables = session.object_tables.as_ref();
    for it in &session.floor_items {
        if !it.visible() {
            continue;
        }
        let Some(id) = it.sprite() else { continue };
        let Some((_, _, w, h)) = t::sprite_rect(tables, id) else { continue };
        let h_tiles = h as f32 / TILE_PX as f32;
        draw_sprite(fb, sprites, tables, crate::items::ITEM_SHADOW_SPRITE, px(it.x + 0.5) - cam_x, px(it.y + 1.0 + h_tiles / 2.0 - 0.5) - cam_y);
        let (cx, cy) = (px(it.x + 1.0), px(it.y + 1.0 - it.lift()));
        draw_sprite(fb, sprites, tables, id, cx - w / 2 - cam_x, cy - h / 2 - cam_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offline::{OfflineSession, DIR_EAST};
    use crate::throwing::{Grounded, Leap, Thrown};

    const BG: [u8; 3] = [10, 120, 10];

    /// A 128 x 402 RGBA sheet, fully transparent except the given
    /// rectangles, which are filled with one colour each.
    fn sheet(fills: &[((i32, i32, i32, i32), [u8; 3])]) -> (u32, Vec<u8>) {
        let (w, h) = (128usize, 402usize);
        let mut px = vec![0u8; w * h * 4];
        for &((x, y, rw, rh), c) in fills {
            for yy in y..y + rh {
                for xx in x..x + rw {
                    let i = (yy as usize * w + xx as usize) * 4;
                    px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
                }
            }
        }
        (128, px)
    }

    fn fb() -> Framebuffer {
        let mut f = Framebuffer::new(640, 640);
        for p in f.pixels.chunks_exact_mut(3) {
            p.copy_from_slice(&BG);
        }
        f
    }

    fn at(f: &Framebuffer, x: i32, y: i32) -> [u8; 3] {
        let i = (y as usize * f.width as usize + x as usize) * 3;
        [f.pixels[i], f.pixels[i + 1], f.pixels[i + 2]]
    }

    fn session(carried: bool) -> OfflineSession {
        let mut s = OfflineSession::new();
        s.player_x = 10.0;
        s.player_y = 20.0;
        s.player_dir = DIR_EAST;
        if carried {
            s.carried = Some(crate::offline::Carried { kind: 0, tiles: [[1, 2], [3, 4]] });
        }
        s
    }

    #[test]
    fn the_carried_bush_is_its_own_pixels_only() {
        // A bush sprite (rect 0,338 32x32) with only a 8x8 patch opaque: the
        // rest of its 32x32 square must leave the grass behind it alone.
        let (w, px) = sheet(&[((12, 350, 8, 8), [200, 30, 30])]);
        let mut f = fb();
        let s = session(true);
        draw_throw_layer(&mut f, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 0, 0);
        // carry origin (10.5, 18.625) -> pixel (168, 298); patch at (+12, +12)
        assert_eq!(at(&f, 168 + 12, 298 + 12), [200, 30, 30]);
        assert_eq!(at(&f, 168 + 19, 298 + 19), [200, 30, 30]);
        assert_eq!(at(&f, 168 + 11, 298 + 12), BG, "left of the patch: transparent");
        assert_eq!(at(&f, 168, 298), BG, "the sprite's corner is not painted");
        assert_eq!(at(&f, 168 + 31, 298 + 31), BG);
        let painted = f.pixels.chunks_exact(3).filter(|p| **p != BG).count();
        assert_eq!(painted, 64, "exactly the object's 8x8 opaque pixels");
    }

    #[test]
    fn a_sheet_without_the_sprite_rows_draws_nothing() {
        let px = vec![255u8; 32 * 338 * 4];
        let mut f = fb();
        draw_throw_layer(&mut f, &session(true), Sheet { px: &px, w: 32, bpp: 4 }, BombSheets::default(), 0, 0);
        assert!(f.pixels.chunks_exact(3).all(|p| *p == BG));
    }

    #[test]
    fn a_flying_object_has_a_shadow_on_the_ground_and_moves_with_the_camera() {
        let (w, px) = sheet(&[((0, 0, 24, 12), [1, 1, 1]), ((0, 338, 32, 32), [200, 30, 30])]);
        let mut s = session(false);
        s.thrown.push(Thrown::launch(0, 10.0, 20.0, DIR_EAST));
        let mut f = fb();
        draw_throw_layer(&mut f, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 16, 32);
        // object at (10.5, 18.625): (168, 298) minus the camera (16, 32)
        assert_eq!(at(&f, 152, 266), [200, 30, 30]);
        // shadow at (10.75, 22.25) -> (172, 356) minus the camera
        assert_eq!(at(&f, 156, 324), [1, 1, 1]);
        assert_eq!(at(&f, 156 + 23, 324 + 11), [1, 1, 1]);
    }

    #[test]
    fn break_pieces_follow_the_frame_and_are_gone_before_the_first_tick() {
        let (w, px) = sheet(&[((24, 82, 16, 16), [90, 90, 90]), ((24, 98, 16, 16), [91, 91, 91]), ((40, 98, 16, 16), [92, 92, 92])]);
        let mut s = session(false);
        s.leaps.push(Leap { kind: crate::throwing::LEAP_ROCK, x: 20.0, y: 30.0, frame: -1 });
        let mut f = fb();
        draw_throw_layer(&mut f, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 0, 0);
        assert!(f.pixels.chunks_exact(3).all(|p| *p == BG), "frame -1 draws nothing");
        s.leaps[0].frame = 0;
        draw_throw_layer(&mut f, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 0, 0);
        // frame 0 of kind 2: (96, -1, 0), (96, 0, 9), (98, 9, -1), (98, 10, 8)
        assert_eq!(at(&f, 320 - 1, 480), [90, 90, 90]);
        assert_eq!(at(&f, 320, 480 + 9), [90, 90, 90]);
        assert_eq!(at(&f, 320 + 9, 480 - 1), [91, 91, 91]);
        assert_eq!(at(&f, 320 + 10 + 15, 480 + 8 + 15), [91, 91, 91]);
        // frame 1 moves outward
        s.leaps[0].frame = 1;
        let mut g = fb();
        draw_throw_layer(&mut g, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 0, 0);
        assert_eq!(at(&g, 320 - 3, 480 - 2), [92, 92, 92]);
    }

    #[test]
    fn a_grounded_bomb_is_drawn_where_it_landed_from_the_loaded_sprite_table() {
        // sprite 65 has no built-in rect: the sprite table of an arrays.dat gives it one
        let (w, px) = sheet(&[((2, 100, 8, 8), [7, 7, 200])]);
        let mut tables = crate::assets::ObjectTables::default();
        tables.sprites = vec![0; 66];
        tables.sprites[65] = (1 << 24) | (50 << 12) | (4 << 6) | 4;
        let mut s = session(false);
        s.object_tables = Some(tables);
        s.grounded.push(Grounded { sprite: 65, x: 5.0, y: 5.0 });
        // a bomb's sprite comes from bomb1.png, not sprites.png
        let (_, blank) = sheet(&[]);
        let one = BombSheets { one: Some(Sheet { px: &px, w, bpp: 4 }), two: None };
        let mut f = fb();
        draw_throw_layer(&mut f, &s, Sheet { px: &blank, w, bpp: 4 }, one, 0, 0);
        assert_eq!(at(&f, 80, 80), [7, 7, 200]);
        assert_eq!(at(&f, 87, 87), [7, 7, 200]);
        assert_eq!(at(&f, 88, 88), BG);
        // without bomb1.png it is not cut from sprites.png instead
        let mut g = fb();
        draw_throw_layer(&mut g, &s, Sheet { px: &px, w, bpp: 4 }, BombSheets::default(), 0, 0);
        assert!(g.pixels.chunks_exact(3).all(|p| *p == BG));
        // without the table nothing is known about it: nothing is drawn
        s.object_tables = None;
        let mut g = fb();
        draw_throw_layer(&mut g, &s, Sheet { px: &px, w, bpp: 4 }, one, 0, 0);
        assert!(g.pixels.chunks_exact(3).all(|p| *p == BG));
    }

    #[test]
    fn a_bomb_and_its_pieces_come_from_the_bomb_sheets() {
        // sprite 65 and piece 278 at the same rect; the power-3 pair 347 too
        let mut tables = crate::assets::ObjectTables::default();
        tables.sprites = vec![0; 349];
        for id in [65, 278, 347] {
            tables.sprites[id] = (4 << 6) | 4; // (0, 0, 8, 8)
        }
        let (w, one_px) = sheet(&[((0, 0, 8, 8), [1, 2, 3])]);
        let (_, two_px) = sheet(&[((0, 0, 8, 8), [4, 5, 6])]);
        let (_, sprites_px) = sheet(&[((0, 0, 8, 8), [9, 9, 9])]);
        let bombs = BombSheets { one: Some(Sheet { px: &one_px, w, bpp: 4 }), two: Some(Sheet { px: &two_px, w, bpp: 4 }) };
        let sprites = Sheet { px: &sprites_px, w, bpp: 4 };
        let mut s = session(false);
        s.object_tables = Some(tables);
        // power 1 at 5,5: the shadow (sprite 0, 24 x 12 built in) at 5.25,6.25,
        // the bomb (8 x 8) centred on 6,6
        s.placed_bombs.push(crate::bombs::Bomb::new(5.0, 5.0, 1, 60));
        let mut f = fb();
        draw_throw_layer(&mut f, &s, sprites, bombs, 0, 0);
        assert_eq!(at(&f, 96 - 4, 96 - 4), [1, 2, 3]);
        assert_eq!(at(&f, 96 + 3, 96 + 3), [1, 2, 3]);
        assert_eq!(at(&f, 84, 100), [9, 9, 9]);
        // power 3 from bomb2.png
        s.placed_bombs[0] = crate::bombs::Bomb::new(5.0, 5.0, 3, 60);
        let mut f = fb();
        draw_throw_layer(&mut f, &s, sprites, bombs, 0, 0);
        assert_eq!(at(&f, 96 - 4, 96 - 4), [4, 5, 6]);
        // a piece of power 1 from bomb1.png, of power 3 from bomb2.png
        s.placed_bombs.clear();
        s.blast.push(crate::bombs::Piece::new(0, 10.0, 10.0, 1));
        s.blast.push(crate::bombs::Piece::new(0, 20.0, 10.0, 3));
        let mut f = fb();
        draw_throw_layer(&mut f, &s, sprites, bombs, 0, 0);
        assert_eq!(at(&f, 160, 160), [1, 2, 3]);
        assert_eq!(at(&f, 320, 160), [4, 5, 6]);
    }
}
