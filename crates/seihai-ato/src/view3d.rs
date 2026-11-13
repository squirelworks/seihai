//! A first view of the Graal3D world (#461): the objects the UDP link
//! keeps, their `.3ds` shapes, the Sun's light and the Sky's colour, drawn
//! flat shaded with a small software rasteriser into the same RGBA frame
//! the flat view uses.
//!
//! Torque's axes: z is up. The view follows the control object from
//! behind and above it, or stands at the camera ghost when there is no
//! control object yet.
//!
//! Shapes: `.3ds` ([`crate::three_ds`]) and `.dts` ([`crate::dts`]).
//! `.dif` interiors, `.ft` shapes and terrain are not drawn yet.
//!
//! The Sky: its six textures as a box around the eye (front +y, right +x,
//! back -y, left -x, top +z, bottom -z), else its solid colour. Past half
//! its visible distance shapes fade into the fog colour, and past the
//! whole of it they are not drawn.
//!
//! Nicknames and chat go over each object's shape as [`Label`]s, which the
//! window writes with its own font.

use crate::three_ds::Model;
use seihai_play::render::Frame;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

/// A position, axis or colour.
pub type Vec3 = [f32; 3];

/// Where an object stands: position, then rotation as an axis and an angle
/// in radians (a ghost's transform section).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub axis: Vec3,
    pub angle: f32,
}

/// One object of the world, as last updated.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Object3d {
    pub shape: Option<String>,
    pub transform: Option<Transform>,
    /// The nickname and its colour (R, G, B, A).
    pub nickname: Option<(String, [u8; 4])>,
    /// What the object last said.
    pub chat: Option<String>,
}

/// A decoded texture, rows top first.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Texture {
    /// A PNG, GIF or BMP file, by its first bytes (the Sky's textures come
    /// without an extension). Other formats give `None`.
    pub fn decode(data: &[u8]) -> Option<Texture> {
        let (width, height, rgba) = if data.starts_with(b"\x89PNG") {
            seihai_core::assets::read_png_rgba(data).ok()?
        } else if data.starts_with(b"GIF8") {
            seihai_core::assets::gif::read_gif_rgba(data).ok()?
        } else if data.starts_with(b"BM") {
            let b = seihai_g3d::bmp::decode(data).ok()?;
            let rgba =
                b.px.iter()
                    .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8, 255])
                    .collect();
            (b.w as u32, b.h as u32, rgba)
        } else {
            return None;
        };
        (width > 0 && height > 0 && rgba.len() == (width * height * 4) as usize).then_some(
            Texture {
                width,
                height,
                rgba,
            },
        )
    }

    /// The colour at (u, v), each 0 to 1 from the top left, nearest.
    fn at(&self, u: f32, v: f32) -> [u8; 3] {
        let x = ((u.clamp(0.0, 1.0) * self.width as f32) as u32).min(self.width - 1);
        let y = ((v.clamp(0.0, 1.0) * self.height as f32) as u32).min(self.height - 1);
        let i = ((y * self.width + x) * 4) as usize;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]]
    }
}

/// A text over an object, in frame pixels: its bottom centre.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub colour: [u8; 4],
    /// Chat (a bubble), not a nickname.
    pub chat: bool,
}

/// What the link has of the 3D world, for the view.
#[derive(Debug, Clone, Default)]
pub struct World3d {
    pub objects: BTreeMap<u32, Object3d>,
    pub control: Option<u32>,
    pub camera: Option<Transform>,
    /// Direction, colour and ambient light.
    pub sun: Option<(Vec3, [f32; 4], [f32; 4])>,
    pub sky_colour: Option<Vec3>,
    pub fog_colour: Option<Vec3>,
    /// How far the Sky lets the eye see.
    pub visible_distance: Option<f32>,
    /// The Sky's textures as named: front, right, back, left, top, bottom
    /// (`-` or a missing one: none).
    pub sky_textures: Vec<String>,
    /// Images by lower-case name: decoded, or `None` when the file was
    /// not sent or is not an image this view reads.
    pub images: HashMap<String, Option<Arc<Texture>>>,
    /// Shapes by lower-case name: read, or `None` when the file was not
    /// sent or is not a shape this view reads.
    pub models: HashMap<String, Option<Arc<Model>>>,
}

