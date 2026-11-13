//! A reader for Torque `.dts` shapes (#461), versions 23 to 25: the
//! versions Graal3D's engine reads (24 or older, `graal3d/torque-engine-4x.md`
//! in Graal_analysis) from the point where skins sit with the other meshes.
//!
//! The layout is the public Torque shape format. After a 32-bit version
//! (the low byte; the exporter's version is in the high half) come the
//! sizes of three buffers, in 32-bit words: the whole buffer, where its
//! 16-bit part starts and where its 8-bit part starts. The shape is read
//! from the three parts side by side, each with its own cursor, and every
//! section ends with a guard: one value from each part, counting up from 0.
//! After the buffer come the sequences and the material list.
//!
//! Only what the view draws is kept: the first detail level that is not a
//! billboard, each of its objects' meshes at rest (frame 0) placed by the
//! nodes' default transforms, as triangles. Skinned meshes are drawn from
//! their initial vertices, which are already in shape space. Animation,
//! textures and the other detail levels are not read.

use crate::three_ds::{Mesh, Model};

/// Why a shape could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not a version this reader takes (23 to 25).
    Version(u32),
    /// The file ends before its buffer does, or a part runs out.
    Short(&'static str),
    /// A guard did not hold the next count: the layout went wrong here.
    Guard(&'static str),
    /// A mesh type this reader does not take (decals are deprecated).
    MeshType(i32),
    /// A count or an index out of range.
    Bad(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Version(v) => write!(f, "dts version {v} (this reader takes 23 to 25)"),
            Error::Short(at) => write!(f, "dts ends early ({at})"),
            Error::Guard(at) => write!(f, "dts guard does not match ({at})"),
            Error::MeshType(t) => write!(f, "dts mesh type {t} not read"),
            Error::Bad(what) => write!(f, "dts: bad {what}"),
        }
    }
}

const FIRST_VERSION: u32 = 23;
const LAST_VERSION: u32 = 25;

/// The three parts of the shape's buffer, each read from the front.
struct Parts<'a> {
    p32: &'a [u8],
    p16: &'a [u8],
    p8: &'a [u8],
    guard: (i32, i16, i8),
}

impl<'a> Parts<'a> {
    fn take(part: &mut &'a [u8], n: usize, at: &'static str) -> Result<&'a [u8], Error> {
        if part.len() < n {
            return Err(Error::Short(at));
        }
        let (a, b) = part.split_at(n);
        *part = b;
        Ok(a)
    }

    fn i32(&mut self, at: &'static str) -> Result<i32, Error> {
        let b = Self::take(&mut self.p32, 4, at)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn f32(&mut self, at: &'static str) -> Result<f32, Error> {
        Ok(f32::from_bits(self.i32(at)? as u32))
    }

    fn i16(&mut self, at: &'static str) -> Result<i16, Error> {
        let b = Self::take(&mut self.p16, 2, at)?;
        Ok(i16::from_le_bytes([b[0], b[1]]))
    }

    /// A count: not negative, and not more than the 32-bit part could hold.
    fn count(&mut self, at: &'static str) -> Result<usize, Error> {
        let n = self.i32(at)?;
        if n < 0 || n as usize > self.p32.len() + self.p16.len() + self.p8.len() {
            return Err(Error::Bad(at));
        }
        Ok(n as usize)
    }

    fn skip32(&mut self, n: usize, at: &'static str) -> Result<(), Error> {
        Self::take(&mut self.p32, n.checked_mul(4).ok_or(Error::Bad(at))?, at).map(|_| ())
    }

    fn skip16(&mut self, n: usize, at: &'static str) -> Result<(), Error> {
        Self::take(&mut self.p16, n.checked_mul(2).ok_or(Error::Bad(at))?, at).map(|_| ())
    }

    fn skip8(&mut self, n: usize, at: &'static str) -> Result<(), Error> {
        Self::take(&mut self.p8, n, at).map(|_| ())
    }

    fn i32s(&mut self, n: usize, at: &'static str) -> Result<Vec<i32>, Error> {
        (0..n).map(|_| self.i32(at)).collect()
    }

    fn f32s(&mut self, n: usize, at: &'static str) -> Result<Vec<f32>, Error> {
        (0..n).map(|_| self.f32(at)).collect()
    }

    fn i16s(&mut self, n: usize, at: &'static str) -> Result<Vec<i16>, Error> {
        (0..n).map(|_| self.i16(at)).collect()
    }

    /// The end of a section: the next count in each part.
    fn guard(&mut self, at: &'static str) -> Result<(), Error> {
        let g32 = self.i32(at)?;
        let g16 = self.i16(at)?;
        let g8 = Self::take(&mut self.p8, 1, at)?[0] as i8;
        if (g32, g16, g8) != self.guard {
            return Err(Error::Guard(at));
        }
        self.guard = (
            self.guard.0.wrapping_add(1),
            self.guard.1.wrapping_add(1),
            self.guard.2.wrapping_add(1),
        );
        Ok(())
    }
}

