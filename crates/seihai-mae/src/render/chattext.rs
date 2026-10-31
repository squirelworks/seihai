//! Chat text over a player's head, drawn the way the real clients draw it.
//!
//! Every build from 1st Graal to 2.31 draws the chat line with the Windows
//! canvas (`TCanvas.TextOut`): a bold TrueType font, no box behind it, the
//! text in white over a one-pixel shadow down and to the right. What changes
//! between builds is the font ([`ChatText`] in the version profile). The rest
//! is the same everywhere and lives here. Read from the 2.05 rev 2 binary and
//! checked against the real client under Wine (Graal_analysis
//! `client-internals/chat-text.md`):
//!
//! - Colour white. The shadow colour comes from the text colour: red and
//!   green 0 when either is above 0xE0, else 0xFF, and the text's own blue.
//!   For white that is pure blue (0, 0, 255).
//! - The shadow is drawn first at (+1, +1), then the text at (0, 0).
//! - Centred on the middle of the player box (x + 1.5 tiles). The top of the
//!   last line is one tile above the box (two while carrying something).
//! - Too wide for the view: word-wrapped at spaces. Lines stack upward, so
//!   the last line keeps its place; a single word wider than the view is not
//!   broken. Each line is pushed back inside the view on the left and right.
//!
//! The fonts themselves are not part of Seihai. [`ChatFont::find`] looks for
//! the build's font in the game folders and the system's font folders; with
//! none found the chat is drawn with the built-in bitmap font, in the same
//! colours and place.

use super::font;
use super::Framebuffer;
use crate::protocol::{ChatFace, ChatText};
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Edge coverage is scaled by this before blending. Windows draws the
/// text hinted and with its own gamma, so its strokes are fuller than plain
/// coverage; 1.8 brings the share of near-white pixels close to the real
/// 2.05 client's (623 against 570 to 602 for 1.7 to 2.0, "hello world Wg").
const COVERAGE_GAIN: f32 = 1.8;

/// The chat colour every build uses.
pub const CHAT_RGB: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// Shadow colour for text of colour `rgb` (the client's rule).
pub fn shadow_of(rgb: [u8; 3]) -> [u8; 3] {
    let v = if rgb[0] > 0xE0 || rgb[1] > 0xE0 { 0 } else { 0xFF };
    [v, v, rgb[2]]
}

/// Font files to try for a face, best first, and whether the file is
/// already bold (otherwise bold is made the way Windows does it).
fn candidates(face: ChatFace) -> &'static [(&'static str, bool)] {
    const ARIAL: &[(&str, bool)] = &[
        ("arialbd.ttf", true),
        ("LiberationSans-Bold.ttf", true),
        ("Arimo-Bold.ttf", true),
        ("DejaVuSans-Bold.ttf", true),
        ("FreeSansBold.ttf", true),
        ("arial.ttf", false),
        ("LiberationSans-Regular.ttf", false),
        ("DejaVuSans.ttf", false),
    ];
    const TEMPUS: &[(&str, bool)] = &[
        ("tempsitc.ttf", false),
        ("arialbd.ttf", true),
        ("LiberationSans-Bold.ttf", true),
        ("Arimo-Bold.ttf", true),
        ("DejaVuSans-Bold.ttf", true),
        ("FreeSansBold.ttf", true),
        ("arial.ttf", false),
        ("LiberationSans-Regular.ttf", false),
        ("DejaVuSans.ttf", false),
    ];
    match face {
        ChatFace::Dialog | ChatFace::Helvetica | ChatFace::Arial => ARIAL,
        ChatFace::TempusSans => TEMPUS,
    }
}

/// The system's font folders.
fn system_font_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(w) = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot")) {
        v.push(PathBuf::from(w).join("Fonts"));
    }
    if let Some(h) = std::env::var_os("HOME") {
        let h = PathBuf::from(h);
        v.push(h.join(".fonts"));
        v.push(h.join(".local/share/fonts"));
        v.push(h.join("Library/Fonts"));
    }
    for d in ["/usr/share/fonts", "/usr/local/share/fonts", "/Library/Fonts", "/System/Library/Fonts"] {
        v.push(PathBuf::from(d));
    }
    v
}

