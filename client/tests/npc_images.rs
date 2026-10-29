//! Item 23: scripts that change what an NPC looks like. `setimg` and
//! `setimgpart` change the picture, `hide` / `show` toggle the NPC,
//! `showimg` / `hideimg` put images up and take them down, `setcharani`
//! switches an NPC between `idle` and `walk`. Each test runs a real NPC
//! script through the engine and then draws the level with
//! `render::draw_npcs`, using Seihai's own default assets for the animation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use seihai::assets::{read_png_rgba, Gani, Level};
use seihai::offline::{OfflineSession, FOOT_DX, FOOT_DY};
use seihai::protocol::ClientVersion;
use seihai::render::{draw_npcs, Framebuffer, NpcArt, NpcPass, PoseSheets, Sheet};
use seihai::scripting::engine::host::SessionScripts;

const RED: [u8; 3] = [200, 30, 30];
const BLUE: [u8; 3] = [30, 30, 200];
const GREEN: [u8; 3] = [30, 200, 30];

/// A `w` x `h` image whose pixels come from `f(x, y)`.
fn image(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> (u32, Vec<u8>) {
    let mut px = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let c = f(x, y);
            px.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    (w, px)
}

fn solid(c: [u8; 3]) -> (u32, Vec<u8>) {
    image(4, 4, |_, _| c)
}

fn images() -> HashMap<String, (u32, Vec<u8>)> {
    let mut m = HashMap::new();
    m.insert("red.png".to_string(), solid(RED));
    m.insert("blue.png".to_string(), solid(BLUE));
    m.insert("halves.png".to_string(), image(8, 8, |x, _| if x < 4 { RED } else { GREEN }));
    m
}

/// A session with `src` as its level and the player away from the NPCs.
fn session(npc_lines: &str) -> (OfflineSession, SessionScripts) {
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::parse(&format!("GLEVNW01\n{npc_lines}")).unwrap());
    s.player_x = 40.0;
    s.player_y = 40.0;
    let mut sc = SessionScripts::new(ClientVersion::V205);
    sc.load_level(&mut s, "a.nw");
    (s, sc)
}

/// Put the player's feet inside the 2x2 touch box of the NPC at (x, y).
fn touch_npc(s: &mut OfflineSession, sc: &mut SessionScripts, x: f32, y: f32) {
    s.player_x = x + 1.0 - FOOT_DX;
    s.player_y = y + 1.0 - FOOT_DY;
    sc.touch(s);
}

/// The pixel colour at `(x, y)` of a 320 x 320 picture of the level (camera at 0, 0).
fn draw(s: &OfflineSession, imgs: &HashMap<String, (u32, Vec<u8>)>) -> Framebuffer {
    draw_with(s, imgs, PoseSheets::default())
}

fn draw_with(s: &OfflineSession, imgs: &HashMap<String, (u32, Vec<u8>)>, sheets: PoseSheets) -> Framebuffer {
    let mut fb = Framebuffer::new(320, 320);
    draw_both(&mut fb, s, &NpcArt { images: imgs, sheets });
    fb
}

/// Both passes, with no player between them.
fn draw_both(fb: &mut Framebuffer, s: &OfflineSession, art: &NpcArt) {
    draw_npcs(fb, s, art, 0, 0, NpcPass::BelowPlayers);
    draw_npcs(fb, s, art, 0, 0, NpcPass::AbovePlayers);
}

fn at(fb: &Framebuffer, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * fb.width + x) * 3) as usize;
    [fb.pixels[i], fb.pixels[i + 1], fb.pixels[i + 2]]
}

const BLACK: [u8; 3] = [0, 0, 0];

// ---- setimg, setimgpart ------------------------------------------------------

#[test]
fn a_scripted_setimg_changes_what_is_drawn() {
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 10 10\nif (playertouchsme) { setimg blue.png; }\nNPCEND\n");
    // the NPC's own image, at 10 * 16 pixels
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 161, 161), RED);
    assert_eq!(at(&fb, 170, 161), BLACK, "the image is 4 pixels wide");
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert_eq!(s.current_level.as_ref().unwrap().npcs[0].image, "blue.png");
    assert!(sc.images_changed, "the host is told to load the picture");
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 161, 161), BLUE, "the new image is drawn");
}