/// What a mesh holds for drawing.
#[derive(Debug, Clone, Default)]
struct RawMesh {
    /// Vertices of every frame; frame 0 comes first.
    verts: Vec<[f32; 3]>,
    /// Vertices per frame (0: all of them).
    verts_per_frame: usize,
    /// Triangles as vertex indices, with each one's material.
    triangles: Vec<([u32; 3], Option<u32>)>,
    /// A skin: its vertices are already in shape space.
    skin: bool,
}

const STANDARD: i32 = 0;
const SKIN: i32 = 1;
const SORTED: i32 = 3;
const NULL: i32 = 4;

/// Primitive flags: the element type in bits 30–31, no material in bit 28.
const TYPE_MASK: u32 = 3 << 30;
const STRIP: u32 = 1 << 30;
const FAN: u32 = 2 << 30;
const NO_MATERIAL: u32 = 1 << 28;
const MATERIAL_MASK: u32 = (1 << 28) - 1;

/// The triangles of one primitive (`start`, `count` elements of `indices`).
fn primitive(
    indices: &[i32],
    start: usize,
    count: usize,
    flags: u32,
    out: &mut Vec<([u32; 3], Option<u32>)>,
) -> Result<(), Error> {
    let el = indices
        .get(start..start + count)
        .ok_or(Error::Bad("primitive range"))?;
    let el: Vec<u32> = el.iter().map(|&i| i as u16 as u32).collect();
    let material = (flags & NO_MATERIAL == 0).then_some(flags & MATERIAL_MASK);
    match flags & TYPE_MASK {
        STRIP => {
            for k in 2..el.len() {
                let t = if k % 2 == 0 {
                    [el[k - 2], el[k - 1], el[k]]
                } else {
                    [el[k - 1], el[k - 2], el[k]]
                };
                out.push((t, material));
            }
        }
        FAN => {
            for k in 2..el.len() {
                out.push(([el[0], el[k - 1], el[k]], material));
            }
        }
        _ => {
            for t in el.chunks_exact(3) {
                out.push(([t[0], t[1], t[2]], material));
            }
        }
    }
    Ok(())
}

