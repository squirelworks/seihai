//! Contract tests for the generated default asset set (`assets/default/`).
//!
//! Each file is loaded with Seihai's existing reader (PNG reader, GANI parser,
//! WAV decoder, level reader, tile type table) and checked against the sizes,
//! layouts and the tile-id mapping contract in `documentation/default-assets.md`
//! and `documentation/asset-reference.md`. Regenerate with
//! `python3 tools/gen_default_assets.py` if a number here changes.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use seihai::assets::{
    check_folder, read_png_rgb, read_png_rgba, AssetResolver, AssetRoot, Gani, Level, Status, TileKind, TileTypes,
};
use seihai::offline::{OfflineSession, FOOT_DX, FOOT_DY};
use seihai::protocol::ClientVersion;
use seihai::render::{Framebuffer, TILE_PX};
use seihai::sound::decode_wav;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("assets").join("default")
}

fn read(rel: &str) -> Vec<u8> {
    fs::read(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// (width, height, bit depth, colour type) from the IHDR chunk, and whether a
/// tRNS chunk is present.
fn png_header(data: &[u8]) -> (u32, u32, u8, u8, bool) {
    assert_eq!(&data[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let w = u32::from_be_bytes(data[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(data[20..24].try_into().unwrap());
    let trns = data.windows(4).any(|c| c == b"tRNS");
    assert_eq!(data[28], 0, "not interlaced");
    (w, h, data[24], data[25], trns)
}

/// The tile type rows of `tiledefs.txt`: (id, x, y, type, name).
fn tiledef_rows() -> Vec<(u16, u32, u32, u8, String)> {
    let text = String::from_utf8(read("tiledefs.txt")).unwrap();
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            assert_eq!(f.len(), 5, "{l}");
            (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].to_string())
        })
        .collect()
}

fn id_of(rows: &[(u16, u32, u32, u8, String)], name: &str) -> u16 {
    rows.iter().find(|r| r.4 == name).unwrap_or_else(|| panic!("no tile named {name}")).0
}

// ---- images ----------------------------------------------------------------

#[test]
fn image_sizes_formats_and_palettes() {
    // (file, width, height, bit depth, transparency chunk expected)
    let sheets: [(&str, u32, u32, u8, bool); 7] = [
        ("pics1.png", 2048, 512, 8, false),
        ("bodies/body.png", 128, 720, 4, true),
        ("heads/head0.png", 32, 560, 4, true),
        ("swords/sword1.png", 128, 96, 4, true),
        ("shields/shield1.png", 38, 20, 4, true),
        ("sprites.png", 128, 402, 4, true),
        ("state.png", 224, 145, 4, true),
    ];
    for (file, w, h, depth, trns) in sheets {
        let data = read(file);
        let (hw, hh, hd, ct, ht) = png_header(&data);
        assert_eq!((hw, hh, hd, ct, ht), (w, h, depth, 3, trns), "{file}: IHDR (palette PNG)");
        let (rw, rh, px) = read_png_rgba(&data).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!((rw, rh), (w, h), "{file}");
        assert_eq!(px.len(), (w * h * 4) as usize);
    }
    let (w, h, rgb) = read_png_rgb(&read("pics1.png")).unwrap();
    assert_eq!((w, h, rgb.len()), (2048, 512, 2048 * 512 * 3));
    // 128 columns x 32 rows of 16 px tiles
    assert_eq!((w / TILE_PX, h / TILE_PX), (128, 32));
    // the whole set stays small
    let total: u64 = walk(&root()).iter().map(|p| fs::metadata(p).unwrap().len()).sum();
    assert!(total < 300 * 1024, "assets/default is {total} bytes");
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            v.extend(walk(&p));
        } else {
            v.push(p);
        }
    }
    v
}

fn opaque_pixels(px: &[u8], w: u32, x: u32, y: u32, cw: u32, ch: u32) -> usize {
    let mut n = 0;
    for yy in y..y + ch {
        for xx in x..x + cw {
            if px[((yy * w + xx) * 4 + 3) as usize] != 0 {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn player_sheets_have_art_in_every_cell_the_code_reads() {
    let (bw, bh, body) = read_png_rgba(&read("bodies/body.png")).unwrap();
    assert_eq!((bw, bh), (128, 720));
    // four 32-px direction columns, 22 pose rows of 32 px (22 * 32 = 704, plus 16 spare)
    assert_eq!(bw / 32, 4);
    assert_eq!(bh / 32, 22);
    for dir in 0..4 {
        // pose 0 (idle, read by draw_player_sprites), poses 1..=8 (walk) and, from item 22,
        // 9 sword wind-up, 10 sword swing, 11 sit, 12 hurt; 13 carry standing and
        // 14..=21 the carry walk (arms raised)
        for pose in 0..=21 {
            let n = opaque_pixels(&body, bw, dir * 32, pose * 32, 32, 32);
            assert!(n > 150 && n < 700, "body dir {dir} pose {pose}: {n} opaque pixels");
        }
        // walk poses differ from each other (a real cycle)
        let cell = |pose: u32| -> Vec<u8> {
            let mut v = Vec::new();
            for y in 0..32 {
                for x in 0..32 {
                    let i = (((pose * 32 + y) * bw + dir * 32 + x) * 4) as usize;
                    v.extend_from_slice(&body[i..i + 4]);
                }
            }
            v
        };
        let distinct: HashSet<Vec<u8>> = (1..=8).map(cell).collect();
        assert!(distinct.len() >= 5, "dir {dir}: only {} distinct walk poses", distinct.len());
        // the four new poses differ from idle and from each other
        let new: HashSet<Vec<u8>> = [0, 9, 10, 11, 12].into_iter().map(cell).collect();
        assert_eq!(new.len(), 5, "dir {dir}: idle, wind-up, swing, sit and hurt look alike");
        // the carry poses are not the plain idle or walk ones
        assert_ne!(cell(13), cell(0), "dir {dir}: carry standing looks like idle");
        for k in 0..8 {
            assert_ne!(cell(14 + k), cell(1 + k), "dir {dir}: carry walk {k} looks like walk {k}");
        }
    }
    let (hw, hh, head) = read_png_rgba(&read("heads/head0.png")).unwrap();
    assert_eq!((hw, hh), (32, 560));
    for dir in 0..4 {
        let n = opaque_pixels(&head, hw, 0, dir * 32, 32, 32);
        assert!(n > 250, "head dir {dir}: {n}");
    }
    // sword sheet: 4 directions x 3 rows of 32x32
    let (sw, sh, sword) = read_png_rgba(&read("swords/sword1.png")).unwrap();
    assert_eq!((sw, sh), (128, 96));
    for dir in 0..4 {
        for row in 0..3 {
            assert!(opaque_pixels(&sword, sw, dir * 32, row * 32, 32, 32) > 10);
        }
    }
    // shield sheet: sprites at x = 0, 6, 14, 30 of sizes 6x18, 8x20, 16x20, 8x20
    let (shw, _, shield) = read_png_rgba(&read("shields/shield1.png")).unwrap();
    for (x, w, h) in [(0u32, 6u32, 18u32), (6, 8, 20), (14, 16, 20), (30, 8, 20)] {
        assert_eq!(opaque_pixels(&shield, shw, x, 0, w, h), (w * h) as usize, "shield at {x}");
    }
}

#[test]
fn sprites_sheet_has_the_shadow_and_both_ripples() {
    let (w, h, px) = read_png_rgba(&read("sprites.png")).unwrap();
    assert!(w >= 128 && h >= 402);
    // shadow 24x12 at (0, 0): drawn, and not a solid box (dithered ellipse)
    let shadow = opaque_pixels(&px, w, 0, 0, 24, 12);
    assert!(shadow > 40 && shadow < 24 * 12 / 2 + 10, "shadow pixels {shadow}");
    // ripples 32x16 at y = 306 and y = 322
    for y in [306, 322] {
        let n = opaque_pixels(&px, w, 0, y, 32, 16);
        assert!(n > 150, "ripple at y={y}: {n}");
    }
    // nothing between the shadow and the ripples is drawn below the break pieces
    assert_eq!(opaque_pixels(&px, w, 0, 144, 128, 162), 0);
}

#[test]
fn sprites_sheet_has_the_carried_objects_and_the_break_pieces() {
    use seihai::throwing::{builtin_sprite_rect, CARRY_SPRITES};
    let (w, h, px) = read_png_rgba(&read("sprites.png")).unwrap();
    // every carried object is drawn inside its own 32x32 cut-out, with
    // transparent corners (the board tile around it is not part of it)
    for id in CARRY_SPRITES {
        let (x, y, rw, rh) = builtin_sprite_rect(id).unwrap();
        assert!((x + rw) as u32 <= w && (y + rh) as u32 <= h, "sprite {id} inside the sheet");
        let n = opaque_pixels(&px, w, x as u32, y as u32, rw as u32, rh as u32);
        assert!(n > 350 && n < (rw * rh) as usize - 60, "sprite {id}: {n} opaque pixels of {}", rw * rh);
        assert_eq!(opaque_pixels(&px, w, x as u32, y as u32, 2, 2), 0, "sprite {id}: corner is transparent");
    }
    // the break pieces (leaves, rock, pot) and the two splash frames carry art
    for id in [25u16, 26, 96, 97, 98, 99, 100, 101, 102, 103, 326, 327] {
        let (x, y, rw, rh) = builtin_sprite_rect(id).unwrap();
        assert!((x + rw) as u32 <= w && (y + rh) as u32 <= h, "sprite {id} inside the sheet");
        assert!(opaque_pixels(&px, w, x as u32, y as u32, rw as u32, rh as u32) > 20, "sprite {id} has art");
    }
}

// ---- tile ids ----------------------------------------------------------------

#[test]
fn tile_id_mapping_contract() {
    let (sw, sh, rgb) = read_png_rgb(&read("pics1.png")).unwrap();
    let rows = tiledef_rows();
    assert!(rows.len() > 40);
    let filler: [[u8; 3]; 2] = [[160, 32, 160], [40, 16, 40]];
    let cell_px = |col: u32, row: u32| -> Vec<[u8; 3]> {
        let mut v = Vec::new();
        for y in 0..16 {
            for x in 0..16 {
                let i = (((row * 16 + y) * sw + col * 16 + x) * 3) as usize;
                v.push([rgb[i], rgb[i + 1], rgb[i + 2]]);
            }
        }
        v
    };
    for (id, x, y, _, name) in &rows {
        let t = *id as u32;
        // the mapping Seihai's blit_tile uses
        let (col, row) = (t % 16 + (t / 512) * 16, (t / 16) % 32);
        assert_eq!((*x, *y), (col * 16, row * 16), "tiledefs x,y of {name}");
        let px = cell_px(col, row);
        if name == "unassigned" {
            assert!(px.iter().all(|p| filler.contains(p)));
            continue;
        }
        assert!(t < 512, "{name}: ids stay in block 0");
        assert!(px.iter().all(|p| !filler.contains(p)), "tile {name} ({id}) is on a filler cell");
        // blit_tile draws exactly that cell
        let mut fb = Framebuffer::new(16, 16);
        fb.blit_tile(&rgb, sw, sh, *id, 0, 0);
        let drawn: Vec<[u8; 3]> = fb.pixels.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
        assert_eq!(drawn, px, "blit_tile({id}) for {name}");
    }
    // ids that no row names are filler (purple/black checker): a wrong id shows
    for id in [5u32, 40, 47, 100, 300, 511, 512, 4095] {
        let (col, row) = (id % 16 + (id / 512) * 16, (id / 16) % 32);
        assert!(cell_px(col, row).iter().all(|p| filler.contains(p)), "id {id}");
    }
    // all 4096 ids have a cell on this sheet
    let cells: HashSet<(u32, u32)> = (0..4096u32).map(|t| (t % 16 + (t / 512) * 16, (t / 16) % 32)).collect();
    assert_eq!(cells.len(), 4096);
    assert!(cells.iter().all(|&(c, r)| c < 128 && r < 32));
}

#[test]
fn tile_types_contract() {
    let types = TileTypes::load(root().join("tiledefs.txt")).expect("tiledefs.txt loads");
    let rows = tiledef_rows();
    let id = |n: &str| id_of(&rows, n);
    // water is exactly type 11 and counts as water, not as a wall
    for n in ["water", "water2"] {
        assert_eq!(types.raw(id(n)), Some(11), "{n}");
        assert_eq!(types.kind(id(n)), TileKind::Water);
    }
    // walls and solid objects are type 20 or more
    for n in ["wall", "wallcap", "roof", "void", "bush", "rock", "tree0", "tree3", "chest0", "chest3", "chestopen1", "sign0", "sign1", "signw0"] {
        assert!(types.raw(id(n)).unwrap() >= 20, "{n}");
        assert_eq!(types.kind(id(n)), TileKind::Blocking, "{n}");
    }
    // floors are open
    for n in ["grass", "grass2", "flowers", "path", "sand", "sand2", "doorway", "woodfloor"] {
        assert_eq!(types.kind(id(n)), TileKind::Open, "{n}");
    }
    // every named id is below 4096 and the table has all 4096 entries
    assert!(rows.iter().all(|r| r.0 < 4096));
    assert_eq!(types.types.len(), 4096);
    // an id that nothing names is open floor (type 0)
    assert_eq!(types.kind(5), TileKind::Open);
    // the foot point the contract is written for
    assert_eq!((FOOT_DX, FOOT_DY), (2.0, 2.75));
}

// ---- GANIs and sounds -------------------------------------------------------

#[test]
fn ganis_parse_with_the_documented_structure() {
    let (bw, bh, body) = read_png_rgba(&read("bodies/body.png")).unwrap();
    let (hw, hh, _) = read_png_rgba(&read("heads/head0.png")).unwrap();
    let (spw, sph, _) = read_png_rgba(&read("sprites.png")).unwrap();
    let (sww, swh, _) = read_png_rgba(&read("swords/sword1.png")).unwrap();
    let sheet_size = |s: &str| match s {
        "BODY" => (bw, bh),
        "HEAD" => (hw, hh),
        "SPRITES" => (spw, sph),
        "SWORD" => (sww, swh),
        other => panic!("unexpected sheet {other}"),
    };
    // name, steps, sound cues as (step, file)
    let want: [(&str, usize, Vec<(usize, &str)>); 6] = [
        ("idle", 1, vec![]),
        ("walk", 8, vec![(0, "steps2.wav"), (4, "steps2.wav")]),
        ("swim", 8, vec![(0, "water.wav")]),
        // item 22: the sword swing has the stock's five steps and a sound after step 0
        ("sword", 5, vec![(0, "sword.wav")]),
        ("sit", 1, vec![]),
        ("hurt", 1, vec![]),
    ];
    for (name, steps, cues) in want {
        let g = Gani::load(root().join("gani").join(format!("{name}.gani"))).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!g.single_direction, "{name}");
        assert_eq!(g.frame_count(), steps, "{name} steps");
        assert_eq!(g.default_body.as_deref(), Some("body.png"));
        assert_eq!(g.default_head.as_deref(), Some("head0.png"));
        for (i, f) in g.frames.iter().enumerate() {
            assert_eq!(f.rows.len(), 4, "{name} step {i}: one row per direction");
            for row in &f.rows {
                assert!(!row.is_empty());
                for p in row {
                    let s = g.sprites.iter().find(|s| s.id == p.sprite_id).unwrap_or_else(|| panic!("{name}: sprite {} undefined", p.sprite_id));
                    let (w, h) = sheet_size(&s.sheet);
                    assert!(s.x >= 0 && s.y >= 0 && (s.x + s.w) as u32 <= w && (s.y + s.h) as u32 <= h, "{name}: sprite {} outside {}", s.id, s.sheet);
                    // placements stay inside the 48x48 player box (the ripple may reach the bottom edge);
                    // a 32x32 sword cell may hang over the edge by its empty margin
                    let slack = if s.sheet == "SWORD" { 16 } else { 0 };
                    assert!(
                        p.x >= -slack && p.x + s.w <= 48 + 8 + slack && p.y >= -slack && p.y + s.h <= 48 + 8 + slack,
                        "{name}: placement {:?}",
                        p
                    );
                }
            }
            let got: Vec<&str> = f.sounds.iter().map(|s| s.file.as_str()).collect();
            let exp: Vec<&str> = cues.iter().filter(|c| c.0 == i).map(|c| c.1).collect();
            assert_eq!(got, exp, "{name} step {i} cues");
            for s in &f.sounds {
                assert_eq!((s.x, s.y), (1.5, 2.0));
                assert!(root().join("sounds").join(&s.file).is_file(), "{} exists", s.file);
            }
        }
        // body sprites that the animation uses carry art
        for s in g.sprites.iter().filter(|s| s.sheet == "BODY") {
            assert!(opaque_pixels(&body, bw, s.x as u32, s.y as u32, s.w as u32, s.h as u32) > 100, "{name}: sprite {}", s.id);
        }
    }
    // idle uses body pose 0 (ids 200..203) and heads 100..103, as draw_player_sprites does
    let idle = Gani::load(root().join("gani/idle.gani")).unwrap();
    for dir in 0..4usize {
        let ids: Vec<u32> = idle.frames[0].rows[dir].iter().map(|p| p.sprite_id).collect();
        assert_eq!(ids, vec![0, 200 + dir as u32, 100 + dir as u32]);
    }
    // sword.gani: five steps that play once and go back to idle; every step has
    // a body pose 9 or 10 and a sword sprite; sit and hurt use their own body poses
    let sword = Gani::load(root().join("gani/sword.gani")).unwrap();
    assert_eq!(sword.setback_to.as_deref(), Some("idle"));
    assert_eq!(Gani::load(root().join("gani/sit.gani")).unwrap().setback_to, None);
    for step in &sword.frames {
        for row in &step.rows {
            assert!(row.iter().any(|p| (236..244).contains(&p.sprite_id)), "a swing body pose (ids 236..243)");
            assert!(row.iter().any(|p| (300..312).contains(&p.sprite_id)), "a sword sprite (ids 300..311)");
        }
    }
    let ids = |name: &str, dir: usize| -> Vec<u32> {
        Gani::load(root().join("gani").join(name)).unwrap().frames[0].rows[dir].iter().map(|p| p.sprite_id).collect()
    };
    for dir in 0..4usize {
        assert!(ids("sit.gani", dir).contains(&(244 + dir as u32)), "sit body pose 11");
        assert!(ids("hurt.gani", dir).contains(&(248 + dir as u32)), "hurt body pose 12");
    }
    // swim places no body
    let swim = Gani::load(root().join("gani/swim.gani")).unwrap();
    assert!(swim.frames.iter().all(|f| f.rows.iter().all(|r| r.iter().all(|p| !(200..300).contains(&p.sprite_id)))));
}

#[test]
fn wav_files_decode_in_the_documented_formats() {
    // name, sample rate, bits per sample, min and max milliseconds
    for (name, rate, bits, lo, hi) in [
        ("steps.wav", 22050u32, 8u16, 80u64, 400u64),
        ("steps2.wav", 22050, 8, 80, 400),
        ("water.wav", 8000, 16, 200, 800),
        ("sword.wav", 8000, 8, 100, 600),
    ] {
        let data = read(&format!("sounds/{name}"));
        assert_eq!(u16::from_le_bytes([data[20], data[21]]), 1, "{name}: PCM");
        assert_eq!(u16::from_le_bytes([data[22], data[23]]), 1, "{name}: mono");
        assert_eq!(u16::from_le_bytes([data[34], data[35]]), bits, "{name}: bits");
        let pcm = decode_wav(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!((pcm.channels, pcm.rate), (1, rate), "{name}");
        let ms = pcm.duration_ms();
        assert!((lo..=hi).contains(&ms), "{name}: {ms} ms");
        let peak = pcm.samples.iter().map(|s| (*s as i32).abs()).max().unwrap();
        assert!(peak > 4000, "{name}: peak {peak}");
        assert!(data.len() < 16 * 1024, "{name}: {} bytes", data.len());
    }
    let (a, b) = (decode_wav(&read("sounds/steps.wav")).unwrap(), decode_wav(&read("sounds/steps2.wav")).unwrap());
    assert_ne!(a.samples, b.samples, "the two footsteps differ");
}

// ---- levels ------------------------------------------------------------------

fn level(name: &str) -> Level {
    Level::load(root().join("levels").join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn foot_tile(l: &Level, types: &TileTypes, x: f32, y: f32) -> (u16, TileKind) {
    let id = l.tile_id_at((x + FOOT_DX).floor() as i32, (y + FOOT_DY).floor() as i32).expect("foot on the board");
    (id, types.kind(id))
}

#[test]
fn starter_levels_load_and_obey_the_tile_contract() {
    let types = TileTypes::load(root().join("tiledefs.txt")).unwrap();
    let rows = tiledef_rows();
    let named: HashSet<u16> = rows.iter().filter(|r| r.4 != "unassigned").map(|r| r.0).collect();
    for name in ["new1.nw", "new2.nw"] {
        let l = level(name);
        // a full 64x64 board, every id named in tiledefs.txt (so it has art)
        assert_eq!(l.boards.len(), 64, "{name}: one BOARD row per tile row");
        for y in 0..64 {
            for x in 0..64 {
                let id = l.tile_id_at(x, y).unwrap();
                assert!(named.contains(&id), "{name}: ({x},{y}) uses tile {id}, which has no art");
            }
        }
        // links: the destination exists, and the arrival foot point is open floor
        assert!(!l.links.is_empty());
        for k in &l.links {
            let dest = level(&k.dest);
            assert!(k.width > 0 && k.height > 0 && k.x >= 0 && k.y >= 0 && k.x + k.width <= 64 && k.y + k.height <= 64);
            let nx = k.new_x.unwrap_or(k.x as f32);
            let ny = k.new_y.unwrap();
            assert_eq!(foot_tile(&dest, &types, nx, ny).1, TileKind::Open, "{name} -> {} arrival", k.dest);
        }
        // chests sit on chest art; signs on sign art; both have text
        let id = |n: &str| id_of(&rows, n);
        for c in &l.chests {
            let (cx, cy) = (c.x, c.y);
            let ids = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| l.tile_id_at(cx + dx, cy + dy).unwrap());
            let closed_g = [id("chest0"), id("chest1"), id("chest2"), id("chest3")];
            let closed_w = [id("chestw0"), id("chestw1"), id("chestw2"), id("chestw3")];
            assert!(ids == closed_g || ids == closed_w, "{name}: chest at {cx},{cy} has tiles {ids:?}");
        }
        for s in &l.signs {
            let ids = [l.tile_id_at(s.x, s.y).unwrap(), l.tile_id_at(s.x + 1, s.y).unwrap()];
            assert!(ids == [id("sign0"), id("sign1")] || ids == [id("signw0"), id("signw1")], "{name}: sign at {},{}", s.x, s.y);
            assert!(s.text.len() > 10);
        }
        assert!(!l.chests.is_empty() && !l.signs.is_empty());
    }
    // new1: the default spawn (30, 30.5) is open, the pond is water, bushes block
    let l = level("new1.nw");
    assert_eq!(foot_tile(&l, &types, 30.0, 30.5).1, TileKind::Open);
    assert!((0..64).any(|y| (0..64).any(|x| types.kind(l.tile_id_at(x, y).unwrap()) == TileKind::Water)));
    let water = (0..64).flat_map(|y| (0..64).map(move |x| (x, y))).find(|&(x, y)| types.is_water(l.tile_id_at(x, y).unwrap())).unwrap();
    let bush = (0..64)
        .flat_map(|y| (0..64).map(move |x| (x, y)))
        .find(|&(x, y)| {
            l.tile_id_at(x, y) == Some(id_of(&rows, "bush"))
                && y > 3
                && x > 3
                && x < 60
                && types.kind(l.tile_id_at(x, y + 1).unwrap()) == TileKind::Open
        })
        .unwrap();
    // a self link and a link to the second level
    assert!(l.links.iter().any(|k| k.dest == "new1.nw"));
    assert!(l.links.iter().any(|k| k.dest == "new2.nw"));
    assert!(level("new2.nw").links.iter().any(|k| k.dest == "new1.nw"));
    // the same through the offline session: walking, swimming and bumping
    let mut s = OfflineSession::default();
    s.load_level(root().join("levels/new1.nw")).unwrap();
    s.load_tiletypes(root().join("tiledefs.txt")).unwrap();
    s.player_x = 30.0;
    s.player_y = 30.5;
    s.refresh_water();
    assert!(!s.in_water);
    s.player_x = water.0 as f32 + 0.5 - FOOT_DX;
    s.player_y = water.1 as f32 + 0.5 - FOOT_DY;
    s.refresh_water();
    assert!(s.in_water, "a foot on the pond swims");
    // standing next to a bush, a step into it does not move the player
    s.player_x = bush.0 as f32 + 0.5 - FOOT_DX;
    s.player_y = bush.1 as f32 + 0.9;
    s.refresh_water();
    for _ in 0..6 {
        s.try_move(0.0, -0.25);
    }
    // the body's top edge (y + 1) may walk up to the bush's lower edge but
    // not into the bush tile
    assert!(s.player_y >= bush.1 as f32, "bush blocks: sprite reached y = {}", s.player_y);
    assert!(s.player_y < bush.1 as f32 + 0.9, "the player did walk up to the bush");
    s.refresh_water();
    assert!(!s.in_water);
}

// ---- the resolver and --check-assets ---------------------------------------

#[test]
fn resolver_and_check_assets_find_every_file() {
    let root = AssetRoot::new(root());
    assert!(root.is_valid());
    for v in [ClientVersion::V205, ClientVersion::V217, ClientVersion::V222, ClientVersion::V231] {
        let r = AssetResolver::new(root.clone(), v.profile());
        for g in ["idle", "walk", "swim", "sword", "sit", "hurt"] {
            assert!(r.gani(g).is_some(), "{}: {g}.gani", v.label());
        }
        assert!(r.sound("steps2").is_some() && r.sound("water").is_some() && r.sound("sword").is_some());
        assert!(r.level("new1.nw").is_some() && r.level("new2.nw").is_some());
        assert!(r.image_in("bodies", "body.png").is_some() && r.image_in("heads", "head0.png").is_some());
        assert!(r.image("sprites.png").is_some() && r.image("state.png").is_some());
        assert!(r.tile_types().is_some());
        assert!(r.tiledef("tiledefsOffline.txt").is_some());
        let rep = check_folder(&root, v.profile());
        for f in &rep.findings {
            assert!(
                !matches!(f.status, Status::BadFile(_)) && !(f.need.required && !f.is_ok()),
                "{}: {} -> {:?}",
                v.label(),
                f.need.name,
                f.status
            );
        }
    }
}
