//! Two 1.39 logins: A walks down a few tiles and swings its sword, sending
//! the sprite numbers Seihai's 1.39 profile computes; B prints the sprite
//! numbers it sees for A (prop 17, `dir + 4 * S`).
//!
//! Usage: cargo run -p seihai-mae --example walk139_probe -- <account prefix> [host:port]
//!
//! The default address is the lab's classic GServer (127.0.0.1:14803), which
//! takes 1.33 to 1.39; its listserver accepts any account.

use seihai::assets::{Gani, GaniPlayer};
use seihai::net::Connection;
use seihai::offline::OfflineSession;
use seihai::protocol::{ClientVersion, SessionState};
use std::path::Path;
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
    let addr = std::env::args().nth(2).unwrap_or_else(|| "127.0.0.1:14803".into());
    let v = ClientVersion::V139;
    let mut b = Connection::connect(&addr).unwrap();
    b.login_versioned(v, 0x42, &format!("{tag}b"), "x").unwrap();
    let rb = b.spawn_reader().unwrap();
    let mut sb = SessionState::for_version(v);
    pump(&rb, &mut sb, 1500);
    let mut a = Connection::connect(&addr).unwrap();
    a.login_versioned(v, 0x42, &format!("{tag}a"), "x").unwrap();
    let ra = a.spawn_reader().unwrap();
    let mut sa = SessionState::for_version(v);
    pump(&ra, &mut sa, 1500);

    // A's session as the 1.39 profile sets it up: walk frame in the sprite,
    // the built-in walk (Seihai's own default set here), the 1.39 swing.
    let mut s = OfflineSession::new();
    s.walk_sprite = !v.profile().ui.ganis;
    s.combat.set_rules(v.combat_rules());
    s.sword_power = 1;
    let walk = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default/gani/walk.gani");
    s.set_walk_gani(GaniPlayer::new(Gani::load(walk).unwrap()));
    s.player_x = 30.0;
    s.player_y = 30.0;
    s.player_dir = 2;

    let mut sent = Vec::new();
    let mut seen = Vec::new();
    let mut look = |sb: &mut SessionState, seen: &mut Vec<u8>| {
        pump(&rb, sb, 60);
        if let Some(sp) = sb.others.first().and_then(|o| o.sprite) {
            if seen.last() != Some(&sp) {
                seen.push(sp);
            }
        }
    };
    // Walk 8 ticks down, a frame each tick; a real 1.39 sends every 100 ms.
    s.walking = true;
    for i in 0..8 {
        s.walk_gani.as_mut().unwrap().step();
        s.player_y += 0.5;
        if i % 2 == 1 {
            let sp = s.sprite_value_for(false);
            sent.push(sp);
            a.send_walk_sprite(&s.player_update(), sp).unwrap();
        }
        look(&mut sb, &mut seen);
    }
    s.walking = false;
    let sp = s.sprite_value_for(false);
    sent.push(sp);
    a.send_walk_sprite(&s.player_update(), sp).unwrap();
    look(&mut sb, &mut seen);
    // The sword: one step a tick, sent every tick here.
    assert!(s.start_swing(), "1.39 swings");
    for _ in 0..6 {
        let sp = s.sprite_value_for(false);
        sent.push(sp);
        a.send_walk_sprite(&s.player_update(), sp).unwrap();
        look(&mut sb, &mut seen);
        s.tick_combat(50);
    }
    pump(&rb, &mut sb, 500);
    look(&mut sb, &mut seen);
    println!("A sent sprites {sent:?}");
    println!("B saw A's sprites {seen:?}");
    println!("B sees A at {:?}", sb.others.first().map(|o| (o.x, o.y)));
    a.shutdown().unwrap();
    b.shutdown().unwrap();
}
