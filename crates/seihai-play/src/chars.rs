//! The players, drawn from their GANIs (#443): the body, head, sword,
//! shield, shadow and attributes each frame of the pose names, in the
//! looks the server sent. The frame is painted by `seihai-draw`'s
//! `draw_gani_frame`, the same drawing the level editor and tools use.
//!
//! The pose GANIs (`idle`, `walk`, `swim`, `sword`, ...) and the pictures
//! come from the asset folders, else from core's built-in set (the default
//! body, `head0`, `sword1`, `shield1` and `sprites.png`). A picture that is
//! in neither is left out of the frame; it is not asked of the server yet.
//!
//! Which pose another player is in follows `seihai-mae`'s rule: the GANI
//! the server named (2.1 and later), else the pre-2.1 SPRITE prop, else
//! standing. Dead players show the last step of `dead.gani`.

use crate::render::Image;
use seihai_core::assets::{builtin, AssetResolver, Gani};
use seihai_core::client::Other;
use seihai_core::looks::{self, Looks};
use seihai_core::pose::WALK_S;
use seihai_draw::{draw_gani_frame, PixelBuffer, SpriteSheets};
use std::collections::HashMap;

/// One GANI step lasts 50 ms.
pub const STEP_MS: u64 = 50;
/// The dead bit of the STATUS prop (18).
pub const STATUS_DEAD: u8 = 0x08;

/// The poses drawn, by the GANI file's stem.
const POSES: [&str; 10] = [
    "idle",
    "walk",
    "swim",
    "sword",
    "hurt",
    "sit",
    "sleep",
    "carry",
    "carrystill",
    "dead",
];

/// The default sword and shield when the server has not named one.
const DEFAULT_SWORD: &str = "sword1.png";
const DEFAULT_SHIELD: &str = "shield1.png";

/// What one player is drawn with: pictures by name and the pose.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    /// Tile position of the player's 48x48 box.
    pub x: f32,
    pub y: f32,
    /// 0 up, 1 left, 2 down, 3 right.
    pub dir: u8,
    /// The pose GANI's stem and the step to draw.
    pub pose: &'static str,
    pub step: Step,
    pub head: String,
    pub body: String,
    pub colors: [u8; 5],
    /// `None`: the default; `Some("")`: nothing.
    pub sword: Option<String>,
    pub shield: Option<String>,
    pub attrs: [Option<String>; 5],
}

/// Which step of the pose GANI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// This step (limited to the GANI's last).
    At(usize),
    /// Run on the clock, looping.
    Clock,
}

/// The pose GANIs and the pictures read so far.
#[derive(Default)]
pub struct Characters {
    ganis: HashMap<&'static str, Gani>,
    /// Pictures by key ([`picture_key`]); `None` is looked for and missing.
    pictures: HashMap<String, Option<Image>>,
}

/// The key a picture is kept under: its folder and name, with the colours
/// for a body.
fn picture_key(dir: &str, name: &str, colors: Option<&[u8; 5]>) -> String {
    match colors {
        Some(c) => format!("{dir}/{}", looks::colored_body_key(name, c)),
        None => format!("{dir}/{name}"),
    }
}

/// A picture's bytes and file name: the asset folders first, else the
/// built-in file of that name in `dir`.
fn picture_bytes(
    resolver: Option<&AssetResolver>,
    dir: &str,
    name: &str,
) -> Option<(String, Vec<u8>)> {
    if let Some(p) = resolver.and_then(|r| r.image_in(dir, name)) {
        if let Ok(d) = std::fs::read(&p) {
            let file = p.file_name()?.to_string_lossy().into_owned();
            return Some((file, d));
        }
    }
    let rel = if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    };
    let lower = rel.to_ascii_lowercase();
    // The built-in set has a PNG and a GIF of each default; either name
    // finds it.
    [lower.clone(), swap_ext(&lower)]
        .into_iter()
        .find_map(|n| builtin::bytes(&n).map(|d| (n, d.to_vec())))
}

fn swap_ext(name: &str) -> String {
    if let Some(stem) = name.strip_suffix(".gif") {
        format!("{stem}.png")
    } else if let Some(stem) = name.strip_suffix(".png") {
        format!("{stem}.gif")
    } else {
        name.to_string()
    }
}

