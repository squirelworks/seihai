//! Item 20: the built-in default set wired into the client.
//!
//! * the set loads with a missing or empty assets folder, for 2.05, 2.17, 2.22
//!   and 2.31;
//! * resolver precedence: server cache, then the user's folder, then the
//!   built-in set;
//! * the built-in set is read-only: a download never touches it;
//! * `FileSession::need` treats a built-in-only file as missing (no mod
//!   time is ever sent for it), and `on_step` asks for the first-step files
//!   only when there is no copy at all;
//! * the offline starter level loads, walks, swims, and draws chests and
//!   signs from the board, with no placeholder boxes.
//!
//! Every file here is our own default art, or a few synthetic bytes.

use std::fs;
use std::path::{Path, PathBuf};

use seihai::assets::{builtin, check_folder, read_png_rgb, AssetResolver, AssetRoot, Layer, Status};
use seihai::filecache::{DownloadStore, FileRequest, FileSession, Stored, FIRST_STEP_REQUESTS};
use seihai::offline::{OfflineSession, FOOT_DX, FOOT_DY};
use seihai::protocol::ClientVersion;
use seihai::render::{render_view_with, Framebuffer, ViewOptions, TILE_PX};

const PROFILES: [ClientVersion; 4] = [ClientVersion::V205, ClientVersion::V217, ClientVersion::V222, ClientVersion::V231];

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("seihai-wiring-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn touch(root: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, bytes).unwrap();
    p
}

/// The built-in set written under a temp folder.
fn builtin_dir(tag: &str) -> (PathBuf, PathBuf) {
    let base = tmp(tag);
    let dir = builtin::ensure_in(&base).unwrap();
    (base, dir)
}

fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    builtin::FILES.iter().map(|(n, _)| ((*n).to_string(), fs::read(dir.join(n)).unwrap())).collect()
}

// ---- the set loads without a data folder ------------------------------------

#[test]
fn the_default_set_loads_with_a_missing_or_empty_assets_folder() {
    let (base, bdir) = builtin_dir("fallback");
    let empty = tmp("empty-assets");
    let missing = base.join("no-such-folder");
    for v in PROFILES {
        for user in [Some(AssetRoot::new(&empty)), Some(AssetRoot::new(&missing)), None] {
            let r = AssetResolver::empty(v.profile()).with_assets(user).with_builtin(Some(bdir.clone()));
            for g in ["idle", "walk", "swim"] {
                let p = r.gani(g).unwrap_or_else(|| panic!("{}: {g}.gani", v.label()));
                assert_eq!(r.layer_of(&p), Some(Layer::Builtin));
            }
            assert!(r.sound("steps2").is_some() && r.sound("steps").is_some() && r.sound("water").is_some());
            assert!(r.level("new1.nw").is_some() && r.level("new2.nw").is_some());
            assert!(r.image_in("bodies", "body.png").is_some() && r.image_in("heads", "head0.png").is_some());
            assert!(r.image("pics1.png").is_some() && r.image("sprites.png").is_some() && r.image("state.png").is_some());
            assert!(r.tile_types().is_some() && r.tiledef("tiledefsOffline.txt").is_some());
        }
        // `--check-assets` on the set: nothing required is missing or bad.
        let rep = check_folder(&AssetRoot::new(&bdir), v.profile());
        for f in &rep.findings {
            assert!(!matches!(f.status, Status::BadFile(_)), "{}: {} -> {:?}", v.label(), f.need.name, f.status);
            assert!(!(f.need.required && !f.is_ok()), "{}: required {} -> {:?}", v.label(), f.need.name, f.status);
        }
    }
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&empty);
}

#[test]
fn a_partial_assets_folder_falls_back_file_by_file() {
    let (base, bdir) = builtin_dir("partial");
    let user = tmp("partial-assets");
    let mine = touch(&user, "gani/walk.gani", b"x");
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_assets(Some(AssetRoot::new(&user))).with_builtin(Some(bdir));
    // The user's file where there is one, ours for the rest.
    assert_eq!(r.gani("walk.gani"), Some(mine.clone()));
    assert_eq!(r.layer_of(&mine), Some(Layer::Assets));
    let swim = r.gani("swim.gani").unwrap();
    assert_eq!(r.layer_of(&swim), Some(Layer::Builtin));
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&user);
}