#[test]
fn a_picture_that_is_not_loaded_draws_nothing_and_does_not_fail() {
    let mut imgs = images();
    imgs.remove("blue.png");
    let (mut s, mut sc) = session("NPC red.png 10 10\nif (playertouchsme) { setimg blue.png; }\nNPCEND\n");
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert_eq!(at(&draw(&s, &imgs), 161, 161), BLACK);
    // when the file arrives it shows
    imgs.insert("blue.png".into(), solid(BLUE));
    assert_eq!(at(&draw(&s, &imgs), 161, 161), BLUE);
}

#[test]
fn setimgpart_draws_a_part_and_setimg_goes_back_to_the_whole_image() {
    let imgs = images();
    let (mut s, mut sc) = session(
        "NPC halves.png 10 10\nif (playertouchsme) { setimgpart halves.png,4,0,4,8; }\nif (timeout) { setimg halves.png; }\nif (playerenters) { timeout = 1; }\nNPCEND\n",
    );
    let whole = draw(&s, &imgs);
    assert_eq!((at(&whole, 161, 161), at(&whole, 167, 161)), (RED, GREEN));
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert_eq!(s.current_level.as_ref().unwrap().npcs[0].image_part, Some((4, 0, 4, 8)));
    let part = draw(&s, &imgs);
    assert_eq!(at(&part, 161, 161), GREEN, "the green half now starts at the NPC's corner");
    assert_eq!(at(&part, 165, 161), BLACK, "and is 4 pixels wide");
    assert_eq!(at(&part, 161, 168), BLACK, "the part is 8 pixels tall: row 8 is outside");
    // setimg resets the part
    for _ in 0..40 {
        sc.tick(&mut s, 50);
    }
    assert_eq!(s.current_level.as_ref().unwrap().npcs[0].image_part, None);
    assert_eq!(at(&draw(&s, &imgs), 161, 161), RED);
}

// ---- hide, show --------------------------------------------------------------

#[test]
fn hide_and_show_toggle_the_drawn_npc() {
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 10 10\nif (playertouchsme) { hide; timeout = 1; }\nif (timeout) { show; }\nNPCEND\n");
    assert_eq!(at(&draw(&s, &imgs), 161, 161), RED);
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert!(!s.current_level.as_ref().unwrap().npcs[0].visible);
    assert_eq!(at(&draw(&s, &imgs), 161, 161), BLACK, "hidden: nothing drawn");
    for _ in 0..20 {
        sc.tick(&mut s, 50);
    }
    assert!(s.current_level.as_ref().unwrap().npcs[0].visible);
    assert_eq!(at(&draw(&s, &imgs), 161, 161), RED);
}

// ---- showimg, hideimg ----------------------------------------------------------

#[test]
fn showimg_puts_an_image_up_and_hideimg_takes_it_down() {
    let imgs = images();
    let (mut s, mut sc) = session(
        "NPC - 2 2\nif (playertouchsme) { showimg 3,blue.png,20,12; showimg 4,red.png,25,12; }\nif (playersays(gone)) { hideimg 3; }\nNPCEND\n",
    );
    assert_eq!(at(&draw(&s, &imgs), 321 - 1, 12 * 16 + 1), BLACK);
    touch_npc(&mut s, &mut sc, 2.0, 2.0);
    let level = s.current_level.as_ref().unwrap();
    assert_eq!(level.npcs[0].images.iter().map(|i| (i.index, i.file.as_str())).collect::<Vec<_>>(), [(3, "blue.png"), (4, "red.png")]);
    assert!(sc.images_changed);
    let mut fb = Framebuffer::new(500, 300);
    draw_both(&mut fb, &s, &NpcArt { images: &imgs, sheets: PoseSheets::default() });
    let px = |x: u32, y: u32| {
        let i = ((y * fb.width + x) * 3) as usize;
        [fb.pixels[i], fb.pixels[i + 1], fb.pixels[i + 2]]
    };
    assert_eq!(px(20 * 16 + 1, 12 * 16 + 1), BLUE);
    assert_eq!(px(25 * 16 + 1, 12 * 16 + 1), RED);
    // the NPC itself had no image, so nothing is drawn at its own place
    assert_eq!(px(2 * 16 + 1, 2 * 16 + 1), BLACK);
    // hideimg deletes one image
    sc.chat(&mut s, "gone");
    let level = s.current_level.as_ref().unwrap();
    assert_eq!(level.npcs[0].images.len(), 1);
    let mut fb = Framebuffer::new(500, 300);
    draw_both(&mut fb, &s, &NpcArt { images: &imgs, sheets: PoseSheets::default() });
    let i = ((12 * 16 + 1) * fb.width + 20 * 16 + 1) as usize * 3;
    assert_eq!(&fb.pixels[i..i + 3], &BLACK);
}

