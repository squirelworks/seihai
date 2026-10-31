//! A small software renderer for Graal3D's meshes: perspective camera,
//! near-plane clipping, a depth buffer, perspective-correct textures
//! (nearest, wrapped) and one directional light.
//!
//! This stands in for Direct3D Retained Mode, which the real client used.
//! It is not a copy of D3DRM's shading.

use crate::bmp::Image;
use crate::xfile::Mesh;
use std::collections::HashMap;

pub type V3 = [f32; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn norm(a: V3) -> V3 {
    let l = dot(a, a).sqrt();
    if l > 1e-9 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u32>,
    /// `1 / view z` per pixel; bigger is nearer.
    depth: Vec<f32>,
}

impl Frame {
    pub fn new(w: usize, h: usize) -> Frame {
        Frame { w, h, px: vec![0; w * h], depth: vec![0.0; w * h] }
    }

    pub fn clear(&mut self, colour: u32) {
        self.px.fill(colour);
        self.depth.fill(0.0);
    }
}

/// A Direct3D-style camera: left-handed, y up, looking along +z in view
/// space.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub pos: V3,
    /// Unit vectors of the view.
    right: V3,
    up: V3,
    fwd: V3,
    /// Focal length in pixels.
    focal: f32,
}

pub const NEAR: f32 = 2.0;

impl Camera {
    /// A camera at `pos` looking at `target`, with a horizontal field of
    /// view of `fov_deg` on a frame `w` pixels wide.
    pub fn look_at(pos: V3, target: V3, fov_deg: f32, w: usize) -> Camera {
        let fwd = norm(sub(target, pos));
        let right = norm(cross([0.0, 1.0, 0.0], fwd));
        let up = cross(fwd, right);
        let focal = (w as f32 / 2.0) / (fov_deg.to_radians() / 2.0).tan();
        Camera { pos, right, up, fwd, focal }
    }

    fn view_of(&self, p: V3) -> V3 {
        let d = sub(p, self.pos);
        [dot(d, self.right), dot(d, self.up), dot(d, self.fwd)]
    }
}

/// A 3×4 transform: rotation (rows) and translation (last column).
pub type Mat = [[f32; 4]; 3];

pub const IDENTITY: Mat = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];

pub fn transform(m: &Mat, v: V3) -> V3 {
    let r = |k: usize| m[k][0] * v[0] + m[k][1] * v[1] + m[k][2] * v[2] + m[k][3];
    [r(0), r(1), r(2)]
}

/// `a` after `b`: first `b`, then `a`.
pub fn compose(a: &Mat, b: &Mat) -> Mat {
    let mut out = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + if j == 3 { a[i][3] } else { 0.0 };
        }
    }
    out
}

/// A turn about y by `deg` (+z towards +x), then a move to `at`.
pub fn yaw_at(deg: f32, at: V3) -> Mat {
    let (s, c) = deg.to_radians().sin_cos();
    [[c, 0.0, s, at[0]], [0.0, 1.0, 0.0, at[1]], [-s, 0.0, c, at[2]]]
}

/// Where and how to draw one mesh.
pub struct Placement {
    pub m: Mat,
    /// Replaces every material's colour (the player colour props).
    pub tint: Option<u32>,
}

impl Placement {
    pub fn world() -> Placement {
        Placement { m: IDENTITY, tint: None }
    }

    fn apply(&self, v: V3) -> V3 {
        transform(&self.m, v)
    }
}

#[derive(Clone, Copy)]
struct Vert {
    v: V3,
    uv: [f32; 2],
}

fn lerp(a: Vert, b: Vert, t: f32) -> Vert {
    Vert {
        v: [a.v[0] + (b.v[0] - a.v[0]) * t, a.v[1] + (b.v[1] - a.v[1]) * t, a.v[2] + (b.v[2] - a.v[2]) * t],
        uv: [a.uv[0] + (b.uv[0] - a.uv[0]) * t, a.uv[1] + (b.uv[1] - a.uv[1]) * t],
    }
}

/// Clip a view-space polygon to `z >= NEAR`.
fn clip_near(poly: &[Vert]) -> Vec<Vert> {
    let mut out = Vec::with_capacity(poly.len() + 2);
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let (ina, inb) = (a.v[2] >= NEAR, b.v[2] >= NEAR);
        if ina {
            out.push(a);
        }
        if ina != inb {
            out.push(lerp(a, b, (NEAR - a.v[2]) / (b.v[2] - a.v[2])));
        }
    }
    out
}

fn shade(c: u32, k: f32) -> u32 {
    let ch = |s: u32| ((((c >> s) & 255) as f32 * k).min(255.0) as u32) << s;
    ch(16) | ch(8) | ch(0)
}

fn rgb(c: [f32; 4]) -> u32 {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u32;
    ch(c[0]) << 16 | ch(c[1]) << 8 | ch(c[2])
}

/// The light: from above and a little to the side.
const LIGHT: V3 = [0.36, 0.86, -0.36];

