//! A reader for `.3ds` meshes, the shape format the lab Graal3D world uses
//! (`Ground`, `labcube.3ds`).
//!
//! `.3ds` is the public Autodesk 3D Studio chunk format: each chunk is a
//! 16-bit id and a 32-bit length (both little-endian, the length counting
//! the 6-byte header), then its data and child chunks. Only what a flat
//! shaded view needs is read: each object's triangle mesh (vertices and
//! faces) and the diffuse colour of the material its faces use.

use std::collections::HashMap;

const MAIN: u16 = 0x4D4D;
const EDITOR: u16 = 0x3D3D;
const OBJECT: u16 = 0x4000;
const TRIMESH: u16 = 0x4100;
const VERTICES: u16 = 0x4110;
const FACES: u16 = 0x4120;
const FACE_MATERIAL: u16 = 0x4130;
const MATERIAL: u16 = 0xAFFF;
const MATERIAL_NAME: u16 = 0xA000;
const DIFFUSE: u16 = 0xA020;
const COLOR_F: u16 = 0x0010;
const COLOR_24: u16 = 0x0011;

/// One object's triangles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    /// Vertex indices, three per triangle.
    pub faces: Vec<[u16; 3]>,
    /// The diffuse colour of each face, when its material gives one.
    pub face_colours: Vec<Option<[u8; 3]>>,
}

/// A `.3ds` file's meshes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

/// Why a file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotThreeDs,
    /// A chunk runs past its parent, at this offset.
    Truncated(usize),
}

struct Chunk<'a> {
    id: u16,
    data: &'a [u8],
}

/// The chunks laid end to end in `b`.
fn chunks(mut b: &[u8], base: usize) -> Result<Vec<Chunk<'_>>, Error> {
    let mut out = Vec::new();
    let mut at = base;
    while b.len() >= 6 {
        let id = u16::from_le_bytes([b[0], b[1]]);
        let len = u32::from_le_bytes([b[2], b[3], b[4], b[5]]) as usize;
        if len < 6 || len > b.len() {
            return Err(Error::Truncated(at));
        }
        out.push(Chunk {
            id,
            data: &b[6..len],
        });
        b = &b[len..];
        at += len;
    }
    Ok(out)
}

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(i)?, *b.get(i + 1)?]))
}

