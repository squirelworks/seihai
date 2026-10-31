//! The M map: the big map picture with the players' heads on it.
//!
//! Read from the 2.05 rev 2 client (Graal_analysis `graal-decompile/v205-delphi`,
//! the routines named in `documentation/map-screen.md`):
//!
//! * The map key (action 7, M) opens the map when it is closed and closes it
//!   when it is open. Opening centres the view on the player's spot on the
//!   map; a player whose level is not on the map gets the middle of the map.
//! * The view is scrolled 8 pixels a key press (with key repeat) by the four
//!   movement keys, and kept inside the picture: `0 ..= picture - view` on
//!   each axis, and when the picture is smaller than the view it is centred
//!   (the 1.38 notes: "the M map is displayed in the vertical middle of the
//!   screen when it is smaller than the screen").
//! * The picture is drawn at the view's top-left minus the scroll, over
//!   black. Every player whose level is on the map gets a 16-pixel head at
//!   `cell * cell_size + position * cell_size / 64`, centred there, and the
//!   own player's nickname under it.
//! * "Map" is written at (20, 20) with a blue shadow one pixel down-right,
//!   and online "Others: n" at (485, 25) of the 640-pixel view.
//! * While the map is open the player does not walk, swing or act. Q swaps
//!   the map for the item list and P closes it.
//!
//! The cell size is the picture's size over the grid's size. In 2.05 it is
//! 56 pixels (`map.png` is 1120×1176 for the 20×21 `bigmap.txt`, the
//! tutorial's 168×224 for 3×4, and the editor's "Generate map" draws 56
//! pixels a level); the 2.22 folder's `bigmap.png` (1024×371) goes with the
//! 58×22 `bigmap2.txt`, about 17.6 pixels a level, which only fits when the
//! client scales to the picture.

use crate::gmap::Gmap;
use crate::render::font::text_width;
use crate::render::Framebuffer;

/// Pixels the view moves for one press of a movement key (2.05).
pub const SCROLL_STEP: i32 = 8;

/// The head drawn for each player, in pixels (2.05: 16).
pub const HEAD_PX: i32 = 16;

/// What the map shows: a level grid and its picture's file name.
#[derive(Debug, Clone, PartialEq)]
pub struct MapSource {
    /// The level-names text (`bigmap.txt`), or a gmap's name.
    pub txt: String,
    /// The picture (`map.png`).
    pub image: String,
    /// Where a player whose level is not on the map is drawn, in levels
    /// (`x,y` of `setmap` and of the server's `bigmap=`), when given.
    pub off_map: Option<(f32, f32)>,
}

impl MapSource {
    pub fn new(txt: &str, image: &str) -> Self {
        Self { txt: txt.trim().to_string(), image: image.trim().to_string(), off_map: None }
    }

    pub fn with_off_map(mut self, x: f32, y: f32) -> Self {
        self.off_map = Some((x, y));
        self
    }

    /// The map every 1.x and 2.x build starts with: `bigmap.txt` and
    /// `map.png` (2.05's defaults; the server or a `setmap` replaces them).
    pub fn default_map() -> Self {
        Self::new("bigmap.txt", "map.png")
    }
}

