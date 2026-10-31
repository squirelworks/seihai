//! Drawing a Graal3D scene: the level, the players and the chat log.
//!
//! Files come from the asset folder (the Graal3D install: textures and
//! the player meshes) and the level folder (levels the server sent). The
//! protocol never sends textures or models, so without an asset folder
//! the level draws in its material colours and players as their parts'
//! colours where the meshes are missing.

use crate::bmp::{self, Image};
use crate::model::{self, Tint};
use crate::props::Player;
use crate::render::{draw_mesh, yaw_at, Camera, Frame, Placement};
use crate::session::safe_file_name;
use crate::world::{self, FollowCam};
use crate::xfile::XFile;
use std::collections::HashMap;
use std::path::PathBuf;

#[path = "../../seihai-mae/src/render/font.rs"]
#[allow(dead_code)]
mod font;

pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;
/// Horizontal field of view (Seihai's choice).
pub const FOV: f32 = 70.0;
const SKY: u32 = 0x5a78b4;

/// How long a chat line stays: 10 s at 50 ms ticks.
pub const CHAT_TICKS: u32 = 200;

/// Files read once and kept.
pub struct Assets {
    dirs: Vec<PathBuf>,
    textures: HashMap<String, Image>,
    meshes: HashMap<String, Option<XFile>>,
}

impl Assets {
    /// Look for files in these folders, first match wins.
    pub fn new(dirs: Vec<PathBuf>) -> Assets {
        Assets { dirs, textures: HashMap::new(), meshes: HashMap::new() }
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let file = safe_file_name(name);
        self.dirs.iter().find_map(|d| std::fs::read(d.join(&file)).ok())
    }

    fn mesh(&mut self, name: &str) -> Option<&XFile> {
        let key = name.to_ascii_lowercase();
        if !self.meshes.contains_key(&key) {
            let x = self.read(name).and_then(|b| XFile::parse(&b).ok());
            if let Some(x) = &x {
                for t in x.textures() {
                    self.texture(&t);
                }
            }
            self.meshes.insert(key.clone(), x);
        }
        self.meshes.get(&key).and_then(Option::as_ref)
    }

    fn texture(&mut self, name: &str) {
        let key = name.to_ascii_lowercase();
        if self.textures.contains_key(&key) {
            return;
        }
        if let Some(img) = self.read(name).and_then(|b| bmp::decode(&b).ok()) {
            self.textures.insert(key, img);
        }
    }

    /// Load a level's textures (the level text is given, not read).
    pub fn level_textures(&mut self, level: &XFile) {
        for t in level.textures() {
            self.texture(&t);
        }
    }
}

/// One line of the chat log.
pub struct ChatLine {
    pub text: String,
    pub ticks: u32,
}

#[derive(Default)]
pub struct ChatLog {
    pub lines: Vec<ChatLine>,
}

impl ChatLog {
    pub fn push(&mut self, nick: &str, text: &str) {
        self.lines.push(ChatLine { text: format!("{nick}: {text}"), ticks: CHAT_TICKS });
        if self.lines.len() > 20 {
            self.lines.remove(0);
        }
    }

    pub fn tick(&mut self) {
        self.lines.iter_mut().for_each(|l| l.ticks = l.ticks.saturating_sub(1));
        self.lines.retain(|l| l.ticks > 0);
    }
}

/// Draw one player: the eight parts posed for prop 5, body and head from
/// props 6–9, the limbs tinted by props 10, 11 and 17.
pub fn draw_player(f: &mut Frame, cam: &Camera, a: &mut Assets, p: &Player, lift: f32) {
    // The model faces −z; the player's angle is where it looks.
    let frame = yaw_at(p.angle as f32 + 180.0, [p.x, p.y + lift, p.z]);
    let pose = model::pose(&frame, model::time(p.anim));
    a.texture(&p.body_texture);
    a.texture(&p.head_texture);
    for (i, part) in model::PARTS.iter().enumerate() {
        let (file, tex) = match i {
            model::HEAD => (p.head_model.as_str(), Some(&p.head_texture)),
            model::BODY => (p.body_model.as_str(), Some(&p.body_texture)),
            _ => (part.mesh, None),
        };
        let tint = match part.tint {
            Tint::None => None,
            Tint::Arm => Some(p.arm_colour),
            Tint::Hand => Some(p.hand_colour),
            Tint::Shoe => Some(p.shoe_colour),
        };
        let Some(x) = a.mesh(file).cloned() else { continue };
        for mut m in x.meshes {
            // Props 8 and 9 replace the body and head textures.
            if let Some(t) = tex {
                for mat in m.materials.iter_mut().filter(|mat| mat.texture.is_some()) {
                    mat.texture = Some(t.clone());
                }
            }
            draw_mesh(f, cam, &m, &Placement { m: pose[i], tint }, &a.textures);
        }
    }
}

/// What to draw this frame.
pub struct Scene<'a> {
    pub level: Option<&'a XFile>,
    pub me: &'a Player,
    pub others: &'a mut dyn Iterator<Item = &'a Player>,
    pub cam: &'a FollowCam,
    pub walking: bool,
    pub chat: &'a ChatLog,
    /// The chat line being typed, if any.
    pub typing: Option<&'a str>,
}

pub fn draw(f: &mut Frame, a: &mut Assets, s: Scene) {
    f.clear(SKY);
    let cam = Camera::look_at(s.cam.pos, s.cam.target(s.me), FOV, f.w);
    if let Some(level) = s.level {
        for m in &level.meshes {
            draw_mesh(f, &cam, m, &Placement::world(), &a.textures);
        }
    }
    for p in s.others {
        if p.in_level {
            draw_player(f, &cam, a, p, world::bob(p.anim, p.anim > 0 && p.anim < 40));
        }
    }
    draw_player(f, &cam, a, s.me, world::bob(s.me.anim, s.walking));
    // The chat log: x 5, newest line at the bottom, white.
    let bottom = f.h as i32 - 29;
    for (k, l) in s.chat.lines.iter().rev().enumerate() {
        text(f, 5, bottom - k as i32 * 16, &l.text, 0xffffff);
    }
    if let Some(t) = s.typing {
        let line = format!("> {t}_");
        fill(f, 0, f.h as i32 - 18, f.w as i32, 18, 0x202020);
        text(f, 5, f.h as i32 - 16, &line, 0xffff80);
    }
}

fn fill(f: &mut Frame, x: i32, y: i32, w: i32, h: i32, c: u32) {
    for yy in y.max(0)..(y + h).min(f.h as i32) {
        for xx in x.max(0)..(x + w).min(f.w as i32) {
            f.px[yy as usize * f.w + xx as usize] = c;
        }
    }
}

/// Text at 2× the 5×7 font, about the 14 px of the original.
pub fn text(f: &mut Frame, x: i32, y: i32, s: &str, c: u32) {
    for (i, ch) in s.chars().enumerate() {
        let g = font::glyph(ch);
        let gx = x + i as i32 * font::CELL_W * 2;
        if gx >= f.w as i32 {
            break;
        }
        for (r, bits) in g.iter().enumerate() {
            for col in 0..5 {
                if bits & (0x10 >> col) != 0 {
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (px, py) = (gx + col * 2 + dx, y + r as i32 * 2 + dy);
                        if px >= 0 && py >= 0 && (px as usize) < f.w && (py as usize) < f.h {
                            f.px[py as usize * f.w + px as usize] = c;
                        }
                    }
                }
            }
        }
    }
}