fn vec3s(f: Vec<f32>) -> Vec<[f32; 3]> {
    f.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

/// One mesh's data, its own or (for a mesh with a parent) its parent's.
fn mesh(p: &mut Parts<'_>, kind: i32, done: &[Option<RawMesh>]) -> Result<RawMesh, Error> {
    p.guard("mesh start")?;
    p.skip32(2, "mesh frames")?;
    let parent = p.i32("mesh parent")?;
    p.skip32(6 + 3 + 1, "mesh bounds")?;
    let own = parent < 0;
    let parent_mesh = if own {
        None
    } else {
        Some(
            done.get(parent as usize)
                .and_then(Option::as_ref)
                .ok_or(Error::Bad("mesh parent"))?,
        )
    };
    let n = p.count("mesh vertices")?;
    let mut verts = if own {
        vec3s(p.f32s(3 * n, "mesh vertices")?)
    } else {
        parent_mesh.map(|m| m.verts.clone()).unwrap_or_default()
    };
    let tv = p.count("mesh texture vertices")?;
    if own {
        p.skip32(2 * tv, "mesh texture vertices")?;
        // Normals, then the encoded normals (one byte each).
        p.skip32(3 * n, "mesh normals")?;
        p.skip8(n, "mesh encoded normals")?;
    }
    let prims = p.count("mesh primitives")?;
    let starts = p.i16s(2 * prims, "mesh primitives")?;
    let flags = p.i32s(prims, "mesh primitive materials")?;
    let ni = p.count("mesh indices")?;
    let indices: Vec<i32> = p
        .i16s(ni, "mesh indices")?
        .into_iter()
        .map(i32::from)
        .collect();
    let merge = p.count("mesh merge indices")?;
    p.skip16(merge, "mesh merge indices")?;
    let verts_per_frame = p.i32("mesh vertices per frame")?.max(0) as usize;
    p.skip32(1, "mesh flags")?;
    p.guard("mesh end")?;

    let mut triangles = Vec::new();
    for (k, &f) in flags.iter().enumerate() {
        let start = starts[2 * k] as u16 as usize;
        let count = starts[2 * k + 1] as u16 as usize;
        primitive(&indices, start, count, f as u32, &mut triangles)?;
    }

    let mut skin = false;
    match kind {
        SKIN => {
            skin = true;
            let n = p.count("skin vertices")?;
            if own {
                let initial = vec3s(p.f32s(3 * n, "skin vertices")?);
                p.skip32(3 * n, "skin normals")?;
                p.skip8(n, "skin encoded normals")?;
                if !initial.is_empty() {
                    verts = initial;
                }
            }
            let transforms = p.count("skin transforms")?;
            if own {
                p.skip32(16 * transforms, "skin transforms")?;
            }
            let weights = p.count("skin weights")?;
            if own {
                p.skip32(3 * weights, "skin weights")?;
            }
            let nodes = p.count("skin nodes")?;
            if own {
                p.skip32(nodes, "skin nodes")?;
            }
            p.guard("skin end")?;
        }
        SORTED => {
            let clusters = p.count("sorted clusters")?;
            p.skip32(8 * clusters, "sorted clusters")?;
            for _ in 0..4 {
                let n = p.count("sorted lists")?;
                p.skip32(n, "sorted lists")?;
            }
            p.skip32(1, "sorted depth")?;
            p.guard("sorted end")?;
        }
        _ => {}
    }
    Ok(RawMesh {
        verts,
        verts_per_frame,
        triangles,
        skin,
    })
}

/// A rotation as Torque stores it: x, y, z, w in 1/32767ths.
fn quat16(q: &[i16]) -> [[f32; 3]; 3] {
    let s = |v: i16| v as f32 / 32767.0;
    let (x, y, z, w) = (s(q[0]), s(q[1]), s(q[2]), s(q[3]));
    if x * x + y * y + z * z < 1e-12 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    // Torque turns points by the conjugate of the quaternion as usually
    // written: this is the transpose of the common matrix.
    let (xs, ys, zs) = (2.0 * x, 2.0 * y, 2.0 * z);
    let (wx, wy, wz) = (w * xs, w * ys, w * zs);
    let (xx, xy, xz) = (x * xs, x * ys, x * zs);
    let (yy, yz, zz) = (y * ys, y * zs, z * zs);
    [
        [1.0 - (yy + zz), xy + wz, xz - wy],
        [xy - wz, 1.0 - (xx + zz), yz + wx],
        [xz + wy, yz - wx, 1.0 - (xx + yy)],
    ]
}

type M3 = [[f32; 3]; 3];

fn mul_m(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            *x = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}

fn mul_v(m: &M3, v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Each node's place in the shape: its own default transform under its
/// parent's.
fn node_transforms(
    parents: &[i32],
    rotations: &[i16],
    translations: &[f32],
) -> Result<Vec<(M3, [f32; 3])>, Error> {
    let n = parents.len();
    let mut out: Vec<Option<(M3, [f32; 3])>> = vec![None; n];
    fn place(
        i: usize,
        parents: &[i32],
        rotations: &[i16],
        translations: &[f32],
        out: &mut [Option<(M3, [f32; 3])>],
        depth: usize,
    ) -> Result<(M3, [f32; 3]), Error> {
        if let Some(t) = out[i] {
            return Ok(t);
        }
        if depth > parents.len() {
            return Err(Error::Bad("node parents (a loop)"));
        }
        let r = quat16(&rotations[4 * i..4 * i + 4]);
        let t = [
            translations[3 * i],
            translations[3 * i + 1],
            translations[3 * i + 2],
        ];
        let p = parents[i];
        let world = if p >= 0 && (p as usize) < parents.len() {
            let (pr, pt) = place(p as usize, parents, rotations, translations, out, depth + 1)?;
            let moved = mul_v(&pr, t);
            (
                mul_m(&pr, &r),
                [moved[0] + pt[0], moved[1] + pt[1], moved[2] + pt[2]],
            )
        } else {
            (r, t)
        };
        out[i] = Some(world);
        Ok(world)
    }
    for i in 0..n {
        place(i, parents, rotations, translations, &mut out, 0)?;
    }
    Ok(out.into_iter().map(Option::unwrap).collect())
}

/// A shape's names and material names, for the log.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Info {
    pub version: u32,
    pub names: Vec<String>,
    pub materials: Vec<String>,
}

/// Read `data` as a `.dts` shape: its rest pose as triangle meshes, one
/// per object, in the shape's own space.
pub fn read(data: &[u8]) -> Result<Model, Error> {
    read_with_info(data).map(|(m, _)| m)
}

/// [`read`], with the shape's names and materials.
pub fn read_with_info(data: &[u8]) -> Result<(Model, Info), Error> {
    let word = |i: usize| -> Result<u32, Error> {
        data.get(4 * i..4 * i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or(Error::Short("header"))
    };
    let version = word(0)? & 0xFF;
    if !(FIRST_VERSION..=LAST_VERSION).contains(&version) {
        return Err(Error::Version(version));
    }
    let (size, start16, start8) = (word(1)? as usize, word(2)? as usize, word(3)? as usize);
    if start16 > start8 || start8 > size {
        return Err(Error::Bad("buffer sizes"));
    }
    let buf = data
        .get(16..16 + size * 4)
        .ok_or(Error::Short("shape buffer"))?;
    let tail = &data[16 + size * 4..];
    let mut p = Parts {
        p32: &buf[..start16 * 4],
        p16: &buf[start16 * 4..start8 * 4],
        p8: &buf[start8 * 4..],
        guard: (0, 0, 0),
    };

    let nodes = p.count("node count")?;
    let objects = p.count("object count")?;
    let decals = p.count("decal count")?;
    let subshapes = p.count("subshape count")?;
    let ifls = p.count("ifl count")?;
    let rots = p.count("rotation count")?;
    let trans = p.count("translation count")?;
    let uniform = p.count("scale count")?;
    let aligned = p.count("scale count")?;
    let arbitrary = p.count("scale count")?;
    let ground = if version > 23 {
        p.count("ground count")?
    } else {
        0
    };
    let object_states = p.count("object state count")?;
    let decal_states = p.count("decal state count")?;
    let triggers = p.count("trigger count")?;
    let details = p.count("detail count")?;
    let meshes = p.count("mesh count")?;
    let names = p.count("name count")?;
    p.skip32(2, "smallest visible")?;
    p.guard("counts")?;
    p.skip32(1 + 1 + 3 + 6, "bounds")?;
    p.guard("bounds")?;
    let node_words = p.i32s(5 * nodes, "nodes")?;
    let parents: Vec<i32> = node_words.chunks_exact(5).map(|n| n[1]).collect();
    p.guard("nodes")?;
    let object_words = p.i32s(6 * objects, "objects")?;
    p.guard("objects")?;
    p.skip32(5 * decals, "decals")?;
    p.guard("decals")?;
    p.skip32(5 * ifls, "ifl materials")?;
    p.guard("ifl materials")?;
    p.skip32(subshapes, "subshapes")?;
    let first_object = p.i32s(subshapes, "subshapes")?;
    p.skip32(subshapes, "subshapes")?;
    p.guard("subshape starts")?;
    p.skip32(subshapes, "subshapes")?;
    let num_objects = p.i32s(subshapes, "subshapes")?;
    p.skip32(subshapes, "subshapes")?;
    p.guard("subshape sizes")?;
    let rotations = p.i16s(4 * nodes, "default rotations")?;
    let translations = p.f32s(3 * nodes, "default translations")?;
    p.skip32(3 * trans, "node translations")?;
    p.skip16(4 * rots, "node rotations")?;
    p.guard("node transforms")?;
    p.skip32(uniform + 3 * aligned + 3 * arbitrary, "node scales")?;
    p.skip16(4 * arbitrary, "node scale rotations")?;
    p.guard("node scales")?;
    if version > 23 {
        p.skip32(3 * ground, "ground translations")?;
        p.skip16(4 * ground, "ground rotations")?;
        p.guard("ground")?;
    }
    p.skip32(3 * object_states, "object states")?;
    p.guard("object states")?;
    p.skip32(decal_states, "decal states")?;
    p.guard("decal states")?;
    p.skip32(2 * triggers, "triggers")?;
    p.guard("triggers")?;
    let detail_words = p.i32s(7 * details, "details")?;
    p.guard("details")?;

    let mut raw: Vec<Option<RawMesh>> = Vec::with_capacity(meshes);
    for _ in 0..meshes {
        let kind = p.i32("mesh type")?;
        match kind {
            NULL => raw.push(None),
            STANDARD | SKIN | SORTED => {
                let m = mesh(&mut p, kind, &raw)?;
                raw.push(Some(m));
            }
            other => return Err(Error::MeshType(other)),
        }
    }
    p.guard("meshes")?;
    let mut name_list = Vec::with_capacity(names);
    for _ in 0..names {
        let end =
            p.p8.iter()
                .position(|&b| b == 0)
                .ok_or(Error::Short("names"))?;
        name_list.push(String::from_utf8_lossy(&p.p8[..end]).into_owned());
        p.p8 = &p.p8[end + 1..];
    }

    let materials = materials(tail, version).unwrap_or_default();
    let nodes_at = node_transforms(&parents, &rotations, &translations)?;

    // The first detail level that is not a billboard.
    let detail = detail_words
        .chunks_exact(7)
        .find(|d| d[1] >= 0)
        .map(|d| (d[1] as usize, d[2]));
    let mut model = Model::default();
    if let Some((sub, level)) = detail {
        let first = *first_object.get(sub).ok_or(Error::Bad("detail subshape"))?;
        let count = *num_objects.get(sub).ok_or(Error::Bad("detail subshape"))?;
        for o in first.max(0)..(first + count).max(0) {
            let o = o as usize;
            let Some(w) = object_words.get(6 * o..6 * o + 6) else {
                return Err(Error::Bad("subshape objects"));
            };
            let (name, num_meshes, start, node) = (w[0], w[1], w[2], w[3]);
            if level < 0 || level >= num_meshes {
                continue;
            }
            let Some(Some(m)) = raw.get((start + level) as usize) else {
                continue;
            };
            let frame = if m.verts_per_frame > 0 {
                m.verts_per_frame.min(m.verts.len())
            } else {
                m.verts.len()
            };
            let place = if m.skin {
                None
            } else {
                nodes_at.get(node as usize).copied()
            };
            let vertices: Vec<[f32; 3]> = m.verts[..frame]
                .iter()
                .map(|&v| match place {
                    Some((r, t)) => {
                        let v = mul_v(&r, v);
                        [v[0] + t[0], v[1] + t[1], v[2] + t[2]]
                    }
                    None => v,
                })
                .collect();
            if vertices.len() > u16::MAX as usize {
                continue;
            }
            let faces: Vec<[u16; 3]> = m
                .triangles
                .iter()
                .map(|(t, _)| *t)
                .filter(|t| t.iter().all(|&i| (i as usize) < vertices.len()))
                .map(|t| t.map(|i| i as u16))
                .collect();
            if faces.is_empty() {
                continue;
            }
            model.meshes.push(Mesh {
                name: name_list
                    .get(name.max(0) as usize)
                    .cloned()
                    .unwrap_or_default(),
                face_colours: vec![None; faces.len()],
                vertices,
                faces,
            });
        }
    }
    Ok((
        model,
        Info {
            version,
            names: name_list,
            materials,
        },
    ))
}

/// The material list after the sequences: the texture names.
fn materials(mut b: &[u8], version: u32) -> Option<Vec<String>> {
    let u32_ = |b: &mut &[u8]| -> Option<u32> {
        let v = u32::from_le_bytes(b.get(..4)?.try_into().ok()?);
        *b = &b[4..];
        Some(v)
    };
    let sequences = u32_(&mut b)?;
    for _ in 0..sequences.min(4096) {
        // Name, flags, keyframes, duration, priority, ground (2), bases (5),
        // triggers (2), tool begin.
        for _ in 0..15 {
            u32_(&mut b)?;
        }
        // Rotation, translation, scale, decal, ifl, visibility, frame and
        // material-frame sets: a dropped count, a size, the words.
        for _ in 0..8 {
            u32_(&mut b)?;
            let n = u32_(&mut b)?;
            for _ in 0..n.min(1 << 16) {
                u32_(&mut b)?;
            }
        }
    }
    let _ = version;
    if *b.first()? != 1 {
        return None;
    }
    b = &b[1..];
    let count = u32_(&mut b)?;
    let mut out = Vec::new();
    for _ in 0..count.min(4096) {
        let len = *b.first()? as usize;
        let name = b.get(1..1 + len)?;
        out.push(String::from_utf8_lossy(name).into_owned());
        b = &b[1 + len..];
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCTAHEDRON: &[u8] = include_bytes!("../tests/fixtures/dts/octahedron.dts");
    const CONE: &[u8] = include_bytes!("../tests/fixtures/dts/simplecone.dts");

    #[test]
    fn reads_a_version_24_shape_with_its_names_and_material() {
        let (m, info) = read_with_info(OCTAHEDRON).unwrap();
        assert_eq!(info.version, 24);
        assert_eq!(info.names, ["detail2", "octahedron2", "octahedron"]);
        assert_eq!(info.materials, ["green"]);
        assert_eq!(m.meshes.len(), 1);
        assert_eq!(m.meshes[0].name, "octahedron");
        assert_eq!(m.meshes[0].faces.len(), 8);
        // Its points sit 0.75 above and below the middle.
        let z: Vec<f32> = m.meshes[0].vertices.iter().map(|v| v[2]).collect();
        let (lo, hi) = z
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        assert!(
            (lo + 0.75).abs() < 1e-4 && (hi - 0.75).abs() < 1e-4,
            "{lo} {hi}"
        );
    }

    #[test]
    fn reads_a_shape_whose_detail_comes_after_its_object_name() {
        let (m, info) = read_with_info(CONE).unwrap();
        assert_eq!(info.materials, ["blue"]);
        assert_eq!(m.meshes.len(), 1);
        assert_eq!(m.meshes[0].name, "simplecone");
        assert_eq!(m.meshes[0].faces.len(), 19);
        assert!(m.meshes[0]
            .faces
            .iter()
            .all(|f| f.iter().all(|&i| (i as usize) < m.meshes[0].vertices.len())));
    }

    #[test]
    fn other_versions_and_cut_files_are_refused() {
        let mut v26 = OCTAHEDRON.to_vec();
        v26[0] = 26;
        assert_eq!(read(&v26), Err(Error::Version(26)));
        assert!(matches!(read(&OCTAHEDRON[..200]), Err(Error::Short(_))));
        assert!(read(b"").is_err());
    }

    #[test]
    fn a_broken_guard_is_caught() {
        // The first guard is the 32-bit word after the 19 counts.
        let mut b = OCTAHEDRON.to_vec();
        let at = 16 + 19 * 4;
        b[at] ^= 0x55;
        assert!(matches!(read(&b), Err(Error::Guard("counts"))));
    }

    #[test]
    fn strips_and_fans_become_triangles() {
        let mut out = Vec::new();
        primitive(&[0, 1, 2, 3], 0, 4, STRIP | NO_MATERIAL, &mut out).unwrap();
        assert_eq!(out, [([0, 1, 2], None), ([2, 1, 3], None)]);
        out.clear();
        primitive(&[0, 1, 2, 3], 0, 4, FAN | 5, &mut out).unwrap();
        assert_eq!(out, [([0, 1, 2], Some(5)), ([0, 2, 3], Some(5))]);
        assert!(primitive(&[0, 1], 0, 3, 0, &mut out).is_err());
    }

    #[test]
    fn a_child_node_is_placed_under_its_parent() {
        // Node 1 is 1 along x from node 0, which is turned a quarter about z
        // and stands at (0, 0, 2).
        let h = (0.5f32.sqrt() * 32767.0) as i16;
        let rot = [0, 0, h, h, 0, 0, 0, 32767];
        let at = node_transforms(&[-1, 0], &rot, &[0.0, 0.0, 2.0, 1.0, 0.0, 0.0]).unwrap();
        let p = at[1].1;
        // Torque turns by the conjugate: +x goes to -y.
        assert!(
            p[0].abs() < 1e-3 && (p[1] + 1.0).abs() < 1e-3 && (p[2] - 2.0).abs() < 1e-3,
            "{p:?}"
        );
        assert!(node_transforms(&[1, 0], &[0; 8], &[0.0; 6]).is_err());
    }
}
