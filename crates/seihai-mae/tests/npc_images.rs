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
    draw_both(&mut fb, s, &NpcArt { images: imgs, sheets, gani_attrs: true });
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
    draw_both(&mut fb, &s, &NpcArt { images: &imgs, sheets: PoseSheets::default(), gani_attrs: true });
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
    draw_both(&mut fb, &s, &NpcArt { images: &imgs, sheets: PoseSheets::default(), gani_attrs: true });
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

// ---- showimg from a weapon ------------------------------------------------------

/// Weapon NPCs act as the player: their `showimg` images go on the
/// player's level at one over the player (Graal_analysis
/// `npc-library/gs1-command-reference-205.md`, `showimg` and the "weapon
/// NPCs act as the player" rule). They are not any NPC's images.
#[test]
fn a_weapon_showimg_is_drawn_over_the_player_and_hideimg_takes_it_down() {
    use seihai::scripting::engine::host::WeaponScript;
    let imgs = images();
    let (mut s, mut sc) = session("NPC red.png 2 2
NPCEND
");
    let script = "if (weaponfired) { if (this.up) { hideimg 7; this.up = 0; } else { showimg 7,blue.png,12,6; changeimgpart 7,0,0,2,2; this.up = 1; } }";
    sc.sync_weapons(&mut s, &[WeaponScript { name: "Lamp", image: "", script }]);
    sc.world.weapon_fired("Lamp");
    sc.apply(&mut s);
    assert!(sc.images_changed, "the host is told to load the picture");
    assert!(s.current_level.as_ref().unwrap().npcs[0].images.is_empty(), "not the NPC's image");
    assert_eq!(s.player_images.len(), 1);
    assert_eq!(s.player_images[0].height, 1, "one over the player");
    // drawn with the NPCs over the players, in its part only
    let fb = draw_pass(&s, &imgs, NpcPass::AbovePlayers);
    assert_eq!(at(&fb, 12 * 16, 6 * 16), BLUE);
    assert_eq!(at(&fb, 12 * 16 + 2, 6 * 16), BLACK, "the part is 2 pixels wide");
    assert_eq!(at(&draw_pass(&s, &imgs, NpcPass::BelowPlayers), 12 * 16, 6 * 16), BLACK);
    sc.world.weapon_fired("Lamp");
    sc.apply(&mut s);
    assert!(s.player_images.is_empty());
    assert_eq!(at(&draw(&s, &imgs), 12 * 16, 6 * 16), BLACK);
}

#[test]
fn a_weapon_image_belongs_to_the_level_it_was_put_up_on() {
    use seihai::scripting::engine::host::WeaponScript;
    let (mut s, mut sc) = session("");
    sc.sync_weapons(&mut s, &[WeaponScript { name: "Lamp", image: "", script: "if (weaponfired) { showimg 1,blue.png,3,3; }" }]);
    sc.world.weapon_fired("Lamp");
    sc.apply(&mut s);
    assert_eq!(s.player_images.len(), 1);
    s.current_level = Some(Level::parse("GLEVNW01\n").unwrap());
    sc.load_level(&mut s, "b.nw");
    assert!(s.player_images.is_empty());
}

// ---- a real NPC script ------------------------------------------------------------

/// One NPC block (`NPC ...` to `NPCEND`) of a script pack in
/// `tests/fixtures/npc-scripts`, by the name on its `// =====` line.
fn fixture_npc(pack: &str, name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../seihai-core/tests/fixtures/npc-scripts").join(pack);
    let text = String::from_utf8_lossy(&std::fs::read(path).expect("read the pack")).into_owned();
    let start = text.find(&format!("// ===== {name} =====")).expect("the NPC is in the pack");
    let body = &text[start..];
    let npc = body.find("\nNPC ").expect("an NPC line") + 1;
    let end = body[npc..].find("\nNPCEND").expect("NPCEND") + npc;
    format!("{}\nNPCEND\n", &body[npc..end])
}

/// The Mollusk's chest from Phoenix's boomerang quest, unchanged: on
/// `playerenters` it `setimg`s the closed chest, on a touch it `setimg`s
/// the open one, puts the key up over it with `showimg 300`, sleeps 3
/// seconds and takes the key down with `hideimg 300`.
#[test]
fn a_real_chest_script_opens_shows_its_key_and_hides_it() {
    let npc = fixture_npc("02_phoenix.npc.txt", "02_phoenix/PhoenixNPC_Phoenix_boomerang_quest/quest_boom2__001__31_10.npc.txt");
    assert!(npc.starts_with("NPC personalchest.png 31 10\n"));
    assert!(npc.contains("setimg personalchest2.png;") && npc.contains("showimg 300,") && npc.contains("hideimg 300;"));
    let mut imgs = HashMap::new();
    imgs.insert("personalchest.png".to_string(), solid(RED));
    imgs.insert("personalchest2.png".to_string(), solid(BLUE));
    imgs.insert("ph_smallkey2.png".to_string(), solid(GREEN));
    let (mut s, mut sc) = session(&npc);
    let draw = |s: &OfflineSession| {
        let mut fb = Framebuffer::new(640, 320);
        draw_both(&mut fb, s, &NpcArt { images: &imgs, sheets: PoseSheets::default(), gani_attrs: true });
        fb
    };
    let (chest, key) = ((31 * 16 + 1, 10 * 16 + 1), (31 * 16 + 8 + 1, 9 * 16 + 1));
    let fb = draw(&s);
    assert_eq!((at(&fb, chest.0, chest.1), at(&fb, key.0, key.1)), (RED, BLACK), "closed, no key");
    touch_npc(&mut s, &mut sc, 31.0, 10.0);
    assert!(sc.images_changed);
    let fb = draw(&s);
    assert_eq!(at(&fb, chest.0, chest.1), BLUE, "setimg: the chest is open");
    assert_eq!(at(&fb, key.0, key.1), GREEN, "showimg 300: the key is up at x+0.5, y-1");
    for _ in 0..62 {
        sc.tick(&mut s, 50);
    }
    let fb = draw(&s);
    assert_eq!((at(&fb, chest.0, chest.1), at(&fb, key.0, key.1)), (BLUE, BLACK), "hideimg 300 after the sleep");
    assert!(s.current_level.as_ref().unwrap().npcs[0].images.is_empty());
}

/// The Hammer weapon from the NPC library (`npc-0565`, `hammer.txt`),
/// unchanged. Fired, it runs ten 0.05 s steps: each of the first nine puts
/// `wham<dir>.gif` up as image 1 next to the player, the tenth takes it
/// down with `hideimg 1`.
const HAMMER: &str = "if (playertouchsme) { toweapons Hammer; set gothammer; set alreadygothammer; destroy; } if (playerenters) { hideimg 1; this.step = 0; timeout = 0; show; } if (weaponfired && this.step<=0) { hideimg 1; hidesword 0.5; freezeplayer 0.5; timeout = 0.05; this.step = 0; } if (timeout) { this.step++; if (this.step<10) { if (this.step<3) { if (playerdir==0) playersprite = 11; else if (playerdir==2) playersprite = 6; else playersprite = 9; } else { if (playerdir==0) playersprite = 10; else if (playerdir==2) playersprite = 11; else playersprite = 11; } if (this.step==6) { set teststake; set testhurt; } if (playerdir==0) showimg 1,wham0.gif,playerx-0.75,playery-1.7; if (playerdir==1) showimg 1,wham1.gif,playerx-1.7,playery-0.2; if (playerdir==2) showimg 1,wham2.gif,playerx+0.6,playery-0.4; if (playerdir==3) showimg 1,wham3.gif,playerx+1.5,playery-0.2; timeout = 0.05; } else { playersprite = 0; hideimg 1; this.step = 0; } } if (teststake) { unset teststake; putx = playerx + 0.5; puty = playery + 2; if (playerdir==0) puty -= 3; if (playerdir==1) { putx -= 2; puty -= 0.5; } if (playerdir==2) { putx += 0.5; puty += 1; } if (playerdir==3) { putx += 2; puty -= 0.5; } putx = int(putx); puty = int(puty); set isstake; for (i=0; i<2; i++) for (j=0; j<2; j++) { b = board[putx+i+64*(puty+j)]; bx = b % 16; by = int(b / 16); if (!(bx==10+i && by==20+j) && !(bx==4+i && by==103+j)) unset isstake; } if (isstake) { unset isstake; board[putx+64*puty] = 10+112*16; board[putx+1+64*puty] = 11+112*16; board[putx+64*(puty+1)] = 10+113*16; board[putx+1+64*(puty+1)] = 11+113*16; updateboard putx,puty,2,2; play axe.wav; } else play arrowon.wav; } if (testhurt) { unset testhurt; testx = playerx+1.5; testy = playery+2; if (playerdir==0) testy-=2; if (playerdir==1) testx-=2; if (playerdir==2) testy+=2; if (playerdir==3) testx+=2; for (i=0; i<compuscount; i++) { if (abs(testx-(compus[i].x+1))<=1 && abs(testy-(compus[i].y+1))<=1.5 && compus[i].mode!=3 && compus[i].mode!=5) hitcompu i,3,testx,testy; } for (i=1; i<playerscount; i++) { if (abs(testx-(players[i].x+1.5))<=1 && abs(testy-(players[i].y+2))<=1.5) { hitplayer i,1,testx,testy; // putleaps 2,testx-1,testy-1; // unset gothammer; // destroy; } } }";

#[test]
fn a_real_hammer_weapon_swings_its_image_up_and_takes_it_down() {
    use seihai::scripting::engine::host::WeaponScript;
    let mut imgs = HashMap::new();
    imgs.insert("wham2.gif".to_string(), solid(RED));
    imgs.insert("wham3.gif".to_string(), solid(BLUE));
    let (mut s, mut sc) = session("");
    sc.sync_weapons(&mut s, &[WeaponScript { name: "Hammer", image: "", script: HAMMER }]);
    s.player_x = 10.0;
    s.player_y = 10.0;
    s.player_dir = 2;
    let fire = |s: &mut OfflineSession, sc: &mut SessionScripts| {
        sc.sync_player(s);
        sc.world.weapon_fired("Hammer");
        sc.apply(s);
    };
    // Facing down the image is at (playerx + 0.6, playery - 0.4) = (10.6, 9.6).
    let down = (10 * 16 + 10, 9 * 16 + 10);
    fire(&mut s, &mut sc);
    assert!(s.player_images.is_empty(), "nothing until the first step");
    sc.tick(&mut s, 50);
    assert!(sc.images_changed);
    assert_eq!(s.player_images.len(), 1);
    assert_eq!(s.player_images[0].file, "wham2.gif");
    assert_eq!(at(&draw(&s, &imgs), down.0, down.1), RED, "drawn over the player at its place");
    for _ in 0..8 {
        sc.tick(&mut s, 50);
    }
    assert_eq!(s.player_images.len(), 1, "still up at step 9");
    sc.tick(&mut s, 50);
    assert!(s.player_images.is_empty(), "hideimg 1 at step 10");
    assert_eq!(at(&draw(&s, &imgs), down.0, down.1), BLACK);
    // Facing right: wham3.gif at (playerx + 1.5, playery - 0.2) = (11.5, 9.8).
    s.player_dir = 3;
    fire(&mut s, &mut sc);
    sc.tick(&mut s, 50);
    assert_eq!(s.player_images[0].file, "wham3.gif");
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 11 * 16 + 8 + 1, 9 * 16 + 13), BLUE);
    assert_eq!(at(&fb, down.0, down.1), BLACK, "one image, moved: index 1 is reused");
}