/// The world, shared between the link's thread and the window.
pub type Shared3d = Arc<Mutex<World3d>>;

/// What the window hands the link: the keys held and lines to say (#462).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Input3d {
    /// 1 forward (up arrow), -1 back, 0 still.
    pub forward: i8,
    /// 1 turn left, -1 right, 0 none.
    pub turn: i8,
    /// Chat lines, sent as the own object's chat section (`e:emoticon`
    /// included, as typed).
    pub say: Vec<String>,
}

/// [`Input3d`], shared between the window and the link's thread.
pub type SharedInput3d = Arc<Mutex<Input3d>>;

/// Walking speed, units a second. Seihai's choice until the walking lab
/// trace (#462) shows the real client's.
pub const WALK_SPEED: f32 = 6.0;
/// Turning speed, radians a second (Seihai's choice, as above).
pub const TURN_SPEED: f32 = 2.5;

/// The heading of a transform: its rotation about z (0 faces +y).
fn heading(t: &Transform) -> f32 {
    if t.axis[2] < 0.0 {
        -t.angle
    } else {
        t.angle
    }
}

/// Move `t` for `dt` seconds of keys: turn about z, then step along the
/// new facing. The rotation comes out as a turn about +z.
pub fn step(t: Transform, forward: i8, turn: i8, dt: f32) -> Transform {
    let h = heading(&t) + turn as f32 * TURN_SPEED * dt;
    let d = forward as f32 * WALK_SPEED * dt;
    let (s, c) = h.sin_cos();
    Transform {
        position: [t.position[0] - s * d, t.position[1] + c * d, t.position[2]],
        axis: [0.0, 0.0, 1.0],
        angle: h,
    }
}

impl World3d {
    /// A GameObject was created or updated: its shape and transform, when
    /// the update carried them.
    pub fn object(&mut self, index: u32, shape: Option<&[u8]>, transform: Option<Transform>) {
        let o = self.objects.entry(index).or_default();
        if let Some(s) = shape {
            o.shape = Some(String::from_utf8_lossy(s).trim().to_string());
        }
        if transform.is_some() {
            o.transform = transform;
        }
    }

    /// An object's nickname and chat, when the update carried them.
    pub fn labels_of(
        &mut self,
        index: u32,
        nickname: Option<(&[u8], [u8; 4])>,
        chat: Option<&[u8]>,
    ) {
        let o = self.objects.entry(index).or_default();
        if let Some((n, c)) = nickname {
            o.nickname = Some((String::from_utf8_lossy(n).trim().to_string(), c));
        }
        if let Some(c) = chat {
            o.chat = Some(String::from_utf8_lossy(c).trim().to_string());
        }
    }

    /// A file came in over TCP: a shape, or an image (the Sky's textures,
    /// named without an extension).
    pub fn file(&mut self, name: &str, data: Option<&[u8]>) {
        let lower = name.to_ascii_lowercase();
        let shape = [".3ds", ".dts", ".dif", ".ft"]
            .iter()
            .any(|e| lower.ends_with(e));
        if shape {
            self.shape_file(name, data);
        } else {
            let t = data.and_then(Texture::decode).map(Arc::new);
            self.images.insert(lower, t);
        }
    }

    /// The Sky's texture for face `i` (front, right, back, left, top,
    /// bottom), when it is in.
    fn sky_face(&self, i: usize) -> Option<&Texture> {
        let name = self.sky_textures.get(i)?;
        self.images.get(&name.to_ascii_lowercase())?.as_deref()
    }

    /// A shape file came in over TCP (`None`: the server had none).
    /// `.3ds` and `.dts` shapes are read; `.dif` and `.ft` are kept as
    /// `None` (not drawn yet).
    pub fn shape_file(&mut self, name: &str, data: Option<&[u8]>) {
        let lower = name.to_ascii_lowercase();
        let model = data
            .and_then(|d| {
                if lower.ends_with(".3ds") {
                    Model::read(d).ok()
                } else if lower.ends_with(".dts") {
                    crate::dts::read(d).ok()
                } else {
                    None
                }
            })
            .map(Arc::new);
        self.models.insert(lower, model);
    }
}

type M3 = [[f32; 3]; 3];

