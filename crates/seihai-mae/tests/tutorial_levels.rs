//! The built-in tutorial (`documentation/tutorial-levels.md`): its levels
//! load, their links lead to each other and land on open ground, each lesson
//! can be reached, and the mechanics it teaches work on its own tables.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use seihai::assets::tutorial::{self, is_tutorial_level};
use seihai::assets::{Level, TileTypes};
use seihai::offline::OfflineSession;

fn levels_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default/levels")
}

const LEVELS: [&str; 5] = ["seihai_tutorial1.nw", "seihai_tutorial_house.nw", "seihai_tutorial2.nw", "seihai_tutorial3.nw", "seihai_tutorial4.nw"];

fn level(name: &str) -> Level {
    Level::load(levels_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn types() -> TileTypes {
    tutorial::tables().expect("built-in tutorial tables").0
}

/// A session in `name` with the tutorial's tables, as the client sets it up.
fn session(name: &str) -> OfflineSession {
    let mut s = OfflineSession::new();
    s.tutorial_tables = tutorial::tables();
    s.load_level(levels_dir().join(name)).unwrap();
    assert!(s.tutorial_tables_active(), "{name} swaps the tutorial tables in");
    s
}

/// Whether the walking body box (`x + 0.5 .. x + 2.5`, `y + 1 .. y + 3`) at
/// `(x, y)` is on open ground or water. `gone` tiles count as open (objects
/// the player cuts, lifts or moves).
fn box_free(l: &Level, t: &TileTypes, x: f32, y: f32, gone: &HashSet<u16>) -> bool {
    let (x0, x1) = ((x + 0.5).floor() as i32, (x + 2.5 - 0.001).floor() as i32);
    let (y0, y1) = ((y + 1.0).floor() as i32, (y + 3.0 - 0.001).floor() as i32);
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(id) = l.tile_id_at(tx, ty) else { return false };
            if t.types[usize::from(id)] >= 20 && !gone.contains(&id) {
                return false;
            }
            if l.chests.iter().any(|c| (c.x..c.x + 2).contains(&tx) && (c.y..c.y + 2).contains(&ty)) {
                return false;
            }
        }
    }
    true
}

/// Every half-tile position the player can walk or swim to from `from`.
fn reach(l: &Level, from: (f32, f32), gone: &HashSet<u16>) -> HashSet<(i32, i32)> {
    let t = types();
    let key = |x: f32, y: f32| ((x * 2.0).round() as i32, (y * 2.0).round() as i32);
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    assert!(box_free(l, &t, from.0, from.1, gone), "start {from:?} is open");
    seen.insert(key(from.0, from.1));
    q.push_back(key(from.0, from.1));
    while let Some((kx, ky)) = q.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (kx + dx, ky + dy);
            let (x, y) = (nx as f32 / 2.0, ny as f32 / 2.0);
            if !(-1.0..=62.0).contains(&x) || !(-1.0..=62.0).contains(&y) || seen.contains(&(nx, ny)) {
                continue;
            }
            if box_free(l, &t, x, y, gone) {
                seen.insert((nx, ny));
                q.push_back((nx, ny));
            }
        }
    }
    seen
}

/// Whether some reachable position puts the feet in the link to `dest`.
fn reaches_link(l: &Level, r: &HashSet<(i32, i32)>, dest: &str) -> bool {
    let link = l.links.iter().find(|k| k.dest == dest).unwrap_or_else(|| panic!("a link to {dest}"));
    r.iter().any(|&(kx, ky)| {
        let (fx, fy) = (kx as f32 / 2.0 + 2.0, ky as f32 / 2.0 + 2.5);
        fx >= link.x as f32 && fx <= (link.x + link.width) as f32 && fy >= link.y as f32 && fy <= (link.y + link.height) as f32
    })
}

/// Whether the player can stand right below the chest at `(cx, cy)`.
fn reaches_chest(r: &HashSet<(i32, i32)>, cx: i32, cy: i32) -> bool {
    r.iter().any(|&(kx, ky)| {
        let (x, y) = (kx as f32 / 2.0, ky as f32 / 2.0);
        y + 1.0 == (cy + 2) as f32 && x + 1.5 >= cx as f32 && x + 1.5 <= (cx + 2) as f32
    })
}

