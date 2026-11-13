//! Stage 3: the flat 2D fallback renderer. It draws the level the way
//! `seihai-mae` does (`render::tiles::render_view`): 16-pixel tiles from a
//! `pics1.png` sheet, the camera centred on the player's 48x48 box, chest
//! records over the board, each link as an outline of its rectangle, the
//! neighbouring levels of a gmap cell past its edges, the level's NPCs as
//! their images (or an outline where the image is not on disk), and the
//! players from their GANIs in the looks the server sent
//! ([`crate::chars`]). A player whose pose GANI is missing is a plain
//! marker. No HUD.
//!
//! Every pixel goes through `seihai-draw` (seihai-core): the tile sheet
//! maths, the sprite copy and the gani frames are the ones the level
//! editor and tools use, not copies of `seihai-mae`'s (#443).
//!
//! The 3.x clients drew their 2D levels from the same board and tile sheet
//! as 2.x, so this is a stand-in until Ato has its own 3.x view (and Issho
//! its 6.x one).

use crate::chars::{self, Characters, Figure};
use seihai_core::assets::gif::read_gif_rgba;
use seihai_core::assets::{
    builtin, read_png_rgb, read_png_rgba, tile_id, AssetResolver, Level, TileTypes, TileTypesFile,
};
use seihai_core::offline::OfflineSession;
use seihai_draw::{draw_sprite, draw_tile, PixelBuffer};
use std::collections::HashMap;

/// Width of a tile in pixels.
pub const TILE_PX: u32 = seihai_draw::TILE_PX;
/// The 2x2 chest art, closed and open (`seihai-mae` `render::tiles`).
pub const CHEST_CLOSED_TILES: [u16; 4] = [1784, 1785, 1800, 1801];
pub const CHEST_OPEN_TILES: [u16; 4] = [829, 830, 845, 846];
/// The player marker: red, blue in water.
const MARKER: [u8; 3] = [0xFF, 0x40, 0x40];
const MARKER_WATER: [u8; 3] = [0x40, 0x80, 0xFF];
/// Another player's marker.
const MARKER_OTHER: [u8; 3] = [0xFF, 0xD0, 0x40];
/// The outline drawn where an NPC's image is not on disk.
pub const NPC_MISSING: [u8; 3] = [0xC0, 0x60, 0xFF];
/// The box an NPC with a missing image is outlined in, in pixels.
const NPC_MISSING_PX: i32 = 32;

/// A decoded NPC image.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    /// A PNG or GIF file's bytes, by the name's extension. Other formats
    /// (`.mng`, `.bmp`) are not read here and give `None`.
    pub fn decode(name: &str, data: &[u8]) -> Option<Image> {
        let lower = name.to_ascii_lowercase();
        let (w, h, rgba) = if lower.ends_with(".png") {
            read_png_rgba(data).ok()?
        } else if lower.ends_with(".gif") {
            read_gif_rgba(data).ok()?
        } else {
            return None;
        };
        Some(Image { w, h, rgba })
    }
}

/// NPC images by name as the level spells them. `None` is an image looked
/// for and not found, so it is not looked for again.
pub type NpcImages = HashMap<String, Option<Image>>;

/// Read every NPC image of the level not looked for yet, from the asset
/// folders. Returns the names that were not found.
pub fn load_npc_images(
    level: &Level,
    resolver: Option<&AssetResolver>,
    images: &mut NpcImages,
) -> Vec<String> {
    let mut missing = Vec::new();
    for npc in &level.npcs {
        let name = npc.image.trim();
        if name.is_empty() || images.contains_key(name) {
            continue;
        }
        let image = resolver
            .and_then(|r| r.image(name))
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|d| Image::decode(name, &d));
        if image.is_none() {
            missing.push(name.to_string());
        }
        images.insert(name.to_string(), image);
    }
    missing
}

/// A link's outline. The real clients draw nothing for a link; this only
/// shows where the level's links are.
pub const LINK_OUTLINE: [u8; 3] = [0xFF, 0xE0, 0x40];

