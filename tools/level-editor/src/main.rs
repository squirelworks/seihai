//! Seihai level editor: the command line, and a minimal window (`gui`).
//!
//!   level-editor [--version V] list  FILE.nw [--rows]
//!   level-editor [--version V] paint FILE.nw X Y W H TILE [--layer L] (--out FILE | --in-place)
//!   level-editor [--version V] save  FILE.nw (--out FILE | --in-place)
//!   level-editor [--version V] [--sheet PNG] gui [FILE.nw]
//!   level-editor versions
//!
//! `level-editor FILE.nw` is `list FILE.nw`. See documentation/level-editor.md.

use level_editor::{gui, Document, EditError, Version, SIZE};
use std::path::{Path, PathBuf};
use std::process;

const USAGE: &str = "\
Usage:
  level-editor [--version V] list  FILE.nw [--rows]
  level-editor [--version V] paint FILE.nw X Y W H TILE [--layer L] (--out FILE | --in-place)
  level-editor [--version V] save  FILE.nw (--out FILE | --in-place)
  level-editor [--version V] [--sheet PNG] gui [FILE.nw]
  level-editor versions

  list    summary and the most used tile ids; --rows dumps the board, one row per line,
          each tile as three hex digits
  paint   fill the rectangle (X, Y, W, H in tiles, 0 to 63) with tile id TILE (0 to 4095,
          decimal or 0x hex); --layer 1 paints layer 1 instead of layer 0
  save    open and write again (the same bytes when the level was not changed)
  gui     open the window: tile palette, click to paint (right click picks the tile
          under the mouse), Save / Save as; --sheet sets the tile sheet (default: the
          built-in pics1.png)
  --version V   editor version to follow: 2.22 (default) or 2.17.1; others say \"not yet\"";

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("level-editor: {msg}");
    process::exit(1);
}

struct Args {
    version: Version,
    out: Option<PathBuf>,
    in_place: bool,
    rows: bool,
    layer: usize,
    sheet: Option<PathBuf>,
    positional: Vec<String>,
}

fn parse_args(raw: Vec<String>) -> Args {
    let mut a = Args { version: Version::V222, out: None, in_place: false, rows: false, layer: 0, sheet: None, positional: Vec::new() };
    let mut it = raw.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| -> String {
            if let Some(v) = arg.strip_prefix(&format!("{name}=")) {
                v.to_string()
            } else {
                it.next().unwrap_or_else(|| fail(format!("{name} needs a value")))
            }
        };
        if arg == "--version" || arg.starts_with("--version=") {
            let v = value("--version");
            a.version = Version::parse(&v).unwrap_or_else(|e| fail(e));
        } else if arg == "--out" || arg.starts_with("--out=") {
            a.out = Some(PathBuf::from(value("--out")));
        } else if arg == "--sheet" || arg.starts_with("--sheet=") {
            a.sheet = Some(PathBuf::from(value("--sheet")));
        } else if arg == "--layer" || arg.starts_with("--layer=") {
            let v = value("--layer");
            a.layer = match v.trim() {
                "0" => 0,
                "1" => 1,
                _ => fail(format!("--layer must be 0 or 1, got \"{v}\"")),
            };
        } else if arg == "--in-place" {
            a.in_place = true;
        } else if arg == "--rows" {
            a.rows = true;
        } else if arg == "-h" || arg == "--help" {
            println!("{USAGE}");
            process::exit(0);
        } else if arg.starts_with("--") {
            fail(format!("unknown option {arg}\n{USAGE}"));
        } else {
            a.positional.push(arg);
        }
    }
    a
}

fn number(name: &str, text: &str) -> u32 {
    let t = text.trim();
    let parsed = match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => t.parse(),
    };
    parsed.unwrap_or_else(|_| fail(format!("{name} must be a whole number, got \"{text}\"")))
}

fn open(path: &str) -> Document {
    Document::open(Path::new(path)).unwrap_or_else(|e: EditError| fail(e))
}

