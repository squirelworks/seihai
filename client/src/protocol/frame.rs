//! Frame helpers: zlib compress/decompress and gen-3/gen-5 wrapping.
//!
//! A frame is `u16be length` + body. The body is zlib for gen-2/3,
//! or a compression-type byte + body for gen-5.

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, Read, Write};

/// Compress `data` with zlib (default level).
pub fn zlib_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data)?;
    enc.finish()
}

/// Decompress a zlib body.
pub fn zlib_decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out)?;
    Ok(out)
}

/// Write a gen-2/3 frame: `u16be len` + zlib(body).
pub fn write_zlib_frame(body: &[u8]) -> io::Result<Vec<u8>> {
    let compressed = zlib_compress(body)?;
    let mut out = Vec::with_capacity(2 + compressed.len());
    out.extend_from_slice(&(compressed.len() as u16).to_be_bytes());
    out.extend_from_slice(&compressed);
    Ok(out)
}

/// Read one gen-2/3 frame from a reader: `u16be len` then that many bytes,
/// decompressed with zlib.
pub fn read_zlib_frame<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 2];
    r.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    zlib_decompress(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zlib_roundtrip() {
        let data = b"hello\nworld\n";
        let c = zlib_compress(data).unwrap();
        let d = zlib_decompress(&c).unwrap();
        assert_eq!(d, data);
    }

    #[test]
    fn frame_roundtrip() {
        let body = b"msg\n";
        let frame = write_zlib_frame(body).unwrap();
        let mut cur = std::io::Cursor::new(frame);
        let got = read_zlib_frame(&mut cur).unwrap();
        assert_eq!(got, body);
    }
}
