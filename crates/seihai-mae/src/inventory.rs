//! The item list (Q): the window that lists the player's weapons and picks
//! the one the weapon key fires.
//!
//! Read from the 2.05 client (derived, see `documentation/inventory.md`).
//! The window is a grid of cells, one per weapon, packed left to right and
//! wrapped. The arrow keys move the selection to the nearest cell in that
//! direction, and moving it selects that weapon at once (nothing is sent).
//! Esc, Q, and the item, attack and action keys close it. While it is open
//! the player does not walk, swing or interact.
//!
//! Seihai adds a line of counts (rupees, bombs, arrows, hearts, sword,
//! shield, glove) above the grid; the real window has no such line.

use crate::render::font::{text_width, CELL_H};
use crate::render::Framebuffer;

/// A key the open list reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKey {
    Up,
    Left,
    Down,
    Right,
    /// Q again, Esc, Enter, or the item, attack or action key.
    Close,
}

/// A cell of the grid, in pixels relative to the window's content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Cell {
    fn right(&self) -> i32 {
        self.x + self.w
    }
    fn bottom(&self) -> i32 {
        self.y + self.h
    }
}

/// Image size the 2.05 client gives a cell with no image (a folder is
/// 32 x 32 too).
pub const ICON: i32 = 32;
/// The padding the 2.05 client puts around an entry.
const PAD: i32 = 5;

/// The list's state: open or not, and the selected entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemList {
    pub open: bool,
    pub selected: Option<usize>,
}

impl ItemList {
    /// Q: open the list, or close it if open.
    pub fn toggle(&mut self, count: usize) {
        self.open = !self.open;
        if self.open {
            self.clamp(count);
        }
    }

    /// Keep the selection inside `count` entries (the first one when there
    /// was none).
    pub fn clamp(&mut self, count: usize) {
        self.selected = match (self.selected, count) {
            (_, 0) => None,
            (None, _) => Some(0),
            (Some(i), n) => Some(i.min(n - 1)),
        };
    }

    /// A key while the list is open. Arrows move to the nearest cell in that
    /// direction (no cell there: the selection stays); anything else
    /// closes the list.
    pub fn key(&mut self, k: ListKey, cells: &[Cell]) {
        if !self.open {
            return;
        }
        let Some(cur) = self.selected.filter(|&i| i < cells.len()) else {
            if k == ListKey::Close {
                self.open = false;
            }
            return;
        };
        if k == ListKey::Close {
            self.open = false;
            return;
        }
        if let Some(n) = nearest(cells, cur, k) {
            self.selected = Some(n);
        }
    }
}

/// The 2.05 rule for an arrow key: stretch the current cell to the edge of
/// the window in that direction, and of the cells that overlap the stretched
/// box take the one whose near edge is closest.
pub fn nearest(cells: &[Cell], cur: usize, k: ListKey) -> Option<usize> {
    let c = cells[cur];
    let (mut x0, mut y0, mut x1, mut y1) = (c.x, c.y, c.right(), c.bottom());
    match k {
        ListKey::Up => y0 = i32::MIN / 2,
        ListKey::Left => x0 = i32::MIN / 2,
        ListKey::Down => y1 = i32::MAX / 2,
        ListKey::Right => x1 = i32::MAX / 2,
        ListKey::Close => return None,
    }
    let mut best: Option<(i32, usize)> = None;
    for (i, o) in cells.iter().enumerate() {
        if i == cur || !(o.x < x1 && x0 < o.right() && o.y < y1 && y0 < o.bottom()) {
            continue;
        }
        let d = match k {
            ListKey::Up => c.y - o.bottom(),
            ListKey::Left => c.x - o.right(),
            ListKey::Down => o.y - c.bottom(),
            _ => o.x - c.right(),
        };
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i));
        }
    }
    best.map(|(_, i)| i)
}

/// An entry's image: its width and its RGBA pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Image<'a> {
    pub w: u32,
    pub px: &'a [u8],
}

impl Image<'_> {
    fn size(&self) -> (i32, i32) {
        let w = self.w as i32;
        (w, if w > 0 { (self.px.len() / 4 / self.w as usize) as i32 } else { 0 })
    }
}

/// The size an entry's image takes in its cell: the image's own size, or
/// the 32 x 32 placeholder when it has none (or it is not here yet).
fn image_size(images: &[Option<Image>], i: usize) -> (i32, i32) {
    images.get(i).copied().flatten().map_or((ICON, ICON), |im| im.size())
}

