//! Fault-injecting proxy for the private Seihai lab (loopback only).
//!
//! faultproxy LISTEN_PORT UPSTREAM_PORT VERSION MODE [P] [AFTER_S] [SEED]
//!
//! Client bytes go to the upstream gserver unchanged. Server frames are
//! decoded with Seihai's own codecs (gen-5 XOR, zlib, bz2), changed as MODE
//! says, and re-encoded with a fresh cipher for the client, so the client
//! sees valid frames carrying bad content (or bad frames, for `framing`).
//!
//! Modes: mutate, inject, oversize, bomb, framing, flood, all.

use seihai_core::protocol::{ClientVersion, EncryptionGen, Gen5, Gen5Comp};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
    fn chance(&mut self, p: f64) -> bool {
        (self.next() % 1_000_000) as f64 / 1_000_000.0 < p
    }
    fn rbytes(&mut self, max: u64, printable: bool) -> Vec<u8> {
        let n = self.below(max) as usize;
        self.bytes(n, printable)
    }
    fn bytes(&mut self, n: usize, printable: bool) -> Vec<u8> {
        (0..n)
            .map(|_| {
                let b = self.next() as u8;
                if printable { 0x20 + b % 0x5E } else { b }
            })
            .collect()
    }
}

#[derive(Clone)]
struct Cfg {
    version: ClientVersion,
    mode: String,
    p: f64,
    after: Duration,
    seed: u64,
}

fn log(conn: u64, what: &str) {
    eprintln!("[{:>6.1}] conn {conn}: {what}", now());
}

fn now() -> f64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn read_exact_or_eof(s: &mut TcpStream, n: usize) -> Option<Vec<u8>> {
    let mut b = vec![0u8; n];
    s.read_exact(&mut b).ok()?;
    Some(b)
}

fn zlib(b: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(b).unwrap();
    e.finish().unwrap()
}

fn bz2(b: &[u8]) -> Vec<u8> {
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
    e.write_all(b).unwrap();
    e.finish().unwrap()
}

/// Server-side reader and client-side writer for one connection.
struct Codec {
    gen5: bool,
    long: bool,
    rx: Option<Gen5>,
    tx: Option<Gen5>,
}

impl Codec {
    /// Read one server frame; return the decompressed body.
    fn read(&mut self, s: &mut TcpStream) -> Option<Vec<u8>> {
        let h = read_exact_or_eof(s, 2)?;
        let mut len = u16::from_be_bytes([h[0], h[1]]) as usize;
        if self.long && len == 0xFFFF {
            let l = read_exact_or_eof(s, 4)?;
            len = u32::from_be_bytes([l[0], l[1], l[2], l[3]]) as usize;
        }
        let raw = read_exact_or_eof(s, len)?;
        if !self.gen5 {
            return seihai_core::protocol::zlib_decompress(&raw).ok();
        }
        if raw.is_empty() {
            return Some(Vec::new());
        }
        let comp = match raw[0] {
            2 => Gen5Comp::None,
            4 => Gen5Comp::Zlib,
            6 => Gen5Comp::Bz2,
            _ => return None,
        };
        let mut body = raw[1..].to_vec();
        self.rx.as_mut().unwrap().xor_body(&mut body, comp);
        match comp {
            Gen5Comp::None => Some(body),
            Gen5Comp::Zlib => seihai_core::protocol::zlib_decompress(&body).ok(),
            Gen5Comp::Bz2 => seihai_core::protocol::bz2_decompress(&body).ok(),
        }
    }

    fn len_header(&self, n: usize) -> Vec<u8> {
        if self.long && n >= 0xFFFF {
            let mut v = vec![0xFF, 0xFF];
            v.extend_from_slice(&(n as u32).to_be_bytes());
            v
        } else {
            (n.min(0xFFFF) as u16).to_be_bytes().to_vec()
        }
    }

