//! The commands 2gta's title level (`splashscreen.nw`) runs before it draws
//! its title: `freezeplayer`, `hideplayer` and `showstats`. Seihai let these
//! fall through to "effect not applied" (debug log, 2026-10-31).

use seihai::assets::Level;
use seihai::offline::OfflineSession;
use seihai::protocol::ClientVersion;
use seihai::scripting::engine::host::SessionScripts;

fn session(script: &str) -> (OfflineSession, SessionScripts) {
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::parse(&format!("GLEVNW01\nNPC - 2 2\n{script}\nNPCEND\n")).unwrap());
    s.player_x = 30.0;
    s.player_y = 30.0;
    let mut sc = SessionScripts::new(ClientVersion::V222);
    sc.load_level(&mut s, "splashscreen.nw");
    (s, sc)
}

#[test]
fn freezeplayer_and_hideplayer_last_their_time() {
    let (mut s, mut sc) = session("if (playerenters) { freezeplayer 2; hideplayer 0.5; }");
    assert_eq!(s.script_freeze_ms, 2000);
    assert_eq!(s.script_hide_ms, 500);
    assert!(sc.unhandled.is_empty(), "{:?}", sc.unhandled);
    sc.tick(&mut s, 400);
    assert_eq!((s.script_freeze_ms, s.script_hide_ms), (1600, 100));
    sc.tick(&mut s, 400);
    assert_eq!((s.script_freeze_ms, s.script_hide_ms), (1200, 0));
    sc.tick(&mut s, 5000);
    assert_eq!(s.script_freeze_ms, 0);
}

/// A script that calls `hideplayer` on each timeout keeps the player hidden.
#[test]
fn hideplayer_on_a_timeout_keeps_the_player_hidden() {
    let (mut s, mut sc) = session("if (playerenters || timeout) { hideplayer 0.1; timeout = 0.05; }");
    for _ in 0..20 {
        sc.tick(&mut s, 50);
        assert!(s.script_hide_ms > 0);
    }
}

#[test]
fn showstats_sets_the_hud_bits() {
    let (s, sc) = session("if (playerenters) { showstats 0; }");
    assert_eq!(s.stats, Some(0));
    assert!(sc.unhandled.is_empty(), "{:?}", sc.unhandled);
    let (s, _) = session("if (playerenters) { showstats 1+2+32; }");
    assert_eq!(s.stats, Some(0x23));
    let (s, _) = session("if (playerenters) { message hi; }");
    assert_eq!(s.stats, None, "no showstats: everything shows");
}

#[test]
fn the_debug_dump_holds_every_npc_script() {
    let level = Level::parse("GLEVNW01\nNPC - 2 2\nif (created) {\n  message a;\n}\nNPCEND\nNPC fallpal.png 28 30\n// second\nNPCEND\n").unwrap();
    let dump = level.npc_scripts_dump();
    assert!(dump.contains("//// npc 0: - at 2,2 (3 lines)\nif (created) {\n  message a;\n}\n"), "{dump}");
    assert!(dump.contains("//// npc 1: fallpal.png at 28,30 (1 lines)\n// second\n"), "{dump}");
}

/// 2gta's title, cut down: the player goes off the board (y 65), the view
/// is held on 30,30, and S (`keydown(5)`) picks "Start".
const TITLE: &str = "if (playerenters) { freezeplayer 99; playery=65; showstats 0; timeout=0.05; }
if (timeout) {
  hideplayer 0.1;
  setfocus 30,30;
  showimg 202,@b@Start,29,40;
  if (keydown(5)) { resetfocus; setlevel2 speciesselect.nw,31.5,30; }
  timeout=0.05;
}";

fn title(version: ClientVersion) -> (OfflineSession, SessionScripts) {
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::parse(&format!("GLEVNW01\nNPC - 32 30\n{TITLE}\nNPCEND\n")).unwrap());
    s.player_x = 30.0;
    s.player_y = 30.0;
    let mut sc = SessionScripts::new(version);
    sc.load_level(&mut s, "splashscreen.nw");
    (s, sc)
}

#[test]
fn the_title_holds_the_view_and_s_starts() {
    let (mut s, mut sc) = title(ClientVersion::V222);
    assert_eq!(s.player_y, 65.0);
    sc.tick(&mut s, 50);
    assert_eq!(s.view_focus, Some((30.0, 30.0)));
    assert!(s.script_hide_ms > 0);
    assert!(s.pending_link.is_none());
    // A (bit 4) is not S.
    s.keys_down = 1 << 4;
    sc.tick(&mut s, 50);
    assert!(s.pending_link.is_none());
    s.keys_down = 1 << 5;
    sc.tick(&mut s, 50);
    assert_eq!(s.view_focus, None, "resetfocus");
    let (dest, x, y) = s.pending_link.clone().expect("setlevel2");
    assert_eq!((dest.as_str(), x, y), ("speciesselect.nw", 31.5, 30.0));
}

#[test]
fn keydown_is_false_while_paused() {
    let (mut s, mut sc) = title(ClientVersion::V222);
    s.keys_down = 1 << 5;
    s.paused = true;
    sc.tick(&mut s, 50);
    assert!(s.pending_link.is_none());
}

/// 2.05 has no executor for `setfocus` (`gs1.rs`): the view stays on the player.
#[test]
fn setfocus_does_nothing_in_205() {
    let (mut s, mut sc) = title(ClientVersion::V205);
    sc.tick(&mut s, 50);
    assert_eq!(s.view_focus, None);
}

#[test]
fn a_new_level_follows_the_player_again() {
    let (mut s, mut sc) = title(ClientVersion::V222);
    sc.tick(&mut s, 50);
    assert!(s.view_focus.is_some());
    s.current_level = Some(Level::parse("GLEVNW01\n").unwrap());
    sc.load_level(&mut s, "other.nw");
    assert_eq!(s.view_focus, None);
}