#[test]
fn showimg_with_an_index_in_use_replaces_it_and_the_same_file_only_moves() {
    let imgs = images();
    let (mut s, mut sc) = session(
        "NPC - 2 2\nif (playertouchsme) { showimg 1,blue.png,3,3; showimg 1,red.png,3,3; }\nif (playersays(move)) { showimg 1,red.png,6,3; }\nNPCEND\n",
    );
    touch_npc(&mut s, &mut sc, 2.0, 2.0);
    let imgs_of = |s: &OfflineSession| s.current_level.as_ref().unwrap().npcs[0].images.clone();
    assert_eq!(imgs_of(&s).len(), 1);
    assert_eq!(imgs_of(&s)[0].file, "red.png");
    sc.chat(&mut s, "move");
    assert_eq!(imgs_of(&s).len(), 1);
    assert_eq!((imgs_of(&s)[0].x, imgs_of(&s)[0].y), (6.0, 3.0));
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 6 * 16 + 1, 3 * 16 + 1), RED);
    assert_eq!(at(&fb, 3 * 16 + 1, 3 * 16 + 1), BLACK);
}

#[test]
fn a_hidden_npc_keeps_its_shown_images() {
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 2 2\nif (playertouchsme) { showimg 1,blue.png,8,8; hide; }\nNPCEND\n");
    touch_npc(&mut s, &mut sc, 2.0, 2.0);
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 2 * 16 + 1, 2 * 16 + 1), BLACK, "the NPC is hidden");
    assert_eq!(at(&fb, 8 * 16 + 1, 8 * 16 + 1), BLUE, "its image stays up");
}

// ---- animations: setcharani idle <-> walk -----------------------------------

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("assets").join("default")
}

struct Art {
    body: (u32, Vec<u8>),
    head: (u32, Vec<u8>),
    sprites: (u32, Vec<u8>),
}

fn art() -> Art {
    let load = |rel: &str| {
        let (w, _, px) = read_png_rgba(&std::fs::read(root().join(rel)).unwrap()).unwrap();
        (w, px)
    };
    Art { body: load("bodies/body.png"), head: load("heads/head0.png"), sprites: load("sprites.png") }
}

fn sheets(a: &Art) -> PoseSheets<'_> {
    fn sh(i: &(u32, Vec<u8>)) -> Option<Sheet<'_>> {
        Some(Sheet { px: i.1.as_slice(), w: i.0, bpp: 4 })
    }
    PoseSheets { body: sh(&a.body), head: sh(&a.head), sprites: sh(&a.sprites), sword: None, shield: None, attrs: Default::default() }
}

fn with_default_ganis(s: &mut OfflineSession) {
    for n in ["idle", "walk"] {
        s.set_npc_gani(n, Gani::load(root().join("gani").join(format!("{n}.gani"))).unwrap());
    }
}

const WALKER: &str = "NPC - 10 10\nif (playerenters) { showcharacter; setcharani idle,; }\n\
    if (playertouchsme) { setcharani walk,; }\nif (playersays(stop)) { setcharani idle,; }\nNPCEND\n";

