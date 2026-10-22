//! Log in to a lab gserver and dump a cleaned `SessionState`.
//!
//! Usage: SEIHAI_ACCOUNT=.. SEIHAI_PASSWORD=.. \
//!     cargo run -p seihai --example session_probe -- <host:port> [seconds]
//!
//! 2.05 (gen 3) only: server→client frames are plain zlib, so
//! `Connection::read_frame_into` can read them directly. Output holds
//! ids, counts, position and the level name; never account names,
//! nicknames or chat text.

use seihai::net::Connection;
use seihai::protocol::{parse_player_props, split_messages, ClientVersion, SessionState};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let addr = args.first().expect("host:port");
    let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let account = std::env::var("SEIHAI_ACCOUNT").expect("SEIHAI_ACCOUNT");
    let password = std::env::var("SEIHAI_PASSWORD").expect("SEIHAI_PASSWORD");

    let mut conn = Connection::connect(addr).expect("connect");
    conn.login_versioned(ClientVersion::V205, 0x42, &account, &password)
        .expect("login");

    let mut state = SessionState::default();
    let mut ids: BTreeMap<u8, usize> = BTreeMap::new();
    let mut prop_ids: BTreeMap<u8, usize> = BTreeMap::new();
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
                }
            }
            Err(e) => {
                println!("read stopped: {}", e.kind());
                break;
            }
        }
    }
    let unhandled: BTreeMap<_, _> = state.unhandled.iter().filter(|(_, &n)| n > 1).collect();
    println!("frames {frames}");
    println!("message ids {ids:?}");
    println!("PLO_PLAYERPROPS prop ids {prop_ids:?}");
    println!("level_name {:?}", state.level_name);
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
    println!("unhandled (>1) {unhandled:?}");
}