/// Rotation about the unit `axis` by `angle` radians.
fn rotation(axis: Vec3, angle: f32) -> M3 {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-6 || angle == 0.0 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let [x, y, z] = [axis[0] / len, axis[1] / len, axis[2] / len];
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    [
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
    ]
}

fn mul(m: &M3, v: Vec3) -> Vec3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l < 1e-9 {
        a
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// Where the eye is and its right, up and forward axes.
struct Eye {
    at: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
}

impl Eye {
    fn looking(at: Vec3, target: Vec3) -> Eye {
        let forward = norm(sub(target, at));
        let mut right = cross(forward, [0.0, 0.0, 1.0]);
        if dot(right, right) < 1e-9 {
            right = [1.0, 0.0, 0.0];
        }
        let right = norm(right);
        let up = cross(right, forward);
        Eye {
            at,
            right,
            up,
            forward,
        }
    }

    /// The view: behind and above the control object, else the camera
    /// ghost looking along its rotation's +y, else from the south.
    fn of(world: &World3d) -> Eye {
        let control = world
            .control
            .and_then(|i| world.objects.get(&i))
            .and_then(|o| o.transform);
        if let Some(t) = control {
            let r = rotation(t.axis, t.angle);
            let back = mul(&r, [0.0, -8.0, 4.0]);
            return Eye::looking(add(t.position, back), add(t.position, [0.0, 0.0, 1.0]));
        }
        if let Some(t) = world.camera {
            let r = rotation(t.axis, t.angle);
            return Eye::looking(t.position, add(t.position, mul(&r, [0.0, 1.0, 0.0])));
        }
        Eye::looking([0.0, -10.0, 5.0], [0.0, 0.0, 0.0])
    }
}

const NEAR: f32 = 0.1;

fn rgb(c: Vec3) -> [u8; 3] {
    let f = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
    [f(c[0]), f(c[1]), f(c[2])]
}

/// Draw `world` into a `width` x `height` frame.
pub fn draw(world: &World3d, width: u32, height: u32) -> Frame {
    let sky = world
        .sky_colour
        .or(world.fog_colour)
        .map(rgb)
        .unwrap_or([96, 140, 200]);
    let mut frame = Frame::filled(width, height, [sky[0], sky[1], sky[2], 255]);
    if width == 0 || height == 0 {
        return frame;
    }
    // 1/z per pixel: larger is nearer, and it is linear on the screen.
    let mut depth = vec![0.0f32; (width * height) as usize];
    let eye = Eye::of(world);
    // Focal length for a 90-degree horizontal field of view.
    let focal = width as f32 / 2.0;
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    sky_box(world, &eye, &mut frame, focal);
    let fog = world.fog_colour.or(world.sky_colour).unwrap_or([
        96.0 / 255.0,
        140.0 / 255.0,
        200.0 / 255.0,
    ]);
    let far = world.visible_distance.filter(|d| *d > 0.0);
    let (sun_dir, sun_col, ambient) =
        world
            .sun
            .unwrap_or(([0.3, 0.4, -0.9], [0.8, 0.8, 0.8, 1.0], [0.3, 0.3, 0.3, 1.0]));
    let to_light = norm([-sun_dir[0], -sun_dir[1], -sun_dir[2]]);
    for o in world.objects.values() {
        let (Some(shape), Some(t)) = (&o.shape, o.transform) else {
            continue;
        };
        let Some(Some(model)) = world.models.get(&shape.to_ascii_lowercase()) else {
            continue;
        };
        let r = rotation(t.axis, t.angle);
        for mesh in &model.meshes {
            let world_v: Vec<Vec3> = mesh
                .vertices
                .iter()
                .map(|&v| add(mul(&r, v), t.position))
                .collect();
            for (f, col) in mesh.faces.iter().zip(&mesh.face_colours) {
                let p = f.map(|i| world_v[i as usize]);
                let n = norm(cross(sub(p[1], p[0]), sub(p[2], p[0])));
                // Both sides lit: .3ds winding is not reliable.
                let lit = dot(n, to_light).abs();
                let base = col.unwrap_or([180, 180, 180]);
                let shade = |k: usize| {
                    let l = ambient[k] + sun_col[k] * lit;
                    (base[k] as f32 * l.clamp(0.0, 1.0)) as u8
                };
                // Fog from half the visible distance; nothing past it.
                let mid = [
                    (p[0][0] + p[1][0] + p[2][0]) / 3.0,
                    (p[0][1] + p[1][1] + p[2][1]) / 3.0,
                    (p[0][2] + p[1][2] + p[2][2]) / 3.0,
                ];
                let mut colour = [shade(0), shade(1), shade(2), 255];
                if let Some(far) = far {
                    let d = dot(sub(mid, eye.at), sub(mid, eye.at)).sqrt();
                    if d > far {
                        continue;
                    }
                    let k = ((d - far / 2.0) / (far / 2.0)).clamp(0.0, 1.0);
                    for c in 0..3 {
                        let f = fog[c].clamp(0.0, 1.0) * 255.0;
                        colour[c] = (colour[c] as f32 * (1.0 - k) + f * k) as u8;
                    }
                }
                // Into eye space, cut at the near plane, then onto the
                // screen.
                let eye_v = p.map(|v| {
                    let d = sub(v, eye.at);
                    [dot(d, eye.right), dot(d, eye.up), dot(d, eye.forward)]
                });
                let poly = clip_near(&eye_v);
                let screen: Vec<[f32; 3]> = poly
                    .iter()
                    .map(|e| {
                        [
                            cx + e[0] * focal / e[2],
                            cy - e[1] * focal / e[2],
                            1.0 / e[2],
                        ]
                    })
                    .collect();
                for k in 1..screen.len().saturating_sub(1) {
                    triangle(
                        &mut frame,
                        &mut depth,
                        [screen[0], screen[k], screen[k + 1]],
                        colour,
                    );
                }
            }
        }
    }
    frame
}

/// The Sky's six textures around the eye, where they are in. A face whose
/// texture is missing keeps the colour the frame was filled with.
fn sky_box(world: &World3d, eye: &Eye, frame: &mut Frame, focal: f32) {
    let faces: Vec<Option<&Texture>> = (0..6).map(|i| world.sky_face(i)).collect();
    if faces.iter().all(Option::is_none) {
        return;
    }
    let (w, h) = (frame.width, frame.height);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = ((x as f32 + 0.5 - cx) / focal, (cy - y as f32 - 0.5) / focal);
            let d = add(
                add(
                    [eye.right[0] * sx, eye.right[1] * sx, eye.right[2] * sx],
                    [eye.up[0] * sy, eye.up[1] * sy, eye.up[2] * sy],
                ),
                eye.forward,
            );
            let (face, u, v) = sky_uv(d);
            if let Some(t) = faces[face] {
                let c = t.at(u, v);
                let i = ((y * w + x) * 4) as usize;
                frame.rgba[i..i + 3].copy_from_slice(&c);
            }
        }
    }
}