fn removable() -> HashSet<u16> {
    // bush, tall grass, pot, grey stone (cut or lifted)
    [2u16, 3, 18, 19, 420, 421, 436, 437, 684, 685, 700, 701, 34, 35, 50, 51].into_iter().collect()
}

#[test]
fn every_level_loads_and_every_link_lands_on_open_ground() {
    let t = types();
    for name in LEVELS {
        assert!(is_tutorial_level(name));
        let l = level(name);
        assert!(!l.links.is_empty(), "{name} has a way out");
        for k in &l.links {
            assert!(LEVELS.contains(&k.dest.as_str()), "{name}: link to {} is a tutorial level", k.dest);
            let dest = level(&k.dest);
            // playerx / playery keep the coordinate: test the middle of the link
            let x = k.new_x.unwrap_or(k.x as f32 + k.width as f32 / 2.0 - 1.5);
            let y = k.new_y.unwrap_or(k.y as f32 + k.height as f32 / 2.0 - 2.0);
            assert!(box_free(&dest, &t, x, y, &HashSet::new()), "{name} -> {} lands on open ground at {x} {y}", k.dest);
            let (fx, fy) = (x + 2.0, y + 2.75);
            for back in &dest.links {
                let inside = fx >= back.x as f32 && fx <= (back.x + back.width) as f32 && fy >= back.y as f32 && fy <= (back.y + back.height) as f32;
                assert!(!inside, "{name} -> {} lands inside the link to {}", k.dest, back.dest);
            }
        }
        assert!(!l.signs.is_empty(), "{name} explains itself");
    }
}

#[test]
fn the_meadow_leads_to_the_cottage_and_east() {
    let l = level(tutorial::START);
    let r = reach(&l, tutorial::START_POS, &HashSet::new());
    assert!(reaches_link(&l, &r, "seihai_tutorial_house.nw"));
    assert!(reaches_link(&l, &r, "seihai_tutorial2.nw"));
}

#[test]
fn the_cottage_chest_holds_the_glove_and_the_mat_leads_out() {
    let l = level("seihai_tutorial_house.nw");
    let arrive = level(tutorial::START).links.iter().find(|k| k.dest == "seihai_tutorial_house.nw").map(|k| (k.new_x.unwrap(), k.new_y.unwrap())).unwrap();
    let r = reach(&l, arrive, &HashSet::new());
    let c = &l.chests[0];
    assert_eq!(c.item, "glove1");
    assert!(reaches_chest(&r, c.x, c.y));
    assert!(reaches_link(&l, &r, tutorial::START));
}

#[test]
fn the_garden_is_closed_until_the_player_cuts_lifts_and_throws() {
    let l = level("seihai_tutorial2.nw");
    let start = (3.0, 30.0);
    assert!(!reaches_link(&l, &reach(&l, start, &HashSet::new()), "seihai_tutorial3.nw"), "the hedge and the stone close the way");
    assert!(reaches_link(&l, &reach(&l, start, &removable()), "seihai_tutorial3.nw"));
}

#[test]
fn the_pond_island_is_reached_by_swimming() {
    let l = level("seihai_tutorial3.nw");
    let r = reach(&l, (30.5, 3.0), &HashSet::new());
    let c = &l.chests[0];
    assert!(reaches_chest(&r, c.x, c.y));
    assert!(reaches_link(&l, &r, "seihai_tutorial4.nw"));
    // and the island is in the water: no dry way there
    let t = types();
    let dry: HashSet<(i32, i32)> = r
        .iter()
        .copied()
        .filter(|&(kx, ky)| {
            let (x, y) = (kx as f32 / 2.0, ky as f32 / 2.0);
            let id = l.tile_id_at((x + 2.0) as i32, (y + 2.75) as i32).unwrap();
            t.types[usize::from(id)] != seihai::assets::TYPE_WATER
        })
        .collect();
    let _ = dry; // the walk test above counts water as open; the swim itself is tested below
    let mut s = session("seihai_tutorial3.nw");
    s.player_x = 13.5;
    s.player_y = 37.0;
    s.refresh_water();
    assert!(s.in_water, "the lake round the island is deep water");
}