#[test]
fn setcharani_switches_an_npc_between_idle_and_walk_and_steps_it() {
    let a = art();
    let imgs = HashMap::new();
    let (mut s, mut sc) = session(WALKER);
    with_default_ganis(&mut s);
    let npc = |s: &OfflineSession| {
        let n = &s.current_level.as_ref().unwrap().npcs[0];
        (n.ani.clone(), n.ani_frame)
    };
    // showcharacter + setcharani idle: drawn as the idle figure
    assert_eq!(npc(&s), (Some("idle".to_string()), 0));
    let idle = draw_with(&s, &imgs, sheets(&a));
    assert_ne!(idle.pixels, draw(&s, &imgs).pixels, "something is drawn where the bare NPC draws nothing");
    // a tick on idle (1 step, no loop) changes nothing
    assert!(!s.tick_npc_anis(1000));
    assert_eq!(draw_with(&s, &imgs, sheets(&a)).pixels, idle.pixels);
    // touch: walk. Frame 0 of walk is not the idle picture
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert_eq!(npc(&s), (Some("walk".to_string()), 0));
    let walk0 = draw_with(&s, &imgs, sheets(&a));
    assert_ne!(walk0.pixels, idle.pixels);
    // every 100 ms is one step; the 8 steps are all different pictures and the loop wraps
    let mut seen = vec![walk0.pixels.clone()];
    for k in 1..8 {
        assert!(s.tick_npc_anis(100));
        assert_eq!(npc(&s).1, k);
        seen.push(draw_with(&s, &imgs, sheets(&a)).pixels);
    }
    for i in 0..8 {
        for j in i + 1..8 {
            assert_ne!(seen[i], seen[j], "walk steps {i} and {j} look alike");
        }
    }
    assert!(s.tick_npc_anis(100));
    assert_eq!(npc(&s), (Some("walk".to_string()), 0), "walk loops");
    // back to idle with a script
    sc.chat(&mut s, "stop");
    assert_eq!(npc(&s), (Some("idle".to_string()), 0));
    assert_eq!(draw_with(&s, &imgs, sheets(&a)).pixels, idle.pixels);
}

#[test]
fn an_animation_that_is_not_loaded_falls_back_to_the_image() {
    let a = art();
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 10 10\nif (playertouchsme) { setcharani walk,; }\nNPCEND\n");
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    assert_eq!(s.wanted_npc_anis(), ["walk"], "the host is told which file to load");
    assert_eq!(at(&draw_with(&s, &imgs, sheets(&a)), 161, 161), RED);
    with_default_ganis(&mut s);
    assert!(s.wanted_npc_anis().is_empty());
    assert_ne!(at(&draw_with(&s, &imgs, sheets(&a)), 161, 161), RED);
}

#[test]
fn setimg_with_a_gani_name_picks_the_animation_and_a_plain_image_ends_it() {
    let a = art();
    let imgs = images();
    let (mut s, mut sc) =
        session("NPC red.png 10 10\nif (playertouchsme) { setimg walk.gani; }\nif (playersays(back)) { setimg blue.png; }\nNPCEND\n");
    with_default_ganis(&mut s);
    touch_npc(&mut s, &mut sc, 10.0, 10.0);
    let n = s.current_level.as_ref().unwrap().npcs[0].clone();
    assert_eq!(n.ani.as_deref(), Some("walk"));
    assert_eq!(n.image, "red.png", "the image name is left alone");
    sc.chat(&mut s, "back");
    let n = &s.current_level.as_ref().unwrap().npcs[0];
    assert_eq!((n.ani.as_deref(), n.image.as_str()), (None, "blue.png"));
    assert_eq!(at(&draw_with(&s, &imgs, sheets(&a)), 161, 161), BLUE);
}