/// A body in `colors`: its palette entries replaced, as the 2.05 client
/// draws it. A picture with no palette is drawn as it is.
fn decode_body(file: &str, data: &[u8], colors: &[u8; 5]) -> Option<Image> {
    let recolor = looks::body_recolor(colors);
    let lower = file.to_ascii_lowercase();
    let read = if lower.ends_with(".gif") {
        seihai_core::assets::gif::read_gif_rgba_recolored(data, &recolor)
    } else {
        seihai_core::assets::read_png_rgba_recolored(data, &recolor)
    };
    match read {
        Ok((w, h, rgba)) => Some(Image { w, h, rgba }),
        Err(_) => Image::decode(file, data),
    }
}

impl Characters {
    /// The pose GANIs from the asset folders, else the built-in ones.
    pub fn load(resolver: Option<&AssetResolver>) -> Characters {
        let mut ganis = HashMap::new();
        for pose in POSES {
            let file = format!("{pose}.gani");
            let from_disk = resolver
                .and_then(|r| r.pose_gani(&file))
                .and_then(|p| Gani::load(p).ok());
            let gani = from_disk.or_else(|| {
                let text = builtin::bytes(&format!("gani/{file}"))?;
                Gani::parse(&String::from_utf8_lossy(text)).ok()
            });
            if let Some(g) = gani {
                ganis.insert(pose, g);
            }
        }
        Characters {
            ganis,
            pictures: HashMap::new(),
        }
    }

    /// The pose GANI `pose`, if it was found.
    pub fn gani(&self, pose: &str) -> Option<&Gani> {
        self.ganis.get(pose)
    }

    /// Read the pictures these figures need that were not looked for yet.
    /// True if any was added.
    pub fn load_pictures(&mut self, resolver: Option<&AssetResolver>, figures: &[Figure]) -> bool {
        let mut added = false;
        for f in figures {
            for (dir, name, colors) in self.wanted(f) {
                let key = picture_key(dir, &name, colors.as_ref());
                if self.pictures.contains_key(&key) {
                    continue;
                }
                let image =
                    picture_bytes(resolver, dir, &name).and_then(|(file, data)| match colors {
                        Some(c) => decode_body(&file, &data, &c),
                        None => Image::decode(&file, &data),
                    });
                added |= image.is_some();
                self.pictures.insert(key, image);
            }
        }
        added
    }

    /// The pictures one figure is drawn with: folder, name and, for the
    /// body, its colours.
    fn wanted(&self, f: &Figure) -> Vec<(&'static str, String, Option<[u8; 5]>)> {
        let mut out = vec![
            ("bodies", f.body.clone(), Some(f.colors)),
            ("heads", f.head.clone(), None),
            ("", "sprites.png".to_string(), None),
        ];
        let sword = f.sword.as_deref().unwrap_or(DEFAULT_SWORD);
        let shield = f.shield.as_deref().unwrap_or(DEFAULT_SHIELD);
        for (dir, name) in [("swords", sword), ("shields", shield)] {
            if !name.is_empty() {
                out.push((dir, name.to_string(), None));
            }
        }
        if let Some(g) = self.ganis.get(f.pose) {
            for i in 0..5 {
                if let Some(a) = looks::attr_image(&f.attrs, g, i) {
                    out.push((looks::attr_dir(i), a.to_string(), None));
                }
            }
        }
        out
    }

    fn picture(&self, dir: &str, name: &str, colors: Option<&[u8; 5]>) -> Option<&Image> {
        self.pictures.get(&picture_key(dir, name, colors))?.as_ref()
    }

    /// Draw `f` with its box's top-left at `(dx, dy)` on `dst`. `clock_ms`
    /// steps a pose that runs on the clock. Nothing is drawn when the
    /// pose GANI is missing.
    pub fn draw(&self, dst: &mut PixelBuffer, f: &Figure, clock_ms: u64, dx: i32, dy: i32) {
        let Some(gani) = self.ganis.get(f.pose) else {
            return;
        };
        let n = gani.frame_count();
        if n == 0 {
            return;
        }
        let step = match f.step {
            Step::At(i) => i.min(n - 1),
            Step::Clock => gani.frame_at_tick((clock_ms / STEP_MS) as usize, true),
        };
        let sword = f.sword.as_deref().unwrap_or(DEFAULT_SWORD);
        let shield = f.shield.as_deref().unwrap_or(DEFAULT_SHIELD);
        let mut attrs = [None; 5];
        for (i, slot) in attrs.iter_mut().enumerate() {
            *slot = looks::attr_image(&f.attrs, gani, i)
                .and_then(|a| view(self.picture(looks::attr_dir(i), a, None)));
        }
        let sheets = SpriteSheets {
            body: view(self.picture("bodies", &f.body, Some(&f.colors))),
            head: view(self.picture("heads", &f.head, None)),
            sprites: view(self.picture("", "sprites.png", None)),
            sword: view(self.picture("swords", sword, None)),
            shield: view(self.picture("shields", shield, None)),
            attrs,
        };
        draw_gani_frame(dst, gani, step, f.dir, &sheets, dx, dy);
    }
}

