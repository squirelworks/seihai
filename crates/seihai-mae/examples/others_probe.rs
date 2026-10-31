//! Two logins see each other: for each pair of 1.39, 2.05 and 2.22,
//! player A moves to (33.5, 31) facing right, player B reports where it
//! sees A, then A logs out and B reports what is left.
//!
//! Usage: cargo run -p seihai-mae --example others_probe -- <account prefix> [host:port]
//!
//! Each run logs in as `<prefix>a…` and `<prefix>b…`; the lab listserver
//! accepts any account. The default address is the lab GServer.

use seihai::net::Connection;
use seihai::protocol::{ClientVersion, SessionState};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
type Rx = Receiver<std::io::Result<Vec<u8>>>;
fn pump(rx: &Rx, st: &mut SessionState, ms: u64) {
    let end = Instant::now() + Duration::from_millis(ms);
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) { Ok(Ok(b)) => st.apply_body(&b), _ => break }
    }
}
fn main() {
    let tag = std::env::args().nth(1).expect("account prefix");
    let addr = std::env::args().nth(2).unwrap_or_else(|| "127.0.0.1:14802".into());
    let vs = [ClientVersion::V139, ClientVersion::V205, ClientVersion::V222];
    for &va in &vs { for &vb in &vs {
        let n = |p: &str| format!("{tag}{p}{}{}", va.label().replace('.', ""), vb.label().replace('.', ""));
        let mut b = Connection::connect(&addr).unwrap();
        b.login_versioned(vb, 0x42, &n("b"), "x").unwrap();
        let rb = b.spawn_reader().unwrap();
        let mut sb = SessionState::for_version(vb);
        pump(&rb, &mut sb, 1000);
        let mut a = Connection::connect(&addr).unwrap();
        a.login_versioned(va, 0x42, &n("a"), "x").unwrap();
        let ra = a.spawn_reader().unwrap();
        let mut sa = SessionState::for_version(va);
        pump(&ra, &mut sa, 1000);
        a.send_position(33.5, 31.0, 3).unwrap();
        pump(&rb, &mut sb, 1000);
        let seen: Vec<_> = sb.others.iter().map(|o| (o.x, o.y, o.dir, o.nick.is_some())).collect();
        a.shutdown().unwrap();
        drop(a);
        pump(&rb, &mut sb, 1500);
        println!("A {} -> B {}: B sees {:?}; after logout B has {} others", va.label(), vb.label(), seen, sb.others.len());
    }}
}