/// Pack the entries into cells inside a content area `width` pixels wide:
/// each cell is as wide as the larger of its image and its label, plus
/// padding, and as tall as both stacked, plus padding. Cells go left to
/// right and wrap to a new row. `images` goes with `labels` by index; an
/// entry with none takes the 32 x 32 placeholder's size.
pub fn layout(labels: &[String], images: &[Option<Image>], width: i32) -> Vec<Cell> {
    let mut out = Vec::with_capacity(labels.len());
    let (mut x, mut y, mut row_h) = (0, 0, 0);
    for (i, l) in labels.iter().enumerate() {
        let (iw, ih) = image_size(images, i);
        let w = iw.max(text_width(l)) + PAD * 2;
        let h = ih + CELL_H + PAD * 3;
        if x > 0 && x + w > width {
            x = 0;
            y += row_h;
            row_h = 0;
        }
        out.push(Cell { x, y, w, h });
        x += w;
        row_h = row_h.max(h);
    }
    out
}

/// Item names of the built-in weapons by `PLO_DEFAULTWEAPON` id (the item
/// table's names). Other ids are not weapons.
pub fn default_weapon_name(id: u8) -> Option<&'static str> {
    Some(match id {
        7 => "bow",
        8 => "bomb",
        12 => "superbomb",
        20 => "fireball",
        21 => "fireblast",
        22 => "nukeshot",
        23 => "joltbomb",
        _ => return None,
    })
}

/// The image of a built-in weapon by its item name (`bow` gives
/// `wbowi1` plus the build's image extension).
pub fn default_weapon_image(name: &str, image_ext: &str) -> Option<String> {
    (0..=24u8)
        .filter_map(crate::weaponbar::default_weapon)
        .find(|d| d.name.eq_ignore_ascii_case(name))
        .map(|d| format!("{}{image_ext}", d.image))
}

/// Built-in weapons the offline session got from chests (`item:bow` and
/// so on in its flags), in item-table order.
pub fn chest_weapons(flags: &std::collections::HashMap<String, bool>) -> Vec<String> {
    (0..=24u8)
        .filter_map(default_weapon_name)
        .filter(|n| flags.get(&format!("item:{n}")).copied().unwrap_or(false))
        .map(str::to_string)
        .collect()
}

/// The list's entries: the built-in weapons, then the named (NPC) weapons.
/// Names starting with `-` are hidden weapons, which the 2.05 client never
/// selects; they are left out here. A name sent twice is listed once.
pub fn entries(default_ids: &[u8], named: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let names = default_ids.iter().filter_map(|&i| default_weapon_name(i)).map(str::to_string).chain(named.iter().cloned());
    for n in names {
        if !n.is_empty() && !n.starts_with('-') && !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

/// The counts line Seihai shows above the grid.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    pub rupees: i32,
    pub bombs: i32,
    pub arrows: i32,
    /// Half hearts now and the maximum.
    pub hearts_half: i32,
    pub max_hearts_half: i32,
    pub sword: u8,
    pub shield: u8,
    pub glove: u8,
}

impl Counts {
    pub fn line(&self) -> String {
        let h = |v: i32| if v % 2 == 0 { format!("{}", v / 2) } else { format!("{}.5", v / 2) };
        format!(
            "Rupees {}  Bombs {}  Arrows {}  Hearts {}/{}  Sword {}  Shield {}  Glove {}",
            self.rupees,
            self.bombs,
            self.arrows,
            h(self.hearts_half),
            h(self.max_hearts_half),
            self.sword,
            self.shield,
            self.glove
        )
    }
}

/// Where the window sits: from (16, 80) to 16 pixels in from the right and
/// bottom edges, as in the 2.05 client. Returns `(x, y, w, h)`.
pub fn window_rect(fb_w: i32, fb_h: i32) -> (i32, i32, i32, i32) {
    let (x, y) = (16, 80);
    (x, y, (fb_w - 16 - x).max(64), (fb_h - 16 - y).max(64))
}

/// The content area inside the window: 32 pixels in on the left, 32 down,
/// 32 in on the right and 48 up from the bottom, as in the 2.05 client.
pub fn content_rect(fb_w: i32, fb_h: i32) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = window_rect(fb_w, fb_h);
    (x + 32, y + 32, (w - 64).max(32), (h - 80).max(32))
}

