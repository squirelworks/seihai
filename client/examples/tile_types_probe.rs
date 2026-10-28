//! Count the tile types an `arrays.dat` gives.
//!
//! `cargo run --example tile_types_probe -- path/to/arrays.dat`

fn main() {
    let path = std::env::args().nth(1).expect("usage: tile_types_probe <arrays.dat>");
    let t = seihai::assets::TileTypes::load_arrays(&path).expect("arrays.dat");
    let mut counts = std::collections::BTreeMap::new();
    for &b in t.types.iter() {
        *counts.entry(b).or_insert(0u32) += 1;
    }
    for (ty, n) in counts {
        println!("type {ty:>2}: {n} tiles");
    }
}