    /// Encode a body for the client, compressed with `comp` (gen-5) or zlib.
    fn frame(&mut self, body: &[u8], comp: Gen5Comp) -> Vec<u8> {
        if !self.gen5 {
            let z = zlib(body);
            let mut out = self.len_header(z.len());
            out.extend(z);
            return out;
        }
        let mut payload = match comp {
            Gen5Comp::None => body.to_vec(),
            Gen5Comp::Zlib => zlib(body),
            Gen5Comp::Bz2 => bz2(body),
        };
        self.tx.as_mut().unwrap().xor_body(&mut payload, comp);
        let mut out = self.len_header(payload.len() + 1);
        out.push(comp as u8);
        out.extend(payload);
        out
    }

    /// One body as one or more frames, each small enough for its length field.
    fn frames(&mut self, body: &[u8], comp: Gen5Comp) -> Vec<u8> {
        if self.long || body.len() <= 30_000 {
            return self.frame(body, comp);
        }
        let mut out = Vec::new();
        for chunk in body.chunks(30_000) {
            out.extend(self.frame(chunk, comp));
        }
        out
    }

    /// A pre-compressed payload (for bombs).
    fn frame_raw(&mut self, mut payload: Vec<u8>, comp: Gen5Comp) -> Vec<u8> {
        if !self.gen5 {
            let mut out = self.len_header(payload.len());
            out.extend(payload);
            return out;
        }
        self.tx.as_mut().unwrap().xor_body(&mut payload, comp);
        let mut out = self.len_header(payload.len() + 1);
        out.push(comp as u8);
        out.extend(payload);
        out
    }
}

/// Split a body into messages (`id payload \n`, raw blocks after id 100).
fn split(body: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    let mut raw_next: Option<usize> = None;
    while i < body.len() {
        if let Some(n) = raw_next.take() {
            let end = (i + n).min(body.len());
            out.push(body[i..end].to_vec());
            i = end;
            continue;
        }
        let end = body[i..].iter().position(|&b| b == b'\n').map_or(body.len(), |p| i + p);
        let m = body[i..end].to_vec();
        if m.first() == Some(&(100 + 32)) && m.len() >= 4 {
            let n = ((m[1] as usize - 32) << 14) | ((m[2] as usize - 32) << 7) | (m[3] as usize - 32);
            raw_next = Some(n);
        }
        out.push(m);
        i = end + 1;
    }
    out
}

fn join(msgs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for m in msgs {
        out.extend_from_slice(m);
        out.push(b'\n');
    }
    out
}

fn gchar(v: usize) -> u8 {
    (v.min(223) + 32) as u8
}

fn gint3(v: usize) -> [u8; 3] {
    let v = v.min((1 << 21) - 1);
    [((v >> 14) & 0x7F) as u8 + 32, ((v >> 7) & 0x7F) as u8 + 32, (v & 0x7F) as u8 + 32]
}

