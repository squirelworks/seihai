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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("assets").join("default")
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

/// Tile ids of the standard Graal layout whose meaning is documented, and
/// which the built-in sheet draws with art of their own
/// (`documentation/default-assets.md`).
const GRASS: u16 = 0;
const GRASS2: u16 = 2047;
const BUSH: u16 = 2;
const STONE: u16 = 34;
const SIGN: u16 = 512;
const POT: u16 = 684;
const BLACK_STONE: u16 = 990;
const TALL_GRASS: u16 = 420;
const CHEST: u16 = 1784;
const CHEST_OPEN: u16 = 829;
const DEEP_WATER: u16 = 322;
const SHALLOW: u16 = 398;
const NO_SWIM: u16 = 3644;
const LAVA: u16 = 2882;
const LAVA_FLOOR: u16 = 2958;

/// The ids a starter level may use: the documented ones, each 2x2 object whole.
fn documented_ids() -> HashSet<u16> {
    let mut v: HashSet<u16> = [GRASS, GRASS2, DEEP_WATER, SHALLOW, NO_SWIM].into_iter().collect();
    for o in [BUSH, STONE, SIGN, POT, BLACK_STONE, TALL_GRASS, CHEST, CHEST_OPEN, 677, 679, 1770, 1802, 1834] {
        v.extend([o, o + 1, o + 16, o + 17]);
    }
    v
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
    let cell_px = |id: u16| -> Vec<[u8; 3]> {
        let t = id as u32;
        // the mapping Seihai's blit_tile uses, and the real sheets' layout
        let (col, row) = (t % 16 + (t / 512) * 16, (t / 16) % 32);
        let mut v = Vec::new();
        for y in 0..16 {
            for x in 0..16 {
                let i = (((row * 16 + y) * sw + col * 16 + x) * 3) as usize;
                v.push([rgb[i], rgb[i + 1], rgb[i + 2]]);
            }
        }
        v
    };
    // blit_tile draws exactly the id's cell, in every block of the sheet
    for id in [GRASS, BUSH, DEEP_WATER, SIGN, CHEST_OPEN, CHEST, GRASS2, LAVA, NO_SWIM, 4095] {
        let mut fb = Framebuffer::new(16, 16);
        fb.blit_tile(&rgb, sw, sh, id, 0, 0);
        let drawn: Vec<[u8; 3]> = fb.pixels.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
        assert_eq!(drawn, cell_px(id), "blit_tile({id})");
    }
    // Every id has art of its type: tiles of one type share a look, and the
    // types look different from each other (grass, wall, water, lava).
    let types = TileTypes::load(root().join("tiledefs.txt")).unwrap();
    let main_colour = |id: u16| -> [u8; 3] {
        let mut n: std::collections::HashMap<[u8; 3], usize> = Default::default();
        for p in cell_px(id) {
            *n.entry(p).or_default() += 1;
        }
        n.into_iter().max_by_key(|(_, k)| *k).unwrap().0
    };
    let (open, wall, water, lava) = (main_colour(1), main_colour(4), main_colour(461), main_colour(2883));
    assert_eq!((types.kind(1), types.kind(4), types.kind(461), types.kind(2883)), (TileKind::Open, TileKind::Blocking, TileKind::Water, TileKind::Lava));
    assert_eq!(HashSet::from([open, wall, water, lava]).len(), 4, "four types, four looks");
    let documented = documented_ids();
    for id in (0..4096u16).filter(|i| !documented.contains(i)) {
        let want = match (types.raw(id).unwrap(), types.kind(id)) {
            (_, TileKind::Water) => water,
            (_, TileKind::Lava) => lava,
            (0, TileKind::Open) => open,
            (22, TileKind::Blocking) => wall,
            _ => continue,
        };
        assert_eq!(main_colour(id), want, "tile {id} looks like its type");
    }
    // the documented objects have art of their own, unlike plain grass or wall
    for id in [BUSH, STONE, SIGN, POT, BLACK_STONE, TALL_GRASS, CHEST, CHEST_OPEN] {
        let px = cell_px(id + 17);
        assert!(px != cell_px(1) && px != cell_px(4), "object {id}");
    }
}