/// `name` in `dir` or (up to `depth` levels) below it, ignoring case.
fn find_file(dir: &Path, name: &str, depth: u32) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            subdirs.push(p);
        } else if e.file_name().to_string_lossy().eq_ignore_ascii_case(name) {
            return Some(p);
        }
    }
    if depth == 0 {
        return None;
    }
    subdirs.sort();
    subdirs.iter().find_map(|d| find_file(d, name, depth - 1))
}

/// One rendered line: coverage per pixel, `w` × the cell height.
struct Line {
    w: i32,
    cov: Vec<u8>,
}

/// A loaded chat font at a build's cell height.
pub struct ChatFont {
    font: FontVec,
    /// The file has no bold face: embolden by one pixel, as Windows does.
    synth_bold: bool,
    height: u8,
    pub path: PathBuf,
    cache: RefCell<HashMap<String, Line>>,
}

impl ChatFont {
    /// Load the font file at `path` for `style`.
    pub fn load(path: &Path, bold_file: bool, style: ChatText) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        let font = FontVec::try_from_vec(data).ok()?;
        Some(Self { font, synth_bold: !bold_file, height: style.height, path: path.to_path_buf(), cache: RefCell::new(HashMap::new()) })
    }

    /// The build's font from the first of `dirs` that has it (the game
    /// folders, top level only), else from the system's font folders.
    pub fn find(style: ChatText, dirs: &[PathBuf]) -> Option<Self> {
        let sys = system_font_dirs();
        for &(name, bold) in candidates(style.face) {
            let hit = dirs.iter().find_map(|d| find_file(d, name, 0)).or_else(|| sys.iter().find_map(|d| find_file(d, name, 3)));
            if let Some(f) = hit.and_then(|p| Self::load(&p, bold, style)) {
                return Some(f);
            }
        }
        None
    }

    fn scale(&self) -> PxScale {
        // `Font.Height` > 0 is the cell height (ascent + descent), which is
        // what a PxScale is.
        PxScale::from(f32::from(self.height))
    }

    /// Pixel advance of `c`: whole pixels per character, as GDI lays out
    /// `TextOut`, plus the overhang of a made-up bold.
    fn advance(&self, c: char) -> i32 {
        let f = self.font.as_scaled(self.scale());
        f.h_advance(f.glyph_id(c)).round() as i32 + i32::from(self.synth_bold)
    }

    /// Width of `text` in pixels (`TextWidth`).
    pub fn width(&self, text: &str) -> i32 {
        text.chars().map(|c| self.advance(c)).sum()
    }

    pub fn line_height(&self) -> i32 {
        i32::from(self.height)
    }

    fn render(&self, text: &str) -> Line {
        let scale = self.scale();
        let sf = self.font.as_scaled(scale);
        let h = self.line_height();
        let w = self.width(text).max(1) + 1;
        let mut cov = vec![0u8; (w * h) as usize];
        let base = sf.ascent().round();
        let mut pen = 0i32;
        for c in text.chars() {
            let id = sf.glyph_id(c);
            let g = id.with_scale_and_position(scale, ab_glyph::point(pen as f32, base));
            if let Some(o) = self.font.outline_glyph(g) {
                let b = o.px_bounds();
                o.draw(|gx, gy, v| {
                    let x = b.min.x as i32 + gx as i32;
                    let y = b.min.y as i32 + gy as i32;
                    let a = ((v * COVERAGE_GAIN).clamp(0.0, 1.0) * 255.0).round() as u8;
                    for dx in 0..=i32::from(self.synth_bold) {
                        let px = x + dx;
                        if px >= 0 && px < w && y >= 0 && y < h {
                            let i = (y * w + px) as usize;
                            cov[i] = cov[i].max(a);
                        }
                    }
                });
            }
            pen += self.advance(c);
        }
        Line { w, cov }
    }

    /// Draw one line with its top-left at (x, y) in `rgb`.
    fn draw_line(&self, fb: &mut Framebuffer, x: i32, y: i32, text: &str, rgb: [u8; 3]) {
        let mut cache = self.cache.borrow_mut();
        if cache.len() > 256 {
            cache.clear();
        }
        let line = cache.entry(text.to_string()).or_insert_with(|| self.render(text));
        let h = self.line_height();
        for ly in 0..h {
            for lx in 0..line.w {
                let a = line.cov[(ly * line.w + lx) as usize];
                if a > 0 {
                    fb.blend_pixel(x + lx, y + ly, rgb, a);
                }
            }
        }
    }
}