/// Hand-made hostile messages: big counts, long strings, raw blocks that lie.
fn hostile(r: &mut Rng) -> Vec<Vec<u8>> {
    let big = 1 + r.below(60_000) as usize;
    let s = r.bytes(big, true);
    let pick = r.below(16);
    let mut m = Vec::new();
    match pick {
        0 => {
            // PLO_LEVELBOARD with a huge line
            m.push(vec![32]);
            m[0].extend(r.bytes(big, true));
        }
        1 => {
            // PLO_RAWDATA that announces 2 MB, then a short block
            let mut a = vec![100 + 32];
            a.extend(gint3(2_000_000));
            m.push(a);
            let mut b = vec![101 + 32];
            b.extend(r.bytes(10, false));
            m.push(b);
        }
        2 => {
            // PLO_RAWDATA of a plausible size followed by a board of the wrong size
            let n = r.below(20_000) as usize;
            let mut a = vec![100 + 32];
            a.extend(gint3(n + 1));
            m.push(a);
            let mut b = vec![101 + 32];
            b.extend(r.bytes(n, false));
            m.push(b);
        }
        3 => {
            // PLO_LEVELNAME (6) with a long / odd name
            let mut a = vec![6 + 32];
            a.extend(&s[..s.len().min(5000)]);
            m.push(a);
        }
        4 => {
            // PLO_PLAYERPROPS (9) / OTHERPLPROPS (8) with random prop bytes
            let mut a = vec![[9u8, 8][r.below(2) as usize] + 32];
            if a[0] == 8 + 32 {
                a.extend([32 + r.below(200) as u8, 32 + r.below(200) as u8]);
            }
            a.extend(r.rbytes(3000, false).into_iter().map(|b| b.max(32).min(255).wrapping_add(0)).filter(|&b| b != b'\n'));
            m.push(a);
        }
        5 => {
            // PLO_NPCPROPS (3) with a huge image/script prop
            let mut a = vec![3 + 32];
            a.extend([32, 32, 32 + r.below(90) as u8]);
            a.extend(s.iter().take(big).filter(|&&b| b != b'\n'));
            m.push(a);
        }
        6 => {
            // PLO_BADDYPROPS (2) junk
            let mut a = vec![2 + 32];
            a.extend(r.rbytes(500, true));
            m.push(a);
        }
        7 => {
            // PLO_LEVELCHEST (4), PLO_LEVELLINK (1), PLO_LEVELSIGN (5) junk
            for id in [1u8, 4, 5] {
                let mut a = vec![id + 32];
                a.extend(r.rbytes(4000, true));
                m.push(a);
            }
        }
        8 => {
            // Chat / to-all / PM / server text with huge text
            for id in [13u8, 37, 82] {
                let mut a = vec![id + 32];
                a.extend([32, 33]);
                a.extend(&s);
                m.push(a);
            }
        }
        9 => {
            // PLAYERWARP (14) / WARP2 (49) out of range
            let mut a = vec![14 + 32, 255, 255];
            a.extend(b"nowhere_level_that_is_not_there.nw");
            m.push(a);
            let mut b = vec![49 + 32, 255, 255, 255, 255, 255];
            b.extend(b"stgm.gmap");
            m.push(b);
        }
        10 => {
            // Board modify (24) with a huge rectangle
            let mut a = vec![24 + 32, 32, 32, 255, 255];
            a.extend(r.rbytes(30_000, true));
            m.push(a);
        }
        11 => {
            // SHOWIMG (32), EXPLOSION (36), BOMBADD (11), ARROWADD (19), HORSEADD (17) junk
            for id in [32u8, 36, 11, 19, 17, 20, 21, 22] {
                let mut a = vec![id + 32];
                a.extend(r.rbytes(300, false).into_iter().filter(|&b| b != b'\n'));
                m.push(a);
            }
        }
        12 => {
            // PLO_FILE (102) via raw data, huge name length
            let body_len = r.below(200_000) as usize;
            let mut a = vec![100 + 32];
            a.extend(gint3(body_len + 1 + 6 + 1 + 1));
            m.push(a);
            let mut b = vec![102 + 32];
            b.extend([32, 32, 32, 32, 32]); // mod time
            b.push(255); // name length
            b.extend(r.bytes(body_len, false));
            b.push(b'\n');
            m.push(b);
        }
        13 => {
            // Weapon add (33) / NPC weapon script with long script
            let mut a = vec![33 + 32, 32 + 20];
            a.extend(b"-stressweapon");
            a.push(32);
            a.extend(&s);
            m.push(a);
        }
        14 => {
            // Server warp (178) to garbage (ignored unless following warps)
            let mut a = vec![178u8.wrapping_add(32)];
            a.extend(&s[..s.len().min(200)]);
            m.push(a);
        }
        _ => {
            // Random id, random payload
            let mut a = vec![32 + r.below(224) as u8];
            a.extend(r.rbytes(4000, false).into_iter().filter(|&b| b != b'\n'));
            m.push(a);
        }
    }
    m
}

