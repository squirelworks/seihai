//! Another player's throw reaches this one through the server: for each pair
//! of 1.39, 2.05 and 2.22, player A lifts the bush at (33.5, 31) facing east
//! and throws it; player B stands at (36, 31). B's side runs what the game
//! loop runs: `PLO_THROWCARRIED` starts the flight from A's position, and the
//! flight is judged against B with the 2.05 rules. Prints B's hearts before
//! and after, and whether a claim would go out at one half heart.
//!
//! Usage: cargo run -p seihai-mae --example throw_hit_probe -- <account prefix> [host:port]
//!
//! The lab listserver accepts any account and password. The default address
//! is the lab GServer.

use seihai::net::{carry_sprite_message, position_message_sprite, throw_carried_message, Connection};
use seihai::offline::OfflineSession;
use seihai::protocol::{ClientVersion, SessionState, COMBAT_205, DEATH_205};
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
fn victim(half: i32) -> OfflineSession {
    let mut s = OfflineSession::new();
    s.combat.set_rules(Some(COMBAT_205));
    s.life.set_rules(Some(DEATH_205));
    s.set_max_hearts_half(6);
    s.set_hearts_half(half);
    s.player_x = 36.0;
    s.player_y = 31.0;
    s
}
fn main() {
    let tag = std::env::args().nth(1).expect("account prefix");
    let addr = std::env::args().nth(2).unwrap_or_else(|| "127.0.0.1:14802".into());
    let vs = [ClientVersion::V139, ClientVersion::V205, ClientVersion::V222];
    for &va in &vs {
        for &vb in &vs {
            let n = |p: &str| format!("{tag}{p}{}{}", va.label().replace('.', ""), vb.label().replace('.', ""));
            let mut b = Connection::connect(&addr).unwrap();
            b.login_versioned(vb, 0x42, &n("b"), "x").unwrap();
            let rb = b.spawn_reader().unwrap();
            let mut sb = SessionState::for_version(vb);
            pump(&rb, &mut sb, 1000);
            b.send_messages(&[position_message_sprite(36.0, 31.0, 2)]).unwrap();
            let mut a = Connection::connect(&addr).unwrap();
            a.login_versioned(va, 0x42, &n("a"), "x").unwrap();
            let ra = a.spawn_reader().unwrap();
            let mut sa = SessionState::for_version(va);
            pump(&ra, &mut sa, 1000);
            a.send_messages(&[carry_sprite_message(Some(261)), position_message_sprite(33.5, 31.0, 3 + 4 * 18)]).unwrap();
            pump(&rb, &mut sb, 1000);
            a.send_messages(&[throw_carried_message(), carry_sprite_message(None), position_message_sprite(33.5, 31.0, 3)]).unwrap();
            pump(&rb, &mut sb, 1000);
            let throws = std::mem::take(&mut sb.throws);
            let mut full = victim(6);
            let mut last = victim(1);
            for o in &throws {
                if let Some(sprite) = o.carry {
                    for s in [&mut full, &mut last] {
                        s.throw_remote(sprite, o.x, o.y, o.dir, o.id);
                        s.tick_throw(9 * 50);
                    }
                }
            }
            let died = last.tick_life(50);
            a.shutdown().unwrap();
            drop(a);
            println!(
                "A {} -> B {}: throws seen {:?}; B hearts 6 -> {} (hurt {}); at 1 -> {} {:?}",
                va.label(),
                vb.label(),
                throws.iter().map(|o| (o.carry, o.x, o.y, o.dir)).collect::<Vec<_>>(),
                full.hearts_half(),
                full.combat.is_hurt(),
                last.hearts_half(),
                died
            );
        }
    }
}
