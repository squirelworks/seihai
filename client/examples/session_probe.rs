//! Log in and dump a cleaned SessionState, including disconnect text.
//!
//! Usage: SEIHAI_ACCOUNT=.. SEIHAI_PASSWORD=.. \
//!     cargo run -p seihai --example session_probe -- <host:port> [seconds] [version]
//!
//! Version: 1.39, 2.05, 2.17, 2.22, 2.31 (default 2.05).

use seihai::net::Connection;
use seihai::protocol::{parse_player_props, split_messages, ClientVersion, SessionState};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn parse_version(s: &str) -> ClientVersion {
    for v in ClientVersion::all_versions() {
        if v.label() == s {
            return *v;
        }
    }
    ClientVersion::V205
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let addr = args.first().expect("host:port");
    let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let version = args.get(2).map(|s| parse_version(s)).unwrap_or(ClientVersion::V205);
    let account = std::env::var("SEIHAI_ACCOUNT").expect("SEIHAI_ACCOUNT");
    let password = std::env::var("SEIHAI_PASSWORD").expect("SEIHAI_PASSWORD");

    println!("version {} ({})", version.label(), version.token());
    let mut conn = Connection::connect(addr).expect("connect");
    conn.login_versioned(version, 0x42, &account, &password)
        .expect("login");

    let mut state = SessionState::default();
    let mut ids: BTreeMap<u8, usize> = BTreeMap::new();
    let mut prop_ids: BTreeMap<u8, usize> = BTreeMap::new();
    let mut disc_msgs: Vec<String> = Vec::new();
    let mut frames = 0;
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        match conn.read_frame_into(&mut state) {
            Ok(body) => {
                frames += 1;
                for m in split_messages(&body) {
                    *ids.entry(m.id).or_default() += 1;
                    if m.id == 9 {
                        for p in parse_player_props(&m.payload) {
                            *prop_ids.entry(p.id).or_default() += 1;
                        }
                    }
                    if m.id == 16 {
                        disc_msgs.push(String::from_utf8_lossy(&m.payload).into_owned());
                    }
                }
            }
            Err(e) => {
                println!("read stopped: {}", e.kind());
                break;
            }
        }
    }
    let unhandled: BTreeMap<_, _> = state.unhandled.iter().collect();
    println!("frames {frames}");
    println!("message ids {ids:?}");
    println!("PLO_PLAYERPROPS prop ids {prop_ids:?}");
    println!("level_name {:?}", state.level_name);
    println!("server board {}", if state.board.is_some() { "received" } else { "none" });
    println!(
        "board_lines {} links {} signs {} chests {} npcs {} baddies {} items {}",
        state.board_lines.len(),
        state.links.len(),
        state.signs.len(),
        state.chests.len(),
        state.npcs.len(),
        state.baddies.len(),
        state.items.len()
    );
    println!(
        "player x {:?} y {:?} dir {:?}",
        state.player_x, state.player_y, state.player_dir
    );
    if !disc_msgs.is_empty() {
        println!("disconnect messages:");
        for m in &disc_msgs {
            println!("  {m}");
        }
    }
    let weapons: Vec<&str> = state.weapons.iter().map(|w| w.name.as_str()).collect();
    println!(
        "flags {} weapons {weapons:?} default weapons {:?} world time {:?}",
        state.flags.len(),
        state.default_weapons,
        state.world_time
    );
    println!("unhandled {unhandled:?}");
}