#[test]
fn the_built_in_tile_table_is_not_applied_to_a_foreign_sheet() {
    let (base, bdir) = builtin_dir("table");
    let user = tmp("table-assets");
    touch(&user, "pics1.png", b"not ours");
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_assets(Some(AssetRoot::new(&user))).with_builtin(Some(bdir.clone()));
    assert!(r.tile_types().is_none(), "the user's sheet has no table; ours does not describe it");
    // With no sheet of the user's, the built-in table goes with the built-in sheet.
    fs::remove_file(user.join("pics1.png")).unwrap();
    let (p, _) = r.tile_types().unwrap();
    assert_eq!(r.layer_of(&p), Some(Layer::Builtin));
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&user);
}

// ---- precedence ----------------------------------------------------------------

#[test]
fn cache_before_user_folder_before_built_in_set() {
    let (base, bdir) = builtin_dir("order");
    let user = tmp("order-user");
    let cache = tmp("order-cache");
    let r = AssetResolver::empty(ClientVersion::V222.profile())
        .with_assets(Some(AssetRoot::new(&user)))
        .with_cache(Some(cache.clone()))
        .with_builtin(Some(bdir.clone()));
    // Built-in only.
    let p = r.image("pics1.png").unwrap();
    assert_eq!(r.layer_of(&p), Some(Layer::Builtin));
    // The user's folder beats the built-in set.
    let up = touch(&user, "pics1.png", b"user");
    assert_eq!(r.image("pics1.png"), Some(up.clone()));
    // The server's cache beats both, for every kind.
    let cp = touch(&cache, "levels/webgifs/pics1.png", b"cache");
    assert_eq!(r.image("pics1.png"), Some(cp));
    let cg = touch(&cache, "levels/ganis/walk.gani", b"x");
    assert_eq!(r.gani("walk.gani"), Some(cg));
    let cs = touch(&cache, "sounds/steps2.wav", b"x");
    assert_eq!(r.sound("steps2"), Some(cs));
    let cb = touch(&cache, "levels/bodies/body.png", b"x");
    assert_eq!(r.image_in("bodies", "body.png"), Some(cb));
    // A name the cache lacks still comes from the later layers.
    let swim = r.gani("swim.gani").unwrap();
    assert_eq!(r.layer_of(&swim), Some(Layer::Builtin));
    // The cache's sounds folder is listed first, even before it exists.
    let dirs = r.sound_dirs();
    assert_eq!(dirs.first(), Some(&cache.join("sounds")));
    assert!(dirs.contains(&bdir.join("sounds")));
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&user);
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn built_in_levels_are_offline_only() {
    let (base, bdir) = builtin_dir("levels");
    let cache = tmp("levels-cache");
    let mk = |offline: bool| {
        AssetResolver::empty(ClientVersion::V222.profile())
            .with_cache(Some(cache.clone()))
            .with_builtin(Some(bdir.clone()))
            .with_builtin_levels(offline)
    };
    assert!(mk(true).level("new1.nw").is_some(), "offline: the starter level opens");
    // Online a server's `new1.nw` must come from the server, not from ours.
    assert!(mk(false).level("new1.nw").is_none());
    let sent = touch(&cache, "levels/new1.nw", b"server's level");
    assert_eq!(mk(false).level("new1.nw"), Some(sent.clone()));
    assert_eq!(mk(false).level_path("new1.nw"), sent);
    // Where a download of an unknown level would go: the cache, not ours.
    assert_eq!(mk(false).level_path("other.nw"), cache.join("levels/other.nw"));
    // Tiles and art stay available online.
    assert!(mk(false).image("pics1.png").is_some());
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&cache);
}

// ---- downloads never touch the built-in set ----------------------------------