/// The rat of `prtrain.graal` (NPC library `npc-0719`; the scrape's
/// trailing `*=ratgf.gif`, the next record's header, is left off). A touch
/// says sign 1 and `setgif`s the rat facing front; when the sign closes it
/// `setgif`s it back.
const RAT: &str = "if (playertouchsme) { say 1; setgif ratf.gif;} if (exploded || washit || wasshooted || waspelt) {hurt1} if (playerendsreading) {setgif ratb.gif}";

#[test]
fn a_real_setgif_level_npc_changes_its_picture_and_back() {
    let mut imgs = HashMap::new();
    imgs.insert("ratb.gif".to_string(), solid(RED));
    imgs.insert("ratf.gif".to_string(), solid(BLUE));
    let (mut s, mut sc) = session(&format!("NPC ratb.gif 15 10\n{RAT}\nNPCEND\nSIGN 1 1\nzero\nSIGNEND\nSIGN 2 2\nSqueak!\nSIGNEND\n"));
    let px = (15 * 16 + 1, 10 * 16 + 1);
    assert_eq!(at(&draw(&s, &imgs), px.0, px.1), RED);
    touch_npc(&mut s, &mut sc, 15.0, 10.0);
    assert_eq!(s.reading.as_deref(), Some("Squeak!"));
    assert!(sc.images_changed);
    assert_eq!(at(&draw(&s, &imgs), px.0, px.1), BLUE, "setgif ratf.gif");
    // A frame with the sign open, then the player closes it.
    sc.tick(&mut s, 50);
    assert_eq!(at(&draw(&s, &imgs), px.0, px.1), BLUE);
    s.reading = None;
    sc.tick(&mut s, 50);
    assert_eq!(at(&draw(&s, &imgs), px.0, px.1), RED, "setgif ratb.gif when the sign closes");
}