fn list(path: &str, rows: bool, version: Version) {
    let doc = open(path);
    let s = doc.summary();
    println!("Level:   {path} (GLEVNW01 text, editor version {})", version.name());
    println!("Board:   {SIZE} x {SIZE}; BOARD rows placed {}, skipped {}, with a layer other than 0 {} (the layer is not read)", s.rows_placed, s.rows_skipped, s.rows_other_layer);
    println!("Layers:  layer 1 cells filled {}; rows of other layers kept as they are {}", doc.layer1_count(), s.rows_other_layers_kept);
    println!("Objects: links {}, signs {}, chests {}, baddies {}, NPCs {}, HEIGHTS block {}", s.links, s.signs, s.chests, s.baddies, s.npcs, if s.heights { "yes" } else { "no" });
    let hist = doc.histogram();
    println!("Tiles:   {} distinct ids; most used:", hist.len());
    for (id, n) in hist.iter().take(10) {
        println!("  {id:4} (0x{id:03X}) x {n}");
    }
    if rows {
        println!("Rows (tile ids in hex, x = 0 to 63 left to right):");
        for y in 0..SIZE {
            let row = doc.row(y).expect("row in range");
            let cells: Vec<String> = row.iter().map(|t| format!("{t:03X}")).collect();
            println!("{y:2}: {}", cells.join(" "));
        }
    }
}

fn target(a: &Args, input: &str) -> PathBuf {
    match (&a.out, a.in_place) {
        (Some(_), true) => fail("use --out or --in-place, not both"),
        (Some(p), false) => p.clone(),
        (None, true) => PathBuf::from(input),
        (None, false) => fail("say where to write: --out FILE or --in-place"),
    }
}

fn main() {
    let a = parse_args(std::env::args().skip(1).collect());
    let mut pos = a.positional.iter().map(String::as_str);
    match pos.next() {
        None => {
            println!("{USAGE}");
            process::exit(1);
        }
        Some("versions") => {
            for v in Version::SUPPORTED {
                println!("{}", v.name());
            }
            println!("(other versions: not yet)");
        }
        Some("list") => {
            let file = pos.next().unwrap_or_else(|| fail("list needs a FILE.nw"));
            list(file, a.rows, a.version);
        }
        Some("paint") => {
            let file = pos.next().unwrap_or_else(|| fail("paint needs FILE.nw X Y W H TILE"));
            let nums: Vec<u32> = ["X", "Y", "W", "H", "TILE"]
                .iter()
                .map(|n| number(n, pos.next().unwrap_or_else(|| fail(format!("paint needs FILE.nw X Y W H TILE (missing {n})")))))
                .collect();
            let out = target(&a, file);
            let mut doc = open(file);
            let (x, y, w, h, tile) = (nums[0] as usize, nums[1] as usize, nums[2] as usize, nums[3] as usize, nums[4]);
            let changed = doc.paint_layer(a.layer, x, y, w, h, Some(tile)).unwrap_or_else(|e| fail(e));
            doc.save(&out).unwrap_or_else(|e| fail(e));
            println!(
                "painted {w} x {h} at ({x}, {y}) on layer {} with tile {tile}: {changed} of {} tiles changed; wrote {}",
                a.layer,
                w * h,
                out.display()
            );
        }
        Some("gui") => {
            let file = pos.next().map(PathBuf::from);
            let sheet = match &a.sheet {
                Some(p) => {
                    let data = std::fs::read(p).unwrap_or_else(|e| fail(format!("cannot read {}: {e}", p.display())));
                    gui::Sheet::from_png(&data).unwrap_or_else(|e| fail(e))
                }
                None => gui::Sheet::builtin(),
            };
            gui::run(a.version, sheet, file.as_deref()).unwrap_or_else(|e| fail(format!("cannot open the window: {e}")));
        }
        Some("save") => {
            let file = pos.next().unwrap_or_else(|| fail("save needs a FILE.nw"));
            let out = target(&a, file);
            let doc = open(file);
            doc.save(&out).unwrap_or_else(|e| fail(e));
            println!("wrote {} ({} bytes, unchanged level)", out.display(), doc.to_bytes().len());
        }
        Some(file) if file.ends_with(".nw") || Path::new(file).exists() => list(file, a.rows, a.version),
        Some(other) => fail(format!("unknown command \"{other}\"\n{USAGE}")),
    }
}