/// An RGBA picture.
#[derive(Debug, Clone)]
pub struct Picture {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

/// One player on the map.
#[derive(Debug, Clone)]
pub struct Marker<'a> {
    /// Point on the picture, in pixels.
    pub at: (i32, i32),
    /// The head image (RGBA, its width) when it is here.
    pub head: Option<(u32, &'a [u8])>,
    /// The name written under the head, if any.
    pub name: Option<String>,
}

/// The map's state: open or not, and the scroll.
#[derive(Debug, Clone, Default)]
pub struct MapScreen {
    pub open: bool,
    /// The picture pixel at the view's top-left. Negative when the picture
    /// is smaller than the view (it is centred).
    pub scroll: (i32, i32),
}

/// Pixels a level takes on the picture, across and down.
pub fn cell_size(grid: &Gmap, pic_w: u32, pic_h: u32) -> (f32, f32) {
    (pic_w as f32 / grid.width.max(1) as f32, pic_h as f32 / grid.height.max(1) as f32)
}

/// Where a player at `(x, y)` (tiles) in `level` is on the picture, if the
/// level is on the grid.
pub fn spot(grid: &Gmap, cell_px: (f32, f32), level: &str, x: f32, y: f32) -> Option<(i32, i32)> {
    let (cx, cy) = grid.cell_of(level.trim())?;
    Some(spot_in_cell(cell_px, (cx, cy), x, y))
}

/// The point for a position `(x, y)` (tiles) inside cell `(cx, cy)`.
pub fn spot_in_cell((cw, ch): (f32, f32), (cx, cy): (i32, i32), x: f32, y: f32) -> (i32, i32) {
    let px = cx as f32 * cw + x * cw / 64.0;
    let py = cy as f32 * ch + y * ch / 64.0;
    (px.round() as i32, py.round() as i32)
}

/// Keep the scroll inside the picture (2.05): `0 ..= pic - view` on each
/// axis; centred when the picture is the smaller.
pub fn clamp_scroll((sx, sy): (i32, i32), (pw, ph): (i32, i32), (vw, vh): (i32, i32)) -> (i32, i32) {
    let axis = |s: i32, p: i32, v: i32| if p < v { (p - v) / 2 } else { s.clamp(0, p - v) };
    (axis(sx, pw, vw), axis(sy, ph, vh))
}

impl MapScreen {
    /// Open the map: centred on `me` (a point on the picture), or on the
    /// middle of the map when the player's level is not on it.
    pub fn open_at(&mut self, me: Option<(i32, i32)>, pic: (i32, i32), view: (i32, i32)) {
        self.open = true;
        self.scroll = match me {
            Some((x, y)) => clamp_scroll((x - view.0 / 2, y - view.1 / 2), pic, view),
            None => ((pic.0 - view.0) / 2, (pic.1 - view.1) / 2),
        };
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Move the view by one key press, `(dx, dy)` in steps (-1, 0 or 1).
    pub fn scroll_by(&mut self, (dx, dy): (i32, i32), pic: (i32, i32), view: (i32, i32)) {
        let s = (self.scroll.0 + dx * SCROLL_STEP, self.scroll.1 + dy * SCROLL_STEP);
        self.scroll = clamp_scroll(s, pic, view);
    }
}

/// The map's files once read: the grid and the picture of one
/// [`MapSource`].
#[derive(Debug, Clone, Default)]
pub struct MapFiles {
    pub source: Option<MapSource>,
    pub grid: Option<Gmap>,
    pub picture: Option<Picture>,
}

impl MapFiles {
    /// Make these the files of `want`. `grid` is a grid the caller already
    /// has (a gmap's, or the `--bigmap` file); without one the text is read
    /// with `read`. A new source drops the old files. Returns the names
    /// still missing, for the caller to ask the server for.
    pub fn sync(&mut self, want: &MapSource, grid: Option<&Gmap>, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> Vec<String> {
        if self.source.as_ref() != Some(want) {
            *self = MapFiles { source: Some(want.clone()), ..Default::default() };
        }
        if let Some(g) = grid {
            if self.grid.as_ref() != Some(g) {
                self.grid = Some(g.clone());
            }
        }
        let mut missing = Vec::new();
        if self.grid.is_none() {
            match read(&want.txt) {
                Some(d) => self.grid = Gmap::parse_grid(&String::from_utf8_lossy(&d)),
                None => missing.push(want.txt.clone()),
            }
        }
        if self.picture.is_none() && !want.image.is_empty() {
            match read(&want.image).and_then(|d| crate::assets::read_png_rgba(&d).ok()) {
                Some((w, h, px)) => self.picture = Some(Picture { w, h, px }),
                None => missing.push(want.image.clone()),
            }
        }
        missing
    }

    /// The picture's size, or the grid at 56 pixels a level while the
    /// picture is not here (2.05's size), or nothing.
    pub fn size(&self) -> (i32, i32) {
        match (&self.picture, &self.grid) {
            (Some(p), _) => (p.w as i32, p.h as i32),
            (None, Some(g)) => (g.width as i32 * 56, g.height as i32 * 56),
            _ => (0, 0),
        }
    }

    /// Pixels a level takes.
    pub fn cell_px(&self) -> Option<(f32, f32)> {
        let g = self.grid.as_ref()?;
        let (w, h) = self.size();
        Some(cell_size(g, w as u32, h as u32))
    }

    /// Where a player in `level` at `(x, y)` is on the picture. A level not
    /// on the map puts the head at the source's off-map point, when it has
    /// one.
    pub fn spot(&self, level: &str, x: f32, y: f32) -> Option<(i32, i32)> {
        let (grid, cell_px) = (self.grid.as_ref()?, self.cell_px()?);
        spot(grid, cell_px, level, x, y).or_else(|| {
            let (ox, oy) = self.source.as_ref()?.off_map?;
            Some(((ox * cell_px.0).round() as i32, (oy * cell_px.1).round() as i32))
        })
    }
}

/// Draw the open map over the whole of `fb`. `others` is `Some(count)`
/// online (the "Others:" line); `pic` is `None` while the picture has not
/// arrived (the view is black, with the title).
pub fn draw(fb: &mut Framebuffer, screen: &MapScreen, pic: Option<&Picture>, markers: &[Marker], others: Option<usize>) {
    let (vw, vh) = (fb.width as i32, fb.height as i32);
    fb.fill_rect(0, 0, vw, vh, [0, 0, 0]);
    let (sx, sy) = screen.scroll;
    if let Some(p) = pic {
        // The part of the picture inside the view.
        let (x0, y0) = (sx.max(0), sy.max(0));
        let (x1, y1) = ((sx + vw).min(p.w as i32), (sy + vh).min(p.h as i32));
        if x1 > x0 && y1 > y0 {
            fb.blit_sprite(&p.px, p.w, 4, x0, y0, x1 - x0, y1 - y0, x0 - sx, y0 - sy);
        }
    }
    for m in markers {
        let (cx, cy) = (m.at.0 - sx, m.at.1 - sy);
        let (hx, hy) = (cx - HEAD_PX / 2, cy - HEAD_PX / 2);
        match m.head {
            Some((w, px)) => draw_head(fb, px, w, hx, hy),
            // Seihai's stand-in until the head image is here.
            None => {
                fb.fill_rect(hx + 4, hy + 4, HEAD_PX - 8, HEAD_PX - 8, [0, 0, 0]);
                fb.fill_rect(hx + 5, hy + 5, HEAD_PX - 10, HEAD_PX - 10, [0xF8, 0xD0, 0x98]);
            }
        }
        if let Some(n) = m.name.as_deref().filter(|n| !n.is_empty()) {
            fb.draw_text_outlined(cx - text_width(n) / 2, hy + HEAD_PX + 1, n, [0xFF, 0xFF, 0xFF]);
        }
    }
    fb.draw_text(22, 22, "Map", [0x00, 0x00, 0xFF]);
    fb.draw_text(20, 20, "Map", [0xFF, 0xFF, 0xFF]);
    if let Some(n) = others {
        // (485, 25) of a 640-pixel view: 155 from the right.
        fb.draw_text(vw - 155, 25, &format!("Others: {n}"), [0xFF, 0xFF, 0xFF]);
    }
}

/// The down-facing head (the third 32×32 frame of a head image), at half
/// size: 16×16 at `(x, y)`. A head image shorter than that draws its first
/// frame.
fn draw_head(fb: &mut Framebuffer, px: &[u8], w: u32, x: i32, y: i32) {
    let rows = px.len() / (w.max(1) as usize * 4);
    let top = if rows >= 96 { 64 } else { 0 };
    let size = (w as i32).min(32);
    for ty in 0..HEAD_PX {
        for tx in 0..HEAD_PX {
            let (sx, sy) = (tx * size / HEAD_PX, top + ty * size / HEAD_PX);
            let i = (sy as usize * w as usize + sx as usize) * 4;
            let Some(c) = px.get(i..i + 4) else { continue };
            if c[3] == 0 {
                continue;
            }
            fb.fill_rect(x + tx, y + ty, 1, 1, [c[0], c[1], c[2]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(w: usize, h: usize) -> Gmap {
        let names = (0..w * h).map(|i| format!("l{}-{}.nw", i % w, i / w)).collect();
        Gmap { width: w, height: h, names, ..Gmap::default() }
    }

    #[test]
    fn the_205_picture_is_56_pixels_a_level() {
        // map.png and bigmap.txt as 2.05 and 2.22 ship them.
        assert_eq!(cell_size(&grid(20, 21), 1120, 1176), (56.0, 56.0));
        assert_eq!(cell_size(&grid(3, 4), 168, 224), (56.0, 56.0));
    }

    #[test]
    fn a_player_is_placed_inside_its_cell() {
        let g = grid(20, 21);
        let c = cell_size(&g, 1120, 1176);
        assert_eq!(spot(&g, c, "l2-3.nw", 0.0, 0.0), Some((112, 168)));
        assert_eq!(spot(&g, c, "L2-3.NW", 32.0, 64.0), Some((140, 224)));
        assert_eq!(spot(&g, c, "elsewhere.nw", 32.0, 32.0), None);
    }

    #[test]
    fn opening_centres_on_the_player_inside_the_picture() {
        let mut m = MapScreen::default();
        m.open_at(Some((560, 588)), (1120, 1176), (640, 480));
        assert!(m.open);
        assert_eq!(m.scroll, (240, 348));
        // Near a corner the view stops at the picture's edge.
        m.open_at(Some((10, 10)), (1120, 1176), (640, 480));
        assert_eq!(m.scroll, (0, 0));
        m.open_at(Some((1110, 1170)), (1120, 1176), (640, 480));
        assert_eq!(m.scroll, (480, 696));
    }

    #[test]
    fn a_player_off_the_map_gets_the_middle() {
        let mut m = MapScreen::default();
        m.open_at(None, (1120, 1176), (640, 480));
        assert_eq!(m.scroll, (240, 348));
    }

    #[test]
    fn a_small_picture_is_centred() {
        // The tutorial's 168×224 in a 640×480 view.
        assert_eq!(clamp_scroll((0, 0), (168, 224), (640, 480)), (-236, -128));
        let mut m = MapScreen::default();
        m.open_at(Some((84, 112)), (168, 224), (640, 480));
        assert_eq!(m.scroll, (-236, -128));
        m.scroll_by((1, 1), (168, 224), (640, 480));
        assert_eq!(m.scroll, (-236, -128), "nothing to scroll");
    }

    #[test]
    fn keys_scroll_eight_pixels_and_stop_at_the_edges() {
        let (pic, view) = ((1120, 1176), (640, 480));
        let mut m = MapScreen::default();
        m.open_at(Some((560, 588)), pic, view);
        m.scroll_by((1, 0), pic, view);
        m.scroll_by((0, -1), pic, view);
        assert_eq!(m.scroll, (248, 340));
        for _ in 0..200 {
            m.scroll_by((-1, -1), pic, view);
        }
        assert_eq!(m.scroll, (0, 0));
        for _ in 0..200 {
            m.scroll_by((1, 1), pic, view);
        }
        assert_eq!(m.scroll, (480, 696));
    }

    #[test]
    fn the_picture_heads_and_title_are_drawn() {
        let mut fb = Framebuffer::new(64, 48);
        let pic = Picture { w: 128, h: 96, px: [10u8, 200, 30, 255].repeat(128 * 96) };
        let head = [250u8, 0, 0, 255].repeat(32 * 128);
        let screen = MapScreen { open: true, scroll: (8, 8) };
        let markers = [Marker { at: (40, 40), head: Some((32, &head)), name: None }];
        draw(&mut fb, &screen, Some(&pic), &markers, None);
        let at = |x: usize, y: usize| fb.pixels[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3].to_vec();
        assert_eq!(at(60, 45), vec![10, 200, 30], "the picture");
        assert_eq!(at(32, 32), vec![250, 0, 0], "the head centred on (40, 40) - scroll");
        assert_eq!(at(40, 40), vec![10, 200, 30], "16 pixels wide only");
    }

    #[test]
    fn files_are_read_once_and_dropped_for_a_new_source() {
        let png = {
            let mut fb = Framebuffer::new(112, 56);
            fb.fill_rect(0, 0, 112, 56, [1, 2, 3]);
            let mut out = Vec::new();
            crate::assets::write_png_rgb(&mut out, 112, 56, &fb.pixels).unwrap();
            out
        };
        let read = |n: &str| match n {
            "bigmap.txt" => Some(b"\"a.nw\",\"b.nw\"\n".to_vec()),
            "map.png" => Some(png.clone()),
            _ => None,
        };
        let mut f = MapFiles::default();
        assert!(f.sync(&MapSource::default_map(), None, &read).is_empty());
        assert_eq!(f.size(), (112, 56));
        assert_eq!(f.spot("b.nw", 32.0, 32.0), Some((84, 28)));
        // The server names another map: its files are asked for.
        let other = MapSource::new("bigmap2.txt", "bigmap.png");
        assert_eq!(f.sync(&other, None, &read), ["bigmap2.txt", "bigmap.png"]);
        assert!(f.grid.is_none() && f.picture.is_none());
    }

    #[test]
    fn a_level_off_the_map_uses_the_sources_point() {
        let read = |n: &str| (n == "t.txt").then(|| b"\"a.nw\",\"b.nw\",\"c.nw\"\n\"d.nw\",\"e.nw\",\"f.nw\"\n".to_vec());
        let mut f = MapFiles::default();
        f.sync(&MapSource::new("t.txt", ""), None, &read);
        assert_eq!(f.spot("elsewhere.nw", 0.0, 0.0), None, "no point given");
        // The tutorial's `setmap ...,1.5,3.8`, at 56 pixels a level.
        f.sync(&MapSource::new("t.txt", "").with_off_map(1.5, 3.8), None, &read);
        assert_eq!(f.spot("elsewhere.nw", 0.0, 0.0), Some((84, 213)));
        assert_eq!(f.spot("e.nw", 0.0, 0.0), Some((56, 56)));
    }

    #[test]
    fn without_a_picture_the_view_is_black() {
        let mut fb = Framebuffer::new(64, 48);
        fb.fill_rect(0, 0, 64, 48, [9, 9, 9]);
        draw(&mut fb, &MapScreen { open: true, scroll: (0, 0) }, None, &[], None);
        assert_eq!(&fb.pixels[(47 * 64 + 63) * 3..(47 * 64 + 63) * 3 + 3], &[0, 0, 0]);
    }
}