/// Which face of the sky box direction `d` meets (front +y, right +x,
/// back -y, left -x, top +z, bottom -z), and where on its texture: u
/// left to right and v top to bottom, as seen from inside the box, with
/// the sides' tops up and the top's and bottom's tops towards the front.
fn sky_uv(d: Vec3) -> (usize, f32, f32) {
    let [x, y, z] = d;
    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
    let half = |a: f32| (a + 1.0) / 2.0;
    if az >= ax && az >= ay {
        if z > 0.0 {
            (4, half(x / az), half(-y / az))
        } else {
            (5, half(x / az), half(y / az))
        }
    } else if ay >= ax {
        if y > 0.0 {
            (0, half(x / ay), half(-z / ay))
        } else {
            (2, half(-x / ay), half(-z / ay))
        }
    } else if x > 0.0 {
        (1, half(-y / ax), half(-z / ax))
    } else {
        (3, half(y / ax), half(-z / ax))
    }
}

/// The nicknames and chat over the objects in view, in frame pixels: each
/// over the top of the object's shape (or 2 units over its place when the
/// shape is not in), the chat above the nickname.
pub fn labels(world: &World3d, width: u32, height: u32) -> Vec<Label> {
    let mut out = Vec::new();
    if width == 0 || height == 0 {
        return out;
    }
    let eye = Eye::of(world);
    let focal = width as f32 / 2.0;
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    let far = world.visible_distance.filter(|d| *d > 0.0);
    for o in world.objects.values() {
        let Some(t) = o.transform else { continue };
        let named = o.nickname.as_ref().filter(|(n, _)| !n.is_empty());
        let said = o.chat.as_ref().filter(|c| !c.is_empty());
        if named.is_none() && said.is_none() {
            continue;
        }
        let top = o
            .shape
            .as_ref()
            .and_then(|s| world.models.get(&s.to_ascii_lowercase()))
            .and_then(|m| m.as_ref())
            .and_then(|m| {
                let r = rotation(t.axis, t.angle);
                m.meshes
                    .iter()
                    .flat_map(|mesh| mesh.vertices.iter())
                    .map(|&v| mul(&r, v)[2])
                    .reduce(f32::max)
            })
            .unwrap_or(2.0);
        let at = add(t.position, [0.0, 0.0, top + 0.3]);
        let d = sub(at, eye.at);
        let z = dot(d, eye.forward);
        if z < NEAR || far.is_some_and(|f| dot(d, d).sqrt() > f) {
            continue;
        }
        let x = cx + dot(d, eye.right) * focal / z;
        let y = cy - dot(d, eye.up) * focal / z;
        if x < 0.0 || x >= width as f32 || y < 0.0 || y >= height as f32 {
            continue;
        }
        let mut y_text = y;
        if let Some((n, c)) = named {
            out.push(Label {
                text: n.clone(),
                x,
                y,
                colour: *c,
                chat: false,
            });
            y_text -= 14.0;
        }
        if let Some(c) = said {
            out.push(Label {
                text: c.clone(),
                x,
                y: y_text,
                colour: [255, 255, 255, 255],
                chat: true,
            });
        }
    }
    out
}

