//! DirectX `.x` text files (`xof 0302txt` / `xof 0303txt`), as Graal3D's
//! levels and player models use them.
//!
//! Only what a Graal3D file holds is read: top-level `Material`s, `Frame`s
//! and `Mesh`es, and inside a mesh its faces, `MeshMaterialList`,
//! `MeshTextureCoords` and `MeshNormals`. Templates are skipped. Binary and
//! compressed `.x` files are refused.

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    NotText,
    Syntax(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotText => write!(f, "not a text .x file"),
            Error::Syntax(s) => write!(f, ".x syntax: {s}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(f64),
    Str(String),
    Open,
    Close,
}

fn tokenize(src: &str) -> Vec<Tok> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let ch = b[i];
        match ch {
            b'{' => {
                out.push(Tok::Open);
                i += 1;
            }
            b'}' => {
                out.push(Tok::Close);
                i += 1;
            }
            b'"' => {
                let start = i + 1;
                i = start;
                while i < b.len() && b[i] != b'"' {
                    i += 1;
                }
                out.push(Tok::Str(
                    String::from_utf8_lossy(&b[start..i.min(b.len())]).into_owned(),
                ));
                i += 1;
            }
            b'<' => {
                // A template GUID.
                while i < b.len() && b[i] != b'>' {
                    i += 1;
                }
                i += 1;
            }
            b'#' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let start = i;
                i += 1;
                while i < b.len() && matches!(b[i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+')
                {
                    i += 1;
                }
                if let Ok(v) = src[start..i].parse::<f64>() {
                    out.push(Tok::Num(v));
                }
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || matches!(b[i], b'_' | b'-')) {
                    i += 1;
                }
                out.push(Tok::Ident(src[start..i].to_string()));
            }
            _ => i += 1, // whitespace, `;`, `,`, `[`, `]`
        }
    }
    out
}

/// A parsed data object: its type, optional name, the numbers and strings
/// of its body in order, and child objects. A `{ name }` reference is a
/// child of kind `{}` so that it keeps its place among the others.
#[derive(Debug, Clone, Default)]
struct Node {
    kind: String,
    name: Option<String>,
    nums: Vec<f64>,
    strs: Vec<String>,
    children: Vec<Node>,
}

struct Parser {
    t: Vec<Tok>,
    i: usize,
}

impl Parser {
    fn peek(&self, k: usize) -> Option<&Tok> {
        self.t.get(self.i + k)
    }

    /// Parse the body after `{` up to the matching `}`.
    fn body(&mut self, node: &mut Node) -> Result<(), Error> {
        loop {
            match self.t.get(self.i).cloned() {
                None => return Err(Error::Syntax(format!("unclosed {}", node.kind))),
                Some(Tok::Close) => {
                    self.i += 1;
                    return Ok(());
                }
                Some(Tok::Num(v)) => {
                    node.nums.push(v);
                    self.i += 1;
                }
                Some(Tok::Str(s)) => {
                    node.strs.push(s);
                    self.i += 1;
                }
                Some(Tok::Open) => {
                    // `{ name }` reference, or an anonymous block.
                    if let (Some(Tok::Ident(n)), Some(Tok::Close)) =
                        (self.peek(1).cloned(), self.peek(2))
                    {
                        node.children.push(Node {
                            kind: "{}".into(),
                            name: Some(n),
                            ..Node::default()
                        });
                        self.i += 3;
                    } else {
                        self.i += 1;
                        let mut skip = Node::default();
                        self.body(&mut skip)?;
                    }
                }
                Some(Tok::Ident(_)) => {
                    if let Some(child) = self.object()? {
                        node.children.push(child);
                    }
                }
            }
        }
    }

    /// `Kind [name] { … }`; `None` for a template or a stray word.
    fn object(&mut self) -> Result<Option<Node>, Error> {
        let Some(Tok::Ident(kind)) = self.t.get(self.i).cloned() else {
            self.i += 1;
            return Ok(None);
        };
        self.i += 1;
        let mut name = None;
        if let Some(Tok::Ident(n)) = self.peek(0).cloned() {
            if self.peek(1) == Some(&Tok::Open) {
                name = Some(n);
                self.i += 1;
            }
        }
        if self.peek(0) != Some(&Tok::Open) {
            return Ok(None);
        }
        self.i += 1;
        let mut node = Node {
            kind: kind.clone(),
            name,
            ..Node::default()
        };
        self.body(&mut node)?;
        Ok((kind != "template").then_some(node))
    }
}

