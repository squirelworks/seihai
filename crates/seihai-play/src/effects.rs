//! What the players' actions put on the level, drawn as `seihai-mae`'s
//! `render::throw` draws it: the items on the floor (bouncing, blinking
//! before they go), the bombs with their shadows, the blast pieces, the
//! arrows, the carried object over the head and thrown ones in the air,
//! the break effects (a pot breaking, a bomb into water) and the item held
//! over the head in the lay mode.
//!
//! Each image is a rectangle of `sprites.png` cut out with core's sprite
//! table ([`seihai_core::throwing::sprite_rect`]); bombs and their blasts
//! come from `bomb1.png` (power 1 and 2) and `bomb2.png` (power 3)
//! instead. A sheet that is not in the asset folders draws nothing.

use crate::render::{Frame, Image, TILE_PX};
use seihai_core::assets::AssetResolver;
use seihai_core::bombs::{self as b, BombSheet};
use seihai_core::offline::OfflineSession;
use seihai_core::throwing as t;
use seihai_draw::draw_sprite;

/// `sprites.png`, `bomb1.png` and `bomb2.png`, where found.
#[derive(Debug, Clone, Default)]
pub struct EffectSheets {
    pub sprites: Option<Image>,
    pub bomb1: Option<Image>,
    pub bomb2: Option<Image>,
}

impl EffectSheets {
    /// The sheets from the asset folders; `sprites.png` falls back on the
    /// built-in one.
    pub fn load(resolver: Option<&AssetResolver>) -> EffectSheets {
        let read = |name: &str| {
            resolver
                .and_then(|r| r.image(name))
                .and_then(|p| std::fs::read(p).ok())
                .and_then(|d| Image::decode(name, &d))
        };
        EffectSheets {
            sprites: read("sprites.png").or_else(|| {
                seihai_core::assets::builtin::bytes("sprites.png")
                    .and_then(|d| Image::decode("sprites.png", d))
            }),
            bomb1: read("bomb1.png").or_else(|| read("bomb1.gif")),
            bomb2: read("bomb2.png").or_else(|| read("bomb2.gif")),
        }
    }

    fn bomb(&self, which: BombSheet) -> Option<&Image> {
        match which {
            BombSheet::One => self.bomb1.as_ref(),
            BombSheet::Two => self.bomb2.as_ref(),
        }
    }

    /// The sheet sprite `id` is cut from: a bomb sheet for a bomb, else
    /// `sprites.png`.
    fn for_sprite(&self, id: u16) -> Option<&Image> {
        match b::sheet_of_sprite(id) {
            Some(which) => self.bomb(which),
            None => self.sprites.as_ref(),
        }
    }
}

fn px(v: f32) -> i32 {
    (v * TILE_PX as f32).round() as i32
}

/// Sprite `id` with its top-left at `(sx, sy)`. A rectangle outside the
/// sheet draws nothing.
fn sprite(f: &mut Frame, sheet: Option<&Image>, s: &OfflineSession, id: u16, sx: i32, sy: i32) {
    let Some(sheet) = sheet else { return };
    let Some((x, y, w, h)) = t::sprite_rect(s.object_tables.as_ref(), id) else {
        return;
    };
    if x < 0 || y < 0 || x + w > sheet.w as i32 || y + h > sheet.h as i32 {
        return;
    }
    let img = seihai_draw::Image::rgba(&sheet.rgba, sheet.w, sheet.h);
    draw_sprite(f, img, (x, y, w, h), sx, sy);
}

/// The size of sprite `id`.
fn size(s: &OfflineSession, id: u16) -> Option<(i32, i32)> {
    t::sprite_rect(s.object_tables.as_ref(), id).map(|(_, _, w, h)| (w, h))
}

/// The items on the floor, as 2.05 draws them: the shadow (sprite 42) at
/// `(x + 0.5, y + 1 + h / 2 - 0.5)` for an item `h` tiles high, then the
/// item centred on `(x + 1, y + 1)`, raised by its bounce. On its blink's
/// off ticks nothing is drawn. Under the players.
pub fn draw_floor_items(f: &mut Frame, s: &OfflineSession, sheets: &EffectSheets, cam: (i32, i32)) {
    let (cam_x, cam_y) = cam;
    let sheet = sheets.sprites.as_ref();
    for it in s.floor_items.iter().filter(|it| it.visible()) {
        let Some(id) = it.sprite() else { continue };
        let Some((w, h)) = size(s, id) else { continue };
        let h_tiles = h as f32 / TILE_PX as f32;
        sprite(
            f,
            sheet,
            s,
            seihai_core::items::ITEM_SHADOW_SPRITE,
            px(it.x + 0.5) - cam_x,
            px(it.y + 1.0 + h_tiles / 2.0 - 0.5) - cam_y,
        );
        let (cx, cy) = (px(it.x + 1.0), px(it.y + 1.0 - it.lift()));
        sprite(f, sheet, s, id, cx - w / 2 - cam_x, cy - h / 2 - cam_y);
    }
}

/// A held object, sprite `id`, with its top-left at `at` (tiles): over
/// the head ([`t::carry_origin`]) or at the feet ([`t::hide_origin`]).
pub fn draw_held(
    f: &mut Frame,
    s: &OfflineSession,
    sheets: &EffectSheets,
    id: u16,
    at: (f32, f32),
    cam: (i32, i32),
) {
    sprite(
        f,
        sheets.for_sprite(id),
        s,
        id,
        px(at.0) - cam.0,
        px(at.1) - cam.1,
    );
}

