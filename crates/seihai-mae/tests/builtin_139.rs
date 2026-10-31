//! 1.39's built-in player animations, its walk sprite and its sword swing.
//!
//! 1.39 has no GANI files: its poses are compiled into the executable, and
//! NewMain 1.41's stock files are those tables written out. So a 1.39
//! profile takes the stock files of the user's own folder for its built-in
//! poses (never a server's), fits them to 1.39's timing, sends its walk
//! frame as the sprite number, and swings one sword step a tick.

use std::fs;
use std::path::{Path, PathBuf};

use seihai::assets::{AssetResolver, AssetRoot, Gani, GaniPlayer};
use seihai::offline::OfflineSession;
use seihai::pose::{fit_builtin_139, PoseKind, WALK_S};
use seihai::protocol::{ClientVersion, COMBAT_139};

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("seihai-builtin139-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// A GANI of `steps` steps (one row each), `wait` after every step, and a
/// `PLAYSOUND` after step 0.
fn gani_text(steps: usize, wait: u32, sound: Option<&str>, extra: &str) -> String {
    let mut t = format!("SPRITE 200 BODY 0 0 32 32 body\nSINGLEDIRECTION\n{extra}ANI\n");
    for k in 0..steps {
        t.push_str("200 8 16\n");
        if k == 0 {
            if let Some(s) = sound {
                t.push_str(&format!("PLAYSOUND {s} 1.5 2\n"));
            }
        }
        if wait > 0 {
            t.push_str(&format!("WAIT {wait}\n"));
        }
        t.push('\n');
    }
    t.push_str("ANIEND\n");
    t
}

fn load_139(r: &AssetResolver, name: &str) -> Option<Gani> {
    let mut g = Gani::load(r.pose_gani(name)?).unwrap();
    fit_builtin_139(name, &mut g);
    Some(g)
}

#[test]
fn the_built_in_poses_come_from_the_users_stock_files_fitted_to_139() {
    let d = tmp("assets");
    let cache = tmp("cache");
    write(&d, "levels/ganis/walk.gani", &gani_text(8, 0, Some("steps2.wav"), "LOOP\n"));
    write(&d, "levels/ganis/push.gani", &gani_text(5, 3, None, "LOOP\n"));
    write(&d, "levels/ganis/hurt.gani", &gani_text(1, 30, Some("lift.wav"), "SETBACKTO idle\n"));
    write(&d, "levels/ganis/sword.gani", &gani_text(5, 0, Some("sword.wav"), "SETBACKTO idle\n"));
    write(&d, "levels/ganis/lift.gani", &gani_text(1, 5, None, ""));
    // What a server would send is not read: 1.39 downloads no GANIs.
    write(&cache, "levels/ganis/idle.gani", &gani_text(1, 0, None, ""));
    let r = AssetResolver::new(AssetRoot::new(&d), ClientVersion::V139.profile()).with_cache(Some(cache.clone()));

    let walk = load_139(&r, "walk.gani").expect("walk is built in");
    assert_eq!(walk.frame_count(), 8);
    assert_eq!(walk.frames[0].sounds[0].file, "steps.wav", "1.39 names steps.wav, not steps2.wav");

    let push = load_139(&r, "push.gani").unwrap();
    assert_eq!(push.total_ticks(), 5 * 2, "about 2 ticks a push frame in 1.39, not 4");

    let hurt = load_139(&r, "hurt.gani").unwrap();
    assert_eq!(hurt.total_ticks(), seihai::pose::HURT_POSE_139_TICKS, "the 1.39 hurt pose is about 10 ticks");
    assert!(hurt.frames[0].sounds.is_empty(), "1.39 has no lift.wav");

    let sword = load_139(&r, "sword.gani").unwrap();
    assert_eq!(sword.total_ticks(), 5);
    assert_eq!(sword.frames[0].sounds[0].file, "sword.wav");

    assert!(r.pose_gani("lift.gani").is_none(), "1.39 has no lift pose");
    assert!(r.pose_gani("idle.gani").is_none(), "the server's copy is not used");
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&cache);
}

/// Seihai's own default set works as the built-in animations too, for a
/// 1.39 with no install folder of its own.
#[test]
fn seihais_default_set_stands_in_when_the_folder_has_none() {
    let empty = tmp("empty");
    let builtin = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default");
    let r = AssetResolver::new(AssetRoot::new(&empty), ClientVersion::V139.profile()).with_builtin(Some(builtin));
    for name in ["idle.gani", "walk.gani", "swim.gani", "sword.gani", "hurt.gani", "carry.gani", "carrystill.gani"] {
        assert!(load_139(&r, name).is_some_and(|g| g.frame_count() > 0), "{name}");
    }
    let _ = fs::remove_dir_all(&empty);
}

fn walk_gani() -> GaniPlayer {
    GaniPlayer::new(Gani::parse(&gani_text(8, 0, None, "LOOP\n")).unwrap())
}

#[test]
fn a_walking_139_player_sends_its_walk_frame_as_the_sprite() {
    let mut s = OfflineSession::new();
    s.player_dir = 2;
    s.walk_sprite = !ClientVersion::V139.profile().ui.ganis;
    s.set_walk_gani(walk_gani());
    assert_eq!(s.sprite_value_for(false), 2, "standing: the direction only");
    s.walking = true;
    let mut seen = Vec::new();
    for _ in 0..8 {
        seen.push(s.sprite_value_for(false));
        s.walk_gani.as_mut().unwrap().step();
    }
    let want: Vec<u8> = WALK_S.iter().map(|w| 2 + 4 * w).collect();
    assert_eq!(seen, want);
    // The lab capture of a real 1.39 walking down: 18 (S 4), 10 (S 2), then 22, 14, 6.
    assert!(seen.contains(&18) && seen.contains(&10) && seen.contains(&22) && seen.contains(&14) && seen.contains(&6));
    // A GANI build sends the name `walk`, not the frame.
    assert_eq!(s.sprite_value_for(true), 2);
    // Swimming sends no walk frame.
    s.in_water = true;
    assert_eq!(s.sprite_value_for(false), 2);
    // A GANI build's session does not put it in the sprite.
    s.in_water = false;
    s.walk_sprite = !ClientVersion::V205.profile().ui.ganis;
    assert_eq!(s.sprite_value_for(false), 2);
}

#[test]
fn the_139_sword_swings_one_step_a_tick() {
    let mut s = OfflineSession::new();
    s.combat.set_rules(ClientVersion::V139.combat_rules());
    assert_eq!(ClientVersion::V139.combat_rules(), Some(COMBAT_139));
    s.sword_power = 1;
    s.player_dir = 2;
    assert!(s.start_swing(), "1.39 has a sword");
    let mut seen = vec![s.sprite_value()];
    for _ in 0..5 {
        s.tick_combat(50);
        seen.push(s.sprite_value());
    }
    // S 6 to 10, then the idle sprite; the capture saw 26, 34, 42 at 100 ms.
    assert_eq!(seen, vec![26, 30, 34, 38, 42, 2]);
    assert!(!s.combat.is_swinging());
}

#[test]
fn the_139_profile_draws_the_built_in_walk() {
    let mut s = OfflineSession::new();
    s.set_walk_gani(walk_gani());
    s.walking = true;
    assert_eq!(s.current_pose().map(|p| p.kind), Some(PoseKind::Walk));
}