/// A material: diffuse colour and an optional texture file name.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: Option<String>,
    pub colour: [f32; 4],
    pub texture: Option<String>,
}

impl Material {
    fn from(n: &Node) -> Material {
        let mut colour = [1.0; 4];
        for (k, v) in n.nums.iter().take(4).enumerate() {
            colour[k] = *v as f32;
        }
        let texture = n
            .children
            .iter()
            .find(|c| c.kind == "TextureFilename")
            .and_then(|c| c.strs.first().cloned());
        Material {
            name: n.name.clone(),
            colour,
            texture,
        }
    }
}

/// One mesh, with its faces split into triangles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub name: Option<String>,
    pub vertices: Vec<[f32; 3]>,
    /// The faces as given (polygons of 3 or more vertices).
    pub faces: Vec<Vec<u32>>,
    /// Material index per face.
    pub face_materials: Vec<u32>,
    pub materials: Vec<Material>,
    pub uvs: Vec<[f32; 2]>,
}

impl Mesh {
    /// The faces fanned into triangles, with each triangle's material.
    pub fn triangles(&self) -> impl Iterator<Item = ([u32; 3], u32)> + '_ {
        self.faces.iter().enumerate().flat_map(move |(fi, f)| {
            let m = self.face_materials.get(fi).copied().unwrap_or(0);
            (1..f.len().saturating_sub(1)).map(move |k| ([f[0], f[k], f[k + 1]], m))
        })
    }

    fn from(n: &Node, named: &HashMap<String, Material>) -> Result<Mesh, Error> {
        let bad = |what: &str| Error::Syntax(format!("mesh: {what}"));
        let mut it = n.nums.iter().copied();
        let mut next = |what: &str| it.next().ok_or_else(|| bad(what));
        let nv = next("vertex count")? as usize;
        let mut vertices = Vec::with_capacity(nv.min(1 << 20));
        for _ in 0..nv {
            vertices.push([
                next("vertex")? as f32,
                next("vertex")? as f32,
                next("vertex")? as f32,
            ]);
        }
        let nf = next("face count")? as usize;
        let mut faces = Vec::with_capacity(nf.min(1 << 20));
        for _ in 0..nf {
            let k = next("face size")? as usize;
            let mut f = Vec::with_capacity(k.min(64));
            for _ in 0..k {
                let v = next("face index")? as u32;
                if v as usize >= nv {
                    return Err(bad("face index out of range"));
                }
                f.push(v);
            }
            faces.push(f);
        }
        let mut mesh = Mesh {
            name: n.name.clone(),
            vertices,
            faces,
            ..Mesh::default()
        };
        for c in &n.children {
            match c.kind.as_str() {
                "MeshMaterialList" => {
                    let mut it = c.nums.iter().copied();
                    let _nm = it.next();
                    let nfi = it.next().unwrap_or(0.0) as usize;
                    mesh.face_materials = it.take(nfi).map(|v| v as u32).collect();
                    for m in &c.children {
                        match m.kind.as_str() {
                            "Material" => mesh.materials.push(Material::from(m)),
                            "{}" => {
                                if let Some(r) = m.name.as_ref().and_then(|r| named.get(r)) {
                                    mesh.materials.push(r.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "MeshTextureCoords" => {
                    let mut it = c.nums.iter().copied();
                    let k = it.next().unwrap_or(0.0) as usize;
                    let v: Vec<f64> = it.take(k * 2).collect();
                    mesh.uvs = v
                        .chunks_exact(2)
                        .map(|p| [p[0] as f32, p[1] as f32])
                        .collect();
                }
                _ => {}
            }
        }
        Ok(mesh)
    }
}

/// Every mesh of a `.x` file, frames flattened (frame transforms are not
/// applied; Graal3D's shipped files have none).
#[derive(Debug, Clone, Default)]
pub struct XFile {
    pub meshes: Vec<Mesh>,
}

impl XFile {
    pub fn parse(src: &[u8]) -> Result<XFile, Error> {
        if src.len() < 16 || &src[..4] != b"xof " || &src[8..11] != b"txt" {
            return Err(Error::NotText);
        }
        let text = String::from_utf8_lossy(&src[16..]);
        let mut p = Parser {
            t: tokenize(&text),
            i: 0,
        };
        let mut top = Vec::new();
        while p.i < p.t.len() {
            if let Some(n) = p.object()? {
                top.push(n);
            }
        }
        let named: HashMap<String, Material> = top
            .iter()
            .filter(|n| n.kind == "Material")
            .filter_map(|n| Some((n.name.clone()?, Material::from(n))))
            .collect();
        let mut meshes = Vec::new();
        fn walk(
            n: &Node,
            named: &HashMap<String, Material>,
            out: &mut Vec<Mesh>,
        ) -> Result<(), Error> {
            match n.kind.as_str() {
                "Mesh" => out.push(Mesh::from(n, named)?),
                "Frame" => {
                    for c in &n.children {
                        walk(c, named, out)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        for n in &top {
            walk(n, &named, &mut meshes)?;
        }
        Ok(XFile { meshes })
    }

    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|m| m.vertices.len()).sum()
    }

    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|m| m.faces.len()).sum()
    }

    /// The texture files the meshes name, once each, in order.
    pub fn textures(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for t in self
            .meshes
            .iter()
            .flat_map(|m| m.materials.iter())
            .filter_map(|m| m.texture.clone())
        {
            if !out.contains(&t) {
                out.push(t);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small level in the shape of the shipped ones: templates, a named
    /// top-level material referenced from the mesh, an inline material.
    const LEVEL: &str = "xof 0302txt 0064
template Vector {
 <3D82AB5E-62DA-11cf-AB39-0020AF71E433>
 FLOAT x;
 FLOAT y;
 FLOAT z;
}

Header {
 1;
 0;
 1;
}

Material grasstex {
 1.000000;1.000000;1.000000;1.000000;;
 0.000000;
 0.000000;0.000000;0.000000;;
 0.000000;0.000000;0.000000;;
 TextureFilename {
  \"grass.bmp\";
 }
}

Mesh obj {
 5;
 -100.0;0.0;-100.0;,
 100.0;0.0;-100.0;,
 100.0;0.0;100.0;,
 -100.0;0.0;100.0;,
 0.0;50.0;0.0;;
 2;
 4;0,1,2,3;,
 3;0,1,4;;
 MeshMaterialList {
  2;
  2;
  0,
  1;;
  { grasstex }
  Material {
   0.5;0.5;0.5;1.0;;
   0.0;
   0.0;0.0;0.0;;
   0.0;0.0;0.0;;
  }
 }
 MeshTextureCoords {
  5;
  0.0;0.0;,
  1.0;0.0;,
  1.0;1.0;,
  0.0;1.0;,
  0.5;0.5;;
 }
}
";

    #[test]
    fn reads_a_level() {
        let x = XFile::parse(LEVEL.as_bytes()).unwrap();
        assert_eq!(x.meshes.len(), 1);
        let m = &x.meshes[0];
        assert_eq!(m.name.as_deref(), Some("obj"));
        assert_eq!(m.vertices.len(), 5);
        assert_eq!(m.vertices[4], [0.0, 50.0, 0.0]);
        assert_eq!(m.faces, vec![vec![0, 1, 2, 3], vec![0, 1, 4]]);
        assert_eq!(m.face_materials, vec![0, 1]);
        assert_eq!(m.materials.len(), 2);
        assert_eq!(m.materials[0].texture.as_deref(), Some("grass.bmp"));
        assert_eq!(m.materials[1].colour, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(m.uvs.len(), 5);
        assert_eq!(m.triangles().count(), 3);
        assert_eq!(x.textures(), vec!["grass.bmp".to_string()]);
    }

    #[test]
    fn refuses_binary_and_bad_indices() {
        assert_eq!(
            XFile::parse(b"xof 0302bin 0032....").unwrap_err(),
            Error::NotText
        );
        let bad = LEVEL.replace("3;0,1,4;;", "3;0,1,9;;");
        assert!(XFile::parse(bad.as_bytes()).is_err());
    }
}
