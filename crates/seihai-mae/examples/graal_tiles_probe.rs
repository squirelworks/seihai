//! Which tile ids of binary levels land on an empty (all-black or
//! transparent) cell of a tile sheet. `cargo run --example graal_tiles_probe
//! -- SHEET.png LEVEL.graal...` prints each such id, how many levels use it
//! and one level that does, and how many board places hold no tile (drawn
//! black).

use std::collections::BTreeMap;

use seihai::assets::{read_png_rgb, tile_id, Level};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((sheet_path, levels)) = args.split_first() else {
        eprintln!("usage: graal_tiles_probe SHEET.png LEVEL.graal...");
        std::process::exit(2);
    };
    let (w, h, px) = read_png_rgb(&std::fs::read(sheet_path).expect("sheet")).expect("sheet");
    let empty = |id: u32| {
        let (col, row) = (id % 16 + (id / 512) * 16, (id / 16) % 32);
        let (sx, sy) = (col * 16, row * 16);
        if sx + 16 > w || sy + 16 > h {
            return true;
        }
        (0..16).all(|y| (0..16).all(|x| {
            let i = (((sy + y) * w + sx + x) * 3) as usize;
            px[i..i + 3] == [0, 0, 0]
        }))
    };
    let mut uses: BTreeMap<u16, (usize, String)> = BTreeMap::new();
    // Levels with board places that hold no tile at all, and how many places.
    let mut holes = (0usize, 0usize);
    let mut by_magic: BTreeMap<String, (usize, u16)> = BTreeMap::new();
    for path in levels {
        let Ok(data) = std::fs::read(path) else { continue };
        let Ok((level, layout)) = Level::parse_binary_with_layout(&data) else { continue };
        let e = by_magic.entry(layout.magic.clone()).or_default();
        e.0 += 1;
        e.1 = e.1.max(layout.max_tile);
        let missing = (0..64).flat_map(|y| (0..64).map(move |x| (x, y))).filter(|&(x, y)| level.tile_at(x, y).is_none()).count();
        if missing > 0 {
            holes.0 += 1;
            holes.1 += missing;
        }
        let mut seen = std::collections::HashSet::new();
        for run in level.boards.iter().filter(|r| r.layer == 0) {
            for &t in &run.tiles {
                let id = tile_id(t);
                if seen.insert(id) && empty(id as u32) {
                    let u = uses.entry(id).or_insert((0, path.clone()));
                    u.0 += 1;
                }
            }
        }
    }
    for (m, (n, max)) in &by_magic {
        println!("{m}: {n} levels, highest tile id {max}");
    }
    println!("levels with places that hold no tile: {}, places: {}", holes.0, holes.1);
    for (id, (n, first)) in &uses {
        println!("{id}\t{n}\t{first}");
    }
}
