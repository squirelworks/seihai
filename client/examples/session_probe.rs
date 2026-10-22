//! Log in to a gserver and dump a cleaned `SessionState`.
//!
//! Usage: SEIHAI_ACCOUNT=.. SEIHAI_PASSWORD=.. \
//!     cargo run -p seihai --example session_probe -- <host:port> [seconds] [version]
//!
//! `version` is a GUI label such as `2.05` (the default), `2.22` or
//! `1.39`. Output holds frame sizes, message ids, counts, position, the
//! level name and the server's disconnect text; never account names,
//! nicknames or chat text. Check the disconnect text before sharing it,
//! since a server may quote the account in it.

use seihai::net::{random_key, Connection};
use seihai::protocol::{parse_player_props_with, split_messages, ClientVersion, SessionState};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let addr = args.first().expect("host:port");
    let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let label = args.get(2).map(String::as_str).unwrap_or("2.05");
    let version = *ClientVersion::all_versions()
        .iter()
        .find(|v| v.label() == label)
        .expect("unknown version label");
    let account = std::env::var("SEIHAI_ACCOUNT").expect("SEIHAI_ACCOUNT");
    let password = std::env::var("SEIHAI_PASSWORD").expect("SEIHAI_PASSWORD");

    let mut conn = Connection::connect(addr).expect("connect");
    conn.login_versioned(version, random_key(), &account, &password)
        .expect("login");
    println!("version {label} ({:?})", version.encryption());

    let mut state = SessionState::default();
    state.file_has_modtime = version.file_has_modtime();
    state.bow_prop10 = version.encryption() == seihai::protocol::EncryptionGen::Gen2;
    let mut ids: BTreeMap<u8, usize> = BTreeMap::new();
    let mut prop_ids: BTreeMap<u8, usize> = BTreeMap::new();
    let mut frames = 0;
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        match conn.read_frame_into(&mut state) {
            Ok(body) => {
                frames += 1;
                if frames <= 8 {
                    println!("frame {frames}: {} bytes after decompress", body.len());
                }
                for m in split_messages(&body) {
                    *ids.entry(m.id).or_default() += 1;
                    if m.id == 9 {
                        for p in parse_player_props_with(&m.payload, state.bow_prop10) {
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
    println!("files {:?}", state.file_requests);
    println!("disconnect message {:?}", state.disconnect_message);
}