fn f32_at(b: &[u8], i: usize) -> Option<f32> {
    Some(f32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

/// A NUL-ended name and the bytes after it.
fn cstring(b: &[u8]) -> (String, &[u8]) {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    let rest = b.get(end + 1..).unwrap_or_default();
    (String::from_utf8_lossy(&b[..end]).into_owned(), rest)
}

/// A colour chunk's RGB.
fn colour(b: &[u8]) -> Option<[u8; 3]> {
    for c in chunks(b, 0).ok()? {
        match c.id {
            COLOR_24 if c.data.len() >= 3 => return Some([c.data[0], c.data[1], c.data[2]]),
            COLOR_F => {
                let f = |i| (f32_at(c.data, i).unwrap_or(0.0).clamp(0.0, 1.0) * 255.0) as u8;
                return Some([f(0), f(4), f(8)]);
            }
            _ => {}
        }
    }
    None
}

impl Model {
    /// Read a `.3ds` file.
    pub fn read(bytes: &[u8]) -> Result<Model, Error> {
        if u16_at(bytes, 0) != Some(MAIN) {
            return Err(Error::NotThreeDs);
        }
        let top = chunks(bytes, 0)?;
        let main = top.iter().find(|c| c.id == MAIN).ok_or(Error::NotThreeDs)?;
        let mut materials: HashMap<String, [u8; 3]> = HashMap::new();
        let mut meshes = Vec::new();
        for ed in chunks(main.data, 6)?.into_iter().filter(|c| c.id == EDITOR) {
            let parts = chunks(ed.data, 0)?;
            for m in parts.iter().filter(|c| c.id == MATERIAL) {
                let mut name = None;
                let mut diffuse = None;
                for c in chunks(m.data, 0)? {
                    match c.id {
                        MATERIAL_NAME => name = Some(cstring(c.data).0),
                        DIFFUSE => diffuse = colour(c.data),
                        _ => {}
                    }
                }
                if let (Some(n), Some(d)) = (name, diffuse) {
                    materials.insert(n, d);
                }
            }
            for o in parts.iter().filter(|c| c.id == OBJECT) {
                let (name, rest) = cstring(o.data);
                for t in chunks(rest, 0)?.into_iter().filter(|c| c.id == TRIMESH) {
                    meshes.push(Self::trimesh(name.clone(), t.data, &materials)?);
                }
            }
        }
        Ok(Model { meshes })
    }

    fn trimesh(
        name: String,
        b: &[u8],
        materials: &HashMap<String, [u8; 3]>,
    ) -> Result<Mesh, Error> {
        let mut mesh = Mesh {
            name,
            ..Default::default()
        };
        for c in chunks(b, 0)? {
            match c.id {
                VERTICES => {
                    let n = u16_at(c.data, 0).unwrap_or(0) as usize;
                    for i in 0..n {
                        let at = 2 + i * 12;
                        match (
                            f32_at(c.data, at),
                            f32_at(c.data, at + 4),
                            f32_at(c.data, at + 8),
                        ) {
                            (Some(x), Some(y), Some(z)) => mesh.vertices.push([x, y, z]),
                            _ => return Err(Error::Truncated(at)),
                        }
                    }
                }
                FACES => {
                    let n = u16_at(c.data, 0).unwrap_or(0) as usize;
                    let end = 2 + n * 8;
                    if c.data.len() < end {
                        return Err(Error::Truncated(end));
                    }
                    for i in 0..n {
                        let at = 2 + i * 8;
                        let v = |k: usize| u16_at(c.data, at + k * 2).unwrap_or(0);
                        mesh.faces.push([v(0), v(1), v(2)]);
                    }
                    mesh.face_colours = vec![None; n];
                    // Face materials are children after the face list.
                    for m in chunks(&c.data[end..], 0)?
                        .into_iter()
                        .filter(|m| m.id == FACE_MATERIAL)
                    {
                        let (mat, rest) = cstring(m.data);
                        let col = materials.get(&mat).copied();
                        let count = u16_at(rest, 0).unwrap_or(0) as usize;
                        for k in 0..count {
                            if let Some(f) = u16_at(rest, 2 + k * 2) {
                                if let Some(slot) = mesh.face_colours.get_mut(f as usize) {
                                    *slot = col;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        mesh.faces
            .retain(|f| f.iter().all(|&v| (v as usize) < mesh.vertices.len()));
        mesh.face_colours.resize(mesh.faces.len(), None);
        Ok(mesh)
    }
}

/// Builds `.3ds` bytes (for tests, and the lab's own shapes).
pub mod write {
    fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
        let mut out = id.to_le_bytes().to_vec();
        out.extend(((body.len() + 6) as u32).to_le_bytes());
        out.extend_from_slice(body);
        out
    }

    /// One object of one triangle mesh, with one coloured material.
    pub fn mesh(name: &str, vertices: &[[f32; 3]], faces: &[[u16; 3]], rgb: [u8; 3]) -> Vec<u8> {
        let mut v = (vertices.len() as u16).to_le_bytes().to_vec();
        for p in vertices {
            for x in p {
                v.extend(x.to_le_bytes());
            }
        }
        let mut f = (faces.len() as u16).to_le_bytes().to_vec();
        for t in faces {
            for i in t {
                f.extend(i.to_le_bytes());
            }
            f.extend(0u16.to_le_bytes());
        }
        let mut fm = b"mat\0".to_vec();
        fm.extend((faces.len() as u16).to_le_bytes());
        for i in 0..faces.len() as u16 {
            fm.extend(i.to_le_bytes());
        }
        f.extend(chunk(super::FACE_MATERIAL, &fm));
        let mut tri = chunk(super::VERTICES, &v);
        tri.extend(chunk(super::FACES, &f));
        let mut obj = name.as_bytes().to_vec();
        obj.push(0);
        obj.extend(chunk(super::TRIMESH, &tri));
        let mut mat = chunk(super::MATERIAL_NAME, b"mat\0");
        mat.extend(chunk(super::DIFFUSE, &chunk(super::COLOR_24, &rgb)));
        let mut ed = chunk(super::MATERIAL, &mat);
        ed.extend(chunk(super::OBJECT, &obj));
        chunk(super::MAIN, &chunk(super::EDITOR, &ed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coloured_triangle_reads_back() {
        let b = write::mesh(
            "tri",
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0, 1, 2]],
            [200, 10, 20],
        );
        let m = Model::read(&b).unwrap();
        assert_eq!(m.meshes.len(), 1);
        let mesh = &m.meshes[0];
        assert_eq!(mesh.name, "tri");
        assert_eq!(mesh.vertices[1], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.faces, [[0, 1, 2]]);
        assert_eq!(mesh.face_colours, [Some([200, 10, 20])]);
    }

    #[test]
    fn other_files_are_refused() {
        assert_eq!(Model::read(b"GIF89a"), Err(Error::NotThreeDs));
        let mut b = write::mesh("t", &[[0.0; 3]; 3], &[[0, 1, 2]], [0; 3]);
        b.truncate(b.len() - 4);
        assert!(Model::read(&b).is_err());
    }

    #[test]
    fn faces_naming_missing_vertices_are_dropped() {
        let b = write::mesh("t", &[[0.0; 3]; 3], &[[0, 1, 2], [0, 1, 9]], [1, 2, 3]);
        let m = Model::read(&b).unwrap();
        assert_eq!(m.meshes[0].faces, [[0, 1, 2]]);
        assert_eq!(m.meshes[0].face_colours.len(), 1);
    }
}