fn mutate(r: &mut Rng, msgs: &mut Vec<Vec<u8>>, p: f64) -> usize {
    let mut n = 0;
    for m in msgs.iter_mut() {
        if m.is_empty() || !r.chance(p) {
            continue;
        }
        n += 1;
        match r.below(5) {
            0 => {
                // flip bytes (never into '\n' outside raw blocks)
                for _ in 0..=r.below(4) {
                    let i = r.below(m.len() as u64) as usize;
                    m[i] = (r.next() as u8).max(11);
                }
            }
            1 => {
                let k = r.below(m.len() as u64) as usize;
                m.truncate(k.max(1));
            }
            2 => {
                let extra = r.below(2000) as usize;
                m.extend(r.bytes(extra, false).into_iter().filter(|&b| b != b'\n'));
            }
            3 => {
                m[0] = 32 + r.below(224) as u8;
            }
            _ => {
                for b in m.iter_mut().skip(1) {
                    if r.chance(0.3) {
                        *b = if r.chance(0.5) { 255 } else { 32 };
                    }
                }
            }
        }
    }
    n
}

fn bomb(r: &mut Rng, c: &mut Codec) -> (Vec<u8>, String) {
    // zlib: ~1030:1. 60 KB on the wire is ~60 MB of body. bz2: much more.
    let size = [8usize << 20, 64 << 20, 256 << 20][r.below(3) as usize];
    let mut body = vec![32 + 31u8]; // a chat-ish id
    body.resize(size, b'A');
    if c.gen5 && r.chance(0.5) {
        let z = bz2(&body);
        let desc = format!("bz2 bomb {} MB body in {} bytes", size >> 20, z.len());
        (c.frame_raw(z, Gen5Comp::Bz2), desc)
    } else {
        let z = zlib(&body);
        if z.len() >= 0xFFFF && !c.long {
            let body2 = &body[..60 << 20];
            let z2 = zlib(body2);
            return (c.frame_raw(z2.clone(), Gen5Comp::Zlib), format!("zlib bomb 60 MB body in {} bytes", z2.len()));
        }
        let desc = format!("zlib bomb {} MB body in {} bytes", size >> 20, z.len());
        (c.frame_raw(z, Gen5Comp::Zlib), desc)
    }
}

fn framing(r: &mut Rng, c: &mut Codec) -> (Vec<u8>, String, bool) {
    // Returns (bytes, description, close_after)
    match r.below(6) {
        0 => (vec![0, 0], "zero-length frame".into(), false),
        1 => {
            let mut v = vec![0x00, 0x10, 0x09];
            v.extend(r.bytes(15, false));
            (v, "bad compression byte".into(), false)
        }
        2 => {
            let mut v = vec![0xFF, 0xFE];
            v.extend(r.bytes(0xFFFE, false));
            (v, "64 KB frame of junk".into(), false)
        }
        3 => {
            let mut v = vec![0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF];
            v.extend(r.bytes(100, false));
            (v, "long length 2 GB (6.x) / 64 KB junk".into(), false)
        }
        4 => {
            let f = c.frame(b"!hello\n", Gen5Comp::Zlib);
            let cut = 1 + r.below(f.len() as u64 - 1) as usize;
            (f[..cut].to_vec(), format!("frame cut at {cut} then close"), true)
        }
        _ => {
            let mut v = vec![0x00, 0x05];
            v.extend([0x04, 0x78, 0x9C, 0x00, 0x00]);
            (v, "zlib stream that ends early".into(), false)
        }
    }
}