// ---- animations: setcharani idle <-> walk -----------------------------------

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("assets").join("default")
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

/// hastur (6.00r2): the gralat NPCs' gani prop is `zlttp_gralats,3`, the
/// name and then its parameter. The file asked for is `zlttp_gralats.gani`
/// (the server answered "not found" for `zlttp_gralats,3.gani`), and the NPC
/// plays it once it is loaded, whatever the value.
#[test]
fn a_gani_prop_with_parameters_names_the_file_before_the_comma() {
    use seihai::npcart::ani_file;
    assert_eq!(ani_file("zlttp_gralats,3"), "zlttp_gralats");
    assert_eq!(ani_file("idle"), "idle");
    assert_eq!(ani_file(""), "");
    let mut s = OfflineSession::new();
    s.current_level = Some(Level::parse("GLEVNW01\nNPC - 5 5\nNPCEND\nNPC - 9 5\nNPCEND\n").unwrap());
    s.current_level.as_mut().unwrap().npcs[0].ani = Some("zlttp_gralats,3".into());
    s.current_level.as_mut().unwrap().npcs[1].ani = Some("zlttp_gralats,1".into());
    assert_eq!(s.wanted_npc_anis(), vec!["zlttp_gralats".to_string()]);
    let two = "LOOP\nANI\n1 0 0\n1 0 0\n1 0 0\n1 0 0\n\n1 0 0\n1 0 0\n1 0 0\n1 0 0\nANIEND\n";
    s.set_npc_gani("zlttp_gralats", Gani::parse(two).unwrap());
    assert!(s.wanted_npc_anis().is_empty());
    assert!(s.npc_gani("zlttp_gralats,3").is_some());
    assert!(s.tick_npc_anis(100));
    let n = &s.current_level.as_ref().unwrap().npcs[0];
    // The prop keeps its parameter (the server sync compares it as sent).
    assert_eq!((n.ani.as_deref(), n.ani_frame), (Some("zlttp_gralats,3"), 1));
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
    // The scripts that put images up on a weapon event, run as weapons:
    // fired, then three seconds of ticks, watching the player's images.
    use seihai::scripting::engine::host::WeaponScript;
    let weapons: Vec<&str> = scripts.iter().copied().filter(|s| s.contains("weaponfired") && s.contains("showimg")).collect();
    let mut wcounts = [0usize; 3]; // ran, an image was up, all down at the end
    for era in [ClientVersion::V205, ClientVersion::V222] {
        for src in &weapons {
            let mut s = OfflineSession::new();
            s.current_level = Some(Level::parse("GLEVNW01\n").unwrap());
            s.player_x = 30.0;
            s.player_y = 30.0;
            let mut sc = SessionScripts::new(era);
            sc.load_level(&mut s, "a.nw");
            sc.sync_weapons(&mut s, &[WeaponScript { name: "W", image: "", script: src }]);
            sc.sync_player(&s);
            sc.world.weapon_fired("W");
            sc.apply(&mut s);
            let mut up = !s.player_images.is_empty();
            for _ in 0..60 {
                sc.tick(&mut s, 50);
                up |= !s.player_images.is_empty();
                let mut idx: Vec<i64> = s.player_images.iter().map(|i| i.index).collect();
                let sorted = idx.windows(2).all(|w| w[0] < w[1]);
                idx.dedup();
                assert!(sorted && idx.len() == s.player_images.len(), "weapon image indexes are unique and sorted");
            }
            let _ = draw(&s, &images());
            wcounts[0] += 1;
            wcounts[1] += usize::from(up);
            wcounts[2] += usize::from(up && s.player_images.is_empty());
        }
    }
    eprintln!("{} weapon runs ({} scripts x 2 versions): an image went up {}, and was down again after 3 s {}", wcounts[0], weapons.len(), wcounts[1], wcounts[2]);
    assert!(counts[1] > 0, "some script changed an image");
}

