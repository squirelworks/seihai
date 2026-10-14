//! Minimal PNG writer matching the Graal client’s image export routines.
//! Supports 8-bit RGB and indexed (palette) images.
//! Uses the same CRC-32 as the client.

use super::crc32::crc32;
use std::io::{self, Write};

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

    // IDAT — uncompressed for simplicity (client uses a compressor; we emit a valid stored block)
    // For a minimal compatible writer we use a single uncompressed deflate block.
    let mut idat = Vec::new();
    // zlib header
    idat.extend_from_slice(&[0x78, 0x01]);
    let raw_len = pixels.len();
    // stored block
    idat.push(0x01); // final block, stored
    idat.extend_from_slice(&(raw_len as u16).to_le_bytes());
    idat.extend_from_slice(&(!raw_len as u16).to_le_bytes());
    idat.extend_from_slice(pixels);
    // adler32
    let adler = adler32(pixels);
    idat.extend_from_slice(&adler.to_be_bytes());
    write_chunk(w, b"IDAT", &idat)?;

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

    // Minimal stored IDAT
    let mut idat = Vec::new();
    idat.extend_from_slice(&[0x78, 0x01]);
    let raw_len = indices.len();
    idat.push(0x01);
    idat.extend_from_slice(&(raw_len as u16).to_le_bytes());
    idat.extend_from_slice(&(!raw_len as u16).to_le_bytes());
    idat.extend_from_slice(indices);
    let adler = adler32(indices);
    idat.extend_from_slice(&adler.to_be_bytes());
    write_chunk(w, b"IDAT", &idat)?;

    write_chunk(w, b"IEND", &[])?;
    Ok(())
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
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
    }
}
