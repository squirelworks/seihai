//! Seihai level editor: the command line, and a minimal window (`gui`).
//!
//!   level-editor [--version|--profile P] list  FILE [--rows]
//!   level-editor [--version|--profile P] paint FILE X Y W H TILE [--layer L] (--out FILE | --in-place)
//!   level-editor [--version|--profile P] fill  FILE X Y TILE [--layer L] (--out FILE | --in-place)
//!   level-editor [--version|--profile P] save  FILE (--out FILE | --in-place)
//!   level-editor [--version|--profile P] [--sheet PNG] gui [FILE]
//!   level-editor versions
//!
//! `level-editor FILE` is `list FILE`. Text (`.nw`) and binary (`GR-V1` /
//! `Z3-V1`) levels open. See documentation/level-editor.md.

use level_editor::{gui, Document, EditError, VersionProfile, SIZE};
use std::path::{Path, PathBuf};
use std::process;

const USAGE: &str = "\
Usage:
  level-editor [--version|--profile P] list  FILE [--rows]
  level-editor [--version|--profile P] paint FILE X Y W H TILE [--layer L] (--out FILE | --in-place)
  level-editor [--version|--profile P] fill  FILE X Y TILE [--layer L] (--out FILE | --in-place)
  level-editor [--version|--profile P] save  FILE (--out FILE | --in-place)
  level-editor [--version|--profile P] [--sheet PNG] gui [FILE]
  level-editor versions

  list    summary and the most used tile ids; --rows dumps the board, one row per line,
          each tile as three hex digits; FILE may be GLEVNW01 text or GR-V1 / Z3-V1 binary
  paint   fill the rectangle (X, Y, W, H in tiles, 0 to 63) with tile id TILE (0 to 4095,
          decimal or 0x hex); --layer 1 paints layer 1 instead of layer 0; --out extension
          picks the write profile (.nw text, .zelda Z3, otherwise GR-V1)
  fill    flood-fill the connected region under (X, Y) with tile id TILE (cells that share
          a side); --layer 1 fills layer 1 instead of layer 0
  save    open and write again (the same bytes when the level was not changed)
  gui     open the window: tile palette, click to paint, Fill tool for flood fill (right
          click picks the tile under the mouse), Save / Save as; --sheet sets the tile
          sheet (default: the built-in pics1.png)
  --version / --profile P
          target profile: 2.22 (default) or 2.17.1 for text .nw; also GR-V1.00..03,
          GR-V1.05, Z3-V1.03/04 (binary open/save ready; open by magic, save by extension)";

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("level-editor: {msg}");
    process::exit(1);
}

struct Args {
    profile: VersionProfile,
    out: Option<PathBuf>,
    in_place: bool,
    rows: bool,
    layer: usize,
    sheet: Option<PathBuf>,
    positional: Vec<String>,
}