/// One pass of the level's NPCs on a fresh picture.
fn draw_pass(s: &OfflineSession, imgs: &HashMap<String, (u32, Vec<u8>)>, pass: NpcPass) -> Framebuffer {
    let mut fb = Framebuffer::new(320, 320);
    draw_npcs(&mut fb, s, &NpcArt { images: imgs, sheets: PoseSheets::default(), gani_attrs: true }, 0, 0, pass);
    fb
}

#[test]
fn drawoverplayer_npcs_and_showimg_images_draw_after_the_players() {
    let imgs = images();
    let (s, _sc) = session(
        "NPC red.png 2 2\nif (playerenters) { drawoverplayer; }\nNPCEND\n\
         NPC blue.png 5 2\nif (playerenters) { drawunderplayer; }\nNPCEND\n\
         NPC blue.png 8 2\nif (playerenters) { showimg 1,blue.png,14,2; }\nNPCEND\n\
         NPC red.png 11 2\nif (playerenters) { drawaslight; }\nNPCEND\n",
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
    let art = NpcArt { images: &imgs, sheets: PoseSheets::default(), gani_attrs: true };
    draw_npcs(&mut fb, &s, &art, 0, 0, NpcPass::BelowPlayers);
    draw_npcs(&mut fb, &s, &art, 0, 0, NpcPass::AbovePlayers);
    // The default NPC is under the shading: red halfway to blue.
    assert_eq!(at(&fb, 2 * 16 + 1, 2 * 16 + 1), [100, 15, 143]);
    // The light is added on top of the shaded black: 128 blue + half red.
    assert_eq!(at(&fb, 6 * 16 + 1, 2 * 16 + 1), [100, 15, 143]);
}

// ---- effects on character NPCs ---------------------------------------------------

#[test]
fn a_character_npc_takes_its_colour_and_zoom_effect() {
    let a = art();
    let imgs = HashMap::new();
    let script = |fx: &str| format!("NPC - 10 10\nif (playerenters) {{ showcharacter; setcharani idle,; {fx} }}\nNPCEND\n");
    let pic = |fx: &str| {
        let (mut s, _sc) = session(&script(fx));
        with_default_ganis(&mut s);
        draw_with(&s, &imgs, sheets(&a))
    };
    let plain = pic("");
    // The same figure with no change to its colour: the same picture.
    assert_eq!(pic("setcoloreffect 1,1,1,1;").pixels, plain.pixels);
    // Red only: every drawn pixel keeps its red and loses green and blue.
    let red = pic("setcoloreffect 1,0,0,1;");
    let mut drawn = 0;
    for (p, q) in plain.pixels.chunks(3).zip(red.pixels.chunks(3)) {
        if p != [0, 0, 0] {
            drawn += 1;
            assert_eq!(q, [p[0], 0, 0]);
        }
    }
    assert!(drawn > 100, "the figure is drawn");
    // Zoom 2: the figure covers more of the picture.
    let count = |fb: &Framebuffer| fb.pixels.chunks(3).filter(|p| *p != [0, 0, 0]).count();
    assert!(count(&pic("setzoomeffect 2;")) > 3 * count(&plain));
}

// ---- a character NPC's own looks (setcharprop) -------------------------------------

/// A character at (10, 10) in the default idle animation, facing down: its
/// box starts at (160, 160), the head (32x32) at (168, 162), the body at
/// (168, 176) and the hat (48x48) at (160, 144), drawn after the head.
fn character_picture(props: &str, gani_attrs: bool) -> Framebuffer {
    let a = art();
    let mut imgs = HashMap::new();
    imgs.insert("ghead.png".to_string(), image(32, 128, |_, _| GREEN));
    imgs.insert("bluehat.png".to_string(), image(192, 144, |_, _| BLUE));
    let colors = [2, 10, 18, 5, 18];
    imgs.insert(seihai::looks::colored_body_key("body.png", &colors), image(128, 32, |_, _| RED));
    let (mut s, _sc) = session(&format!("NPC - 10 10\nif (playerenters) {{ showcharacter; setcharani idle,; {props} }}\nNPCEND\n"));
    with_default_ganis(&mut s);
    let mut fb = Framebuffer::new(320, 320);
    draw_both(&mut fb, &s, &NpcArt { images: &imgs, sheets: sheets(&a), gani_attrs });
    fb
}

#[test]
fn a_character_npc_draws_its_own_head_body_colours_and_hat() {
    let props = "setcharprop #3,ghead.png; setcharprop #C1,blue; setcharprop #P1,bluehat.png;";
    let fb = character_picture(props, true);
    assert_eq!(at(&fb, 184, 193), GREEN, "its own head");
    assert_eq!(at(&fb, 184, 205), RED, "its body in its own colours");
    assert_eq!(at(&fb, 184, 150), BLUE, "its hat, over the head");
    assert_eq!(at(&fb, 184, 180), BLUE, "the hat is drawn after the head");
    // A build without GANI attributes (before 2.02) draws no hat.
    let fb = character_picture(props, false);
    assert_eq!(at(&fb, 184, 150), [0, 0, 0]);
    assert_eq!(at(&fb, 184, 180), GREEN);
}

#[test]
fn a_character_npc_without_setcharprop_keeps_the_default_pictures() {
    let a = art();
    let with = character_picture("", true);
    let (mut s, _sc) = session("NPC - 10 10\nif (playerenters) { showcharacter; setcharani idle,; }\nNPCEND\n");
    with_default_ganis(&mut s);
    let plain = draw_with(&s, &HashMap::new(), sheets(&a));
    assert_eq!(with.pixels, plain.pixels);
}

// ---- changeimgvis ----------------------------------------------------------------

#[test]
fn changeimgvis_moves_an_image_between_the_heights() {
    let imgs = images();
    let (mut s, mut sc) = session(
        "NPC - 2 2\nif (playerenters) { showimg 1,red.png,4,2; showimg 2,red.png,6,2; showimg 3,red.png,8,2; \
         showimg 4,red.png,10,2; showimg 5,blue.png,100,40; showimg 6,red.png,12,2; \
         changeimgvis 1,0; changeimgvis 2,1; changeimgvis 3,2; changeimgvis 4,3; changeimgvis 5,4; changeimgvis 6,9; }\n\
         if (playersays(again)) { changeimgcolors 5,1,1,1,0.5; showimg 5,blue.png,100,40; }\nNPCEND\n",
    );
    let heights = |s: &OfflineSession| s.current_level.as_ref().unwrap().npcs[0].images.iter().map(|i| i.height).collect::<Vec<_>>();
    // Under, level with and over the NPC (height 0); light; screen; and a
    // value past 4 leaves the default (one over the NPC).
    assert_eq!(heights(&s), vec![-1, 0, 1, 9, 10, 1]);
    let below = draw_pass(&s, &imgs, NpcPass::BelowPlayers);
    let above = draw_pass(&s, &imgs, NpcPass::AbovePlayers);
    let screen = draw_pass(&s, &imgs, NpcPass::Screen);
    let p = |x: u32| x * 16 + 1;
    for (x, b, a) in [(4, RED, BLACK), (6, RED, BLACK), (8, BLACK, RED), (10, BLACK, RED), (12, BLACK, RED)] {
        assert_eq!((at(&below, p(x), p(2)), at(&above, p(x), p(2))), (b, a), "image at x {x}");
    }
    // The screen layer: at screen pixels, and only in its own pass.
    assert_eq!(at(&screen, 101, 41), BLUE);
    assert_eq!(at(&above, 101, 41), BLACK);
    assert_eq!(at(&screen, p(8), p(2)), BLACK);
    // showimg on an index in use puts it back one over the NPC; its colour stays.
    sc.chat(&mut s, "again");
    assert_eq!(heights(&s)[4], 1);
    assert_eq!(s.current_level.as_ref().unwrap().npcs[0].images[4].effect.color, [1.0, 1.0, 1.0, 0.5]);
}

#[test]
fn an_image_of_an_npc_over_the_player_sits_one_higher() {
    let (s, _sc) = session("NPC - 2 2\nif (playerenters) { drawoverplayer; showimg 1,red.png,4,2; showimg 2,red.png,6,2; changeimgvis 2,0; }\nNPCEND\n");
    let imgs = &s.current_level.as_ref().unwrap().npcs[0].images;
    // drawoverplayer is height 1: showimg gives 2, changeimgvis 0 gives 0.
    assert_eq!((imgs[0].height, imgs[1].height), (2, 0));
}

/// An MNG made the way Graal's `gif2mng` writes them (as in the 2.22
/// `bluelampani.mng`): `MHDR`, `TERM` (repeat for ever), then a `FRAM`
/// and one PNG stream per frame. Each frame is a solid colour.
fn lamp_mng(w: u32, h: u32, colours: &[[u8; 3]], delays: &[u32]) -> Vec<u8> {
    fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]) {
        out.extend_from_slice(&(body.len() as u32).to_be_bytes());
        let mut c = tag.to_vec();
        c.extend_from_slice(body);
        out.extend_from_slice(&c);
        out.extend_from_slice(&seihai::assets::crc32(&c).to_be_bytes());
    }
    let mut out = vec![0x8A, b'M', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut mhdr = Vec::new();
    for v in [w, h, 1000, 3, 5, 0, 1] {
        mhdr.extend_from_slice(&v.to_be_bytes());
    }
    chunk(&mut out, b"MHDR", &mhdr);
    chunk(&mut out, b"TERM", &[3, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
    for (c, d) in colours.iter().zip(delays) {
        let mut fram = vec![2, 0, 2, 0, 0, 0];
        fram.extend_from_slice(&d.to_be_bytes());
        chunk(&mut out, b"FRAM", &fram);
        let mut png = Vec::new();
        let px: Vec<u8> = (0..w * h).flat_map(|_| *c).collect();
        seihai::assets::write_png_rgb(&mut png, w, h, &px).unwrap();
        out.extend_from_slice(&png[8..]);
    }
    chunk(&mut out, b"MEND", &[]);
    out
}

/// The lamps of Graal the Adventure (`bluelampani.mng`, three frames with
/// delays 100, 0, 0) drew nothing: the image reader took PNG and GIF only.
/// Now the first frame is read like any picture, and `ImageAnimations`
/// plays the frames in the image map, so the drawn NPC changes colour on
/// the MNG's clock, a 0 delay lasting `ZERO_DELAY_MS`.
#[test]
fn mng_npc_draws_and_animates() {
    use seihai::assets::{mng::ZERO_DELAY_MS, ImageAnimations};
    let data = lamp_mng(32, 64, &[RED, GREEN, BLUE], &[100, 0, 0]);
    let (s, _sc) = session("NPC lamp.mng 2 2\nNPCEND\n");
    let mut imgs = images();
    let (w, _, px) = read_png_rgba(&data).expect("an MNG reads as its first frame");
    imgs.insert("lamp.mng".into(), (w, px));
    assert_eq!(at(&draw(&s, &imgs), 40, 40), RED);

    let mut anims = ImageAnimations::new();
    let read = |_: &str| Some(data.clone());
    anims.update(&mut imgs, 0, read);
    assert_eq!(anims.len(), 1);
    assert_eq!(at(&draw(&s, &imgs), 40, 40), RED);
    assert!(anims.update(&mut imgs, 100, read));
    assert_eq!(at(&draw(&s, &imgs), 40, 40), GREEN);
    assert!(!anims.update(&mut imgs, 100 + u64::from(ZERO_DELAY_MS) - 1, read));
    assert!(anims.update(&mut imgs, 100 + u64::from(ZERO_DELAY_MS), read));
    assert_eq!(at(&draw(&s, &imgs), 40, 40), BLUE);
    // Loops: back to the first frame after one pass.
    assert!(anims.update(&mut imgs, 100 + 2 * u64::from(ZERO_DELAY_MS), read));
    assert_eq!(at(&draw(&s, &imgs), 40, 40), RED);
    // The whole 32 x 64 picture is drawn, the NPC at tile (2, 2).
    let fb = draw(&s, &imgs);
    assert_eq!(at(&fb, 32 + 31, 32 + 63), RED);
    assert_eq!(at(&fb, 32 + 32, 32 + 63), BLACK);
}