/// The bombs, the item held in the lay mode, the blasts, the arrows and
/// the break effects, over the players.
pub fn draw_effects(f: &mut Frame, s: &OfflineSession, sheets: &EffectSheets, cam: (i32, i32)) {
    let (cam_x, cam_y) = cam;
    let sprites = sheets.sprites.as_ref();
    for g in &s.grounded {
        sprite(
            f,
            sheets.for_sprite(g.sprite),
            s,
            g.sprite,
            px(g.x) - cam_x,
            px(g.y) - cam_y,
        );
    }
    for bomb in &s.placed_bombs {
        // The shadow, then the bomb centred on (x + 1, y + 1).
        sprite(
            f,
            sprites,
            s,
            t::SHADOW_SPRITE,
            px(bomb.x + 0.25) - cam_x,
            px(bomb.y + 1.25) - cam_y,
        );
        let Some((w, h)) = size(s, bomb.sprite) else {
            continue;
        };
        sprite(
            f,
            sheets.bomb(b::sheet_of_bomb(bomb.power)),
            s,
            bomb.sprite,
            px(bomb.x + 1.0) - w / 2 - cam_x,
            px(bomb.y + 1.0) - h / 2 - cam_y,
        );
    }
    // The shadows of what is in the air.
    for o in &s.thrown {
        sprite(
            f,
            sprites,
            s,
            t::SHADOW_SPRITE,
            px(o.x + 0.25) - cam_x,
            px(o.ground + 1.25) - cam_y,
        );
    }
    // The carried object above the head; hidden under it (2.05 on), where
    // the player stands.
    if let Some(id) = s.carried.as_ref().and_then(|c| t::carry_sprite(c.kind)) {
        let at = if s.figure_hidden() {
            t::hide_origin(s.player_x, s.player_y)
        } else {
            t::carry_origin(s.player_x, s.player_y)
        };
        draw_held(f, s, sheets, id, at, cam);
    }
    // The item picked in the lay mode, centred above the head.
    if let Some(id) = s.laying.and_then(|i| {
        seihai_core::items::ITEM_SPRITES
            .get(usize::from(i))
            .copied()
    }) {
        if let Some((w, h)) = size(s, id) {
            let (x, y) = seihai_core::offline::lay_held_centre(s.player_x, s.player_y);
            sprite(
                f,
                sprites,
                s,
                id,
                px(x) - w / 2 - cam_x,
                px(y) - h / 2 - cam_y,
            );
        }
    }
    // What is in the air.
    for o in &s.thrown {
        if let Some(id) = o.sprite {
            sprite(
                f,
                sheets.for_sprite(id),
                s,
                id,
                px(o.x) - cam_x,
                px(o.y) - cam_y,
            );
        }
    }
    for p in &s.blast {
        sprite(
            f,
            sheets.bomb(b::sheet_of_piece(p.power)),
            s,
            p.sprite,
            px(p.x) - cam_x,
            px(p.y) - cam_y,
        );
    }
    for a in &s.arrows {
        sprite(f, sprites, s, a.sprite, px(a.x) - cam_x, px(a.y) - cam_y);
    }
    for l in s.leaps.iter().filter(|l| l.frame >= 0) {
        let frames = t::leap_frames(s.object_tables.as_ref(), l.kind);
        let Some(frame) = frames.get(l.frame as usize) else {
            continue;
        };
        for &(id, dx, dy) in frame {
            sprite(
                f,
                sprites,
                s,
                id,
                px(l.x) + i32::from(dx) - cam_x,
                px(l.y) + i32::from(dy) - cam_y,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::assets::Level;

    /// A session whose sprite table gives every id the built-in sheet's
    /// top-left 16 x 12 pixels (the build's `arrays.dat` names the real
    /// rectangles).
    fn session() -> OfflineSession {
        let mut s = OfflineSession::default();
        s.object_tables = Some(seihai_core::assets::ObjectTables {
            sprites: vec![(8 << 6) | 6; 400],
            ..Default::default()
        });
        s.current_level = Some(Level::from_board(&[0; 4096]));
        s.player_x = 30.0;
        s.player_y = 30.0;
        s
    }

    fn sheets() -> EffectSheets {
        EffectSheets::load(None)
    }

    /// An arrow in flight is drawn from the built-in sprite sheet.
    #[test]
    fn an_arrow_is_drawn() {
        let mut s = session();
        let before = Frame::filled(320, 240, [0, 0, 0, 255]);
        let mut f = before.clone();
        s.shoot_arrow(30.0, 30.0, 2.0, 2.0, 2, true);
        assert_eq!(s.arrows.len(), 1);
        let cam = (px(30.0) - 160, px(30.0) - 120);
        draw_effects(&mut f, &s, &sheets(), cam);
        assert!(f.rgba != before.rgba, "nothing drawn");
    }

    /// A floor item and its shadow are drawn; with no items, nothing is.
    #[test]
    fn a_floor_item_is_drawn() {
        let mut s = session();
        let before = Frame::filled(320, 240, [0, 0, 0, 255]);
        let mut f = before.clone();
        let cam = (px(30.0) - 160, px(30.0) - 120);
        draw_floor_items(&mut f, &s, &sheets(), cam);
        assert!(f.rgba == before.rgba, "drawn with no items");
        s.floor_items
            .push(seihai_core::items::FloorItem::new(0, 31.0, 31.0));
        draw_floor_items(&mut f, &s, &sheets(), cam);
        assert!(f.rgba != before.rgba, "nothing drawn");
    }
}