fn view(p: Option<&Image>) -> Option<seihai_draw::Image<'_>> {
    p.map(|i| seihai_draw::Image::rgba(&i.rgba, i.w, i.h))
}

/// The GANI another player is in, lower case, without `.gani` (GServer-v2
/// adds `.gif` to a name a 2.05 player sends in the bow slot).
fn gani_name(o: &Other) -> Option<String> {
    o.looks.gani.as_deref().map(|n| {
        n.to_ascii_lowercase()
            .trim_end_matches(".gani")
            .trim_end_matches(".gif")
            .to_string()
    })
}

/// The pose and step another player is drawn in.
pub fn other_pose(o: &Other) -> (&'static str, Step) {
    if o.status.is_some_and(|s| s & STATUS_DEAD != 0) {
        return ("dead", Step::At(usize::MAX));
    }
    if let Some(name) = gani_name(o) {
        let named = match name.as_str() {
            "idle" => Some(("idle", Step::At(0))),
            "walk" => Some(("walk", Step::Clock)),
            "swim" => Some(("swim", Step::Clock)),
            "sword" => Some(("sword", Step::Clock)),
            "hurt" => Some(("hurt", Step::Clock)),
            "sit" => Some(("sit", Step::At(0))),
            "sleep" => Some(("sleep", Step::At(0))),
            "carrystill" | "lift" => Some(("carrystill", Step::At(0))),
            "carry" => Some(("carry", Step::Clock)),
            "dead" => Some(("dead", Step::At(usize::MAX))),
            _ => None,
        };
        if let Some(p) = named {
            return p;
        }
    }
    let s = o.sprite.unwrap_or(0) / 4;
    match s {
        1..=5 => (
            "walk",
            Step::At(WALK_S.iter().position(|&w| w == s).unwrap_or(0)),
        ),
        6..=10 => ("sword", Step::At(usize::from(s - 6))),
        18 => ("carrystill", Step::At(0)),
        19 | 20 => ("carry", Step::At(usize::from(s - 19))),
        28 => ("sit", Step::At(0)),
        32 => ("hurt", Step::At(0)),
        _ => ("idle", Step::At(0)),
    }
}

/// The figure for another player.
pub fn other_figure(o: &Other) -> Figure {
    let l = &o.looks;
    let (pose, step) = other_pose(o);
    let named = |n: &Option<String>, d: &str| {
        n.as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(d)
            .to_string()
    };
    Figure {
        x: o.x,
        y: o.y,
        dir: o.dir % 4,
        pose,
        step,
        head: named(&l.head, looks::DEFAULT_HEAD),
        body: named(&l.body, looks::DEFAULT_BODY),
        colors: l.colors.unwrap_or(looks::DEFAULT_COLORS),
        sword: l.sword.clone(),
        shield: l.shield.clone(),
        attrs: l.attrs.clone(),
    }
}

