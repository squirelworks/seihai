//! Gen-5 partial-XOR cipher (cleaned observation of 2.22+).
//!
//! Same LCG family as gen-3 (`it = it * 0x08088405 + key`, seed `0x04A80B38`).
//! Applied to the compressed body, not to individual messages.
//!
//! On send: compress first, then XOR.
//! On receive: undo XOR first, then decompress.
//!
//! Group limits:
//! - uncompressed (`0x02`): `ceil(len/4)` capped at 12
//! - zlib (`0x04`) / bz2 (`0x06`): `floor(len/4)` capped at 4
//!
//! A partial last group is XORed byte by byte.
//! Login frame is plain zlib with no compression byte and no XOR.

use super::gen3::{GEN3_MUL, GEN3_SEED};

/// Compression type byte for gen-5 frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gen5Comp {
    /// Uncompressed (`0x02`).
    None = 0x02,
    /// zlib (`0x04`).
    Zlib = 0x04,
    /// bz2 (`0x06`).
    Bz2 = 0x06,
}

/// Gen-5 partial-XOR state.
#[derive(Debug, Clone)]
pub struct Gen5 {
    it: u32,
    key: u8,
}

impl Gen5 {
    /// Create a new cipher. `key` is the login key byte.
    pub fn new(key: u8) -> Self {
        Self {
            it: GEN3_SEED,
            key,
        }
    }

    /// Resume from a known iterator state (for replaying captures that
    /// start mid-session).
    pub fn with_state(it: u32, key: u8) -> Self {
        Self { it, key }
    }

    /// Current iterator state.
    pub fn state(&self) -> u32 {
        self.it
    }

    /// XOR `body` in place. `comp` selects the group limit.
    pub fn xor_body(&mut self, body: &mut [u8], comp: Gen5Comp) {
        let limit = match comp {
            Gen5Comp::None => (body.len() + 3) / 4,
            Gen5Comp::Zlib | Gen5Comp::Bz2 => body.len() / 4,
        };
        let limit = limit.min(match comp {
            Gen5Comp::None => 12,
            Gen5Comp::Zlib | Gen5Comp::Bz2 => 4,
        });
        let mut off = 0;
        for _ in 0..limit {
            self.it = self.it.wrapping_mul(GEN3_MUL).wrapping_add(self.key as u32);
            let bytes = self.it.to_le_bytes();
            for b in &bytes {
                if off >= body.len() {
                    return;
                }
                body[off] ^= b;
                off += 1;
            }
        }
    }

    /// Encode a body: XOR then return. Caller supplies the already-compressed
    /// bytes and the compression type that was used.
    pub fn encode(&mut self, mut body: Vec<u8>, comp: Gen5Comp) -> Vec<u8> {
        self.xor_body(&mut body, comp);
        body
    }

    /// Decode a body: undo the XOR. Caller then decompresses.
    pub fn decode(&mut self, mut body: Vec<u8>, comp: Gen5Comp) -> Vec<u8> {
        self.xor_body(&mut body, comp);
        body
    }
}

/// Choose the gen-5 compression type for a body of the given length.
///
/// - ≥ 0x1000 → bz2
/// - ≥ 0x30 → zlib
/// - < 0x30 → uncompressed (caller may still prefer zlib if it is smaller)
pub fn choose_comp(len: usize) -> Gen5Comp {
    if len >= 0x1000 {
        Gen5Comp::Bz2
    } else if len >= 0x30 {
        Gen5Comp::Zlib
    } else {
        Gen5Comp::None
    }
}

/// Write a gen-5 frame: `u16be (1 + body.len())` + comp byte + body.
pub fn write_frame(comp: Gen5Comp, body: &[u8]) -> Vec<u8> {
    let total = 1 + body.len();
    let mut out = Vec::with_capacity(2 + total);
    out.extend_from_slice(&(total as u16).to_be_bytes());
    out.push(comp as u8);
    out.extend_from_slice(body);
    out
}

/// Read one gen-5 frame: `u16be (1+body)` + comp byte + XORed body.
///
/// Returns the decompressed body. `cipher` is advanced.
pub fn read_frame<R: std::io::Read>(
    r: &mut R,
    cipher: &mut Gen5,
) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut len_buf = [0u8; 2];
    r.read_exact(&mut len_buf)?;
    let total = u16::from_be_bytes(len_buf) as usize;
    if total == 0 {
        return Ok(Vec::new());
    }
    let mut raw = vec![0u8; total];
    r.read_exact(&mut raw)?;
    let comp = match raw[0] {
        0x02 => Gen5Comp::None,
        0x04 => Gen5Comp::Zlib,
        0x06 => Gen5Comp::Bz2,
        _ => Gen5Comp::None,
    };
    let mut body = raw[1..].to_vec();
    cipher.xor_body(&mut body, comp);
    match comp {
        Gen5Comp::None => Ok(body),
        Gen5Comp::Zlib => super::zlib_decompress(&body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        Gen5Comp::Bz2 => super::bz2_decompress(&body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_roundtrip() {
        let key = 0x42;
        let mut enc = Gen5::new(key);
        let mut dec = Gen5::new(key);
        let body = b"hello world, this is a test payload for gen5";
        let encoded = enc.encode(body.to_vec(), Gen5Comp::Zlib);
        assert_ne!(encoded, body);
        let decoded = dec.decode(encoded, Gen5Comp::Zlib);
        assert_eq!(decoded, body);
    }

    #[test]
    fn uncompressed_limit() {
        let mut g = Gen5::new(1);
        let mut body = vec![0u8; 100];
        g.xor_body(&mut body, Gen5Comp::None);
        // 12 groups * 4 = 48 bytes should be touched; rest stay zero.
        assert!(body[..48].iter().any(|&b| b != 0));
        assert!(body[48..].iter().all(|&b| b == 0));
    }

    #[test]
    fn frame_header() {
        let frame = write_frame(Gen5Comp::Zlib, b"abc");
        assert_eq!(&frame[..2], &4u16.to_be_bytes());
        assert_eq!(frame[2], 0x04);
        assert_eq!(&frame[3..], b"abc");
    }
}
