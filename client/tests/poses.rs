//! Item 22: the sword, hurt and sit poses, with Seihai's own default assets.
//!
//! State level: the offline session steps `sword.gani` with the swing, plays
//! its sound cue, shows the hurt pose on a hit and goes back to idle.
//! Render level: the pose GANIs draw something different from the idle figure
//! and from each other.

use std::path::{Path, PathBuf};

use seihai::assets::{read_png_rgba, Gani};
use seihai::combat::HurtPlayer;
use seihai::offline::OfflineSession;
use seihai::pose::{PoseKind, HURT_POSE_MS};
use seihai::protocol::COMBAT_205;
use seihai::render::{draw_gani_frame, draw_player_sprites, Framebuffer, PoseSheets, Sheet};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("assets").join("default")
}

fn gani(name: &str) -> Gani {
    Gani::load(root().join("gani").join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// A session with the 2.05 combat rules and the three pose animations.
fn session() -> OfflineSession {
    let mut s = OfflineSession::new();
    s.combat.set_rules(Some(COMBAT_205));
    s.life.set_rules(Some(seihai::protocol::DEATH_205));
    s.player_x = 30.0;
    s.player_y = 30.0;
    for k in [PoseKind::Sword, PoseKind::Hurt, PoseKind::Sit] {
        s.set_pose_gani(k, gani(k.file()));
    }
    s
}

fn pose(s: &OfflineSession) -> Option<(PoseKind, usize)> {
    s.current_pose().map(|p| (p.kind, p.frame))
}

fn hurt_packet(half: i32) -> HurtPlayer {
    HurtPlayer { attacker: 7, dx: 0.0, dy: 0.0, half, npc: None }
}

#[test]
fn the_s_key_plays_the_sword_animation_and_goes_back_to_idle() {
    let mut s = session();
    assert_eq!(pose(&s), None, "idle before the swing");
    assert!(s.start_swing());
    assert_eq!(pose(&s), Some((PoseKind::Sword, 0)));
    // the sound after step 0 is queued once, at the player (cue offset 1.5, 2)
    let ev = s.take_sound_events();
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].file, "sword.wav");
    assert_eq!((ev[0].x, ev[0].y), (31.5, 32.0));
    // the sprite value other players see follows the same step
    let mut seen = vec![(pose(&s).unwrap().1, s.sprite_value())];
    for _ in 0..4 {
        s.tick_combat(100); // a swing step is 2 ticks of 50 ms
        seen.push((pose(&s).map_or(usize::MAX, |p| p.1), s.sprite_value()));
    }
    let dir = u32::from(s.player_dir);
    let expect: Vec<(usize, u8)> = (0..5).map(|i| (i, (dir + 4 * (6 + i as u32)) as u8)).collect();
    assert_eq!(seen[..4], expect[..4]);
    assert_eq!(seen[4].0, 4);
    // the swing ends: the pose is gone (sword.gani says SETBACKTO idle)
    assert!(s.tick_combat(100));
    assert_eq!(pose(&s), None);
    assert!(!s.combat.is_swinging());
    assert_eq!(s.sprite_value(), s.player_dir);
    assert_eq!(gani("sword.gani").setback_to.as_deref(), Some("idle"));
    assert!(s.take_sound_events().is_empty(), "no further cue: only step 0 has one");
}

#[test]
fn a_swing_in_one_long_tick_still_plays_every_cue_once() {
    let mut s = session();
    // a second cue on step 2, to see that skipped steps still count
    let mut g = gani("sword.gani");
    g.frames[2].sounds.push(seihai::assets::GaniSound { file: "x.wav".into(), x: 1.5, y: 2.0 });
    s.set_pose_gani(PoseKind::Sword, g);
    assert!(s.start_swing());
    s.tick_combat(10_000);
    let names: Vec<String> = s.take_sound_events().into_iter().map(|e| e.file).collect();
    assert_eq!(names, ["sword.wav", "x.wav"]);
}