/// An RGB tile sheet.
pub struct Sheet {
    pub w: u32,
    pub h: u32,
    pub rgb: Vec<u8>,
}

impl Sheet {
    /// `pics1.png` from the asset folders, or the built-in sheet.
    pub fn load(resolver: Option<&AssetResolver>) -> Result<Sheet, String> {
        let data = match resolver.and_then(|r| r.image("pics1.png")) {
            Some(p) => std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?,
            None => builtin::bytes("pics1.png")
                .ok_or("no built-in pics1.png")?
                .to_vec(),
        };
        let (w, h, rgb) = read_png_rgb(&data).map_err(|e| format!("pics1.png: {e}"))?;
        Ok(Sheet { w, h, rgb })
    }
}

/// The tile types (walls, water) from the asset folders, or the built-in
/// `tiledefs.txt`, which uses the same layout as the built-in sheet.
pub fn tile_types(resolver: Option<&AssetResolver>) -> Option<TileTypes> {
    match resolver.and_then(|r| r.tile_types()) {
        Some((p, TileTypesFile::Text)) => TileTypes::load(p).ok(),
        Some((p, TileTypesFile::ArraysDat)) => TileTypes::load_arrays(p).ok(),
        None => TileTypes::parse(&String::from_utf8_lossy(builtin::bytes("tiledefs.txt")?)).ok(),
    }
}

/// A frame: row-major RGBA, opaque.
pub type Frame = PixelBuffer;

/// An opaque pixel.
fn rgba([r, g, b]: [u8; 3]) -> [u8; 4] {
    [r, g, b, 255]
}

impl Sheet {
    fn image(&self) -> seihai_draw::Image<'_> {
        seihai_draw::Image::rgb(&self.rgb, self.w, self.h)
    }
}

impl Image {
    fn view(&self) -> seihai_draw::Image<'_> {
        seihai_draw::Image::rgba(&self.rgba, self.w, self.h)
    }
}

/// `f` as RGB bytes, for a texture (the frame is opaque).
pub fn rgb_of(f: &Frame) -> Vec<u8> {
    let (pixels, _) = f.rgba.as_chunks::<4>();
    pixels.iter().flat_map(|&[r, g, b, _]| [r, g, b]).collect()
}

/// Top-left of the view in level pixels: centred on the middle of the
/// player's 48x48 box (player x/y + 1.5 tiles).
pub fn camera_origin(s: &OfflineSession, view_w: u32, view_h: u32) -> (i32, i32) {
    let t = TILE_PX as f32;
    (
        ((s.player_x + 1.5) * t - view_w as f32 * t / 2.0) as i32,
        ((s.player_y + 1.5) * t - view_h as f32 * t / 2.0) as i32,
    )
}

/// The tile at `(x, y)`: layer 0, else any board run that covers it.
fn tile_at(level: &Level, x: i32, y: i32) -> Option<u16> {
    level.tile_id_at(x, y).or_else(|| {
        level
            .boards
            .iter()
            .find(|r| r.y == y && x >= r.x && x < r.x + r.width)
            .and_then(|r| r.tiles.get((x - r.x) as usize).copied())
            .map(tile_id)
    })
}

/// The players drawn over the level: the other players' figures, the pose
/// GANIs and pictures, and the clock the walking poses run on.
#[derive(Clone, Copy)]
pub struct Players<'a> {
    pub chars: &'a Characters,
    pub others: &'a [Figure],
    pub clock_ms: u64,
    /// The sheets for bombs, arrows and floor items; `None` draws none.
    pub effects: Option<&'a crate::effects::EffectSheets>,
}

impl Players<'_> {
    /// No GANIs: every player is a marker.
    pub fn markers(others: &[Figure]) -> Players<'_> {
        static NONE: std::sync::OnceLock<Characters> = std::sync::OnceLock::new();
        Players {
            chars: NONE.get_or_init(Characters::default),
            others,
            clock_ms: 0,
            effects: None,
        }
    }
}