#[test]
fn tile_types_contract() {
    // One table for this sheet and for a Graal pics1.png: the types of the
    // 2.22 arrays.dat (tile-types.md), by tile id.
    let types = TileTypes::load(root().join("tiledefs.txt")).expect("tiledefs.txt loads");
    assert_eq!(types.types.len(), 4096);
    let kind = |id: u16| types.kind(id);
    // the new4.nw tiles, as tile-types.md lists them for the 2.22 file
    assert_eq!(kind(SHALLOW), TileKind::Open);
    assert_eq!((types.raw(461), types.raw(472)), (Some(11), Some(11)));
    assert_eq!(kind(NO_SWIM), TileKind::Blocking);
    assert_eq!(types.raw(LAVA), Some(12));
    assert_eq!(types.raw(LAVA_FLOOR), Some(13));
    assert!(types.is_lava(LAVA) && types.is_lava(LAVA_FLOOR));
    // deep water swims; grass is open
    assert_eq!(kind(DEEP_WATER), TileKind::Water);
    assert_eq!((kind(GRASS), kind(GRASS2)), (TileKind::Open, TileKind::Open));
    // bushes, stones, signs, pots, black stones and chests block, and thrown
    // objects fly over them (array 2)
    for o in [BUSH, STONE, SIGN, POT, BLACK_STONE, CHEST, CHEST_OPEN] {
        for t in [o, o + 1, o + 16, o + 17] {
            assert_eq!(types.raw(t), Some(20), "{t}");
        }
    }
    // a plain wall
    assert_eq!(types.raw(4), Some(22));
    // bed tops are walls and still bed tops (the `bedtop:` line), so the
    // sleep pose works with only the built-in table loaded
    for id in 708..=711 {
        assert!(types.is_wall(id) && types.is_bed_top(id), "{id}");
    }
    assert!(!types.is_bed_top(724) && !types.is_bed_top(707));
    // what a cut or lift leaves is open, and so is tall grass
    for t in [677, 679, 1770, 1802, 1834, TALL_GRASS] {
        assert_eq!(kind(t), TileKind::Open, "{t}");
    }
    // a chair (arrays.dat array 4) and a bed bottom (array 6)
    assert_eq!((types.raw(681), types.raw(724)), (Some(3), Some(5)));
    // the list counts of the 2.22 file: 934 passable, 116 liquid
    let liquid = (0..4096u16).filter(|&t| matches!(types.raw(t), Some(11 | 12))).count();
    assert_eq!(liquid, 116);
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
    let (shw, shh, _) = read_png_rgba(&read("shields/shield1.png")).unwrap();
    let sheet_size = |s: &str| match s {
        "BODY" => (bw, bh),
        "HEAD" => (hw, hh),
        "SPRITES" => (spw, sph),
        "SWORD" => (sww, swh),
        "SHIELD" => (shw, shh),
        // The hat: Seihai ships none; the stock hat sheet is 192x144.
        "ATTR1" => (192, 144),
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
                for (k, p) in row.iter().enumerate() {
                    let s = g.sprites.iter().find(|s| s.id == p.sprite_id).unwrap_or_else(|| panic!("{name}: sprite {} undefined", p.sprite_id));
                    let (w, h) = sheet_size(&s.sheet);
                    assert!(s.x >= 0 && s.y >= 0 && (s.x + s.w) as u32 <= w && (s.y + s.h) as u32 <= h, "{name}: sprite {} outside {}", s.id, s.sheet);
                    // the hat comes right after its head, at the stock offset for the head's facing
                    if s.sheet == "ATTR1" {
                        let head = &row[k - 1];
                        let facing = (head.sprite_id - 100) as usize;
                        assert_eq!(p.sprite_id, 40 + facing as u32, "{name}: hat of the head's facing");
                        let (dx, dy) = [(-8, -18), (-12, -18), (-8, -18), (-4, -18)][facing];
                        assert_eq!((p.x - head.x, p.y - head.y), (dx, dy), "{name}: hat offset");
                        continue;
                    }
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
    // idle uses body pose 0 (ids 200..203) and heads 100..103, as draw_player_sprites does,
    // then the hat (ids 40..43); the shield (ids 10..13) is behind the body, or in front
    // facing down, at the stock idle.gani places against the head
    let idle = Gani::load(root().join("gani/idle.gani")).unwrap();
    assert_eq!(idle.default_attrs, <[Option<String>; 5]>::default(), "Seihai ships no hat");
    let shield_at = [(36, 16), (6, 16), (8, 24), (34, 16)];
    for dir in 0..4usize {
        let row = &idle.frames[0].rows[dir];
        let ids: Vec<u32> = row.iter().map(|p| p.sprite_id).collect();
        let (d, s) = (dir as u32, 10 + dir as u32);
        let want = if dir == 2 { vec![0, 200 + d, 100 + d, 40 + d, s] } else { vec![0, s, 200 + d, 100 + d, 40 + d] };
        assert_eq!(ids, want);
        let sh = row.iter().find(|p| p.sprite_id == s).unwrap();
        assert_eq!((sh.x, sh.y), shield_at[dir]);
    }
    // the shield is in idle, walk and sword only, as in the stock GANIs
    for (name, has) in [("idle", true), ("walk", true), ("sword", true), ("sit", false), ("swim", false), ("hurt", false), ("carry", false), ("dead", false)] {
        let g = Gani::load(root().join("gani").join(format!("{name}.gani"))).unwrap();
        let any = g.frames.iter().flat_map(|f| &f.rows).flatten().any(|p| (10..14).contains(&p.sprite_id));
        assert_eq!(any, has, "{name} shield");
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
    let documented = documented_ids();
    for name in ["new1.nw", "new2.nw"] {
        let l = level(name);
        // a full 64x64 board of documented ids only, so it reads the same on
        // a Graal pics1.png
        assert_eq!(l.boards.len(), 64, "{name}: one BOARD row per tile row");
        for y in 0..64 {
            for x in 0..64 {
                let id = l.tile_id_at(x, y).unwrap();
                assert!(documented.contains(&id), "{name}: ({x},{y}) uses tile {id}, whose meaning is not documented");
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
        // the board under a chest is open grass (the chest is drawn from its
        // record); a sign record is the lower row of a sign; both have text
        for c in &l.chests {
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                assert_eq!(l.tile_id_at(c.x + dx, c.y + dy), Some(GRASS2), "{name}: chest at {},{}", c.x, c.y);
            }
        }
        for s in &l.signs {
            let ids = [l.tile_id_at(s.x, s.y).unwrap(), l.tile_id_at(s.x + 1, s.y).unwrap()];
            assert_eq!(ids, [SIGN + 16, SIGN + 17], "{name}: sign at {},{}", s.x, s.y);
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
            l.tile_id_at(x, y) == Some(BUSH + 16)
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