#[test]
fn no_swing_in_water_or_without_the_gani_and_no_pose_when_dead() {
    let mut s = session();
    s.in_water = true;
    assert!(!s.start_swing());
    assert_eq!(pose(&s), None);
    s.in_water = false;

    // no sword.gani loaded: the swing still runs (combat), nothing is drawn, no cue
    let mut bare = OfflineSession::new();
    bare.combat.set_rules(Some(COMBAT_205));
    assert!(bare.start_swing());
    assert_eq!(pose(&bare), None);
    assert!(bare.take_sound_events().is_empty());
    bare.tick_combat(1000);

    // dead: no pose
    let mut d = session();
    assert!(d.start_swing());
    d.damage_half(d.max_hearts, Some(3));
    d.tick_life(100);
    assert!(d.life.is_dead());
    d.tick_combat(0);
    assert_eq!(pose(&d), None);
}

#[test]
fn a_hit_plays_the_hurt_pose_through_the_existing_hurt_path() {
    let mut s = session();
    let at = Some((s.player_x + 2.0, s.player_y));
    let rep = s.receive_hurtplayer(&hurt_packet(1), at);
    assert_eq!(rep.outcome, Ok(1));
    assert!(s.combat.is_hurt());
    assert_eq!(pose(&s), Some((PoseKind::Hurt, 0)));
    // the sprite others see is still the hurt sprite (S = 32) for the whole hurt mode
    assert_eq!(s.sprite_value(), s.player_dir + 4 * 32);
    // the pose is shown for HURT_POSE_MS, hurt mode goes on for its 2 s
    s.tick_combat(HURT_POSE_MS - 50);
    assert_eq!(pose(&s), Some((PoseKind::Hurt, 0)));
    s.tick_combat(100);
    assert_eq!(pose(&s), None);
    assert!(s.combat.is_hurt());
    s.tick_combat(2_000);
    assert!(!s.combat.is_hurt());
    assert_eq!(s.sprite_value(), s.player_dir);
}

#[test]
fn a_swing_wins_over_hurt_and_sit_shows_only_when_asked() {
    let mut s = session();
    s.receive_hurtplayer(&hurt_packet(1), Some((s.player_x + 2.0, s.player_y)));
    assert!(s.start_swing());
    assert_eq!(pose(&s), Some((PoseKind::Sword, 0)));
    let mut t = session();
    assert_eq!(pose(&t), None);
    t.sitting = true;
    assert_eq!(pose(&t), Some((PoseKind::Sit, 0)));
    // sitting waits while an arrow key keeps the walk going
    t.walking = true;
    assert_ne!(pose(&t).map(|p| p.0), Some(PoseKind::Sit));
    t.walking = false;
    t.in_water = true;
    assert_eq!(pose(&t), None, "not while swimming");
}

// ---- render level ------------------------------------------------------------

struct Art {
    body: (u32, Vec<u8>),
    head: (u32, Vec<u8>),
    sprites: (u32, Vec<u8>),
    sword: (u32, Vec<u8>),
}

fn art() -> Art {
    let load = |rel: &str| {
        let (w, _, px) = read_png_rgba(&std::fs::read(root().join(rel)).unwrap()).unwrap();
        (w, px)
    };
    Art { body: load("bodies/body.png"), head: load("heads/head0.png"), sprites: load("sprites.png"), sword: load("swords/sword1.png") }
}

fn sheets(a: &Art) -> PoseSheets<'_> {
    fn sh(i: &(u32, Vec<u8>)) -> Option<Sheet<'_>> {
        Some(Sheet { px: i.1.as_slice(), w: i.0, bpp: 4 })
    }
    PoseSheets { body: sh(&a.body), head: sh(&a.head), sprites: sh(&a.sprites), sword: sh(&a.sword), shield: None, attrs: Default::default() }
}

fn draw(g: &Gani, step: usize, dir: u8, a: &Art) -> Vec<u8> {
    let mut fb = Framebuffer::new(64, 64);
    draw_gani_frame(&mut fb, g, step, dir, &sheets(a), 8, 8);
    fb.pixels.clone()
}

fn blank() -> Vec<u8> {
    Framebuffer::new(64, 64).pixels.clone()
}

