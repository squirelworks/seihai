//! Minimal PNG writer matching the Graal client’s image export routines.
//! Supports 8-bit RGB and indexed (palette) images.
//! Uses the same CRC-32 as the client.

use super::crc32::crc32;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, Read, Write};

fn write_chunk<W: Write>(w: &mut W, tag: &[u8; 4], data: &[u8]) -> io::Result<()> {
    let len = data.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(tag)?;
    w.write_all(data)?;
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(tag);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    w.write_all(&crc.to_be_bytes())?;
    Ok(())
}

/// Write an 8-bit RGB PNG (no palette).
/// `pixels` must be width * height * 3 bytes, row-major, no padding.
pub fn write_png_rgb<W: Write>(
    w: &mut W,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> io::Result<()> {
    if pixels.len() != (width as usize) * (height as usize) * 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pixel buffer size mismatch",
        ));
    }

    // Signature
    w.write_all(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])?;

    // IHDR
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(2); // color type: RGB
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_chunk(w, b"IHDR", &ihdr)?;

    write_chunk(w, b"IDAT", &zlib_rows(pixels, width as usize * 3)?)?;

    // IEND
    write_chunk(w, b"IEND", &[])?;
    Ok(())
}

/// Write an indexed-color PNG (palette).
/// `indices` is width * height bytes; `palette` is RGB triples (up to 256).
pub fn write_png_indexed<W: Write>(
    w: &mut W,
    width: u32,
    height: u32,
    indices: &[u8],
    palette: &[u8],
) -> io::Result<()> {
    if indices.len() != (width as usize) * (height as usize) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "index buffer size mismatch",
        ));
    }
    if palette.len() % 3 != 0 || palette.len() > 256 * 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid palette",
        ));
    }

    w.write_all(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])?;

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(3); // indexed
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(w, b"IHDR", &ihdr)?;

    write_chunk(w, b"PLTE", palette)?;

    write_chunk(w, b"IDAT", &zlib_rows(indices, width as usize)?)?;

    write_chunk(w, b"IEND", &[])?;
    Ok(())
}

/// zlib-compress image rows, each prefixed with filter type 0.
fn zlib_rows(data: &[u8], stride: usize) -> io::Result<Vec<u8>> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    for row in data.chunks(stride.max(1)) {
        enc.write_all(&[0])?;
        enc.write_all(row)?;
    }
    enc.finish()
}

/// Minimal PNG reader.
///
/// Handles 8-bit grayscale, RGB, palette, gray+alpha and RGBA images
/// (alpha dropped) with no interlace, and all five row filters.
/// Returns `(width, height, pixels)` where `pixels` is row-major RGB.
pub fn read_png_rgb(data: &[u8]) -> io::Result<(u32, u32, Vec<u8>)> {
    let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
    if data.len() < 8 || &data[..8] != &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Err(bad("not a PNG"));
    }
    let mut pos = 8;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut interlace = 0u8;
    let mut palette = Vec::new();
    let mut idat = Vec::new();
    while pos + 8 <= data.len() {
        let len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        let tag = &data[pos + 4..pos + 8];
        let chunk = data.get(pos + 8..pos + 8 + len).ok_or_else(|| bad("truncated chunk"))?;
        if tag == b"IHDR" && chunk.len() >= 13 {
            width = u32::from_be_bytes(chunk[0..4].try_into().unwrap());
            height = u32::from_be_bytes(chunk[4..8].try_into().unwrap());
            bit_depth = chunk[8];
            color_type = chunk[9];
            interlace = chunk[12];
        } else if tag == b"PLTE" {
            palette = chunk.to_vec();
        } else if tag == b"IDAT" {
            idat.extend_from_slice(chunk);
        } else if tag == b"IEND" {
            break;
        }
        pos += 12 + len;
    }
    let channels = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        _ => return Err(bad("unknown PNG color type")),
    };
    if bit_depth != 8 || interlace != 0 {
        return Err(bad("only 8-bit, non-interlaced PNGs are supported"));
    }
    if color_type == 3 && palette.is_empty() {
        return Err(bad("palette PNG without PLTE"));
    }
    let mut dec = ZlibDecoder::new(&idat[..]);
    let mut raw = Vec::new();
    dec.read_to_end(&mut raw)?;

    let bpp = channels;
    let stride = width as usize * bpp;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 3);
    let mut prev = vec![0u8; stride];
    let mut row = vec![0u8; stride];
    let mut i = 0;
    for _ in 0..height {
        let line = raw.get(i..i + 1 + stride).ok_or_else(|| bad("truncated image data"))?;
        i += 1 + stride;
        let filter = line[0];
        for x in 0..stride {
            let v = line[1 + x];
            let a = if x >= bpp { row[x - bpp] } else { 0 };
            let b = prev[x];
            let c = if x >= bpp { prev[x - bpp] } else { 0 };
            row[x] = match filter {
                0 => v,
                1 => v.wrapping_add(a),
                2 => v.wrapping_add(b),
                3 => v.wrapping_add(((a as u16 + b as u16) / 2) as u8),
                4 => v.wrapping_add(paeth(a, b, c)),
                _ => return Err(bad("unknown PNG filter")),
            };
        }
        for px in row.chunks(bpp) {
            match color_type {
                0 | 4 => pixels.extend_from_slice(&[px[0], px[0], px[0]]),
                2 | 6 => pixels.extend_from_slice(&px[..3]),
                _ => {
                    let p = px[0] as usize * 3;
                    pixels.extend_from_slice(palette.get(p..p + 3).unwrap_or(&[0, 0, 0]));
                }
            }
        }
        std::mem::swap(&mut prev, &mut row);
    }
    Ok((width, height, pixels))
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_roundtrip_header() {
        let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]; // 2x2
        let mut buf = Vec::new();
        write_png_rgb(&mut buf, 2, 2, &pixels).unwrap();
        assert!(buf.starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(buf.windows(4).any(|w| w == b"IHDR"));
        assert!(buf.windows(4).any(|w| w == b"IEND"));
        let (w, h, back) = read_png_rgb(&buf).unwrap();
        assert_eq!((w, h), (2, 2));
        assert_eq!(back, pixels);
    }
}