#[test]
fn a_gani_that_does_not_loop_plays_once_then_follows_setbackto() {
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::parse("GLEVNW01\nNPC - 5 5\nNPCEND\n").unwrap());
    // two steps, no LOOP, back to idle; idle is one step
    let two = "SETBACKTO idle\nANI\n1 0 0\n1 0 0\n1 0 0\n1 0 0\n\n1 0 0\n1 0 0\n1 0 0\n1 0 0\nANIEND\n";
    s.set_npc_gani("hit", Gani::parse(two).unwrap());
    s.set_npc_gani("idle", Gani::parse("ANI\n1 0 0\n1 0 0\n1 0 0\n1 0 0\nANIEND\n").unwrap());
    s.current_level.as_mut().unwrap().npcs[0].ani = Some("hit".into());
    assert!(s.tick_npc_anis(100));
    assert_eq!(s.current_level.as_ref().unwrap().npcs[0].ani_frame, 1);
    assert!(s.tick_npc_anis(100));
    let n = &s.current_level.as_ref().unwrap().npcs[0];
    assert_eq!((n.ani.as_deref(), n.ani_frame), (Some("idle"), 0));
    assert!(!s.tick_npc_anis(5000), "idle is one step, nothing changes");
    // with no SETBACKTO the last step stays
    s.set_npc_gani("hit", Gani::parse("ANI\n1 0 0\n1 0 0\n1 0 0\n1 0 0\n\n1 0 0\n1 0 0\n1 0 0\n1 0 0\nANIEND\n").unwrap());
    s.current_level.as_mut().unwrap().npcs[0].ani = Some("hit".into());
    s.tick_npc_anis(100);
    s.tick_npc_anis(100);
    s.tick_npc_anis(100);
    let n = &s.current_level.as_ref().unwrap().npcs[0];
    assert_eq!((n.ani.as_deref(), n.ani_frame), (Some("hit"), 1));
}

#[test]
fn ani_names_are_read_like_the_scripts_write_them() {
    use seihai::npcart::ani_name;
    assert_eq!(ani_name("walk"), Some("walk".into()));
    assert_eq!(ani_name(" Walk.GANI "), Some("walk".into()));
    assert_eq!(ani_name("\"idle\""), Some("idle".into()));
    assert_eq!(ani_name(""), None);
    assert_eq!(ani_name(".gani"), None);
}

// ---- smoke test against real NPC scripts ----------------------------------------

/// Runs the scripts of a private script collection that use the image and
/// animation commands, and checks facts derived from the runs (counts and
/// invariants). `SEIHAI_SCRIPT_CORPUS` is a text file with one script per
/// line (blank lines between); nothing of the scripts is printed or kept.
/// Ignored without it: the collection is private and not in the repository.
#[test]
#[ignore]
fn real_npc_scripts_that_change_images_run_cleanly() {
    let Ok(path) = std::env::var("SEIHAI_SCRIPT_CORPUS") else {
        eprintln!("SEIHAI_SCRIPT_CORPUS not set; skipped");
        return;
    };
    let text = String::from_utf8_lossy(&std::fs::read(&path).expect("read the script file")).into_owned();
    let uses = |t: &str| ["setimg", "setgif", "showimg", "hideimg", "setcharani", "setani", "showcharacter", "changeimgpart"].iter().any(|c| t.contains(c));
    let scripts: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty() && uses(l) && !l.contains("NPCEND")).collect();
    assert!(!scripts.is_empty(), "no script in the file uses the image commands");
    let mut counts = [0usize; 6]; // ran, image changed, showimg up at the end, ani at the end, hidden at the end, part set
    for era in [ClientVersion::V205, ClientVersion::V222] {
        for src in &scripts {
            let mut s = OfflineSession::new();
            s.current_level = Some(Level::parse(&format!("GLEVNW01\nNPC pic.png 10 10\n{src}\nNPCEND\n")).unwrap());
            s.player_x = 40.0;
            s.player_y = 40.0;
            let mut sc = SessionScripts::new(era);
            sc.load_level(&mut s, "a.nw");
            let mut changed = sc.images_changed;
            touch_npc(&mut s, &mut sc, 10.0, 10.0);
            for _ in 0..60 {
                sc.tick(&mut s, 50);
                s.tick_npc_anis(50);
            }
            for said in ["yes", "no", "hello"] {
                sc.chat(&mut s, said);
            }
            changed |= sc.images_changed;
            let n = &s.current_level.as_ref().unwrap().npcs[0];
            // invariants of the state a script can leave
            let mut idx: Vec<i64> = n.images.iter().map(|i| i.index).collect();
            let sorted = idx.windows(2).all(|w| w[0] < w[1]);
            idx.dedup();
            assert!(sorted && idx.len() == n.images.len(), "image indexes are unique and sorted");
            assert!(n.images.iter().all(|i| i.x.is_finite() && i.y.is_finite()));
            if let Some((x, y, w, h)) = n.image_part {
                assert!(x >= 0 && y >= 0 && w >= 0 && h >= 0 && w <= 220 && h <= 220, "part inside the clamps");
            }
            // drawing never panics, whatever the images are
            let imgs = images();
            let _ = draw(&s, &imgs);
            counts[0] += 1;
            counts[1] += usize::from(changed);
            counts[2] += usize::from(!n.images.is_empty());
            counts[3] += usize::from(n.ani.is_some());
            counts[4] += usize::from(!n.visible);
            counts[5] += usize::from(n.image_part.is_some());
        }
    }
    eprintln!(
        "{} script runs ({} scripts x 2 versions): image changed {}, showimg up {}, animation {}, hidden {}, part {}",
        counts[0],
        scripts.len(),
        counts[1],
        counts[2],
        counts[3],
        counts[4],
        counts[5]
    );
    assert!(counts[1] > 0, "some script changed an image");
}

