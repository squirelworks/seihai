//! A second player sees what the first one carries: for each pair of 1.39,
//! 2.05 and 2.22, player A lifts (CARRYSPRITE = the bush, then a walk update
//! with the carry sprite), player B reports what it sees on A; then A throws
//! (PLI_THROWCARRIED, CARRYSPRITE -1) and B reports again.
//!
//! Usage: cargo run -p seihai --example carry_probe -- <account prefix> [host:port] [hold seconds]
//!
//! With a hold time, A keeps the object that long before throwing (to look
//! at it from a GUI client on the same level).
//!
//! The lab listserver accepts any account and password. The default address
//! is the lab GServer.

use seihai::net::{carry_sprite_message, position_message_sprite, throw_carried_message, Connection};
use seihai::protocol::{ClientVersion, SessionState};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
type Rx = Receiver<std::io::Result<Vec<u8>>>;
fn pump(rx: &Rx, st: &mut SessionState, ms: u64) {
    let end = Instant::now() + Duration::from_millis(ms);
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(Ok(b)) => st.apply_body(&b),
            _ => break,
        }
    }
}
fn main() {
    let tag = std::env::args().nth(1).expect("account prefix");
    let addr = std::env::args().nth(2).unwrap_or_else(|| "127.0.0.1:14802".into());
    let hold = std::env::args().nth(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let vs = [ClientVersion::V139, ClientVersion::V205, ClientVersion::V222];
    for &va in &vs {
        for &vb in &vs {
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
            // Lift: the bush (sprite 261), then S = 18 facing right.
            a.send_messages(&[carry_sprite_message(Some(261)), position_message_sprite(33.5, 31.0, 3 + 4 * 18)]).unwrap();
            pump(&rb, &mut sb, 1000);
            let lifted: Vec<_> = sb.others.iter().map(|o| (o.carry, o.sprite.map(|s| s / 4))).collect();
            pump(&rb, &mut sb, hold * 1000);
            // Throw.
            a.send_messages(&[throw_carried_message(), carry_sprite_message(None), position_message_sprite(33.5, 31.0, 3)]).unwrap();
            pump(&rb, &mut sb, 1000);
            let thrown: Vec<_> = sb.others.iter().map(|o| (o.carry, o.sprite.map(|s| s / 4))).collect();
            a.shutdown().unwrap();
            drop(a);
            println!("A {} -> B {}: after lift B sees (carry, S) {:?}; after throw {:?}", va.label(), vb.label(), lifted, thrown);
        }
    }
}