/// Draw the open list. `images` goes with `labels` by index; an entry with
/// no image (or one not here yet) gets an outlined 32 x 32 placeholder.
pub fn draw(fb: &mut Framebuffer, list: &ItemList, labels: &[String], images: &[Option<Image>], counts: &Counts) {
    let (fw, fh) = (fb.width as i32, fb.height as i32);
    let (wx, wy, ww, wh) = window_rect(fw, fh);
    fb.shade_rect(wx, wy, ww, wh, 60);
    for (x, y, w, h) in [(wx, wy, ww, 1), (wx, wy + wh - 1, ww, 1), (wx, wy, 1, wh), (wx + ww - 1, wy, 1, wh)] {
        fb.fill_rect(x, y, w, h, [0xC0, 0xC0, 0xC0]);
    }
    fb.draw_text(wx + 8, wy + 6, "Items", [0xFF, 0xE0, 0x70]);
    fb.draw_text(wx + 8, wy + 8 + CELL_H, &counts.line(), [0xFF, 0xFF, 0xFF]);
    let (cx, cy, cw, ch) = content_rect(fw, fh);
    if labels.is_empty() {
        fb.draw_text(cx, cy, "No weapons", [0xA0, 0xA0, 0xA0]);
        return;
    }
    let cells = layout(labels, images, cw);
    // Scroll so the selected cell is inside the content area.
    let sel = list.selected.and_then(|i| cells.get(i));
    let scroll = sel.map_or(0, |c| (c.bottom() - ch).max(0));
    for (i, (c, l)) in cells.iter().zip(labels).enumerate() {
        let (x, y) = (cx + c.x, cy + c.y - scroll);
        if y < cy || y + c.h > cy + ch {
            continue;
        }
        let selected = list.selected == Some(i);
        if selected {
            fb.fill_rect(x, y, c.w, c.h, [0xFF, 0xD4, 0x80]);
        }
        // The image, centred with 5 px above it; the placeholder outline
        // when there is none.
        let (iw, ih) = image_size(images, i);
        let ix = x + (c.w - iw) / 2;
        let iy = y + PAD;
        match images.get(i).copied().flatten() {
            Some(im) => fb.blit_sprite(im.px, im.w, 4, 0, 0, iw, ih, ix, iy),
            None => {
                let edge = if selected { [0x60, 0x40, 0x10] } else { [0x80, 0x80, 0x80] };
                for (x, y, w, h) in [(ix, iy, ICON, 1), (ix, iy + ICON - 1, ICON, 1), (ix, iy, 1, ICON), (ix + ICON - 1, iy, 1, ICON)] {
                    fb.fill_rect(x, y, w, h, edge);
                }
            }
        }
        let tx = x + (c.w - text_width(l)) / 2;
        let colour = if selected { [0x20, 0x20, 0x20] } else { [0xFF, 0xFF, 0xFF] };
        fb.draw_text(tx, iy + ih + PAD, l, colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(x: i32, y: i32) -> Cell {
        Cell { x, y, w: 40, h: 50 }
    }

    #[test]
    fn q_opens_and_closes_and_the_first_entry_is_selected() {
        let mut l = ItemList::default();
        l.toggle(3);
        assert!(l.open);
        assert_eq!(l.selected, Some(0));
        l.toggle(3);
        assert!(!l.open);
        // Empty list: opens with nothing selected and does not crash.
        let mut e = ItemList::default();
        e.toggle(0);
        assert!(e.open);
        assert_eq!(e.selected, None);
        e.key(ListKey::Right, &[]);
        e.key(ListKey::Close, &[]);
        assert!(!e.open);
    }

    #[test]
    fn arrows_take_the_nearest_cell_that_way_and_stop_at_the_edge() {
        // Two rows of three.
        let cells: Vec<Cell> = (0..6).map(|i| cell((i % 3) * 40, (i / 3) * 50)).collect();
        let mut l = ItemList { open: true, selected: Some(0) };
        l.key(ListKey::Right, &cells);
        assert_eq!(l.selected, Some(1));
        l.key(ListKey::Down, &cells);
        assert_eq!(l.selected, Some(4));
        l.key(ListKey::Down, &cells);
        assert_eq!(l.selected, Some(4), "nothing below");
        l.key(ListKey::Left, &cells);
        l.key(ListKey::Left, &cells);
        assert_eq!(l.selected, Some(3));
        l.key(ListKey::Up, &cells);
        assert_eq!(l.selected, Some(0));
        l.key(ListKey::Close, &cells);
        assert!(!l.open);
        assert_eq!(l.selected, Some(0), "closing keeps the selected weapon");
    }

    #[test]
    fn cells_wrap_inside_the_width() {
        let labels: Vec<String> = ["bow", "bomb", "fireball", "a"].iter().map(|s| s.to_string()).collect();
        let cells = layout(&labels, &[], 100);
        assert_eq!((cells[0].x, cells[0].y), (0, 0));
        assert_eq!(cells[1].x, cells[0].w);
        assert!(cells.iter().all(|c| c.x == 0 || c.right() <= 100));
        assert!(cells.iter().any(|c| c.y > 0), "a second row");
        assert!(cells.iter().all(|c| c.w >= ICON + 10 && c.h == ICON + CELL_H + 15));
    }

    #[test]
    fn a_cell_takes_its_image_size_and_the_placeholder_without_one() {
        let labels: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let px = vec![0u8; 48 * 20 * 4];
        let cells = layout(&labels, &[Some(Image { w: 48, px: &px }), None], 1000);
        assert_eq!((cells[0].w, cells[0].h), (48 + 10, 20 + CELL_H + 15));
        assert_eq!((cells[1].w, cells[1].h), (ICON + 10, ICON + CELL_H + 15));
    }

    #[test]
    fn the_image_is_drawn_in_its_cell() {
        let mut fb = Framebuffer::new(640, 480);
        let labels = vec!["bow".to_string()];
        let px: Vec<u8> = [0x11, 0x22, 0x33, 0xFF].repeat(32 * 32);
        draw(&mut fb, &ItemList { open: true, selected: None }, &labels, &[Some(Image { w: 32, px: &px })], &Counts::default());
        let (cx, cy, _, _) = content_rect(640, 480);
        let c = layout(&labels, &[Some(Image { w: 32, px: &px })], 1000)[0];
        let (x, y) = (cx + (c.w - 32) / 2 + 16, cy + PAD + 16);
        let i = (y as usize * 640 + x as usize) * 3;
        assert_eq!(fb.pixels[i..i + 3], [0x11, 0x22, 0x33]);
    }

    #[test]
    fn built_in_weapon_images_by_item_name() {
        assert_eq!(default_weapon_image("bow", ".gif").as_deref(), Some("wbowi1.gif"));
        assert_eq!(default_weapon_image("Bomb", ".png").as_deref(), Some("wbomb1.png"));
        assert_eq!(default_weapon_image("Sword Spin", ".png"), None);
    }

    #[test]
    fn entries_list_default_weapons_then_named_ones_without_hidden_or_repeats() {
        let named = vec!["Sword Spin".to_string(), "-system".to_string(), "bow".to_string()];
        assert_eq!(entries(&[7, 8, 3], &named), vec!["bow", "bomb", "Sword Spin"]);
        assert!(entries(&[], &[]).is_empty());
    }

    #[test]
    fn chest_weapons_come_from_the_offline_item_flags() {
        let mut f = std::collections::HashMap::new();
        f.insert("item:bomb".to_string(), true);
        f.insert("item:bow".to_string(), true);
        f.insert("item:redrupee".to_string(), true);
        assert_eq!(chest_weapons(&f), vec!["bow", "bomb"]);
    }

    #[test]
    fn counts_line_shows_half_hearts() {
        let c = Counts { rupees: 12, hearts_half: 5, max_hearts_half: 6, sword: 1, ..Counts::default() };
        assert_eq!(c.line(), "Rupees 12  Bombs 0  Arrows 0  Hearts 2.5/3  Sword 1  Shield 0  Glove 0");
    }

    #[test]
    fn drawing_does_not_crash_on_small_or_empty_frames() {
        for (w, h) in [(640, 480), (160, 120), (32, 32)] {
            let mut fb = Framebuffer::new(w, h);
            let labels = entries(&[7, 8], &["Weapon".into()]);
            let px = vec![0xFFu8; 48 * 16 * 4];
            let images = [Some(Image { w: 48, px: &px }), None];
            draw(&mut fb, &ItemList { open: true, selected: Some(2) }, &labels, &images, &Counts::default());
            draw(&mut fb, &ItemList { open: true, selected: None }, &[], &[], &Counts::default());
        }
    }
}