/// The part of an eye-space triangle (right, up, forward) in front of the
/// near plane: none, a triangle or a quad, as a fan.
fn clip_near(t: &[Vec3; 3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(4);
    for i in 0..3 {
        let (a, b) = (t[i], t[(i + 1) % 3]);
        let (ina, inb) = (a[2] >= NEAR, b[2] >= NEAR);
        if ina {
            out.push(a);
        }
        if ina != inb {
            let k = (NEAR - a[2]) / (b[2] - a[2]);
            out.push([a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, NEAR]);
        }
    }
    out
}

/// Fill one screen triangle (x, y, 1/z) where it is nearer than what is
/// drawn.
fn triangle(frame: &mut Frame, depth: &mut [f32], s: [[f32; 3]; 3], colour: [u8; 4]) {
    let (w, h) = (frame.width as i32, frame.height as i32);
    let area =
        (s[1][0] - s[0][0]) * (s[2][1] - s[0][1]) - (s[2][0] - s[0][0]) * (s[1][1] - s[0][1]);
    if area.abs() < 1e-6 {
        return;
    }
    let min_x = s
        .iter()
        .map(|p| p[0])
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_x = s
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min((w - 1) as f32) as i32;
    let min_y = s
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_y = s
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min((h - 1) as f32) as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let edge = |a: [f32; 3], b: [f32; 3]| {
                (b[0] - a[0]) * (py - a[1]) - (b[1] - a[1]) * (px - a[0])
            };
            let w0 = edge(s[1], s[2]) / area;
            let w1 = edge(s[2], s[0]) / area;
            let w2 = edge(s[0], s[1]) / area;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let z = w0 * s[0][2] + w1 * s[1][2] + w2 * s[2][2];
            let i = (y * w + x) as usize;
            if z > depth[i] {
                depth[i] = z;
                frame.rgba[i * 4..i * 4 + 4].copy_from_slice(&colour);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::three_ds::write;

    fn cube() -> Vec<u8> {
        let v = [
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 2.0],
            [1.0, -1.0, 2.0],
            [1.0, 1.0, 2.0],
            [-1.0, 1.0, 2.0],
        ];
        let f = [
            [0, 1, 5],
            [0, 5, 4],
            [1, 2, 6],
            [1, 6, 5],
            [2, 3, 7],
            [2, 7, 6],
            [3, 0, 4],
            [3, 4, 7],
            [4, 5, 6],
            [4, 6, 7],
            [0, 2, 1],
            [0, 3, 2],
        ];
        write::mesh("cube", &v, &f, [220, 40, 40])
    }

    fn world() -> World3d {
        let mut w = World3d::default();
        w.object(
            1,
            Some(b"labcube.3ds"),
            Some(Transform {
                position: [0.0, 0.0, 0.0],
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
            }),
        );
        w.control = Some(1);
        w.shape_file("labcube.3ds", Some(&cube()));
        w
    }

    fn pixel(f: &Frame, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * f.width + x) * 4) as usize;
        f.rgba[i..i + 4].try_into().unwrap()
    }

    #[test]
    fn the_control_objects_shape_is_drawn_in_the_middle_over_the_sky() {
        let mut w = world();
        w.sky_colour = Some([0.0, 0.0, 1.0]);
        let f = draw(&w, 160, 120);
        assert_eq!(pixel(&f, 2, 2), [0, 0, 255, 255], "sky in the corner");
        let mid = pixel(&f, 80, 60);
        assert_ne!(mid, [0, 0, 255, 255], "the cube in the middle");
        assert!(mid[0] > mid[1] && mid[0] > mid[2], "red, shaded: {mid:?}");
    }

    #[test]
    fn a_shape_not_yet_in_is_not_drawn() {
        let mut w = world();
        w.models.clear();
        let f = draw(&w, 64, 48);
        assert!(f.rgba.chunks(4).all(|p| p == [96, 140, 200, 255]));
        w.shape_file("labcube.3ds", None);
        assert_eq!(draw(&w, 64, 48), f);
    }

    /// A ground slab that reaches behind the eye is cut at the near plane
    /// and still fills the bottom of the view.
    #[test]
    fn ground_reaching_behind_the_eye_is_still_drawn() {
        let mut w = world();
        let g = [
            [-40.0, -40.0, 0.0],
            [40.0, -40.0, 0.0],
            [40.0, 40.0, 0.0],
            [-40.0, 40.0, 0.0],
        ];
        w.shape_file(
            "ground.3ds",
            Some(&write::mesh("g", &g, &[[0, 1, 2], [0, 2, 3]], [0, 200, 0])),
        );
        w.object(
            0,
            Some(b"ground.3ds"),
            Some(Transform {
                position: [0.0; 3],
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
            }),
        );
        let f = draw(&w, 160, 120);
        let p = pixel(&f, 5, 115);
        assert!(
            p[1] > p[0] && p[1] > p[2],
            "green ground at the bottom: {p:?}"
        );
    }

    #[test]
    fn walking_steps_along_the_facing_and_turning_turns() {
        let t = Transform {
            position: [0.0; 3],
            axis: [0.0, 0.0, 1.0],
            angle: 0.0,
        };
        let a = step(t, 1, 0, 0.5);
        assert!((a.position[1] - WALK_SPEED * 0.5).abs() < 1e-5 && a.position[0].abs() < 1e-5);
        let quarter = std::f32::consts::FRAC_PI_2 / TURN_SPEED;
        let b = step(t, 0, 1, quarter);
        assert!((b.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(b.position, [0.0; 3]);
        // A left quarter turn from +y faces -x.
        let c = step(b, 1, 0, 1.0);
        assert!(
            (c.position[0] + WALK_SPEED).abs() < 1e-4,
            "{:?}",
            c.position
        );
    }

    fn red_png() -> Vec<u8> {
        let mut out = Vec::new();
        seihai_core::assets::write_png_rgb(&mut out, 2, 2, &[255, 0, 0].repeat(4)).unwrap();
        out
    }

    #[test]
    fn images_decode_by_their_first_bytes() {
        let t = Texture::decode(&red_png()).unwrap();
        assert_eq!((t.width, t.height), (2, 2));
        assert_eq!(t.at(0.9, 0.9), [255, 0, 0]);
        let bmp = seihai_g3d::bmp::encode(1, 1, &[0x00_10_20_30]);
        assert_eq!(
            Texture::decode(&bmp).unwrap().at(0.0, 0.0),
            [0x10, 0x20, 0x30]
        );
        assert!(Texture::decode(b"not an image").is_none());
    }

    #[test]
    fn files_go_to_shapes_or_images_by_name() {
        let mut w = World3d::default();
        w.file("Sky_FR", Some(&red_png()));
        w.file("labcube.3ds", Some(&cube()));
        w.file("tree.dif", Some(b"interior"));
        assert!(w.images["sky_fr"].is_some());
        assert!(w.models["labcube.3ds"].is_some());
        assert_eq!(w.models.get("tree.dif"), Some(&None), "kept, not drawn yet");
    }

    #[test]
    fn a_dts_shape_is_drawn() {
        let mut w = world();
        w.shape_file(
            "octahedron.dts",
            Some(include_bytes!("../tests/fixtures/dts/octahedron.dts")),
        );
        // Its middle where the view looks: 1 over the control object.
        w.models.remove("labcube.3ds");
        w.object(
            2,
            Some(b"octahedron.dts"),
            Some(Transform {
                position: [0.0, 0.0, 1.0],
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
            }),
        );
        w.sky_colour = Some([0.0, 0.0, 1.0]);
        let f = draw(&w, 160, 120);
        assert_ne!(
            pixel(&f, 80, 60),
            [0, 0, 255, 255],
            "the shape in the middle"
        );
    }

    #[test]
    fn the_sky_box_faces_are_picked_by_direction() {
        assert_eq!(sky_uv([0.0, 1.0, 0.0]), (0, 0.5, 0.5));
        assert_eq!(sky_uv([1.0, 0.0, 0.0]).0, 1);
        assert_eq!(sky_uv([0.0, -1.0, 0.0]).0, 2);
        assert_eq!(sky_uv([-1.0, 0.0, 0.0]).0, 3);
        assert_eq!(sky_uv([0.0, 0.0, 1.0]).0, 4);
        assert_eq!(sky_uv([0.0, 0.0, -1.0]).0, 5);
        // Up the front face is up the texture; right is right.
        let (_, u, v) = sky_uv([0.5, 1.0, 0.5]);
        assert!(u > 0.5 && v < 0.5);
    }

    #[test]
    fn the_sky_box_is_drawn_behind_the_shapes() {
        let mut w = world();
        w.sky_textures = ["fr", "rt", "bk", "lf", "up", "dn"]
            .map(String::from)
            .to_vec();
        for n in ["fr", "rt", "bk", "lf", "up", "dn"] {
            w.file(n, Some(&red_png()));
        }
        let f = draw(&w, 160, 120);
        assert_eq!(
            pixel(&f, 2, 2),
            [255, 0, 0, 255],
            "sky texture in the corner"
        );
        let mid = pixel(&f, 80, 60);
        assert_ne!(mid, [255, 0, 0, 255], "the cube over it");
    }

    #[test]
    fn shapes_fade_into_the_fog_and_go_past_the_visible_distance() {
        let mut w = world();
        w.sky_colour = Some([0.0, 0.0, 0.0]);
        w.fog_colour = Some([1.0, 1.0, 1.0]);
        // The eye is about 8.9 from the cube.
        w.visible_distance = Some(100.0);
        let near = pixel(&draw(&w, 160, 120), 80, 60);
        w.visible_distance = Some(10.0);
        let fogged = pixel(&draw(&w, 160, 120), 80, 60);
        assert!(fogged[1] > near[1] + 40, "greyer: {near:?} {fogged:?}");
        w.visible_distance = Some(5.0);
        assert_eq!(pixel(&draw(&w, 160, 120), 80, 60), [0, 0, 0, 255]);
    }

    #[test]
    fn nicknames_and_chat_go_over_the_objects() {
        let mut w = world();
        w.labels_of(1, Some((b"Alice ", [255, 0, 0, 255])), Some(b"hi"));
        let l = labels(&w, 160, 120);
        assert_eq!(l.len(), 2);
        assert_eq!(
            (l[0].text.as_str(), l[0].colour, l[0].chat),
            ("Alice", [255, 0, 0, 255], false)
        );
        assert!((l[0].x - 80.0).abs() < 1.0, "centred over the cube");
        assert!(l[0].y < 60.0, "over the middle of the view");
        assert!(l[1].chat && l[1].y < l[0].y, "chat over the nickname");
        // Behind the eye: none.
        w.camera = None;
        w.control = None;
        w.objects.get_mut(&1).unwrap().transform = Some(Transform {
            position: [0.0, -20.0, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle: 0.0,
        });
        assert!(labels(&w, 160, 120).is_empty());
    }

    #[test]
    fn a_turn_about_z_maps_x_to_y() {
        let r = rotation([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let v = mul(&r, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-6 && (v[1] - 1.0).abs() < 1e-6);
    }
}