/// The player's own figure: swinging the sword, swimming, walking or
/// standing, in the session's looks.
pub fn own_figure(s: &OfflineSession) -> Figure {
    let mut f = chars::own_figure(
        s.player_x,
        s.player_y,
        s.player_dir,
        &s.looks,
        s.in_water,
        s.walking,
        s.walk_gani.as_ref().map(|g| g.frame),
    );
    // Lifting and carrying hold the object up (the lift shows as the
    // still carry, as `lift.gani` is not among the poses drawn).
    if s.carried.is_some() && !s.in_water {
        f.pose = if s.walking { "carry" } else { "carrystill" };
        f.step = chars::Step::Clock;
    }
    // A sword swing wins over the walk (core's pose order), and the spin
    // over both.
    if let Some(step) = s.combat.swing_frame().filter(|_| !s.in_water) {
        f.pose = "sword";
        f.step = chars::Step::At(step);
    }
    if let Some(ms) = s.combat.spin_elapsed_ms() {
        f.pose = "spin";
        let tick = (ms / seihai_core::life::TICK_MS) as usize;
        f.step = chars::Step::At(
            s.poses
                .spin
                .as_ref()
                .map_or(0, |g| g.frame_at_tick(tick, false)),
        );
    }
    // Hit: the hurt pose while hurt mode lasts. Dead: the last step of
    // `dead.gani`, over everything else.
    if s.combat.is_hurt() {
        f.pose = "hurt";
        f.step = chars::Step::Clock;
    }
    if s.life.is_dead() {
        f.pose = "dead";
        f.step = chars::Step::At(usize::MAX);
    }
    f
}