#[test]
fn the_pose_ganis_draw_and_differ_from_idle_and_each_other() {
    let a = art();
    for dir in 0..4u8 {
        let idle = draw(&gani("idle.gani"), 0, dir, &a);
        assert_ne!(idle, blank());
        // idle through the GANI renderer is the same picture draw_player_sprites makes
        let mut fb = Framebuffer::new(64, 64);
        draw_player_sprites(&mut fb, &a.body.1, a.body.0, 4, &a.head.1, a.head.0, 4, Some((&a.sprites.1, a.sprites.0, 4)), dir, 8, 8, false);
        assert_eq!(idle, fb.pixels, "dir {dir}: idle.gani and the built-in idle drawing agree");

        let sword = gani("sword.gani");
        let steps: Vec<Vec<u8>> = (0..5).map(|i| draw(&sword, i, dir, &a)).collect();
        let mut all = vec![idle.clone(), draw(&gani("sit.gani"), 0, dir, &a), draw(&gani("hurt.gani"), 0, dir, &a)];
        all.extend(steps);
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                // sword steps 2 and 3 are the same picture on purpose (the thrust is held for a step)
                if (i, j) == (5, 6) {
                    continue;
                }
                assert_ne!(all[i], all[j], "dir {dir}: pictures {i} and {j} are the same");
            }
        }
    }
}

#[test]
fn a_missing_sheet_or_a_bad_step_draws_nothing_and_does_not_panic() {
    let a = art();
    let mut fb = Framebuffer::new(64, 64);
    draw_gani_frame(&mut fb, &gani("sword.gani"), 99, 0, &PoseSheets::default(), 0, 0);
    assert_eq!(fb.pixels, blank());
    // the sword sheet alone draws only the sword
    let only = PoseSheets { sword: sheets(&a).sword, ..PoseSheets::default() };
    draw_gani_frame(&mut fb, &gani("sword.gani"), 2, 2, &only, 8, 8);
    assert_ne!(fb.pixels, blank());
}

#[test]
fn an_attr_sprite_draws_from_its_attribute_sheet() {
    let g = Gani::parse("SPRITE 1 ATTR1 0 0 2 2 hat\nANI\n1 3 4\nANIEND\n").unwrap();
    let px = vec![200u8, 10, 20, 255].repeat(4);
    let mut fb = Framebuffer::new(64, 64);
    draw_gani_frame(&mut fb, &g, 0, 2, &PoseSheets::default(), 8, 8);
    assert_eq!(fb.pixels, blank(), "no hat sheet, nothing drawn");
    let mut sheets = PoseSheets::default();
    sheets.attrs[0] = Some(Sheet { px: &px, w: 2, bpp: 4 });
    draw_gani_frame(&mut fb, &g, 0, 2, &sheets, 8, 8);
    assert_ne!(fb.pixels, blank());
    // ATTR2 does not take the hat's sheet.
    let g2 = Gani::parse("SPRITE 1 ATTR2 0 0 2 2 x\nANI\n1 3 4\nANIEND\n").unwrap();
    let mut fb = Framebuffer::new(64, 64);
    draw_gani_frame(&mut fb, &g2, 0, 2, &sheets, 8, 8);
    assert_eq!(fb.pixels, blank());
}

// ---- idle, walking and swimming ------------------------------------------

/// A session with every default animation, standing on open floor.
fn walker() -> OfflineSession {
    use seihai::assets::{GaniPlayer, Level};
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::from_board(&[0u16; 64 * 64]));
    let mut t = seihai::assets::TileTypes { types: [0; 4096] };
    t.types[1] = 11; // water
    s.tiletypes = Some(t);
    s.player_x = 30.0;
    s.player_y = 30.0;
    s.set_pose_gani(PoseKind::Idle, gani("idle.gani"));
    s.set_walk_gani(GaniPlayer::new(gani("walk.gani")));
    s.set_swim_gani(GaniPlayer::new(gani("swim.gani")));
    s
}

