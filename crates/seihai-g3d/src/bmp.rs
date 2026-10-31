//! Windows BMP textures, as Graal3D ships them: 24-bit, and 8-bit with a
//! palette, plain or RLE8. Also a 24-bit writer for screenshots.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    /// `0x00RRGGBB`, top row first.
    pub px: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub &'static str);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bmp: {}", self.0)
    }
}

impl std::error::Error for Error {}

fn u16le(b: &[u8], o: usize) -> Result<u16, Error> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or(Error("short header"))
}

fn u32le(b: &[u8], o: usize) -> Result<u32, Error> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(Error("short header"))
}

/// The largest texture side accepted.
const MAX_SIDE: usize = 4096;

pub fn decode(b: &[u8]) -> Result<Image, Error> {
    if b.get(..2) != Some(b"BM") {
        return Err(Error("not a BMP"));
    }
    let data = u32le(b, 10)? as usize;
    let hdr = u32le(b, 14)? as usize;
    let w = u32le(b, 18)? as i32;
    let h = u32le(b, 22)? as i32;
    let bpp = u16le(b, 28)?;
    let comp = if hdr >= 40 { u32le(b, 30)? } else { 0 };
    let colours = if hdr >= 40 { u32le(b, 46)? as usize } else { 0 };
    let (w, top_down, h) = (w.unsigned_abs() as usize, h < 0, h.unsigned_abs() as usize);
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(Error("bad size"));
    }
    let pix = b.get(data..).ok_or(Error("no pixel data"))?;
    let mut px = vec![0u32; w * h];
    // Row r of the file is image row `row(r)`.
    let row = |r: usize| if top_down { r } else { h - 1 - r };
    match (bpp, comp) {
        (24, 0) => {
            let stride = (w * 3 + 3) & !3;
            for r in 0..h {
                let line = pix.get(r * stride..r * stride + w * 3).ok_or(Error("short pixel data"))?;
                for x in 0..w {
                    let p = &line[x * 3..x * 3 + 3];
                    px[row(r) * w + x] = (p[2] as u32) << 16 | (p[1] as u32) << 8 | p[0] as u32;
                }
            }
        }
        (8, 0) | (8, 1) => {
            let n = if colours == 0 { 256 } else { colours.min(256) };
            let pal_at = 14 + hdr;
            let mut pal = [0u32; 256];
            for (i, slot) in pal.iter_mut().enumerate().take(n) {
                let p = b.get(pal_at + i * 4..pal_at + i * 4 + 3).ok_or(Error("short palette"))?;
                *slot = (p[2] as u32) << 16 | (p[1] as u32) << 8 | p[0] as u32;
            }
            let idx = if comp == 0 {
                let stride = (w + 3) & !3;
                let mut v = vec![0u8; w * h];
                for r in 0..h {
                    let line = pix.get(r * stride..r * stride + w).ok_or(Error("short pixel data"))?;
                    v[r * w..r * w + w].copy_from_slice(line);
                }
                v
            } else {
                rle8(pix, w, h)?
            };
            for r in 0..h {
                for x in 0..w {
                    px[row(r) * w + x] = pal[idx[r * w + x] as usize];
                }
            }
        }
        _ => return Err(Error("only 24-bit and 8-bit images are read")),
    }
    Ok(Image { w, h, px })
}

/// RLE8 into file-order rows of indices.
fn rle8(src: &[u8], w: usize, h: usize) -> Result<Vec<u8>, Error> {
    let mut out = vec![0u8; w * h];
    let (mut x, mut y, mut i) = (0usize, 0usize, 0usize);
    let mut put = |x: &mut usize, y: usize, v: u8| {
        if *x < w && y < h {
            out[y * w + *x] = v;
        }
        *x += 1;
    };
    while i + 1 < src.len() && y < h {
        let (n, v) = (src[i], src[i + 1]);
        i += 2;
        if n > 0 {
            for _ in 0..n {
                put(&mut x, y, v);
            }
            continue;
        }
        match v {
            0 => {
                x = 0;
                y += 1;
            }
            1 => break,
            2 => {
                let d = src.get(i..i + 2).ok_or(Error("short RLE delta"))?;
                x += d[0] as usize;
                y += d[1] as usize;
                i += 2;
            }
            k => {
                let k = k as usize;
                let lit = src.get(i..i + k).ok_or(Error("short RLE run"))?;
                for &b in lit {
                    put(&mut x, y, b);
                }
                i += k + (k & 1);
            }
        }
    }
    Ok(out)
}

/// A 24-bit bottom-up BMP of `px` (`0x00RRGGBB`, top row first).
pub fn encode(w: usize, h: usize, px: &[u32]) -> Vec<u8> {
    let stride = (w * 3 + 3) & !3;
    let size = 54 + stride * h;
    let mut b = Vec::with_capacity(size);
    b.extend_from_slice(b"BM");
    b.extend_from_slice(&(size as u32).to_le_bytes());
    b.extend_from_slice(&[0; 4]);
    b.extend_from_slice(&54u32.to_le_bytes());
    b.extend_from_slice(&40u32.to_le_bytes());
    b.extend_from_slice(&(w as u32).to_le_bytes());
    b.extend_from_slice(&(h as u32).to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&24u16.to_le_bytes());
    b.extend_from_slice(&[0; 24]);
    for r in (0..h).rev() {
        for x in 0..w {
            let p = px[r * w + x];
            b.extend_from_slice(&[p as u8, (p >> 8) as u8, (p >> 16) as u8]);
        }
        b.resize(b.len() + stride - w * 3, 0);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_24() {
        let px = vec![0xff0000, 0x00ff00, 0x0000ff, 0x123456, 0xabcdef, 0];
        let b = encode(3, 2, &px);
        assert_eq!(decode(&b).unwrap(), Image { w: 3, h: 2, px });
    }

    #[test]
    fn rle8_palette() {
        // 4×2, two palette entries, RLE8: bottom row "0 0 1 1", top row
        // literal "1 0 1 0".
        let mut b = Vec::new();
        b.extend_from_slice(b"BM");
        b.extend_from_slice(&[0; 8]);
        b.extend_from_slice(&(54u32 + 8).to_le_bytes());
        b.extend_from_slice(&40u32.to_le_bytes());
        b.extend_from_slice(&4u32.to_le_bytes());
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&8u16.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(&[0; 12]);
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&[0; 4]);
        b.extend_from_slice(&[0, 0, 0, 0, 255, 255, 255, 0]);
        b.extend_from_slice(&[2, 0, 2, 1, 0, 0, 0, 4, 1, 0, 1, 0, 0, 1]);
        let img = decode(&b).unwrap();
        assert_eq!(img.px, vec![0xffffff, 0, 0xffffff, 0, 0, 0, 0xffffff, 0xffffff]);
    }
}