#[test]
fn a_download_goes_to_the_cache_and_never_into_the_built_in_set_or_the_users_folder() {
    let (base, bdir) = builtin_dir("download");
    let before = snapshot(&bdir);
    let user = tmp("download-user");
    let mine = touch(&user, "levels/ganis/walk.gani", b"mine");
    let cache = tmp("download-cache");
    let r = AssetResolver::empty(ClientVersion::V222.profile())
        .with_assets(Some(AssetRoot::new(&user)))
        .with_cache(Some(cache.clone()))
        .with_builtin(Some(bdir.clone()));
    let mut store = DownloadStore::new(r.clone(), true);
    store.set_cache_meta(Some("host=h\nport=1\n".into()));
    // Names the built-in set has, and one the user's folder has.
    for (name, rel) in [
        ("pics1.png", "levels/webgifs/pics1.png"),
        ("steps.wav", "sounds/steps.wav"),
        ("body.png", "levels/bodies/body.png"),
        ("swim.gani", "levels/ganis/swim.gani"),
        ("new1.nw", "levels/new1.nw"),
        ("walk.gani", "levels/ganis/walk.gani"),
    ] {
        let got = store.store(name, b"from the server", Some(1_700_000_000)).unwrap();
        assert_eq!(got, Stored::Disk(cache.join(rel)), "{name}");
    }
    assert!(cache.join("server.txt").exists());
    // Our files are byte for byte what they were; the user's too.
    assert_eq!(snapshot(&bdir), before);
    assert_eq!(fs::read(&mine).unwrap(), b"mine");
    // And the cache's copy is the one the resolver now finds.
    for p in [r.image("pics1.png"), r.gani("walk.gani"), r.sound("steps"), r.level("new1.nw")] {
        assert_eq!(r.layer_of(&p.unwrap()), Some(Layer::Cache));
    }
    // A second download of the same name replaces the cache's copy in place.
    let again = store.store("steps.wav", b"newer", Some(1_700_000_100)).unwrap();
    assert_eq!(again, Stored::Disk(cache.join("sounds/steps.wav")));
    assert_eq!(fs::read(cache.join("sounds/steps.wav")).unwrap(), b"newer");
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&user);
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn with_no_cache_a_download_cannot_land_in_the_built_in_set() {
    let (base, bdir) = builtin_dir("ro");
    let before = snapshot(&bdir);
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_builtin(Some(bdir.clone()));
    let mut store = DownloadStore::new(r, true);
    for name in ["pics1.png", "steps.wav", "new1.nw", "levels/new1.nw", "gani/walk.gani"] {
        let e = store.store(name, b"x", None).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied, "{name}");
    }
    assert_eq!(snapshot(&bdir), before);
    // Even with a cache that (wrongly) is the built-in folder itself.
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_cache(Some(bdir.clone())).with_builtin(Some(bdir.clone()));
    let mut store = DownloadStore::new(r, true);
    let e = store.store("pics1.png", b"x", None).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(snapshot(&bdir), before);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn save_downloads_off_keeps_files_in_memory_and_writes_no_cache_files() {
    let (base, bdir) = builtin_dir("nosave");
    let cache = tmp("nosave-cache");
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_cache(Some(cache.clone())).with_builtin(Some(bdir));
    let mut store = DownloadStore::new(r, false);
    assert_eq!(store.store("steps.wav", b"abc", None).unwrap(), Stored::Memory);
    assert_eq!(store.store("sword1.png", b"abc", None).unwrap(), Stored::Memory);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0, "nothing written");
    assert_eq!(store.read("steps.wav").unwrap(), b"abc");
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&cache);
}

// ---- what is asked of the server ---------------------------------------------

fn sent(msgs: Vec<Vec<u8>>) -> Vec<String> {
    msgs.iter()
        .map(|m| {
            let id = m[0] - 0x20;
            let name = String::from_utf8_lossy(&m[if id == 34 { 6 } else { 1 }..]).trim_end().to_string();
            format!("{id}:{name}")
        })
        .collect()
}

