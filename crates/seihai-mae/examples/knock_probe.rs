//! A hit packet pushes the victim: for each pair of 2.05 and 2.22 (the
//! builds with combat rules), player B stands at (30, 30) and player A, at
//! (27, 30), sends `PLI_HURTPLAYER` (26) at B with a push of (+1, 0) and one
//! half heart. B's side runs what the game loop runs: the relayed
//! `PLO_HURTPLAYER` (40) is judged and the push is stepped on the 50 ms
//! clock. Prints the payload B got, the judge's result, and where B ends up.
//!
//! Usage: cargo run -p seihai-mae --example knock_probe -- <account prefix> [host:port]
//!
//! The lab listserver accepts any account and password. The default address
//! is the lab GServer.

use seihai::combat::{hurtplayer_message, Target};
use seihai::net::{position_message_sprite, Connection};
use seihai::offline::OfflineSession;
use seihai::protocol::{ClientVersion, SessionState};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
type Rx = Receiver<std::io::Result<Vec<u8>>>;
/// Apply what arrives for `ms`; returns the raw `PLO_HURTPLAYER` (40)
/// payloads seen.
fn pump(rx: &Rx, st: &mut SessionState, ms: u64) -> Vec<Vec<u8>> {
    let mut raw40 = Vec::new();
    let end = Instant::now() + Duration::from_millis(ms);
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(Ok(b)) => {
                for m in seihai::protocol::split_messages(&b) {
                    if m.id == 40 {
                        raw40.push(m.payload.clone());
                    }
                }
                st.apply_body(&b);
            }
            _ => break,
        }
    }
    raw40
}
fn main() {
    let tag = std::env::args().nth(1).expect("account prefix");
    let addr = std::env::args().nth(2).unwrap_or_else(|| "127.0.0.1:14802".into());
    let vs = [ClientVersion::V205, ClientVersion::V222];
    for &va in &vs {
        for &vb in &vs {
            let n = |p: &str| format!("{tag}{p}{}{}", va.label().replace('.', ""), vb.label().replace('.', ""));
            let mut b = Connection::connect(&addr).unwrap();
            b.login_versioned(vb, 0x42, &n("b"), "x").unwrap();
            let rb = b.spawn_reader().unwrap();
            let mut sb = SessionState::for_version(vb);
            pump(&rb, &mut sb, 1000);
            b.send_messages(&[position_message_sprite(30.0, 30.0, 2)]).unwrap();
            let mut a = Connection::connect(&addr).unwrap();
            a.login_versioned(va, 0x42, &n("a"), "x").unwrap();
            let ra = a.spawn_reader().unwrap();
            let mut sa = SessionState::for_version(va);
            a.send_messages(&[position_message_sprite(27.0, 30.0, 3)]).unwrap();
            pump(&ra, &mut sa, 1000);
            pump(&rb, &mut sb, 500);
            let Some(v) = sa.others.iter().find(|o| o.nick.as_deref().is_some_and(|s| s.contains(&n("b"))) || o.account.as_deref() == Some(n("b").as_str())).or(sa.others.last()) else {
                println!("A {} -> B {}: A does not see B", va.label(), vb.label());
                continue;
            };
            // B stands at (30, 30); A's view of B's position can lag.
            let t = Target { id: v.id, x: 30.0, y: 30.0, status: v.status, sprite: v.sprite };
            let m = match hurtplayer_message(va, (27.0, 30.0), &t, (1.0, 0.0), 1) {
                Ok(m) => m,
                Err(e) => {
                    println!("A {} -> B {}: not sent ({e:?}); A sees B at {:?}", va.label(), vb.label(), (t.x, t.y));
                    continue;
                }
            };
            a.send_messages(&[m]).unwrap();
            let raw = pump(&rb, &mut sb, 1000);
            let hits = std::mem::take(&mut sb.hits);
            let mut s = OfflineSession::new();
            s.combat.set_rules(vb.profile().life.combat);
            s.life.set_rules(vb.profile().life.death);
            s.set_max_hearts_half(6);
            s.set_hearts_half(6);
            s.player_x = 30.0;
            s.player_y = 30.0;
            let reports = s.receive_all(&sb.others, None, None, 0, &[], &hits);
            let mut path = vec![s.player_x];
            for _ in 0..8 {
                s.tick_combat(50);
                path.push(s.player_x);
            }
            a.shutdown().unwrap();
            drop(a);
            println!(
                "A {} -> B {}: raw 40 {:02x?}; B got {:?}; judged {:?}; hearts {}; x per tick {:?}",
                va.label(),
                vb.label(),
                raw,
                hits,
                reports.iter().map(|r| r.outcome).collect::<Vec<_>>(),
                s.hearts_half(),
                path
            );
        }
    }
}