#[test]
fn standing_is_idle_walking_steps_the_walk_frames_and_stopping_is_idle_again() {
    let mut s = walker();
    assert_eq!(pose(&s), Some((PoseKind::Idle, 0)));
    let frames = gani("walk.gani").frame_count();
    assert_eq!(frames, 8, "the default walk has 8 steps");
    // one GANI frame per step taken, the way the footstep cues are stepped
    let mut seen = Vec::new();
    for _ in 0..10 {
        assert!(s.try_move(0.5, 0.0).is_some());
        seen.push(pose(&s).unwrap());
    }
    let expect: Vec<(PoseKind, usize)> = (1..=10).map(|i| (PoseKind::Walk, i % frames)).collect();
    assert_eq!(seen, expect);
    // the sprite value other players get does not depend on the pose
    assert_eq!(s.sprite_value(), s.player_dir);
    assert!(s.stop_walking());
    assert_eq!(pose(&s), Some((PoseKind::Idle, 0)));
    assert!(!s.stop_walking(), "stopping twice changes nothing");
}

#[test]
fn a_blocked_step_runs_in_place() {
    let mut s = walker();
    // walk into the board's east edge until a step is refused
    s.player_x = 58.0;
    while s.try_move(0.5, 0.0).is_some() {}
    let at = s.player_x;
    assert!(at > 58.0 && at < 64.0);
    assert!(s.stop_walking());
    let frame = s.walk_frame().unwrap();
    s.take_sound_events();
    assert!(s.try_move(0.5, 0.0).is_none());
    assert_eq!(s.player_x, at);
    // 2.05 sets the walk gani on every walk tick, moved or not: the legs
    // keep running against the edge until the push pose takes over.
    let next = (frame + 1) % s.walk_gani.as_ref().unwrap().gani.frames.len();
    assert_eq!(s.walk_frame(), Some(next), "a refused step steps the animation");
    assert_eq!(pose(&s), Some((PoseKind::Walk, next)), "a refused step is drawn walking");
    assert!(s.take_sound_events().is_empty(), "no footsteps in place");
}

#[test]
fn in_water_the_swim_animation_is_drawn_and_a_swing_or_hurt_is_not() {
    let mut s = walker();
    s.combat.set_rules(Some(COMBAT_205));
    s.life.set_rules(Some(seihai::protocol::DEATH_205));
    s.set_pose_gani(PoseKind::Sword, gani("sword.gani"));
    let tiles: Vec<u16> = (0..64 * 64).map(|i| if (28..40).contains(&(i % 64)) && (20..40).contains(&(i / 64)) { 1 } else { 0 }).collect();
    s.current_level = Some(seihai::assets::Level::from_board(&tiles));
    s.player_x = 24.0;
    s.player_y = 30.0;
    for _ in 0..6 {
        s.try_move(0.5, 0.0);
    }
    assert!(s.in_water, "the foot is in the pond");
    let p = s.current_pose().unwrap();
    assert_eq!(p.kind, PoseKind::Swim);
    assert!(p.frame < gani("swim.gani").frame_count());
    // the swim animation is not the walk one
    let a = art();
    assert_ne!(draw(&gani("swim.gani"), 0, 2, &a), draw(&gani("walk.gani"), 1, 2, &a));
    assert_ne!(draw(&gani("walk.gani"), 1, 2, &a), draw(&gani("idle.gani"), 0, 2, &a));
}

// ---- carrying -------------------------------------------------------------

#[test]
fn holding_an_object_shows_the_carry_poses_and_sends_the_carry_sprite() {
    let mut s = walker();
    s.set_pose_gani(PoseKind::CarryStill, gani("carrystill.gani"));
    s.set_pose_gani(PoseKind::Carry, gani("carry.gani"));
    s.carried = Some(seihai::offline::Carried { kind: 0, tiles: [[0, 0], [0, 0]] });
    assert_eq!(pose(&s), Some((PoseKind::CarryStill, 0)));
    let dir = s.player_dir;
    assert_eq!(s.sprite_value(), dir + 4 * seihai::combat::CARRY_SPRITE_S, "S = 18 while carrying");
    // walking steps the carry walk with the walk's step
    let mut seen = Vec::new();
    for _ in 0..3 {
        assert!(s.try_move(0.5, 0.0).is_some());
        seen.push(pose(&s).unwrap());
    }
    assert_eq!(seen, vec![(PoseKind::Carry, 1), (PoseKind::Carry, 2), (PoseKind::Carry, 3)]);
    assert_eq!(s.sprite_value(), s.player_dir + 4 * 18);
    // empty hands: back to the plain poses and S = 0
    s.carried = None;
    assert_eq!(pose(&s).map(|p| p.0), Some(PoseKind::Walk));
    assert!(s.stop_walking());
    assert_eq!(pose(&s), Some((PoseKind::Idle, 0)));
    assert_eq!(s.sprite_value(), s.player_dir);
    // the carry pictures differ from idle and walk
    let a = art();
    for d in 0..4u8 {
        assert_ne!(draw(&gani("carrystill.gani"), 0, d, &a), draw(&gani("idle.gani"), 0, d, &a));
        assert_ne!(draw(&gani("carry.gani"), 1, d, &a), draw(&gani("walk.gani"), 1, d, &a));
    }
}