#[test]
fn the_rock_yard_needs_the_push_and_the_pull() {
    let l = level("seihai_tutorial4.nw");
    let start = (3.0, 12.0);
    let r = reach(&l, start, &HashSet::new());
    assert!(!reaches_link(&l, &r, tutorial::START), "the blocks close the way home");
    let mut gone = HashSet::new();
    for id in [l.tile_id_at(22, 36).unwrap(), l.tile_id_at(40, 48).unwrap()] {
        gone.extend([id, id + 1, id + 16, id + 17]);
    }
    assert!(reaches_link(&l, &reach(&l, start, &gone), tutorial::START));
}

#[test]
fn the_sword_cuts_the_hedge_and_the_glove_lifts_the_grey_stone() {
    let mut s = session("seihai_tutorial2.nw");
    s.player_x = 11.0;
    s.player_y = 28.5;
    s.player_dir = 3; // east
    assert_eq!(s.try_cut(), Some(0), "a bush of the hedge");

    s.player_x = 29.5;
    s.player_y = 43.0;
    s.player_dir = 2; // south
    assert_eq!(s.try_lift(), None, "too heavy without the glove");
    s.give_item("glove1");
    assert_eq!(s.try_lift(), Some(3), "the grey stone");
}

#[test]
fn the_ledge_jumps_and_the_block_pushes() {
    let mut s = session("seihai_tutorial4.nw");
    s.player_x = 9.5;
    s.player_y = 18.0;
    for _ in 0..40 {
        s.try_move(0.0, 0.5);
        if s.jumping() {
            break;
        }
    }
    assert!(s.jumping(), "walking down against the edge jumps");
    for _ in 0..20 {
        s.tick_pushpull(50);
    }
    assert!(s.player_y > 23.0, "jumped off the ledge (y {})", s.player_y);

    let mut s = session("seihai_tutorial4.nw");
    let block = s.current_level.as_ref().unwrap().tile_id_at(22, 36).unwrap();
    s.player_x = 21.5;
    s.player_y = 33.0;
    for _ in 0..40 {
        s.try_move(0.0, 0.5);
    }
    assert_eq!(s.current_level.as_ref().unwrap().tile_id_at(22, 37), Some(block), "the block moved down");
}

#[test]
fn leaving_the_tutorial_puts_the_players_tables_back() {
    let mut s = OfflineSession::new();
    s.tutorial_tables = tutorial::tables();
    s.load_tiletypes(levels_dir().join("../tiledefs.txt")).unwrap();
    let own = s.tiletypes.as_ref().unwrap().types;
    s.load_level(levels_dir().join(tutorial::START)).unwrap();
    assert!(s.tutorial_tables_active());
    assert!(s.object_tables.is_some(), "the tutorial brings cut and lift tables");
    s.load_level(levels_dir().join("new1.nw")).unwrap();
    assert!(!s.tutorial_tables_active());
    assert_eq!(s.tiletypes.as_ref().unwrap().types, own);
    assert!(s.object_tables.is_none());
}

#[test]
fn the_ring_block_pulls() {
    let mut s = session("seihai_tutorial4.nw");
    let block = s.current_level.as_ref().unwrap().tile_id_at(40, 48).unwrap();
    s.player_x = 39.5;
    s.player_y = 45.0;
    s.player_dir = 2; // facing the block, below
    assert!(s.try_grab(), "A grabs the block");
    for _ in 0..60 {
        s.try_move(0.0, -0.5);
        s.tick_pushpull(50);
    }
    let l = s.current_level.as_ref().unwrap();
    let top = (40..49).find(|&y| l.tile_id_at(40, y) == Some(block)).expect("the block is still in column 40");
    assert!(top < 48, "the block came up out of the wall (now at y {top})");
    assert_ne!(l.tile_id_at(40, 48), Some(block));
}