/// Split `text` into lines no wider than `max_w` the way the client does:
/// words are added while the line fits; a word that does not fit starts the
/// next line, and a word that does not fit on a line of its own is a line.
pub fn wrap(text: &str, max_w: i32, width: impl Fn(&str) -> i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    let mut words = text.split(' ').peekable();
    while let Some(word) = words.peek() {
        let mut next = cur.clone();
        if !next.is_empty() {
            next.push(' ');
        }
        next.push_str(word);
        if width(&next) > max_w {
            if cur.is_empty() {
                lines.push(next);
                words.next();
            } else {
                lines.push(std::mem::take(&mut cur));
            }
        } else {
            cur = next;
            words.next();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// Left edge of a line `w` wide centred on `cx`, kept inside `0..view_w`.
pub fn line_left(cx: i32, w: i32, view_w: i32) -> i32 {
    let mut x = cx - w / 2;
    if x + w > view_w {
        x = view_w - w;
    }
    x.max(0)
}

/// Draw `text` centred on `cx` with the top of its last line at `top`, in
/// `rgb` over its shadow: the client's one routine for text over sprites
/// (chat, nicknames). Lines that do not fit the view wrap upward.
fn draw_label(fb: &mut Framebuffer, font: Option<&ChatFont>, text: &str, rgb: [u8; 3], cx: i32, top: i32) {
    let view_w = fb.width as i32;
    let shadow = shadow_of(rgb);
    match font {
        Some(f) => {
            let lines = wrap(text, view_w, |s| f.width(s));
            let lh = f.line_height();
            let mut y = top - (lines.len() as i32 - 1) * lh;
            for l in &lines {
                let x = line_left(cx, f.width(l), view_w);
                f.draw_line(fb, x + 1, y + 1, l, shadow);
                f.draw_line(fb, x, y, l, rgb);
                y += lh;
            }
        }
        None => {
            let lines = wrap(text, view_w, font::text_width);
            let lh = font::CELL_H;
            // The bitmap font is shorter than the real one: sit its last
            // line on the real text's baseline (about 18 pixels down the
            // 24-pixel cell; the glyphs are 7 high).
            let mut y = top + 18 - 7 - (lines.len() as i32 - 1) * lh;
            for l in &lines {
                let x = line_left(cx, font::text_width(l), view_w);
                fb.draw_text(x + 1, y + 1, l, shadow);
                fb.draw_text(x, y, l, rgb);
                y += lh;
            }
        }
    }
}

/// Draw `text` over the player whose 48×48 box has its top-left at
/// (`dx`, `dy`). `carrying` lifts it one more tile. `font` is `None` when no
/// TrueType font was found: the built-in bitmap font is used.
pub fn draw_chat_text(fb: &mut Framebuffer, font: Option<&ChatFont>, text: &str, dx: i32, dy: i32, carrying: bool) {
    if text.is_empty() {
        return;
    }
    let tile = super::TILE_PX as i32;
    let top = dy - tile * if carrying { 2 } else { 1 };
    draw_label(fb, font, text, CHAT_RGB, dx + tile * 3 / 2, top);
}

/// Draw an NPC's `message` centred on `cx` with the top of its last line at
/// `top`, in the chat colour and style.
pub fn draw_npc_message(fb: &mut Framebuffer, font: Option<&ChatFont>, text: &str, cx: i32, top: i32) {
    if !text.is_empty() {
        draw_label(fb, font, text, CHAT_RGB, cx, top);
    }
}

/// The nickname colour for a player with `ap` alignment points, the ramp
/// every build from 1.31 to 2.31 uses (and 6.x, per the v6 notes). Read
/// from 2.05 rev 2 and checked live for AP 0, 20, 40, 50, 58, 65, 85, 100.
/// Each step is `floor(n * k / d)`.
pub fn ap_colour(ap: u8) -> [u8; 3] {
    let f = |n: i32, k: i32, d: i32| (n * k / d) as u8;
    let a = i32::from(ap);
    match a {
        0 => [0x40, 0, 0],
        // Dark red to red.
        1..=26 => {
            let d = 0x40 - f(a - 1, 64, 25);
            [0x80 + f(a - 1, 127, 25), d, d]
        }
        // Red to pale pink.
        27..=49 => {
            let v = f(a - 26, 255, 24);
            [0xFF, v, v]
        }
        50..=55 => [0xFF, 0xFF, 0xFF],
        // White to light green.
        56..=60 => {
            let v = 0xFF - f(a - 55, 127, 5);
            [v, 0xFF, v]
        }
        // Light green to cyan.
        61..=70 => [0x80 - f(a - 61, 128, 10), 0xFF, 0x80 + f(a - 61, 127, 10)],
        // Cyan to blue.
        71..=99 => [0, 0xFF - f(a - 70, 255, 29), 0xFF],
        100 => [0xE0, 0xC4, 0],
        _ => [0xFF, 0, 0],
    }
}

/// Draw a nickname under the player box at (`dx`, `dy`): the top of its
/// cell 2.5 tiles below the box's top, in the AP colour.
pub fn draw_nick_text(fb: &mut Framebuffer, font: Option<&ChatFont>, nick: &str, ap: u8, dx: i32, dy: i32) {
    if nick.is_empty() {
        return;
    }
    let tile = super::TILE_PX as i32;
    draw_label(fb, font, nick, ap_colour(ap), dx + tile * 3 / 2, dy + tile * 5 / 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> i32 {
        s.chars().count() as i32 * 10
    }

    #[test]
    fn white_text_gets_a_blue_shadow() {
        assert_eq!(shadow_of([255, 255, 255]), [0, 0, 255]);
        assert_eq!(shadow_of([0, 0, 0]), [255, 255, 0]);
        assert_eq!(shadow_of([0xE0, 0xE0, 7]), [255, 255, 7]);
        assert_eq!(shadow_of([0xE1, 0, 9]), [0, 0, 9]);
    }

    #[test]
    fn nickname_colour_follows_ap() {
        // The colours read off the real 2.05 rev 2 screen in the lab.
        assert_eq!(ap_colour(0), [64, 0, 0]);
        assert_eq!(ap_colour(20), [224, 16, 16]);
        assert_eq!(ap_colour(40), [255, 148, 148]);
        assert_eq!(ap_colour(50), [255, 255, 255]);
        assert_eq!(ap_colour(58), [179, 255, 179]);
        assert_eq!(ap_colour(65), [77, 255, 178]);
        assert_eq!(ap_colour(85), [0, 124, 255]);
        assert_eq!(ap_colour(100), [224, 196, 0]);
        // Ends of the ramps.
        assert_eq!(ap_colour(1), [128, 64, 64]);
        assert_eq!(ap_colour(26), [255, 0, 0]);
        assert_eq!(ap_colour(99), [0, 0, 255]);
    }

    #[test]
    fn nickname_sits_under_the_box() {
        let mut fb = Framebuffer::new(320, 240);
        draw_nick_text(&mut fb, None, "bob", 100, 100, 60);
        let gold: Vec<(usize, usize)> =
            (0..240).flat_map(|y| (0..320).map(move |x| (x, y))).filter(|&(x, y)| fb.pixels[(y * 320 + x) * 3..][..3] == [224, 196, 0]).collect();
        assert!(!gold.is_empty());
        assert!(gold.iter().all(|p| p.1 >= 100 && p.1 < 124), "within the cell 40 px below the box top");
    }

    #[test]
    fn short_text_is_one_line() {
        assert_eq!(wrap("hello world", 640, w), vec!["hello world"]);
        assert_eq!(wrap("", 640, w), Vec::<String>::new());
    }

    #[test]
    fn long_text_wraps_at_spaces() {
        // 5 characters per line at most.
        assert_eq!(wrap("aa bb cc dd", 50, w), vec!["aa bb", "cc dd"]);
        assert_eq!(wrap("aa bbbb cc", 50, w), vec!["aa", "bbbb", "cc"]);
    }

    #[test]
    fn a_word_wider_than_the_view_is_its_own_line() {
        assert_eq!(wrap("aa bbbbbbbb cc", 50, w), vec!["aa", "bbbbbbbb", "cc"]);
        assert_eq!(wrap("bbbbbbbb", 50, w), vec!["bbbbbbbb"]);
    }

    #[test]
    fn lines_stay_inside_the_view() {
        assert_eq!(line_left(320, 100, 640), 270);
        assert_eq!(line_left(10, 100, 640), 0);
        assert_eq!(line_left(630, 100, 640), 540);
        assert_eq!(line_left(320, 700, 640), 0);
    }

    #[test]
    fn bitmap_fallback_draws_text_and_blue_shadow_above_the_player() {
        let mut fb = Framebuffer::new(320, 240);
        draw_chat_text(&mut fb, None, "hi", 100, 120, false);
        let px = |x: i32, y: i32| {
            let i = (y as usize * 320 + x as usize) * 3;
            [fb.pixels[i], fb.pixels[i + 1], fb.pixels[i + 2]]
        };
        let mut white = Vec::new();
        let mut blue = 0;
        for y in 0..240 {
            for x in 0..320 {
                match px(x, y) {
                    [255, 255, 255] => white.push((x, y)),
                    [0, 0, 255] => blue += 1,
                    _ => {}
                }
            }
        }
        assert!(!white.is_empty() && blue > 0);
        // Centred on x = 100 + 24, sitting on the baseline near the top of
        // the box.
        let (minx, maxx) = (white.iter().map(|p| p.0).min().unwrap(), white.iter().map(|p| p.0).max().unwrap());
        assert!((minx + maxx) / 2 >= 120 && (minx + maxx) / 2 <= 128);
        let maxy = white.iter().map(|p| p.1).max().unwrap();
        assert!((112..=122).contains(&maxy), "maxy {maxy}");
        // Carrying lifts it one tile.
        let mut fb2 = Framebuffer::new(320, 240);
        draw_chat_text(&mut fb2, None, "hi", 100, 120, true);
        let low = (0..240).rev().find(|&y| (0..320).any(|x| fb2.pixels[(y * 320 + x) * 3..][..3] == [255, 255, 255])).unwrap();
        assert_eq!(low as i32, maxy - 16);
    }

    /// With a real TrueType font on the machine, the text is that tall and
    /// white over blue.
    #[test]
    fn truetype_text_when_a_font_is_found() {
        let style = ChatText { face: ChatFace::Helvetica, height: 24 };
        let Some(f) = ChatFont::find(style, &[]) else {
            eprintln!("no Arial-like font on this machine; skipped");
            return;
        };
        assert!(f.width("hello") > 30);
        let mut fb = Framebuffer::new(320, 240);
        draw_chat_text(&mut fb, Some(&f), "hello", 100, 120, false);
        let white: Vec<(usize, usize)> =
            (0..240).flat_map(|y| (0..320).map(move |x| (x, y))).filter(|&(x, y)| fb.pixels[(y * 320 + x) * 3..][..3] == [255, 255, 255]).collect();
        assert!(!white.is_empty());
        let top = white.iter().map(|p| p.1).min().unwrap();
        let bottom = white.iter().map(|p| p.1).max().unwrap();
        // Inside the 24-pixel cell that starts 16 pixels above the box.
        assert!(top >= 104 && bottom < 128, "{top}..{bottom}");
    }
}