/// Draw the session's level around the player, `view_w` x `view_h` tiles,
/// with the other players and the player over it. Off the level is black.
pub fn draw(
    s: &OfflineSession,
    sheet: &Sheet,
    npc_images: &NpcImages,
    players: Players<'_>,
    view_w: u32,
    view_h: u32,
) -> Frame {
    let mut f = Frame::filled(view_w * TILE_PX, view_h * TILE_PX, [0, 0, 0, 255]);
    let Some(level) = &s.current_level else {
        return f;
    };
    let (cam_x, cam_y) = camera_origin(s, view_w, view_h);
    let t = TILE_PX as i32;
    let (ox, oy) = (cam_x.div_euclid(t), cam_y.div_euclid(t));
    let (sub_x, sub_y) = (cam_x.rem_euclid(t), cam_y.rem_euclid(t));
    // One extra row and column cover the sub-tile scroll.
    for ty in 0..=view_h as i32 {
        for tx in 0..=view_w as i32 {
            let (lx, ly) = (ox + tx, oy + ty);
            // Past an edge of a gmap cell: the neighbour's tile, when that
            // level is loaded; otherwise black, as off a plain level.
            let (from, lx, ly) = if (0..64).contains(&lx) && (0..64).contains(&ly) {
                (Some(level), lx, ly)
            } else {
                let off = (lx.div_euclid(64), ly.div_euclid(64));
                let near = s
                    .gmap
                    .as_ref()
                    .and_then(|w| w.levels.get(&(w.cell.0 + off.0, w.cell.1 + off.1)));
                (near, lx.rem_euclid(64), ly.rem_euclid(64))
            };
            let Some(from) = from else {
                continue;
            };
            if let Some(id) = tile_at(from, lx, ly) {
                draw_tile(&mut f, sheet.image(), id, tx * t - sub_x, ty * t - sub_y);
            }
        }
    }
    // Chests from their records, over the board; skipped outside 0 to 62
    // as the client does.
    for c in &level.chests {
        if !(0..=62).contains(&c.x) || !(0..=62).contains(&c.y) {
            continue;
        }
        let opened = c.opened || s.opened_chests.contains(&(c.x, c.y));
        let art = if opened {
            CHEST_OPEN_TILES
        } else {
            CHEST_CLOSED_TILES
        };
        for (i, id) in art.into_iter().enumerate() {
            let (dx, dy) = ((i % 2) as i32, (i / 2) as i32);
            draw_tile(
                &mut f,
                sheet.image(),
                id,
                (c.x + dx) * t - cam_x,
                (c.y + dy) * t - cam_y,
            );
        }
    }
    // Links over the tiles and chests, under the player.
    for l in &level.links {
        f.stroke_rect(
            l.x * t - cam_x,
            l.y * t - cam_y,
            l.width * t,
            l.height * t,
            rgba(LINK_OUTLINE),
        );
    }
    // The visible NPCs (file and server), in level order, as their
    // images: the gmap neighbours' first, past the cell's edges, then the
    // player's level over them. One whose image is not on disk is outlined
    // instead; one with no image name draws nothing, as in the clients.
    let mut levels: Vec<((i32, i32), &Level)> = Vec::new();
    if let Some(w) = &s.gmap {
        let mut near: Vec<_> = w
            .levels
            .iter()
            .map(|(c, l)| ((c.0 - w.cell.0, c.1 - w.cell.1), l))
            .filter(|(off, _)| *off != (0, 0))
            .collect();
        near.sort_by_key(|(off, _)| *off);
        levels.extend(near);
    }
    levels.push(((0, 0), level));
    for (off, from) in levels {
        let (bx, by) = (off.0 * 64 * t - cam_x, off.1 * 64 * t - cam_y);
        for npc in from.npcs.iter().filter(|n| n.visible) {
            let name = npc.image.trim();
            if name.is_empty() {
                continue;
            }
            let (nx, ny) = (
                (npc.x * t as f32) as i32 + bx,
                (npc.y * t as f32) as i32 + by,
            );
            match npc_images.get(name) {
                Some(Some(img)) => {
                    let part = npc.image_part.unwrap_or((0, 0, img.w as i32, img.h as i32));
                    draw_sprite(&mut f, img.view(), part, nx, ny);
                }
                _ => f.stroke_rect(nx, ny, NPC_MISSING_PX, NPC_MISSING_PX, rgba(NPC_MISSING)),
            }
        }
    }
    // The items on the floor, under the players.
    if let Some(sheets) = players.effects {
        crate::effects::draw_floor_items(&mut f, s, sheets, (cam_x, cam_y));
    }
    // Other players, then the player over them: their GANI's frame, or a
    // marker where the pose GANI is missing.
    let figure = |f: &mut Frame, fig: &Figure, marker: [u8; 3]| {
        let (bx, by) = (
            (fig.x * t as f32) as i32 - cam_x,
            (fig.y * t as f32) as i32 - cam_y,
        );
        if players.chars.gani(fig.pose).is_some() {
            players.chars.draw(f, fig, players.clock_ms, bx, by);
        } else {
            // The marker sits where the body would be.
            f.fill_rect(bx + 16, by + 24, 16, 16, rgba(marker));
        }
    };
    for o in players.others {
        figure(&mut f, o, MARKER_OTHER);
    }
    let mut own = own_figure(s);
    // Without `spin.gani` (it is not built in) the spin shows as the swing.
    if own.pose == "spin" && players.chars.gani("spin").is_none() {
        own.pose = "sword";
        own.step = chars::Step::Clock;
    }
    figure(&mut f, &own, if s.in_water { MARKER_WATER } else { MARKER });
    // Bombs, blasts and arrows over them.
    if let Some(sheets) = players.effects {
        crate::effects::draw_effects(&mut f, s, sheets, (cam_x, cam_y));
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(fill: u16) -> OfflineSession {
        let mut s = OfflineSession::default();
        s.current_level = Some(Level::from_board(&[fill; 4096]));
        s.player_x = 30.0;
        s.player_y = 30.0;
        s
    }

    fn px(f: &Frame, x: u32, y: u32) -> [u8; 3] {
        let p = f.pixel(x, y).unwrap();
        [p[0], p[1], p[2]]
    }

    /// No GANIs and no other players: the player is a marker.
    fn none() -> Players<'static> {
        Players::markers(&[])
    }

    #[test]
    fn the_builtin_sheet_and_types_load() {
        let s = Sheet::load(None).unwrap();
        assert_eq!((s.w, s.h), (2048, 512));
        assert!(tile_types(None).is_some());
    }

    #[test]
    fn tiles_come_from_the_sheet_position_mae_uses() {
        let sheet = Sheet::load(None).unwrap();
        // Tile 600: column 600 % 16 + (600 / 512) * 16 = 24, row (600 / 16) % 32 = 5.
        let f = draw(&session(600), &sheet, &NpcImages::new(), none(), 40, 30);
        let (cam_x, cam_y) = camera_origin(&session(600), 40, 30);
        // A screen pixel at a tile corner is that sheet tile's corner pixel.
        let (sx, sy) = (
            (16 - cam_x.rem_euclid(16)) as u32,
            (16 - cam_y.rem_euclid(16)) as u32,
        );
        let si = ((5 * 16 * sheet.w + 24 * 16) * 3) as usize;
        assert_eq!(
            px(&f, sx, sy),
            [sheet.rgb[si], sheet.rgb[si + 1], sheet.rgb[si + 2]]
        );
    }

    #[test]
    fn the_player_marker_is_at_the_centre() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(&session(0), &sheet, &NpcImages::new(), none(), 40, 30);
        // The box (48x48) is centred; the marker is its middle-bottom 16x16.
        assert_eq!(px(&f, 320, 248), MARKER);
    }

    #[test]
    fn a_link_is_outlined_on_its_rectangle() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(0);
        let level = s.current_level.as_mut().unwrap();
        level.add_server_links(&["next.nw 20 25 2 3 30 30".to_string()]);
        let f = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        let (x0, y0) = ((20 * 16 - cam_x) as u32, (25 * 16 - cam_y) as u32);
        // Its corners and edges, 2 x 3 tiles; not its inside.
        assert_eq!(px(&f, x0, y0), LINK_OUTLINE);
        assert_eq!(px(&f, x0 + 31, y0 + 47), LINK_OUTLINE);
        assert_eq!(px(&f, x0 + 16, y0), LINK_OUTLINE);
        assert_ne!(px(&f, x0 + 16, y0 + 24), LINK_OUTLINE);
    }

    #[test]
    fn another_player_gets_a_marker() {
        let sheet = Sheet::load(None).unwrap();
        let s = session(0);
        let (ox, oy) = (s.player_x + 3.0, s.player_y + 2.0);
        let others = [chars::other_figure(&seihai_core::client::Other {
            x: ox,
            y: oy,
            ..Default::default()
        })];
        let f = draw(
            &s,
            &sheet,
            &NpcImages::new(),
            Players::markers(&others),
            40,
            30,
        );
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        let x = (ox * 16.0) as i32 - cam_x + 20;
        let y = (oy * 16.0) as i32 - cam_y + 28;
        assert_eq!(px(&f, x as u32, y as u32), MARKER_OTHER);
    }

    #[test]
    fn off_the_level_is_black() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(600);
        s.player_x = 0.0;
        s.player_y = 0.0;
        let f = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        assert_eq!(px(&f, 0, 0), [0, 0, 0]);
    }

    #[test]
    fn no_level_draws_black() {
        let sheet = Sheet::load(None).unwrap();
        let f = draw(
            &OfflineSession::default(),
            &sheet,
            &NpcImages::new(),
            none(),
            4,
            4,
        );
        assert!(rgb_of(&f).iter().all(|&b| b == 0));
    }

    /// With the built-in GANIs, the player and another player are drawn
    /// from their idle frames where the markers were, over the tiles.
    #[test]
    fn players_are_drawn_from_their_ganis() {
        let sheet = Sheet::load(None).unwrap();
        let s = session(0);
        let plain = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        let mut chars = Characters::load(None);
        let other = chars::other_figure(&seihai_core::client::Other {
            x: s.player_x + 4.0,
            y: s.player_y,
            dir: 2,
            ..Default::default()
        });
        let own = own_figure(&s);
        assert!(chars.load_pictures(None, &[own, other.clone()]));
        let others = [other];
        let players = Players {
            chars: &chars,
            others: &others,
            clock_ms: 0,
            effects: None,
        };
        let f = draw(&s, &sheet, &NpcImages::new(), players, 40, 30);
        // The player's box is 320-24 .. 320+24 across; the other's 64 px
        // to the right. Both differ from the bare board, and no marker
        // colour is left.
        let differs = |x0: u32| {
            (0..48)
                .flat_map(|dx| (0..48).map(move |dy| (x0 + dx, 216 + dy)))
                .filter(|&(x, y)| px(&f, x, y) != px(&plain, x, y))
                .count()
        };
        assert!(differs(296) > 100, "the player: {}", differs(296));
        assert!(differs(360) > 100, "the other player: {}", differs(360));
        assert_ne!(px(&f, 320, 248), MARKER);
    }

    /// A 4x4 image: opaque red on its left half, clear on its right.
    fn half_red() -> Image {
        let mut rgba = Vec::new();
        for _ in 0..4 {
            for x in 0..4 {
                rgba.extend_from_slice(if x < 2 {
                    &[255, 0, 0, 255]
                } else {
                    &[0, 0, 255, 0]
                });
            }
        }
        Image { w: 4, h: 4, rgba }
    }

    fn with_npc(image: &str, dx: f32, dy: f32) -> (OfflineSession, i32, i32) {
        let mut s = session(0);
        let (x, y) = (s.player_x + dx, s.player_y + dy);
        s.current_level
            .as_mut()
            .unwrap()
            .npcs
            .push(seihai_core::assets::Npc::new(
                image.into(),
                x,
                y,
                String::new(),
            ));
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        (s, (x * 16.0) as i32 - cam_x, (y * 16.0) as i32 - cam_y)
    }

    #[test]
    fn an_npc_is_drawn_as_its_image_with_the_clear_pixels_left_out() {
        let sheet = Sheet::load(None).unwrap();
        let (s, x, y) = with_npc("block.png", 5.0, -4.0);
        let images = NpcImages::from([("block.png".to_string(), Some(half_red()))]);
        let under = px(
            &draw(&session(0), &sheet, &NpcImages::new(), none(), 40, 30),
            x as u32 + 3,
            y as u32,
        );
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), [255, 0, 0]);
        assert_eq!(px(&f, x as u32 + 1, y as u32 + 3), [255, 0, 0]);
        // The clear half shows the tiles under it.
        assert_eq!(px(&f, x as u32 + 3, y as u32), under);
    }

    #[test]
    fn an_image_part_draws_only_that_part() {
        let sheet = Sheet::load(None).unwrap();
        let (mut s, x, y) = with_npc("block.png", 5.0, -4.0);
        // Only the clear half: nothing red is drawn.
        s.current_level.as_mut().unwrap().npcs[0].image_part = Some((2, 0, 2, 4));
        let images = NpcImages::from([("block.png".to_string(), Some(half_red()))]);
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_ne!(px(&f, x as u32, y as u32), [255, 0, 0]);
    }

    #[test]
    fn a_missing_image_is_outlined_and_a_hidden_npc_is_not_drawn() {
        let sheet = Sheet::load(None).unwrap();
        let (mut s, x, y) = with_npc("gone.png", 5.0, -4.0);
        let images = NpcImages::from([("gone.png".to_string(), None)]);
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), NPC_MISSING);
        assert_eq!(px(&f, x as u32 + 31, y as u32 + 31), NPC_MISSING);
        s.current_level.as_mut().unwrap().npcs[0].visible = false;
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_ne!(px(&f, x as u32, y as u32), NPC_MISSING);
    }

    #[test]
    fn an_npc_with_no_image_draws_nothing() {
        let sheet = Sheet::load(None).unwrap();
        let (s, x, y) = with_npc("", 5.0, -4.0);
        let plain = draw(&session(0), &sheet, &NpcImages::new(), none(), 40, 30);
        let f = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        assert_eq!(px(&f, x as u32, y as u32), px(&plain, x as u32, y as u32));
    }

    #[test]
    fn png_and_gif_names_decode_and_others_do_not() {
        let mut png = Vec::new();
        seihai_core::assets::write_png_rgb(&mut png, 2, 1, &[1, 2, 3, 4, 5, 6]).unwrap();
        let img = Image::decode("Thing.PNG", &png).unwrap();
        assert_eq!((img.w, img.h), (2, 1));
        assert_eq!(&img.rgba[..4], &[1, 2, 3, 255]);
        assert!(Image::decode("thing.mng", &png).is_none());
        assert!(Image::decode("thing.gif", &png).is_none(), "not a GIF");
    }

    #[test]
    fn images_are_looked_for_once_and_missing_ones_reported() {
        let (s, _, _) = with_npc("nowhere.png", 1.0, 1.0);
        let level = s.current_level.as_ref().unwrap();
        let mut images = NpcImages::new();
        assert_eq!(load_npc_images(level, None, &mut images), ["nowhere.png"]);
        assert_eq!(images.get("nowhere.png"), Some(&None));
        assert!(load_npc_images(level, None, &mut images).is_empty());
    }

    #[test]
    fn a_gmap_neighbour_is_drawn_past_the_edge() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(0);
        s.player_x = 60.0;
        let grid = "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n";
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        // The middle of tile (64, 30): one tile past the east edge.
        let (ex, ey) = ((64 * 16 - cam_x + 8) as u32, (30 * 16 - cam_y + 8) as u32);
        let f = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        assert_eq!(px(&f, ex, ey), [0, 0, 0], "no gmap: black past the edge");
        s.enter_gmap(
            "w.gmap".into(),
            seihai_core::gmap::Gmap::parse(grid).unwrap(),
            (0, 0),
        );
        s.gmap
            .as_mut()
            .unwrap()
            .levels
            .insert((1, 0), Level::from_board(&[600; 4096]));
        let f = draw(&s, &sheet, &NpcImages::new(), none(), 40, 30);
        let want = draw(&session(600), &sheet, &NpcImages::new(), none(), 40, 30);
        // Tile 600's middle pixel, wherever it is drawn.
        let (cx, cy) = camera_origin(&session(600), 40, 30);
        let (wx, wy) = ((30 * 16 - cx + 8) as u32, (30 * 16 - cy + 8) as u32);
        assert_eq!(px(&f, ex, ey), px(&want, wx, wy));
    }

    /// A gmap neighbour's NPCs are drawn past the cell's edge, where its
    /// tiles are.
    #[test]
    fn a_gmap_neighbours_npc_is_drawn_past_the_edge() {
        let sheet = Sheet::load(None).unwrap();
        let mut s = session(0);
        s.player_x = 60.0;
        let grid = "GRMAP001\nWIDTH 2\nHEIGHT 1\nLEVELNAMES\n\"a.nw\",\"b.nw\"\nLEVELNAMESEND\n";
        s.enter_gmap(
            "w.gmap".into(),
            seihai_core::gmap::Gmap::parse(grid).unwrap(),
            (0, 0),
        );
        let mut east = Level::from_board(&[0; 4096]);
        east.npcs.push(seihai_core::assets::Npc::new(
            "gone.png".into(),
            2.0,
            30.0,
            String::new(),
        ));
        s.gmap.as_mut().unwrap().levels.insert((1, 0), east);
        let images = NpcImages::from([("gone.png".to_string(), None)]);
        let (cam_x, cam_y) = camera_origin(&s, 40, 30);
        // Tile (2, 30) of the east cell is tile (66, 30) from this one.
        let (nx, ny) = ((66 * 16 - cam_x) as u32, (30 * 16 - cam_y) as u32);
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_eq!(px(&f, nx, ny), NPC_MISSING);
        s.gmap.as_mut().unwrap().levels.clear();
        let f = draw(&s, &sheet, &images, none(), 40, 30);
        assert_ne!(px(&f, nx, ny), NPC_MISSING);
    }
}