fn handle(client: TcpStream, upstream_port: u16, cfg: Cfg, id: u64) {
    let mut server = match TcpStream::connect(("127.0.0.1", upstream_port)) {
        Ok(s) => s,
        Err(e) => {
            log(id, &format!("upstream connect failed: {e}"));
            return;
        }
    };
    let _ = server.set_nodelay(true);
    let _ = client.set_nodelay(true);
    let mut cl_r = client.try_clone().unwrap();
    let mut cl_w = client;
    // First client frame: the login line (plain zlib). Read it to get the key.
    let Some(h) = read_exact_or_eof(&mut cl_r, 2) else { return };
    let len = u16::from_be_bytes([h[0], h[1]]) as usize;
    let Some(body) = read_exact_or_eof(&mut cl_r, len) else { return };
    let login = seihai_core::protocol::zlib_decompress(&body).unwrap_or_default();
    let key = login.get(1).map_or(0, |k| k.wrapping_sub(32));
    let mut first = h.clone();
    first.extend(&body);
    server.write_all(&first).unwrap();
    let v = cfg.version;
    let gen5 = v.encryption() == EncryptionGen::Gen5;
    let pr = &v.profile().protocol;
    let mk = || if pr.gen5_round_down { Gen5::new(key).round_down() } else { Gen5::new(key) };
    let mut codec = Codec { gen5, long: pr.gen5_long_frames, rx: gen5.then(mk), tx: gen5.then(mk) };
    log(id, &format!("login {} key {key} gen5 {gen5} long {}", v.label(), codec.long));
    // Client -> server: pass through.
    let mut sv_w = server.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut buf = [0u8; 16384];
        loop {
            match cl_r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if sv_w.write_all(&buf[..n]).is_err() {
                        break;
                    }
                }
            }
        }
        let _ = sv_w.shutdown(std::net::Shutdown::Both);
    });
    let mut r = Rng(cfg.seed ^ (id.wrapping_mul(0x9E37_79B9_7F4A_7C15)) | 1);
    let start = Instant::now();
    let mut faults = 0u64;
    let mut recent: Vec<Vec<u8>> = Vec::new();
    let mut sv_r = server;
    // Server frames are decoded on their own thread so that faults can also
    // go out while the server is quiet (every 200 ms).
    let (btx, brx) = std::sync::mpsc::channel::<Option<Vec<u8>>>();
    let mut rd = sv_r.try_clone().unwrap();
    let mut rcodec = Codec { gen5, long: codec.long, rx: codec.rx.take(), tx: None };
    std::thread::spawn(move || loop {
        let b = rcodec.read(&mut rd);
        let end = b.is_none();
        if btx.send(b).is_err() || end {
            break;
        }
    });
    loop {
        let body = match brx.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(b)) => b,
            Ok(None) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log(id, "server closed or sent an unreadable frame");
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Vec::new(),
        };
        let idle = body.is_empty();
        let active = start.elapsed() >= cfg.after;
        let mut msgs = split(&body);
        for m in &msgs {
            if recent.len() >= 300 {
                recent.remove(0);
            }
            recent.push(m.clone());
        }
        let mode = if cfg.mode == "all" {
            ["mutate", "inject", "oversize", "bomb", "framing", "flood"][r.below(6) as usize].to_string()
        } else {
            cfg.mode.clone()
        };
        let mut out: Vec<u8> = Vec::new();
        let mut close = false;
        if active {
            match mode.as_str() {
                "mutate" => {
                    if idle && !recent.is_empty() && r.chance(cfg.p) {
                        for _ in 0..=r.below(5) {
                            let m = recent[r.below(recent.len() as u64) as usize].clone();
                            if !matches!(m.first(), Some(&132) | Some(&133) | Some(&134)) {
                                msgs.push(m);
                            }
                        }
                    }
                    let n = mutate(&mut r, &mut msgs, if idle { 1.0 } else { cfg.p });
                    if n > 0 {
                        faults += 1;
                        log(id, &format!("mutated {n} of {} messages", msgs.len()));
                    }
                }
                "inject" | "oversize" => {
                    if r.chance(cfg.p) {
                        let h = hostile(&mut r);
                        faults += 1;
                        log(id, &format!("{mode}: {} hostile message(s), first id {} len {}", h.len(), h[0][0].wrapping_sub(32), h[0].len()));
                        let at = r.below(msgs.len() as u64 + 1) as usize;
                        if mode == "oversize" {
                            // one oversized body: pad with many hostile messages
                            let mut all = h;
                            for _ in 0..r.below(40) {
                                all.extend(hostile(&mut r));
                            }
                            let b = join(&all);
                            let comp = if gen5 && b.len() > 0x1000 { Gen5Comp::Bz2 } else { Gen5Comp::Zlib };
                            out.extend(codec.frames(&b, comp));
                        } else {
                            for (k, m) in h.into_iter().enumerate() {
                                msgs.insert(at + k, m);
                            }
                        }
                    }
                }
                "bomb" => {
                    if r.chance(cfg.p / 4.0) {
                        let (b, desc) = bomb(&mut r, &mut codec);
                        faults += 1;
                        log(id, &desc);
                        out.extend(b);
                    }
                }
                "framing" => {
                    if r.chance(cfg.p / 4.0) {
                        let (b, desc, c) = framing(&mut r, &mut codec);
                        faults += 1;
                        log(id, &format!("framing: {desc}"));
                        let _ = cl_w.write_all(&b);
                        // After raw junk the cipher states no longer match; end it.
                        close = true;
                        let _ = c;
                    }
                }
                "flood" => {
                    if r.chance(cfg.p) && !msgs.is_empty() {
                        let k = r.below(msgs.len() as u64) as usize;
                        let times = 200 + r.below(5000) as usize;
                        let m = msgs[k].clone();
                        if m.first() != Some(&(100 + 32)) && m.first() != Some(&(101 + 32)) && m.first() != Some(&(102 + 32)) {
                            faults += 1;
                            log(id, &format!("flood: id {} x{times}", m[0].wrapping_sub(32)));
                            let b = join(&vec![m; times]);
                            let comp = if gen5 && b.len() >= 0x1000 { Gen5Comp::Bz2 } else { Gen5Comp::Zlib };
                            out.extend(codec.frames(&b, comp));
                        }
                    }
                }
                _ => {}
            }
        }
        if close {
            std::thread::sleep(Duration::from_millis(500));
            let _ = cl_w.shutdown(std::net::Shutdown::Both);
            let _ = sv_r.shutdown(std::net::Shutdown::Both);
            break;
        }
        if idle && msgs.is_empty() {
            if !out.is_empty() && cl_w.write_all(&out).is_err() {
                log(id, "client gone");
                break;
            }
            continue;
        }
        let b = join(&msgs);
        let b = if b.len() > body.len() && !body.ends_with(b"\n") { b[..b.len() - 1].to_vec() } else { b };
        let comp = if gen5 { if b.len() >= 0x1000 { Gen5Comp::Bz2 } else { Gen5Comp::Zlib } } else { Gen5Comp::Zlib };
        // A long body for a short-frame client is split so each frame fits.
        if !codec.long && b.len() > 30_000 {
            for chunk in b.chunks(30_000) {
                out.extend(codec.frame(chunk, comp));
            }
        } else {
            out.extend(codec.frame(&b, comp));
        }
        if cl_w.write_all(&out).is_err() {
            log(id, "client gone");
            break;
        }
    }
    log(id, &format!("closed after {:.1} s, {faults} fault(s)", start.elapsed().as_secs_f64()));
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 5 {
        eprintln!("usage: faultproxy LISTEN UPSTREAM VERSION MODE [P] [AFTER_S] [SEED]");
        std::process::exit(2);
    }
    let listen: u16 = a[1].parse().unwrap();
    let up: u16 = a[2].parse().unwrap();
    let version = ClientVersion::from_label(&a[3]).expect("version");
    let cfg = Cfg {
        version,
        mode: a[4].clone(),
        p: a.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.05),
        after: Duration::from_secs_f64(a.get(6).and_then(|s| s.parse().ok()).unwrap_or(8.0)),
        seed: a.get(7).and_then(|s| s.parse().ok()).unwrap_or(1),
    };
    let l = TcpListener::bind(("127.0.0.1", listen)).expect("bind");
    let n = Arc::new(AtomicU64::new(0));
    now();
    eprintln!("faultproxy {listen} -> 127.0.0.1:{up} as {} mode {} p {}", version.label(), cfg.mode, cfg.p);
    for c in l.incoming().flatten() {
        let id = n.fetch_add(1, Ordering::SeqCst);
        let cfg = cfg.clone();
        std::thread::spawn(move || handle(c, up, cfg, id));
    }
}