/// A single-direction GANI of `n` steps, looping or not.
fn steps(n: usize, looped: bool) -> Gani {
    let mut text = String::from(if looped { "SINGLEDIRECTION\nLOOP\nANI\n" } else { "SINGLEDIRECTION\nANI\n" });
    for _ in 0..n {
        text.push_str("1 0 0\n\n");
    }
    text.push_str("ANIEND\n");
    Gani::parse(&text).unwrap()
}

#[test]
fn a_dead_player_shows_dead_gani_stepped_by_the_death_timer() {
    let mut s = session();
    s.set_pose_gani(PoseKind::Dead, steps(3, false));
    s.damage_half(100, None);
    s.tick_life(0);
    assert!(s.life.is_dead());
    assert_eq!(pose(&s), Some((PoseKind::Dead, 0)));
    s.tick_life(50);
    assert_eq!(pose(&s), Some((PoseKind::Dead, 1)));
    // Held on the last step.
    s.tick_life(500);
    assert_eq!(pose(&s), Some((PoseKind::Dead, 2)));
}

#[test]
fn the_default_dead_gani_falls_and_lies_down() {
    let dead = gani("dead.gani");
    assert_eq!(dead.frame_count(), 15);
    let a = art();
    for dir in 0..4u8 {
        let hurt = draw(&dead, 0, dir, &a);
        let kneel = draw(&dead, 6, dir, &a);
        let lying = draw(&dead, 14, dir, &a);
        assert_eq!(hurt, draw(&gani("hurt.gani"), 0, dir, &a), "dir {dir}: the fall starts from the hurt pose");
        assert_ne!(kneel, hurt);
        assert_ne!(lying, kneel);
        assert_ne!(lying, blank());
    }
    // Lying is the same whatever the direction.
    assert_eq!(draw(&dead, 14, 0, &a), draw(&dead, 14, 3, &a));
    // In the session, the death timer reaches the last step and holds it.
    let mut s = session();
    s.set_pose_gani(PoseKind::Dead, dead);
    s.damage_half(100, None);
    s.tick_life(0);
    s.tick_life(5000);
    assert_eq!(pose(&s), Some((PoseKind::Dead, 14)));
}

#[test]
fn hurt_gani_steps_and_a_looping_one_lasts_the_hurt_mode() {
    let mut s = session();
    s.set_pose_gani(PoseKind::Hurt, steps(12, false));
    s.receive_hurtplayer(&hurt_packet(1), Some((s.player_x + 2.0, s.player_y)));
    s.tick_combat(150);
    assert_eq!(pose(&s), Some((PoseKind::Hurt, 3)));
    // 12 steps are 600 ms, longer than HURT_POSE_MS: shown to the end.
    s.tick_combat(400);
    assert_eq!(pose(&s), Some((PoseKind::Hurt, 11)));
    s.tick_combat(100);
    assert_eq!(pose(&s), None);
    let mut l = session();
    l.set_pose_gani(PoseKind::Hurt, steps(2, true));
    l.receive_hurtplayer(&hurt_packet(1), Some((l.player_x + 2.0, l.player_y)));
    l.tick_combat(1_550);
    assert_eq!(pose(&l), Some((PoseKind::Hurt, 1)));
}