/// One pass of the level's NPCs on a fresh picture.
fn draw_pass(s: &OfflineSession, imgs: &HashMap<String, (u32, Vec<u8>)>, pass: NpcPass) -> Framebuffer {
    let mut fb = Framebuffer::new(320, 320);
    draw_npcs(&mut fb, s, &NpcArt { images: imgs, sheets: PoseSheets::default() }, 0, 0, pass);
    fb
}

#[test]
fn drawoverplayer_npcs_and_showimg_images_draw_after_the_players() {
    let imgs = images();
    let (s, _sc) = session(
        "NPC red.png 2 2\nif (playerenters) { drawoverplayer; }\nNPCEND\n\
         NPC blue.png 5 2\nif (playerenters) { drawunderplayer; }\nNPCEND\n\
         NPC blue.png 8 2\nNPCEND\n\
         NPC red.png 11 2\nif (playerenters) { drawaslight; showimg 1,blue.png,14,2; }\nNPCEND\n",
    );
    let below = draw_pass(&s, &imgs, NpcPass::BelowPlayers);
    let above = draw_pass(&s, &imgs, NpcPass::AbovePlayers);
    let p = |x: u32| x * 16 + 1;
    // drawoverplayer: only after the players.
    assert_eq!((at(&below, p(2), p(2)), at(&above, p(2), p(2))), (BLACK, RED));
    // drawunderplayer and the default height: before the players.
    assert_eq!((at(&below, p(5), p(2)), at(&above, p(5), p(2))), (BLUE, BLACK));
    assert_eq!((at(&below, p(8), p(2)), at(&above, p(8), p(2))), (BLUE, BLACK));
    // drawaslight and showimg images: after the players.
    assert_eq!((at(&below, p(11), p(2)), at(&above, p(11), p(2))), (BLACK, RED));
    assert_eq!((at(&below, p(14), p(2)), at(&above, p(14), p(2))), (BLACK, BLUE));
}

#[test]
fn drawunderplayer_npcs_draw_before_the_default_ones() {
    let imgs = images();
    // The same place: the default NPC (red) covers the one under the player.
    let (s, _sc) = session("NPC red.png 2 2\nNPCEND\nNPC blue.png 2 2\nif (playerenters) { drawunderplayer; }\nNPCEND\n");
    assert_eq!(at(&draw_pass(&s, &imgs, NpcPass::BelowPlayers), 2 * 16 + 1, 2 * 16 + 1), RED);
}

