//! A reader for `.3ds` meshes, the shape format the lab Graal3D world uses
//! (`Ground`, `labcube.3ds`).
//!
//! `.3ds` is the public Autodesk 3D Studio chunk format: each chunk is a
//! 16-bit id and a 32-bit length (both little-endian, the length counting
//! the 6-byte header), then its data and child chunks. Only what a flat
//! shaded view needs is read: each object's triangle mesh (vertices,
//! texture coordinates and faces) and the diffuse colour and texture map
//! of the material its faces use.

use std::collections::HashMap;

const MAIN: u16 = 0x4D4D;
const EDITOR: u16 = 0x3D3D;
const OBJECT: u16 = 0x4000;
const TRIMESH: u16 = 0x4100;
const VERTICES: u16 = 0x4110;
const FACES: u16 = 0x4120;
const FACE_MATERIAL: u16 = 0x4130;
const TEX_VERTS: u16 = 0x4140;
const MATERIAL: u16 = 0xAFFF;
const MATERIAL_NAME: u16 = 0xA000;
const DIFFUSE: u16 = 0xA020;
const TEXTURE_MAP: u16 = 0xA200;
const MAP_NAME: u16 = 0xA300;
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
    /// Texture coordinates, one per vertex, `v` from the top of the image
    /// (empty: the mesh has none).
    pub uvs: Vec<[f32; 2]>,
    /// The texture of each face, as its material names the file (empty, or
    /// `None` for a face: no texture).
    pub face_textures: Vec<Option<String>>,
}

impl Model {
    /// The texture files the model's faces name, each once, in first-use
    /// order.
    pub fn textures(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for m in &self.meshes {
            for t in m.face_textures.iter().flatten() {
                if !out.iter().any(|o| o.eq_ignore_ascii_case(t)) {
                    out.push(t.clone());
                }
            }
        }
        out
    }
}

/// A material: its diffuse colour and texture map, when given.
#[derive(Debug, Clone, Default)]
struct Material {
    diffuse: Option<[u8; 3]>,
    texture: Option<String>,
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
        let mut materials: HashMap<String, Material> = HashMap::new();
        let mut meshes = Vec::new();
        for ed in chunks(main.data, 6)?.into_iter().filter(|c| c.id == EDITOR) {
            let parts = chunks(ed.data, 0)?;
            for m in parts.iter().filter(|c| c.id == MATERIAL) {
                let mut name = None;
                let mut mat = Material::default();
                for c in chunks(m.data, 0)? {
                    match c.id {
                        MATERIAL_NAME => name = Some(cstring(c.data).0),
                        DIFFUSE => mat.diffuse = colour(c.data),
                        TEXTURE_MAP => {
                            mat.texture = chunks(c.data, 0)?
                                .iter()
                                .find(|t| t.id == MAP_NAME)
                                .map(|t| cstring(t.data).0)
                                .filter(|n| !n.trim().is_empty());
                        }
                        _ => {}
                    }
                }
                if let Some(n) = name {
                    materials.insert(n, mat);
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
        materials: &HashMap<String, Material>,
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
                    mesh.face_textures = vec![None; n];
                    // Face materials are children after the face list.
                    for m in chunks(&c.data[end..], 0)?
                        .into_iter()
                        .filter(|m| m.id == FACE_MATERIAL)
                    {
                        let (mat, rest) = cstring(m.data);
                        let mat = materials.get(&mat);
                        let col = mat.and_then(|m| m.diffuse);
                        let tex = mat.and_then(|m| m.texture.clone());
                        let count = u16_at(rest, 0).unwrap_or(0) as usize;
                        for k in 0..count {
                            if let Some(f) = u16_at(rest, 2 + k * 2) {
                                if let Some(slot) = mesh.face_colours.get_mut(f as usize) {
                                    *slot = col;
                                }
                                if let Some(slot) = mesh.face_textures.get_mut(f as usize) {
                                    slot.clone_from(&tex);
                                }
                            }
                        }
                    }
                }
                TEX_VERTS => {
                    // 3D Studio's v runs up from the bottom of the image.
                    let n = u16_at(c.data, 0).unwrap_or(0) as usize;
                    for i in 0..n {
                        let at = 2 + i * 8;
                        match (f32_at(c.data, at), f32_at(c.data, at + 4)) {
                            (Some(u), Some(v)) => mesh.uvs.push([u, 1.0 - v]),
                            _ => return Err(Error::Truncated(at)),
                        }
                    }
                }
                _ => {}
            }
        }
        if mesh.uvs.len() != mesh.vertices.len() {
            mesh.uvs.clear();
        }
        // Drop faces naming missing vertices, with their colour and texture.
        let keep: Vec<bool> = mesh
            .faces
            .iter()
            .map(|f| f.iter().all(|&v| (v as usize) < mesh.vertices.len()))
            .collect();
        mesh.face_colours.resize(mesh.faces.len(), None);
        mesh.face_textures.resize(mesh.faces.len(), None);
        let mut k = keep.iter();
        mesh.faces.retain(|_| *k.next().unwrap_or(&false));
        let mut k = keep.iter();
        mesh.face_colours.retain(|_| *k.next().unwrap_or(&false));
        let mut k = keep.iter();
        mesh.face_textures.retain(|_| *k.next().unwrap_or(&false));
        if mesh.uvs.is_empty() {
            mesh.face_textures.iter_mut().for_each(|t| *t = None);
        }
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
        textured(name, vertices, faces, rgb, None)
    }

    /// [`mesh`], with texture coordinates (3D Studio's: `v` up from the
    /// bottom) and a texture map on its material.
    pub fn textured(
        name: &str,
        vertices: &[[f32; 3]],
        faces: &[[u16; 3]],
        rgb: [u8; 3],
        texture: Option<(&[[f32; 2]], &str)>,
    ) -> Vec<u8> {
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
        if let Some((uvs, _)) = texture {
            let mut t = (uvs.len() as u16).to_le_bytes().to_vec();
            for uv in uvs {
                t.extend(uv[0].to_le_bytes());
                t.extend(uv[1].to_le_bytes());
            }
            tri.extend(chunk(super::TEX_VERTS, &t));
        }
        tri.extend(chunk(super::FACES, &f));
        let mut obj = name.as_bytes().to_vec();
        obj.push(0);
        obj.extend(chunk(super::TRIMESH, &tri));
        let mut mat = chunk(super::MATERIAL_NAME, b"mat\0");
        mat.extend(chunk(super::DIFFUSE, &chunk(super::COLOR_24, &rgb)));
        if let Some((_, file)) = texture {
            let mut n = file.as_bytes().to_vec();
            n.push(0);
            mat.extend(chunk(super::TEXTURE_MAP, &chunk(super::MAP_NAME, &n)));
        }
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
    fn a_texture_map_and_its_coordinates_read_back() {
        let uvs = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let b = write::textured(
            "tri",
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0, 1, 2]],
            [1, 2, 3],
            Some((&uvs, "GRASS.PNG")),
        );
        let m = Model::read(&b).unwrap();
        let mesh = &m.meshes[0];
        // v is turned to run down from the top of the image.
        assert_eq!(mesh.uvs, [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]]);
        assert_eq!(mesh.face_textures, [Some("GRASS.PNG".to_string())]);
        assert_eq!(m.textures(), ["GRASS.PNG"]);
        // Without coordinates the texture is not used.
        let b = write::mesh("t", &[[0.0; 3]; 3], &[[0, 1, 2]], [0; 3]);
        assert!(Model::read(&b).unwrap().textures().is_empty());
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
