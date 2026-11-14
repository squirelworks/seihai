//! Lab tool (#458 pass 2): read a pcap of a Graal3D UDP link and print each
//! client-to-server data packet's game section (protocol 21/24): the reported
//! position and the control object's changed sections, with the 5.12 client's
//! section numbers (2 velocity, 5 transform, 9 chat).
//!
//!   cargo run --example g3d_pcap -- capture.pcap 14831
use seihai_ato::torque::net::bits::BitReader;
use seihai_ato::torque::net::ghosts::GameObjectUpdate;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("pcap");
    let port: u16 = a.get(2).map_or(14831, |p| p.parse().unwrap());
    let mut off = 24;
    let mut t0 = None;
    while off + 16 <= data.len() {
        let sec = u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as f64;
        let usec = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as f64;
        let len = u32::from_le_bytes(data[off + 8..off + 12].try_into().unwrap()) as usize;
        let pkt = &data[off + 16..off + 16 + len];
        off += 16 + len;
        let t = sec + usec / 1e6;
        if pkt.len() < 42 || pkt[12..14] != [8, 0] || pkt[23] != 17 {
            continue;
        }
        let ihl = (pkt[14] & 15) as usize * 4;
        let u = 14 + ihl;
        let dport = u16::from_be_bytes([pkt[u + 2], pkt[u + 3]]);
        if dport != port {
            continue;
        }
        let body = &pkt[u + 8..];
        let t0 = *t0.get_or_insert(t);
        if let Some(line) = decode(body) {
            println!("{t:.3} +{:.3} {line}", t - t0);
        }
    }
}

fn decode(p: &[u8]) -> Option<String> {
    let mut r = BitReader::new(p);
    if !r.flag().ok()? {
        return None; // handshake
    }
    let _bit = r.int(1).ok()?;
    let seq = r.int(9).ok()?;
    let _hi = r.int(9).ok()?;
    let ty = r.int(2).ok()?;
    let ack_bytes = r.int(3).ok()?;
    r.int(ack_bytes * 8).ok()?;
    if ty != 0 {
        return None;
    }
    for _ in 0..2 {
        if r.flag().ok()? {
            r.int(10).ok()?;
            r.int(10).ok()?;
        }
    }
    let fp = r.flag().ok()?;
    let pos = [r.f32().ok()?, r.f32().ok()?, r.f32().ok()?];
    let mut s = format!("seq {seq} fp {} pos ({:.3}, {:.3}, {:.3})", fp as u8, pos[0], pos[1], pos[2]);
    if r.flag().ok()? {
        s += &format!(" control {}", r.int(12).ok()?);
    }
    if r.flag().ok()? {
        match GameObjectUpdate::read(&mut r, None) {
            Ok(u) if u == GameObjectUpdate::default() => s += " update: empty",
            Ok(u) => {
                s += " update:";
                if let Some(n) = &u.name { s += &format!(" [1 name {}]", String::from_utf8_lossy(n)); }
                if let Some(v) = u.velocity { s += &format!(" [2 velocity {:.3} {:.3} {:.3}]", v[0], v[1], v[2]); }
                if let Some(n) = &u.shape { s += &format!(" [3 shape {}]", String::from_utf8_lossy(n)); }
                if let Some(t) = u.transform {
                    s += &format!(
                        " [5 transform pos {:.3} {:.3} {:.3} axis {:.3} {:.3} {:.3} angle {:.4}]",
                        t.position[0], t.position[1], t.position[2], t.axis[0], t.axis[1], t.axis[2], t.angle
                    );
                }
                if let Some(c) = &u.chat { s += &format!(" [9 chat {:?}]", String::from_utf8_lossy(c)); }
                if let Some(f) = u.flags { s += &format!(" [13 flags {f:#x}]"); }
                if u.movetype.is_some() || u.movement.is_some() || u.skin.is_some() || u.attributes.iter().any(|a| a.is_some()) || u.nickname.is_some() || u.script_classes.is_some() || u.shape_data.is_some() {
                    s += &format!(" [other {u:?}]");
                }
            }
            Err(e) => s += &format!(" update: unreadable ({e:?})"),
        }
    }
    Some(s)
}