#[test]
fn blockagain_puts_an_npc_back_with_the_players() {
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 2 2\nif (playerenters) { drawoverplayer; }\nif (playersays(back)) { blockagain; }\nNPCEND\n");
    assert_eq!(at(&draw_pass(&s, &imgs, NpcPass::BelowPlayers), 2 * 16 + 1, 2 * 16 + 1), BLACK);
    sc.chat(&mut s, "back");
    assert_eq!(at(&draw_pass(&s, &imgs, NpcPass::BelowPlayers), 2 * 16 + 1, 2 * 16 + 1), RED);
}

#[test]
fn setcoloreffect_tints_and_blends_the_npc_picture() {
    let imgs = images();
    // Half the red and alpha 0.5 over black: 200 * 0.5 * 0.5 = 50.
    let (s, _sc) = session("NPC red.png 2 2\nif (playerenters) { drawoverplayer; setcoloreffect 0.5,1,1,0.5; }\nNPCEND\n");
    let fb = draw_pass(&s, &imgs, NpcPass::AbovePlayers);
    assert_eq!(at(&fb, 2 * 16 + 1, 2 * 16 + 1), [50, 15, 15]);
}

#[test]
fn setzoomeffect_scales_about_the_centre() {
    let imgs = images();
    // red.png is 4x4 at (32, 32); zoom 2 makes it 8x8 from (30, 30).
    let (s, _sc) = session("NPC red.png 2 2\nif (playerenters) { setzoomeffect 2; }\nNPCEND\n");
    let fb = draw_pass(&s, &imgs, NpcPass::BelowPlayers);
    assert_eq!(at(&fb, 30, 30), RED);
    assert_eq!(at(&fb, 37, 37), RED);
    assert_eq!(at(&fb, 29, 30), BLACK);
    assert_eq!(at(&fb, 38, 37), BLACK);
}

#[test]
fn changeimgcolors_and_changeimgzoom_change_one_showimg_image() {
    let imgs = images();
    let (s, _sc) = session(
        "NPC - 2 2\nif (playerenters) { showimg 1,blue.png,10,2; showimg 2,blue.png,14,2; changeimgcolors 1,1,1,0,1; changeimgzoom 2,0.5; }\nNPCEND\n",
    );
    let fb = draw_pass(&s, &imgs, NpcPass::AbovePlayers);
    assert_eq!(at(&fb, 10 * 16 + 1, 2 * 16 + 1), [30, 30, 0], "blue taken out");
    // 4x4 at half size is 2x2 from (14*16 + 1, 33).
    assert_eq!(at(&fb, 14 * 16 + 1, 2 * 16 + 1), BLUE);
    assert_eq!(at(&fb, 14 * 16, 2 * 16), BLACK);
}

#[test]
fn seteffect_shades_the_map_under_the_lights() {
    let imgs = images();
    let (s, _sc) = session(
        "NPC red.png 2 2\nif (playerenters) { seteffect 0,0,1,0.5; }\nNPCEND\n\
         NPC red.png 6 2\nif (playerenters) { drawaslight; setcoloreffect 1,1,1,0.5; }\nNPCEND\n",
    );
    assert_eq!(s.screen_effect, [0.0, 0.0, 1.0, 0.5]);
    let mut fb = Framebuffer::new(320, 320);
    let art = NpcArt { images: &imgs, sheets: PoseSheets::default() };
    draw_npcs(&mut fb, &s, &art, 0, 0, NpcPass::BelowPlayers);
    draw_npcs(&mut fb, &s, &art, 0, 0, NpcPass::AbovePlayers);
    // The default NPC is under the shading: red halfway to blue.
    assert_eq!(at(&fb, 2 * 16 + 1, 2 * 16 + 1), [100, 15, 143]);
    // The light is added on top of the shaded black: 128 blue + half red.
    assert_eq!(at(&fb, 6 * 16 + 1, 2 * 16 + 1), [100, 15, 143]);
}
