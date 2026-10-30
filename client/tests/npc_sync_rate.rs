//! How many NPC prop messages Seihai sends while the scripts run: none for
//! NPCs whose scripts keep setting the same values, and none for an NPC a
//! script keeps moving (its position is this client's business; only a
//! push or a pull sends X and Y). See `documentation/ledges-push-pull.md`.

use seihai::offline::{Linked, OfflineSession, ServerNpcSync};
use seihai::protocol::npcprops::NpcProps;
use seihai::protocol::ClientVersion;
use seihai::scripting::engine::host::SessionScripts;
use std::collections::BTreeMap;
use std::time::Duration;

/// Run `scripts` (id, script) as server NPCs for `secs` seconds of 16 ms
/// frames and count the messages each frame's sync sent, per second.
fn messages_per_second(scripts: &[(u32, &str)], secs: u64) -> Vec<usize> {
    let mut s = OfflineSession::new();
    s.current_level = Some(seihai::assets::Level::from_board(&vec![0u16; 4096]));
    s.player_x = 30.0;
    s.player_y = 50.0;
    let mut sc = SessionScripts::new(ClientVersion::V205);
    sc.load_level(&mut s, "x.nw");
    let mut sync = ServerNpcSync::default();
    let mut heard = BTreeMap::new();
    for (k, &(id, body)) in scripts.iter().enumerate() {
        let p = NpcProps { image: Some("a.png".into()), script: Some(body.into()), x: Some(20.0 + 4.0 * k as f32), y: Some(30.0), ..Default::default() };
        let Linked::Added(i) = s.take_server_npc(id, &p) else { panic!("{id} not added") };
        sc.add_npc(&mut s, i);
        heard.insert(id, (i, p));
    }
    for (id, (i, p)) in &heard {
        sync.heard(*id, p, &s.current_level.as_ref().unwrap().npcs[*i]);
    }
    let frame = Duration::from_millis(16);
    let mut per_second = vec![0; secs as usize];
    let mut t = Duration::ZERO;
    while t < Duration::from_secs(secs) {
        sc.tick(&mut s, frame.as_millis() as u64);
        per_second[t.as_secs() as usize] += sync.tick(s.current_level.as_ref().unwrap(), frame).len();
        t += frame;
    }
    per_second
}

#[test]
fn npcs_whose_scripts_set_the_same_values_send_nothing_after_the_first_round() {
    let n = messages_per_second(
        &[
            (1, "//"),
            (2, "if (created||timeout) { message hi; dir=2; timeout=0.05; }\n"),
            (3, "if (created||timeout) { showcharacter; setcharprop #3,head0.png; setcharani idle,; message yo; timeout=0.05; }\n"),
            (4, "if (created||timeout) { canbepushed; hide; show; timeout=0.1; }\n"),
        ],
        5,
    );
    // The first round says what the scripts set up (one message per NPC at
    // most); after that, five seconds of nothing.
    assert!(n[0] <= 4, "{n:?}");
    assert_eq!(n[1..].iter().sum::<usize>(), 0, "{n:?}");
}

#[test]
fn an_npc_a_script_keeps_moving_stays_on_this_client() {
    let n = messages_per_second(&[(1, "if (created||timeout) { x+=0.5; if (x>40) x=20; timeout=0.05; }\n")], 5);
    // The first round is its VISFLAGS, as 2.05 sends for every NPC.
    assert!(n[0] <= 1, "{n:?}");
    assert_eq!(n[1..].iter().sum::<usize>(), 0, "{n:?}");
}
