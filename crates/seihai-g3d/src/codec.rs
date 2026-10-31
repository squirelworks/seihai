//! The Graal3D wire codec: the 2D Graal `+32` text codec in zlib frames,
//! with no key byte and no scramble.
//!
//! - A frame is a big-endian `u16` length and a zlib block.
//! - The block holds messages, each `c(id)` + payload + `\n`.
//! - Text that has line breaks (level files, admin messages) carries `§`
//!   (0xA7) in place of each `\n`.

use seihai_core::protocol::{zlib_compress, zlib_decompress};
use std::io;

/// The line-break stand-in inside a message.
pub const LINE_BREAK: u8 = 0xA7;

/// `c(v)`: one byte, `v + 32`.
pub fn c(v: u8) -> u8 {
    v.wrapping_add(32)
}

/// The value of a `c` byte.
pub fn uc(b: u8) -> u8 {
    b.wrapping_sub(32)
}

/// `sN(v)`: `n` bytes of 7 bits each, most significant first, each `+ 32`.
pub fn put_sn(out: &mut Vec<u8>, v: u32, n: usize) {
    for i in (0..n).rev() {
        out.push(((v >> (7 * i)) & 0x7f) as u8 + 32);
    }
}

/// `str(s)`: `c(len)` then the bytes. Text longer than 223 bytes is cut,
/// since the length must fit one `c` byte.
pub fn put_str(out: &mut Vec<u8>, s: &[u8]) {
    let s = &s[..s.len().min(223)];
    out.push(c(s.len() as u8));
    out.extend_from_slice(s);
}

/// A reader over one message payload.
pub struct Reader<'a> {
    b: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Reader { b, pos: 0 }
    }

    pub fn done(&self) -> bool {
        self.pos >= self.b.len()
    }

    pub fn c(&mut self) -> Option<u8> {
        let v = uc(*self.b.get(self.pos)?);
        self.pos += 1;
        Some(v)
    }

    pub fn sn(&mut self, n: usize) -> Option<u32> {
        let bytes = self.b.get(self.pos..self.pos + n)?;
        self.pos += n;
        Some(
            bytes
                .iter()
                .fold(0u32, |v, &b| (v << 7) | (uc(b) & 0x7f) as u32),
        )
    }

    pub fn bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.pos..self.pos + n)?;
        self.pos += n;
        Some(s)
    }

    pub fn str(&mut self) -> Option<&'a [u8]> {
        let n = self.c()? as usize;
        self.bytes(n)
    }

    pub fn rest(&mut self) -> &'a [u8] {
        let s = &self.b[self.pos.min(self.b.len())..];
        self.pos = self.b.len();
        s
    }
}

/// One message: `c(id)` and the payload, without the `\n`.
pub fn message(id: u8, payload: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(payload.len() + 1);
    m.push(c(id));
    m.extend_from_slice(payload);
    m
}

/// Pack messages into one frame. A message must not hold `\n`.
pub fn frame(messages: &[Vec<u8>]) -> io::Result<Vec<u8>> {
    let mut raw = Vec::new();
    for m in messages {
        debug_assert!(!m.contains(&b'\n'));
        raw.extend_from_slice(m);
        raw.push(b'\n');
    }
    let z = zlib_compress(&raw)?;
    if z.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame is over 64 KB compressed",
        ));
    }
    let mut out = Vec::with_capacity(z.len() + 2);
    out.extend_from_slice(&(z.len() as u16).to_be_bytes());
    out.extend_from_slice(&z);
    Ok(out)
}

/// Collects TCP bytes and yields whole messages as `(id, payload)`.
#[derive(Default)]
pub struct Deframer {
    buf: Vec<u8>,
}

impl Deframer {
    pub fn push(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// The messages of every whole frame received so far.
    pub fn messages(&mut self) -> io::Result<Vec<(u8, Vec<u8>)>> {
        let mut out = Vec::new();
        while self.buf.len() >= 2 {
            let n = u16::from_be_bytes([self.buf[0], self.buf[1]]) as usize;
            if self.buf.len() < 2 + n {
                break;
            }
            let raw = zlib_decompress(&self.buf[2..2 + n])?;
            self.buf.drain(..2 + n);
            for line in raw.split(|&b| b == b'\n') {
                if let Some((&id, payload)) = line.split_first() {
                    out.push((uc(id), payload.to_vec()));
                }
            }
        }
        Ok(out)
    }
}

/// Text with `§` turned back into `\n`.
pub fn unescape_lines(b: &[u8]) -> Vec<u8> {
    b.iter()
        .map(|&x| if x == LINE_BREAK { b'\n' } else { x })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_strings() {
        let mut v = Vec::new();
        put_sn(&mut v, 8000, 2);
        put_str(&mut v, b"body.x");
        assert_eq!(v, b"^`&body.x");
        let mut r = Reader::new(&v);
        assert_eq!(r.sn(2), Some(8000));
        assert_eq!(r.str(), Some(&b"body.x"[..]));
        assert!(r.done());
    }

    #[test]
    fn frames_round_trip() {
        let f = frame(&[message(0, b"GR3DV002%alice#pw1"), message(1, b"/   ")]).unwrap();
        let mut d = Deframer::default();
        d.push(&f[..3]);
        assert!(d.messages().unwrap().is_empty());
        d.push(&f[3..]);
        let m = d.messages().unwrap();
        assert_eq!(
            m,
            vec![(0, b"GR3DV002%alice#pw1".to_vec()), (1, b"/   ".to_vec())]
        );
    }

    #[test]
    fn line_breaks() {
        assert_eq!(unescape_lines(b"a\xa7b"), b"a\nb");
    }
}