/// Draw `mesh` into `f`. `textures` maps a material's texture file name
/// (lower case) to its image; missing textures draw in the material
/// colour.
pub fn draw_mesh(f: &mut Frame, cam: &Camera, mesh: &Mesh, place: &Placement, textures: &HashMap<String, Image>) {
    let (cx, cy) = (f.w as f32 / 2.0, f.h as f32 / 2.0);
    for (tri, mat) in mesh.triangles() {
        let world: Vec<V3> = tri.iter().map(|&i| place.apply(mesh.vertices[i as usize])).collect();
        // Direct3D faces are clockwise from the front; draw both sides,
        // lit by the face normal facing the camera.
        let mut n = norm(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        if dot(n, sub(cam.pos, world[0])) > 0.0 {
            n = [-n[0], -n[1], -n[2]];
        }
        let k = 0.45 + 0.55 * dot(n, LIGHT).abs();
        let m = mesh.materials.get(mat as usize);
        let tex = m
            .and_then(|m| m.texture.as_ref())
            .and_then(|t| textures.get(&t.to_ascii_lowercase()))
            .filter(|_| place.tint.is_none());
        let base = place.tint.unwrap_or_else(|| m.map(|m| rgb(m.colour)).unwrap_or(0xc0c0c0));
        let poly: Vec<Vert> = tri
            .iter()
            .zip(&world)
            .map(|(&i, &w)| Vert { v: cam.view_of(w), uv: mesh.uvs.get(i as usize).copied().unwrap_or([0.0, 0.0]) })
            .collect();
        let poly = clip_near(&poly);
        if poly.len() < 3 {
            continue;
        }
        // Screen x, y, 1/z, u/z, v/z.
        let s: Vec<[f32; 5]> = poly
            .iter()
            .map(|p| {
                let iz = 1.0 / p.v[2];
                [cx + p.v[0] * cam.focal * iz, cy - p.v[1] * cam.focal * iz, iz, p.uv[0] * iz, p.uv[1] * iz]
            })
            .collect();
        for k2 in 1..s.len() - 1 {
            raster(f, [s[0], s[k2], s[k2 + 1]], tex, base, k);
        }
    }
}

fn raster(f: &mut Frame, t: [[f32; 5]; 3], tex: Option<&Image>, base: u32, light: f32) {
    let area = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[2][0] - t[0][0]) * (t[1][1] - t[0][1]);
    if area.abs() < 1e-6 {
        return;
    }
    let min_x = t.iter().map(|p| p[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_x = t.iter().map(|p| p[0]).fold(f32::MIN, f32::max).ceil().min(f.w as f32 - 1.0);
    let min_y = t.iter().map(|p| p[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_y = t.iter().map(|p| p[1]).fold(f32::MIN, f32::max).ceil().min(f.h as f32 - 1.0);
    if max_x < 0.0 || max_y < 0.0 {
        return;
    }
    let (max_x, max_y) = (max_x as usize, max_y as usize);
    let flat = shade(base, light);
    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let w0 = ((t[1][0] - px) * (t[2][1] - py) - (t[2][0] - px) * (t[1][1] - py)) / area;
            let w1 = ((t[2][0] - px) * (t[0][1] - py) - (t[0][0] - px) * (t[2][1] - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let iz = w0 * t[0][2] + w1 * t[1][2] + w2 * t[2][2];
            let i = y * f.w + x;
            if iz <= f.depth[i] {
                continue;
            }
            f.depth[i] = iz;
            f.px[i] = match tex {
                Some(img) => {
                    let u = (w0 * t[0][3] + w1 * t[1][3] + w2 * t[2][3]) / iz;
                    let v = (w0 * t[0][4] + w1 * t[1][4] + w2 * t[2][4]) / iz;
                    let tx = ((u - u.floor()) * img.w as f32) as usize % img.w;
                    let ty = ((v - v.floor()) * img.h as f32) as usize % img.h;
                    shade(img.px[ty * img.w + tx], light)
                }
                None => flat,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad() -> Mesh {
        Mesh {
            vertices: vec![[-50.0, 0.0, -50.0], [50.0, 0.0, -50.0], [50.0, 0.0, 50.0], [-50.0, 0.0, 50.0]],
            faces: vec![vec![0, 1, 2, 3]],
            ..Mesh::default()
        }
    }

    #[test]
    fn draws_a_floor_in_front() {
        let mut f = Frame::new(64, 48);
        f.clear(0);
        let cam = Camera::look_at([0.0, 40.0, -120.0], [0.0, 0.0, 0.0], 60.0, 64);
        draw_mesh(&mut f, &cam, &quad(), &Placement::world(), &HashMap::new());
        // The middle of the frame is floor; the top row is sky.
        assert_ne!(f.px[24 * 64 + 32], 0);
        assert_eq!(f.px[0], 0);
    }

    #[test]
    fn nearer_wins_and_behind_is_clipped() {
        let mut f = Frame::new(32, 32);
        f.clear(0);
        let cam = Camera::look_at([0.0, 200.0, 0.0], [0.0, 0.0, 0.01], 60.0, 32);
        let far = Placement { m: IDENTITY, tint: Some(0x0000ff) };
        let near = Placement { m: yaw_at(0.0, [0.0, 50.0, 0.0]), tint: Some(0xff0000) };
        draw_mesh(&mut f, &cam, &quad(), &near, &HashMap::new());
        draw_mesh(&mut f, &cam, &quad(), &far, &HashMap::new());
        let c = f.px[16 * 32 + 16];
        assert!(c >> 16 > 0 && c & 0xff == 0, "{c:06x}");
        // A camera inside the quad's plane sees nothing and doesn't panic.
        let v = transform(&compose(&yaw_at(90.0, [1.0, 0.0, 0.0]), &yaw_at(0.0, [0.0, 2.0, 0.0])), [0.0, 0.0, 1.0]);
        assert!((v[0] - 2.0).abs() < 1e-5 && (v[1] - 2.0).abs() < 1e-5 && v[2].abs() < 1e-5, "{v:?}");
        let cam = Camera::look_at([0.0, 0.0, -200.0], [0.0, 0.0, 0.0], 60.0, 32);
        draw_mesh(&mut f, &cam, &quad(), &near, &HashMap::new());
    }
}