#[test]
fn a_built_in_only_file_is_asked_for_as_missing_with_no_mod_time() {
    let (base, bdir) = builtin_dir("need");
    let user = tmp("need-user");
    touch(&user, "levels/bodies/body.png", b"mine");
    let cache = tmp("need-cache");
    touch(&cache, "levels/swords/sword1.png", b"cached");
    let r = AssetResolver::empty(ClientVersion::V222.profile())
        .with_assets(Some(AssetRoot::new(&user)))
        .with_cache(Some(cache.clone()))
        .with_builtin(Some(bdir));
    let mut fs_ = FileSession::new(r, true);
    // idle.gani and head0.png: only ours has them -> WANTFILE (23).
    assert_eq!(sent(fs_.need("idle.gani")), ["23:idle.gani"]);
    assert_eq!(sent(fs_.need("head0.png")), ["23:head0.png"]);
    // The user's copy and the cache's copy -> UPDATEFILE (34) with their time.
    assert_eq!(sent(fs_.need("body.png")), ["34:body.png"]);
    assert_eq!(sent(fs_.need("sword1.png")), ["34:sword1.png"]);
    // The built-in file's own mtime never goes on the wire.
    let m = fs_.need("pics1.png");
    assert_eq!(sent(m), ["23:pics1.png"]);
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&user);
    let _ = fs::remove_dir_all(&cache);
}

fn step_session(v: ClientVersion, user: Option<&Path>, cache: Option<&Path>, bdir: Option<&Path>) -> FileSession {
    let r = AssetResolver::empty(v.profile())
        .with_assets(user.map(AssetRoot::new))
        .with_cache(cache.map(Path::to_path_buf))
        .with_builtin(bdir.map(Path::to_path_buf));
    FileSession::new(r, true)
}

