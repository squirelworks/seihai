//! A first view of the Graal3D world (#461): the objects the UDP link
//! keeps, their `.3ds` shapes, the Sun's light and the Sky's colour, drawn
//! flat shaded with a small software rasteriser into the same RGBA frame
//! the flat view uses.
//!
//! Torque's axes: z is up. The view follows the control object from
//! behind and above it, or stands at the camera ghost when there is no
//! control object yet. `.dts`, `.dif` and `.ft` shapes, the Sky's
//! textures, terrain and nicknames are not drawn yet.

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
#[derive(Debug, Clone, PartialEq)]
pub struct Object3d {
    pub shape: Option<String>,
    pub transform: Option<Transform>,
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
        let o = self.objects.entry(index).or_insert(Object3d {
            shape: None,
            transform: None,
        });
        if let Some(s) = shape {
            o.shape = Some(String::from_utf8_lossy(s).trim().to_string());
        }
        if transform.is_some() {
            o.transform = transform;
        }
    }

    /// A shape file came in over TCP (`None`: the server had none).
    pub fn shape_file(&mut self, name: &str, data: Option<&[u8]>) {
        let model = data
            .filter(|_| name.to_ascii_lowercase().ends_with(".3ds"))
            .and_then(|d| Model::read(d).ok())
            .map(Arc::new);
        self.models.insert(name.to_ascii_lowercase(), model);
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
    let (sun_dir, sun_col, ambient) =
        world
            .sun
            .unwrap_or(([0.3, 0.4, -0.9], [0.8, 0.8, 0.8, 1.0], [0.3, 0.3, 0.3, 1.0]));
    let to_light = norm([-sun_dir[0], -sun_dir[1], -sun_dir[2]]);
    // Focal length for a 90-degree horizontal field of view.
    let focal = width as f32 / 2.0;
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
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
                let colour = [shade(0), shade(1), shade(2), 255];
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

    #[test]
    fn a_turn_about_z_maps_x_to_y() {
        let r = rotation([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let v = mul(&r, [1.0, 0.0, 0.0]);
        assert!(v[0].abs() < 1e-6 && (v[1] - 1.0).abs() < 1e-6);
    }
}