/// The figure for the player: swimming, walking or standing, in their own
/// looks. `walk_step` is the step of the session's `walk.gani` when it has
/// one; the swim stroke runs on the clock.
pub fn own_figure(
    x: f32,
    y: f32,
    dir: u8,
    looks: &Looks,
    in_water: bool,
    walking: bool,
    walk_step: Option<usize>,
) -> Figure {
    let (pose, step) = if in_water {
        ("swim", Step::Clock)
    } else if walking {
        ("walk", walk_step.map_or(Step::Clock, Step::At))
    } else {
        ("idle", Step::At(0))
    };
    Figure {
        x,
        y,
        dir: dir % 4,
        pose,
        step,
        head: looks.head_name().to_string(),
        body: looks.body_name().to_string(),
        colors: looks.colors,
        sword: None,
        shield: None,
        attrs: looks.attrs.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn other(sprite: Option<u8>, gani: Option<&str>) -> Other {
        let mut o = Other {
            id: 1,
            dir: 2,
            sprite,
            ..Default::default()
        };
        o.looks.gani = gani.map(String::from);
        o
    }

    #[test]
    fn the_named_gani_wins_then_the_sprite_then_idle() {
        assert_eq!(other_pose(&other(None, None)), ("idle", Step::At(0)));
        assert_eq!(
            other_pose(&other(Some(4 * 6), Some("walk.gani"))),
            ("walk", Step::Clock)
        );
        // A 2.05 player's bow slot name, with GServer's `.gif`.
        assert_eq!(
            other_pose(&other(None, Some("Sword.gif"))),
            ("sword", Step::Clock)
        );
        assert_eq!(
            other_pose(&other(Some(4 * 7 + 2), None)),
            ("sword", Step::At(1))
        );
        assert_eq!(other_pose(&other(Some(4 * 3), None)).0, "walk");
        assert_eq!(other_pose(&other(Some(4 * 28), None)), ("sit", Step::At(0)));
        // A GANI Seihai does not draw falls back to the sprite.
        assert_eq!(
            other_pose(&other(Some(4 * 32), Some("dance"))),
            ("hurt", Step::At(0))
        );
        let mut dead = other(Some(4 * 2), Some("walk"));
        dead.status = Some(STATUS_DEAD);
        assert_eq!(other_pose(&dead).0, "dead");
    }

    #[test]
    fn the_built_in_poses_and_pictures_load_with_no_asset_folder() {
        let mut c = Characters::load(None);
        for pose in ["idle", "walk", "swim", "sword", "dead"] {
            assert!(c.gani(pose).is_some(), "{pose}");
        }
        let f = other_figure(&other(None, None));
        assert_eq!(
            (f.head.as_str(), f.body.as_str()),
            ("head0.png", "body.png")
        );
        assert!(c.load_pictures(None, std::slice::from_ref(&f)));
        assert!(c.picture("heads", "head0.png", None).is_some());
        assert!(c.picture("bodies", "body.png", Some(&f.colors)).is_some());
        assert!(c.picture("swords", DEFAULT_SWORD, None).is_some());
        assert!(c.picture("", "sprites.png", None).is_some());
        // Looked for once.
        assert!(!c.load_pictures(None, &[f]));
    }

    #[test]
    fn a_missing_picture_is_left_out_and_not_looked_for_again() {
        let mut c = Characters::load(None);
        let mut o = other(None, None);
        o.looks.head = Some("nowhere.png".into());
        let f = other_figure(&o);
        c.load_pictures(None, std::slice::from_ref(&f));
        assert_eq!(c.pictures.get("heads/nowhere.png"), Some(&None));
    }

    #[test]
    fn a_player_is_drawn_from_the_idle_gani_in_their_colours() {
        let mut c = Characters::load(None);
        let mut o = other(None, None);
        o.x = 1.0;
        o.y = 1.0;
        let plain = other_figure(&o);
        o.looks.colors = Some([0, 10, 10, 10, 10]);
        let blue = other_figure(&o);
        c.load_pictures(None, &[plain.clone(), blue.clone()]);
        let draw = |f: &Figure| {
            let mut buf = PixelBuffer::filled(96, 96, [0, 0, 0, 255]);
            c.draw(&mut buf, f, 0, 16, 16);
            buf
        };
        let a = draw(&plain);
        let b = draw(&blue);
        let painted =
            |buf: &PixelBuffer| buf.rgba.chunks(4).filter(|p| p[..3] != [0, 0, 0]).count();
        assert!(painted(&a) > 200, "the figure is drawn: {}", painted(&a));
        assert_ne!(a.rgba, b.rgba, "other colours draw another body");
    }

    #[test]
    fn the_player_swims_walks_or_stands() {
        let l = Looks::default();
        assert_eq!(
            own_figure(0.0, 0.0, 2, &l, true, true, Some(3)).pose,
            "swim"
        );
        let w = own_figure(0.0, 0.0, 6, &l, false, true, Some(3));
        assert_eq!((w.pose, w.step, w.dir), ("walk", Step::At(3), 2));
        assert_eq!(own_figure(0.0, 0.0, 2, &l, false, false, None).pose, "idle");
    }
}
