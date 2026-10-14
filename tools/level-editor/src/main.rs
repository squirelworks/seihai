//! Simple .nw level editor / viewer.
//!
//! Usage:
//!   level-editor <file.nw>              # show summary + ASCII map
//!   level-editor <file.nw> --export out.nw

use seihai::assets::Level;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: level-editor <file.nw> [--export out.nw]");
        process::exit(1);
    }

    let path = &args[1];
    let level = match Level::load(path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load {path}: {e}");
            process::exit(1);
        }
    };

    println!("Level: {path}");
    println!("  BOARD runs: {}", level.boards.len());
    println!("  NPCs:       {}", level.npcs.len());

    if !level.npcs.is_empty() {
        println!("\nNPCs:");
        for (i, npc) in level.npcs.iter().enumerate() {
            println!("  [{i}] image={} pos=({:.1}, {:.1})", npc.image, npc.x, npc.y);
        }
    }

    // ASCII overview of layer 0 (first 40x20)
    println!("\nLayer 0 preview (.) = empty/Id, # = other:");
    let max_x = 40;
    let max_y = 20;
    for y in 0..max_y {
        for x in 0..max_x {
            match level.tile_at(x, y) {
                Some(t) if t == *b"Id" => print!("."),
                Some(_) => print!("#"),
                None => print!(" "),
            }
        }
        println!();
    }

    // Optional re-export
    if args.len() >= 4 && args[2] == "--export" {
        let out = &args[3];
        if let Err(e) = export_level(&level, out) {
            eprintln!("Export failed: {e}");
            process::exit(1);
        }
        println!("\nExported to {out}");
    }
}

fn export_level(level: &Level, path: &str) -> Result<(), String> {
    let mut out = String::from("GLEVNW01\n");
    for run in &level.boards {
        let tiles: String = run
            .tiles
            .iter()
            .map(|t| String::from_utf8_lossy(t).into_owned())
            .collect();
        out.push_str(&format!(
            "BOARD {} {} {} {} {}\n",
            run.x, run.y, run.width, run.layer, tiles
        ));
    }
    for npc in &level.npcs {
        out.push_str(&format!("NPC {} {} {}\n", npc.image, npc.x, npc.y));
        out.push_str(&npc.script);
        if !npc.script.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("NPCEND\n");
    }
    fs::write(path, out).map_err(|e| e.to_string())
}