#[test]
fn the_first_step_requests_follow_the_real_client_only_when_no_copy_exists() {
    assert_eq!(FIRST_STEP_REQUESTS, &[(1, &["steps.wav", "walk.gani"][..]), (2, &["steps2.wav"][..])]);
    let (base, bdir) = builtin_dir("steps");
    let empty = tmp("steps-empty");

    // With the built-in set (all three files) nothing is asked, however far it walks.
    let mut s = step_session(ClientVersion::V231, Some(&empty), None, Some(&bdir));
    for _ in 0..6 {
        assert!(s.on_step().is_empty());
    }

    // A clean client: steps.wav and walk.gani with the first step, steps2.wav with the second.
    let mut s = step_session(ClientVersion::V231, Some(&empty), None, None);
    assert_eq!(sent(s.on_step()), ["23:steps.wav", "23:walk.gani"]);
    assert_eq!(sent(s.on_step()), ["23:steps2.wav"]);
    for _ in 0..5 {
        assert!(s.on_step().is_empty());
    }
    // A new connection starts counting again, and asks again (the lists are per connection).
    s.new_connection();
    assert_eq!(sent(s.on_step()), ["23:steps.wav", "23:walk.gani"]);

    // A copy anywhere counts: the cache has walk.gani, the user's folder has steps.wav.
    let cache = tmp("steps-cache");
    touch(&cache, "levels/ganis/walk.gani", b"x");
    touch(&empty, "sounds/steps.wav", b"x");
    let mut s = step_session(ClientVersion::V222, Some(&empty), Some(&cache), None);
    assert!(s.on_step().is_empty());
    assert_eq!(sent(s.on_step()), ["23:steps2.wav"]);

    // A file received this session counts too (kept in memory with saving off).
    let mut s = FileSession::new(
        AssetResolver::empty(ClientVersion::V222.profile()).with_assets(Some(AssetRoot::new(tmp("steps-mem")))),
        false,
    );
    s.store.store("steps.wav", b"abc", None).unwrap();
    s.store.store("walk.gani", b"abc", None).unwrap();
    assert!(s.on_step().is_empty());

    // Builds with no GANIs, or with no WANTFILE, ask for none of these.
    let mut s = step_session(ClientVersion::V139, Some(&empty), None, None);
    assert!(s.on_step().is_empty() && s.on_step().is_empty());
    let _ = fs::remove_dir_all(&base);
    for d in [&empty, &cache] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn the_first_step_messages_are_plain_wantfiles() {
    let mut s = step_session(ClientVersion::V222, None, Some(&tmp("steps-plain")), None);
    let m = s.on_step();
    assert_eq!(m[0], FileRequest::Want("steps.wav".into()).message());
    assert_eq!(m[1], FileRequest::Want("walk.gani".into()).message());
}

// ---- the offline starter level -------------------------------------------------

fn offline_session(bdir: &Path) -> (OfflineSession, AssetResolver) {
    let r = AssetResolver::empty(ClientVersion::V222.profile()).with_builtin(Some(bdir.to_path_buf()));
    let mut s = OfflineSession::default();
    s.load_level(r.level_path("new1.nw")).unwrap();
    let (tp, kind) = r.tile_types().unwrap();
    assert_eq!(kind, seihai::assets::TileTypesFile::Text);
    s.load_tiletypes(tp).unwrap();
    (s, r)
}

#[test]
fn the_starter_level_loads_the_player_walks_and_swims() {
    let (base, bdir) = builtin_dir("offline");
    let (mut s, r) = offline_session(&bdir);
    let l = s.current_level.clone().unwrap();
    assert!(!l.chests.is_empty() && !l.signs.is_empty());
    // Walk: spawn on open grass, five steps east move the player.
    s.player_x = 30.0;
    s.player_y = 30.5;
    s.refresh_water();
    assert!(!s.in_water);
    let x0 = s.player_x;
    for _ in 0..6 {
        s.try_move(0.5, 0.0);
    }
    assert!(s.player_x > x0 + 1.0, "walked east: {} -> {}", x0, s.player_x);
    // The walk and swim animations (ours) drive the sound cues.
    for (name, swim) in [("walk.gani", false), ("swim.gani", true)] {
        let g = seihai::assets::Gani::load(r.gani(name).unwrap()).unwrap();
        if swim {
            s.set_swim_gani(seihai::assets::GaniPlayer::new(g));
        } else {
            s.set_walk_gani(seihai::assets::GaniPlayer::new(g));
        }
    }
    s.player_x = 30.0;
    s.player_y = 30.5;
    for i in 0..10 {
        s.try_move(if i % 2 == 0 { 0.5 } else { -0.5 }, 0.0);
    }
    let cues: Vec<String> = s.take_sound_events().into_iter().map(|e| e.file).collect();
    assert!(!cues.is_empty() && cues.iter().all(|c| c == "steps2.wav"), "{cues:?}");
    // Swim: put the foot on the pond.
    let types = s.tiletypes.clone().unwrap();
    let water = (0..64)
        .flat_map(|y| (0..64).map(move |x| (x, y)))
        .find(|&(x, y)| l.tile_id_at(x, y).is_some_and(|id| types.is_water(id)))
        .expect("a pond");
    s.player_x = water.0 as f32 + 0.5 - FOOT_DX;
    s.player_y = water.1 as f32 + 0.5 - FOOT_DY;
    s.refresh_water();
    assert!(s.in_water);
    for i in 0..10 {
        s.try_move(if i % 2 == 0 { 0.25 } else { -0.25 }, 0.0);
    }
    let cues: Vec<String> = s.take_sound_events().into_iter().map(|e| e.file).collect();
    assert!(!cues.is_empty() && cues.iter().all(|c| c == "water.wav"), "{cues:?}");
    let _ = fs::remove_dir_all(&base);
}

/// The 32x32 pixels at `(px, py)` of a view.
fn region(fb: &Framebuffer, px: i32, py: i32, w: i32, h: i32) -> Vec<u8> {
    let mut v = Vec::new();
    for y in py..py + h {
        for x in px..px + w {
            let i = ((y as u32 * fb.width + x as u32) * 3) as usize;
            v.extend_from_slice(&fb.pixels[i..i + 3]);
        }
    }
    v
}

/// What blitting these tile ids on the sheet draws, 2 tiles wide.
fn tiles_image(sheet: &[u8], sw: u32, sh: u32, ids: &[u16], cols: i32) -> Vec<u8> {
    let rows = ids.len() as i32 / cols;
    let mut fb = Framebuffer::new((cols as u32) * TILE_PX, (rows as u32) * TILE_PX);
    for (i, id) in ids.iter().enumerate() {
        let (cx, cy) = (i as i32 % cols, i as i32 / cols);
        fb.blit_tile(sheet, sw, sh, *id, cx * TILE_PX as i32, cy * TILE_PX as i32);
    }
    fb.pixels
}

#[test]
fn chests_and_signs_are_drawn_from_the_board_with_no_placeholder_boxes() {
    let (base, bdir) = builtin_dir("render");
    let (mut s, r) = offline_session(&bdir);
    let l = s.current_level.clone().unwrap();
    let sheet_path = r.image("pics1.png").unwrap();
    let (sw, sh, sheet) = read_png_rgb(&fs::read(sheet_path).unwrap()).unwrap();
    let (vw, vh) = (40u32, 30u32);
    let t = TILE_PX as i32;
    let opts = ViewOptions { builtin_sheet: true };
    let cam = |s: &OfflineSession| {
        (((s.player_x + 1.5) * 16.0 - (vw as f32 * 16.0) / 2.0) as i32, ((s.player_y + 1.5) * 16.0 - (vh as f32 * 16.0) / 2.0) as i32)
    };
    let chest = l.chests[0].clone();
    let sign = l.signs[0].clone();
    for (label, ax, ay) in [("chest", chest.x, chest.y), ("sign", sign.x, sign.y)] {
        // Centre the camera so the object is in view.
        s.player_x = ax as f32 - 2.0;
        s.player_y = ay as f32 - 2.0;
        let (cx, cy) = cam(&s);
        let fb = render_view_with(&s, &sheet, sw, sh, vw, vh, opts);
        let (w, h) = if label == "chest" { (2, 2) } else { (2, 1) };
        let ids: Vec<u16> = (0..h).flat_map(|dy| (0..w).map(move |dx| (dx, dy))).map(|(dx, dy)| l.tile_id_at(ax + dx, ay + dy).unwrap()).collect();
        let got = region(&fb, ax * t - cx, ay * t - cy, w * t, h * t);
        assert_eq!(got, tiles_image(&sheet, sw, sh, &ids, w), "{label}: the view shows the board's tiles and nothing on top");
    }
    // An opened chest shows the open tiles, with our sheet.
    s.player_x = chest.x as f32 - 2.0;
    s.player_y = chest.y as f32 - 2.0;
    s.opened_chests.insert((chest.x, chest.y));
    let (cx, cy) = cam(&s);
    let closed: Vec<u16> = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| l.tile_id_at(chest.x + dx, chest.y + dy).unwrap()).to_vec();
    let open: Vec<u16> = closed.iter().map(|i| i + 2).collect();
    let fb = render_view_with(&s, &sheet, sw, sh, vw, vh, opts);
    let got = region(&fb, chest.x * t - cx, chest.y * t - cy, 2 * t, 2 * t);
    assert_eq!(got, tiles_image(&sheet, sw, sh, &open, 2));
    assert_ne!(got, tiles_image(&sheet, sw, sh, &closed, 2));
    // With another sheet the chest is drawn from its record with the
    // sheet's own chest ids (open art at 829 and on).
    let fb = render_view_with(&s, &sheet, sw, sh, vw, vh, ViewOptions::default());
    assert_eq!(region(&fb, chest.x * t - cx, chest.y * t - cy, 2 * t, 2 * t), tiles_image(&sheet, sw, sh, &[829, 830, 845, 846], 2));
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn the_starter_level_chest_and_sign_tiles_have_open_art_two_ids_on() {
    // The open-chest swap rests on this: every closed chest tile of the
    // built-in sheet has the open tile two ids to the right, named so in tiledefs.txt.
    let text = String::from_utf8(builtin::bytes("tiledefs.txt").unwrap().to_vec()).unwrap();
    let name_of = |id: u16| text.lines().find_map(|l| l.split(',').collect::<Vec<_>>().get(..5).filter(|f| f[0].parse::<u16>() == Ok(id)).map(|f| f[4].to_string()));
    for id in seihai::render::BUILTIN_CHEST_CLOSED {
        let (c, o) = (name_of(id).unwrap(), name_of(id + 2).unwrap());
        assert!(c.starts_with("chest") && !c.contains("open"), "{id} is {c}");
        assert!(o.starts_with("chest") && o.contains("open"), "{} is {o}", id + 2);
    }
}

// ---- downloads land where the resolver looks (laundry list P6, 2026-10-29) ----

/// Every image a server sends is found again by the resolver, at the path
/// the store wrote, for each save layout.
#[test]
fn a_downloaded_image_is_found_where_it_was_saved() {
    for v in [ClientVersion::V139, ClientVersion::V205, ClientVersion::V222, ClientVersion::V231] {
        let cache = tmp(&format!("agree-{}", v.label()));
        let r = AssetResolver::empty(v.profile()).with_cache(Some(cache.clone()));
        let mut store = DownloadStore::new(r.clone(), true);
        for name in [
            "Block.PNG", "door.gif", "hat3.png", "bomy1.png", "b_fire.png", "emoticon_a.png", "minimap.png", "worldmap.gif",
            "head927.png", "body13.png", "sword2.png", "shield3.png", "letters.mng", "npcs/sign.png",
        ] {
            let Stored::Disk(at) = store.store(name, b"x", Some(1_700_000_000)).unwrap() else { panic!("{name}: not on disk") };
            assert_eq!(r.image(name).as_ref(), Some(&at), "{} {name}", v.label());
            assert_eq!(r.image(&name.to_ascii_lowercase()).as_ref(), Some(&at), "{} {name} in lower case", v.label());
            assert_eq!(store.local_path(name).as_ref(), Some(&at), "{} {name}: the store's own lookup", v.label());
        }
        let _ = fs::remove_dir_all(&cache);
    }
}

// ---- the Seihai folder's cache (laundry list P2, 2026-10-29) ---------------------

/// `_cache/<host>_<port>/` in the Seihai folder is searched before the
/// settings' cache, and a download goes there only. The settings' cache gets
/// a copy when the user named it, and the download when the Seihai folder
/// cannot be written.
#[test]
fn the_seihai_folder_cache_is_searched_first_and_takes_the_downloads() {
    let game = tmp("game-cache");
    let cache = tmp("game-cache-settings");
    let r = AssetResolver::empty(ClientVersion::V222.profile())
        .with_game_cache(Some(game.clone()))
        .with_cache(Some(cache.clone()));
    // An older copy in the settings' cache only: found, and offered with its time.
    let old = touch(&cache, "levels/heads/head927.png", b"old");
    assert_eq!(r.image("head927.png"), Some(old));
    // A copy in the Seihai folder wins.
    let new = touch(&game, "levels/heads/head927.png", b"new");
    assert_eq!(r.image("head927.png"), Some(new.clone()));
    assert_eq!(r.layer_of(&new), Some(Layer::Cache));
    assert_eq!(r.download_root().map(|d| d.root.clone()), Some(game.clone()));

    let mut store = DownloadStore::new(r.clone(), true);
    store.set_cache_meta(Some("host=h\nport=1\n".into()));
    let got = store.store("body13.png", b"server", Some(1_700_000_000)).unwrap();
    assert_eq!(got, Stored::Disk(game.join("levels/bodies/body13.png")));
    assert!(!cache.join("levels/bodies/body13.png").exists(), "no copy in the settings' cache by default");
    assert!(game.join("server.txt").exists() && !cache.join("server.txt").exists());
    assert_eq!(r.image("body13.png"), Some(game.join("levels/bodies/body13.png")));
    assert_eq!(store.local_path("BODY13.PNG"), Some(game.join("levels/bodies/body13.png")));
    // A replaced file in the settings' cache only lands in the Seihai folder.
    let got = store.store("head927.png", b"newer", None).unwrap();
    assert_eq!(got, Stored::Disk(new.clone()));
    assert_eq!(fs::read(cache.join("levels/heads/head927.png")).unwrap(), b"old");
    // A cache folder the user named keeps a copy.
    store.set_settings_copy(true);
    store.store("body14.png", b"server", None).unwrap();
    assert_eq!(fs::read(cache.join("levels/bodies/body14.png")).unwrap(), b"server");
    assert_eq!(fs::read(game.join("levels/bodies/body14.png")).unwrap(), b"server");

    // The Seihai folder's cache cannot be made (a file is in the way).
    let blocked = tmp("game-cache-blocked").join("file");
    fs::write(&blocked, b"").unwrap();
    let r2 = AssetResolver::empty(ClientVersion::V222.profile())
        .with_game_cache(Some(blocked.join("_cache")))
        .with_cache(Some(cache.clone()));
    let mut store2 = DownloadStore::new(r2, true);
    let got = store2.store("hat0.png", b"x", None).unwrap();
    assert_eq!(got, Stored::Disk(cache.join("levels/hats/hat0.png")));
    let _ = fs::remove_dir_all(&game);
    let _ = fs::remove_dir_all(&cache);
}