fn parse_args(raw: Vec<String>) -> Args {
    let mut a = Args {
        profile: VersionProfile::DEFAULT,
        out: None,
        in_place: false,
        rows: false,
        layer: 0,
        sheet: None,
        positional: Vec::new(),
    };
    let mut it = raw.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| -> String {
            if let Some(v) = arg.strip_prefix(&format!("{name}=")) {
                v.to_string()
            } else {
                it.next()
                    .unwrap_or_else(|| fail(format!("{name} needs a value")))
            }
        };
        if arg == "--version"
            || arg.starts_with("--version=")
            || arg == "--profile"
            || arg.starts_with("--profile=")
        {
            let name = if arg.starts_with("--profile") {
                "--profile"
            } else {
                "--version"
            };
            let v = value(name);
            a.profile = VersionProfile::parse(&v).unwrap_or_else(|e| fail(e));
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

fn open(path: &str, profile: VersionProfile) -> Document {
    Document::open_with_profile(Path::new(path), profile).unwrap_or_else(|e: EditError| fail(e))
}

fn list(path: &str, rows: bool, profile: VersionProfile) {
    let doc = open(path, profile);
    let s = doc.summary();
    println!(
        "Level:   {path} ({}, profile {})",
        doc.profile().name(),
        profile.name()
    );
    println!("Board:   {SIZE} x {SIZE}; BOARD rows placed {}, skipped {}, with a layer other than 0 {} (the layer is not read)", s.rows_placed, s.rows_skipped, s.rows_other_layer);
    println!(
        "Layers:  layer 1 cells filled {}; rows of other layers kept as they are {}",
        doc.layer1_count(),
        s.rows_other_layers_kept
    );
    println!(
        "Objects: links {}, signs {}, chests {}, baddies {}, NPCs {}, HEIGHTS block {}",
        s.links,
        s.signs,
        s.chests,
        s.baddies,
        s.npcs,
        if s.heights { "yes" } else { "no" }
    );
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
            for p in VersionProfile::ALL {
                let status = if p.io_ready() { "ready" } else { "parked" };
                println!("{}  [{}]  {}", p.name(), status, p.label());
            }
        }
        Some("list") => {
            let file = pos.next().unwrap_or_else(|| fail("list needs a FILE.nw"));
            list(file, a.rows, a.profile);
        }
        Some("paint") => {
            let file = pos
                .next()
                .unwrap_or_else(|| fail("paint needs FILE.nw X Y W H TILE"));
            let nums: Vec<u32> = ["X", "Y", "W", "H", "TILE"]
                .iter()
                .map(|n| {
                    number(
                        n,
                        pos.next().unwrap_or_else(|| {
                            fail(format!("paint needs FILE.nw X Y W H TILE (missing {n})"))
                        }),
                    )
                })
                .collect();
            let out = target(&a, file);
            let mut doc = open(file, a.profile);
            let (x, y, w, h, tile) = (
                nums[0] as usize,
                nums[1] as usize,
                nums[2] as usize,
                nums[3] as usize,
                nums[4],
            );
            let changed = doc
                .paint_layer(a.layer, x, y, w, h, Some(tile))
                .unwrap_or_else(|e| fail(e));
            doc.save(&out).unwrap_or_else(|e| fail(e));
            println!(
                "painted {w} x {h} at ({x}, {y}) on layer {} with tile {tile}: {changed} of {} tiles changed; wrote {}",
                a.layer,
                w * h,
                out.display()
            );
        }
        Some("fill") => {
            let file = pos
                .next()
                .unwrap_or_else(|| fail("fill needs FILE.nw X Y TILE"));
            let nums: Vec<u32> = ["X", "Y", "TILE"]
                .iter()
                .map(|n| {
                    number(
                        n,
                        pos.next().unwrap_or_else(|| {
                            fail(format!("fill needs FILE.nw X Y TILE (missing {n})"))
                        }),
                    )
                })
                .collect();
            let out = target(&a, file);
            let mut doc = open(file, a.profile);
            let (x, y, tile) = (nums[0] as usize, nums[1] as usize, nums[2]);
            let changed = doc
                .flood_fill_layer(a.layer, x, y, Some(tile))
                .unwrap_or_else(|e| fail(e));
            doc.save(&out).unwrap_or_else(|e| fail(e));
            println!(
                "filled at ({x}, {y}) on layer {} with tile {tile}: {changed} tiles changed; wrote {}",
                a.layer,
                out.display()
            );
        }
        Some("gui") => {
            let file = pos.next().map(PathBuf::from);
            let sheet = match &a.sheet {
                Some(p) => {
                    let data = std::fs::read(p)
                        .unwrap_or_else(|e| fail(format!("cannot read {}: {e}", p.display())));
                    gui::Sheet::from_png(&data).unwrap_or_else(|e| fail(e))
                }
                None => gui::Sheet::builtin(),
            };
            gui::run(a.profile, sheet, file.as_deref())
                .unwrap_or_else(|e| fail(format!("cannot open the window: {e}")));
        }
        Some("save") => {
            let file = pos.next().unwrap_or_else(|| fail("save needs a FILE.nw"));
            let out = target(&a, file);
            let doc = open(file, a.profile);
            doc.save(&out).unwrap_or_else(|e| fail(e));
            println!(
                "wrote {} ({} bytes, unchanged level)",
                out.display(),
                doc.to_bytes().len()
            );
        }
        Some(file) if file.ends_with(".nw") || Path::new(file).exists() => {
            list(file, a.rows, a.profile)
        }
        Some(other) => fail(format!("unknown command \"{other}\"\n{USAGE}")),
    }
}
