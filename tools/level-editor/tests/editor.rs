//! Tests of the level editor's first slice: round trips, painting, bounds,
//! versions and the command line. The levels are the starter levels of the
//! default asset set and small synthetic ones; no corpus level is used.

use level_editor::{tile_code, Document, EditError, Version, SIZE};
use std::path::{Path, PathBuf};
use std::process::Command;

fn starter(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default/levels").join(name)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seihai-level-editor-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn row(y: usize, ids: &[u16]) -> String {
    let data: String = ids.iter().map(|&t| String::from_utf8(tile_code(t).to_vec()).unwrap()).collect();
    format!("BOARD 0 {y} {} 0 {data}\n", ids.len())
}

/// A small level that uses every awkward case of the format: comments, a
/// BOARD-looking line inside an NPC script, HEIGHTS, links, signs, chests,
/// baddies, partial and skipped rows, a row with a layer, unordered rows.
fn synthetic() -> String {
    let mut s = String::from("GLEVNW01\n// a comment\n");
    s.push_str(&row(3, &[1; 64]));
    s.push_str("BOARD 10 2 4 0 AEAFAGAH\n"); // partial row at x = 10
    s.push_str("BOARD 62 5 4 0 AAAAAAAA\n"); // x + w > 64: skipped
    s.push_str("BOARD 0 70 2 0 AAAA\n"); // y out of range: skipped
    s.push_str("BOARD 0 6 3 0 AAAA\n"); // too short: skipped
    s.push_str("BOARD 4 7 3 1 BAB!BC\n"); // layer 1 is still placed; '!' ends the row
    s.push_str("LINK other.nw 63 20 1 2 playerx playery\n");
    s.push_str("NPC image.png 12.5 30\n// inside a script:\nBOARD 0 0 64 0 not-a-row\nsetimg x.png;\nNPCEND\n");
    s.push_str("SIGN 3 4\nhello\nSIGNEND\nCHEST 5 6 1 -1\n");
    s.push_str("BADDY 8 9 0\none\ntwo\nthree\nBADDYEND\n");
    s.push_str("HEIGHTS\n0,1,2\nHEIGHTSEND\n");
    s.push_str(&row(0, &[7; 64]));
    s
}

fn non_board_lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    // BOARD lines outside a block only; NPC blocks may contain a BOARD-looking line.
    let text = String::from_utf8_lossy(bytes).into_owned();
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim();
        if inside {
            out.push(line.as_bytes().to_vec());
            inside = !t.ends_with("END");
        } else if t.starts_with("NPC ") || t.starts_with("SIGN ") || t.starts_with("BADDY ") || t == "HEIGHTS" {
            out.push(line.as_bytes().to_vec());
            inside = true;
        } else if !t.starts_with("BOARD ") {
            out.push(line.as_bytes().to_vec());
        }
    }
    out
}

#[test]
fn untouched_starter_levels_round_trip_byte_for_byte() {
    for name in ["new1.nw", "new2.nw"] {
        let original = std::fs::read(starter(name)).unwrap();
        let doc = Document::parse(&original).unwrap();
        assert!(!doc.is_modified());
        assert_eq!(doc.to_bytes(), original, "{name} through to_bytes");
        let out = scratch(&format!("rt-{name}"));
        Document::open(&starter(name)).unwrap().save(&out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), original, "{name} through save");
        let s = doc.summary();
        assert_eq!(s.rows_placed, 64, "{name} has 64 board rows");
        assert_eq!(s.rows_skipped, 0);
    }
}

#[test]
fn starter_summary_matches_what_seihai_reads() {
    let data = std::fs::read(starter("new1.nw")).unwrap();
    let doc = Document::parse(&data).unwrap();
    let level = seihai::assets::Level::parse(&String::from_utf8_lossy(&data)).unwrap();
    assert_eq!(doc.summary().npcs, level.npcs.len());
    assert_eq!(doc.summary().links, level.links.len());
    assert_eq!(doc.summary().signs, level.signs.len());
    assert_eq!(doc.summary().chests, level.chests.len());
    assert_eq!(doc.summary().baddies, level.baddies.len());
    for (x, y) in [(0, 0), (5, 2), (63, 63), (10, 40), (33, 17)] {
        assert_eq!(doc.tile(x, y), level.tile_id_at(x as i32, y as i32), "tile ({x}, {y})");
    }
}

#[test]
fn synthetic_level_round_trips_in_every_line_ending_and_ending() {
    let lf = synthetic();
    let crlf = lf.replace('\n', "\r\n");
    let no_final = lf.trim_end_matches('\n').to_string();
    for (what, text) in [("lf", lf.clone()), ("crlf", crlf), ("no final newline", no_final)] {
        let doc = Document::parse(text.as_bytes()).unwrap();
        assert_eq!(doc.to_bytes(), text.as_bytes(), "{what}");
    }
}

#[test]
fn board_rows_follow_the_text_reader_rules() {
    let doc = Document::parse(synthetic().as_bytes()).unwrap();
    let s = doc.summary();
    // placed: row 3, the partial row at y 2, the layer 1 row at y 7, the final row 0
    assert_eq!((s.rows_placed, s.rows_skipped, s.rows_other_layer), (4, 3, 1));
    assert_eq!((s.links, s.signs, s.chests, s.baddies, s.npcs, s.heights), (1, 1, 1, 1, 1, true));
    assert_eq!(doc.tile(0, 3), Some(1));
    assert_eq!(doc.tile(10, 2), Some(4)); // "AE"
    assert_eq!(doc.tile(13, 2), Some(7)); // "AH"
    assert_eq!(doc.tile(14, 2), Some(0));
    assert_eq!(doc.tile(0, 6), Some(0), "the short row was skipped");
    assert_eq!(doc.tile(62, 5), Some(0), "the row that left the board was skipped");
    assert_eq!(doc.tile(4, 7), Some(64), "layer 1 is placed like layer 0; 'BA' is 1 * 64 + 0");
    assert_eq!(doc.tile(5, 7), Some(0), "'B!' holds an invalid character and ends the row");
    assert_eq!(doc.tile(6, 7), Some(0));
    // the later row 0 overwrites nothing else; row 0 is all 7
    assert!((0..64).all(|x| doc.tile(x, 0) == Some(7)));
    // the BOARD-looking line inside the NPC script was not a board row
    assert_eq!(doc.summary().rows_skipped, 3);
}

#[test]
fn painting_changes_only_the_rectangle_and_keeps_every_other_line() {
    let original = synthetic();
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let before: Vec<Vec<u16>> = (0..SIZE).map(|y| doc.row(y).unwrap().to_vec()).collect();
    let changed = doc.paint(20, 30, 5, 4, 1234).unwrap();
    assert_eq!(changed, 20);
    assert!(doc.is_modified());
    for y in 0..SIZE {
        for x in 0..SIZE {
            let inside = (20..25).contains(&x) && (30..34).contains(&y);
            let want = if inside { 1234 } else { before[y][x] };
            assert_eq!(doc.tile(x, y), Some(want), "({x}, {y})");
        }
    }
    let bytes = doc.to_bytes();
    // everything that is not a board row is untouched, in order
    assert_eq!(non_board_lines(&bytes), non_board_lines(original.as_bytes()));
    // the saved file reads back to the same board, as 64 canonical rows
    let again = Document::parse(&bytes).unwrap();
    assert_eq!(again.summary().rows_placed, 64);
    assert_eq!(again.summary().rows_skipped, 0);
    for y in 0..SIZE {
        assert_eq!(again.row(y), doc.row(y), "row {y}");
    }
    // and they sit where the first old BOARD row was: after the comment line
    let text = String::from_utf8(bytes).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(&lines[..2], &["GLEVNW01", "// a comment"]);
    assert!(lines[2].starts_with("BOARD 0 0 64 0 AH"));
    assert!(lines[3].starts_with("BOARD 0 1 64 0 "));
}

#[test]
fn seihai_reads_what_the_editor_saved() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    doc.paint(2, 3, 10, 6, 0x5A5).unwrap();
    let out = scratch("seihai-reads.nw");
    doc.save(&out).unwrap();
    let level = seihai::assets::Level::load(&out).unwrap();
    for y in 0..64 {
        for x in 0..64 {
            let want = if (2..12).contains(&x) && (3..9).contains(&y) { 0x5A5 } else { Document::open(&starter("new1.nw")).unwrap().tile(x, y).unwrap() };
            assert_eq!(level.tile_id_at(x as i32, y as i32), Some(want), "({x}, {y})");
        }
    }
    // links, signs, chests and NPCs survive
    let orig = seihai::assets::Level::load(starter("new1.nw")).unwrap();
    assert_eq!((level.links.len(), level.signs.len(), level.chests.len(), level.npcs.len()), (orig.links.len(), orig.signs.len(), orig.chests.len(), orig.npcs.len()));
}

#[test]
fn painting_back_the_old_tiles_restores_the_starter_bytes() {
    // the starter is in the writer's canonical shape, so a change and its undo are byte-stable
    for name in ["new1.nw", "new2.nw"] {
        let original = std::fs::read(starter(name)).unwrap();
        let mut doc = Document::parse(&original).unwrap();
        let old = doc.tile(7, 9).unwrap();
        doc.paint(7, 9, 1, 1, (old as u32 + 1) % 4096).unwrap();
        assert_ne!(doc.to_bytes(), original);
        doc.paint(7, 9, 1, 1, old as u32).unwrap();
        assert!(doc.is_modified());
        assert_eq!(doc.to_bytes(), original, "{name}");
    }
}

#[test]
fn painting_the_tile_that_is_already_there_changes_nothing() {
    let original = synthetic();
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    assert_eq!(doc.paint(0, 0, 64, 1, 7).unwrap(), 0);
    assert!(!doc.is_modified());
    assert_eq!(doc.to_bytes(), original.as_bytes());
}

#[test]
fn bounds_and_id_range_are_checked_and_nothing_changes_on_error() {
    let original = synthetic();
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let cases: [(usize, usize, usize, usize, u32); 7] = [
        (60, 0, 5, 1, 1),   // x + w = 65
        (0, 60, 1, 5, 1),   // y + h = 65
        (64, 0, 1, 1, 1),   // x outside
        (0, 64, 1, 1, 1),   // y outside
        (0, 0, 0, 1, 1),    // empty
        (0, 0, 1, 0, 1),    // empty
        (0, 0, 65, 65, 1),  // too big
    ];
    for (x, y, w, h, t) in cases {
        let err = doc.paint(x, y, w, h, t).unwrap_err();
        assert!(matches!(err, EditError::Bounds(_)), "{x} {y} {w} {h}: {err}");
    }
    assert_eq!(doc.paint(0, 0, 1, 1, 4096), Err(EditError::TileRange(4096)));
    assert_eq!(doc.paint(0, 0, 1, 1, u32::MAX), Err(EditError::TileRange(u32::MAX)));
    assert!(EditError::TileRange(5000).to_string().contains("0 to 4095"));
    assert!(!doc.is_modified());
    assert_eq!(doc.to_bytes(), original.as_bytes());
    // the corners and the largest id are fine
    assert_eq!(doc.paint(63, 63, 1, 1, 4095).unwrap(), 1);
    assert_eq!(doc.paint(0, 0, 64, 64, 4095).unwrap(), 64 * 64 - 1);
}

#[test]
fn every_tile_id_survives_a_save_and_read() {
    let mut doc = Document::parse(b"GLEVNW01\n").unwrap();
    // the file has no BOARD row: the board is written right after the header
    for id in 0..4096u32 {
        doc.paint((id % 64) as usize, (id / 64) as usize, 1, 1, id).unwrap();
    }
    let bytes = doc.to_bytes();
    assert!(bytes.starts_with(b"GLEVNW01\nBOARD 0 0 64 0 AAAB"));
    let again = Document::parse(&bytes).unwrap();
    let level = seihai::assets::Level::parse(&String::from_utf8(bytes).unwrap()).unwrap();
    for id in 0..4096u32 {
        let (x, y) = ((id % 64) as usize, (id / 64) as usize);
        assert_eq!(again.tile(x, y), Some(id as u16));
        assert_eq!(level.tile_id_at(x as i32, y as i32), Some(id as u16), "seihai reads id {id}");
    }
}

#[test]
fn header_only_without_newline_gets_a_board_after_it() {
    let mut doc = Document::parse(b"GLEVNW01").unwrap();
    doc.paint(0, 0, 1, 1, 1).unwrap();
    let text = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(text.starts_with("GLEVNW01\nBOARD 0 0 64 0 AB"));
}

#[test]
fn crlf_levels_stay_crlf_after_a_paint() {
    let text = synthetic().replace('\n', "\r\n");
    let mut doc = Document::parse(text.as_bytes()).unwrap();
    doc.paint(1, 1, 2, 2, 9).unwrap();
    let out = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(out.matches('\n').count() == out.matches("\r\n").count(), "every new line ends with CRLF");
}

#[test]
fn only_text_levels_open() {
    for magic in ["GR-V1.03", "Z3-V1.04"] {
        let err = Document::parse(format!("{magic}\n...").as_bytes()).unwrap_err();
        assert!(err.to_string().contains("not yet"), "{err}");
    }
    assert!(matches!(Document::parse(b"hello\n"), Err(EditError::NotText(_))));
    assert!(matches!(Document::parse(b""), Err(EditError::NotText(_))));
    assert!(matches!(Document::open(Path::new("/nonexistent/x.nw")), Err(EditError::Io(_))));
}

#[test]
fn version_selector_accepts_222_and_217_and_says_not_yet_for_the_rest() {
    assert_eq!(Version::parse("2.22"), Ok(Version::V222));
    assert_eq!(Version::parse("2.17"), Ok(Version::V217));
    assert_eq!(Version::parse("2.17.1"), Ok(Version::V217));
    assert_eq!(Version::SUPPORTED[0], Version::V222, "2.22 comes first");
    for v in ["2.05", "2.31", "2.04", "2.00", "1.41", "2.12"] {
        let msg = Version::parse(v).unwrap_err().to_string();
        assert!(msg.contains("not yet supported") && msg.contains(v), "{v}: {msg}");
    }
    assert!(Version::parse("2.05").unwrap_err().to_string().contains("HEIGHTS"));
    assert!(Version::parse("banana").unwrap_err().to_string().contains("unknown version"));
}

// ---- the command line ----

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_level-editor")).args(args).output().unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

#[test]
fn cli_list_shows_a_summary_and_a_row_dump() {
    let f = starter("new1.nw");
    let f = f.to_str().unwrap();
    let out = run(&["list", f]);
    assert!(out.status.success());
    let t = text(&out.stdout);
    assert!(t.contains("editor version 2.22") && t.contains("BOARD rows placed 64, skipped 0"), "{t}");
    assert!(t.contains("signs 1") && t.contains("distinct ids"), "{t}");
    // no subcommand: the same as list
    assert_eq!(text(&run(&[f]).stdout), t);
    let rows = text(&run(&["list", f, "--rows"]).stdout);
    let dump: Vec<&str> = rows.lines().filter(|l| l.len() > 190).collect();
    assert_eq!(dump.len(), 64, "one line per board row");
    let doc = Document::open(&starter("new1.nw")).unwrap();
    let first: Vec<String> = doc.row(0).unwrap().iter().map(|t| format!("{t:03X}")).collect();
    assert_eq!(dump[0], format!(" 0: {}", first.join(" ")));
}

#[test]
fn cli_paint_writes_a_new_file_and_leaves_the_input_alone() {
    let input = starter("new1.nw");
    let before = std::fs::read(&input).unwrap();
    let out_path = scratch("cli-paint.nw");
    let out = run(&["paint", input.to_str().unwrap(), "4", "5", "3", "2", "0x1F0", "--out", out_path.to_str().unwrap()]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("tiles changed"));
    assert_eq!(std::fs::read(&input).unwrap(), before);
    let doc = Document::open(&out_path).unwrap();
    assert_eq!(doc.tile(4, 5), Some(0x1F0));
    assert_eq!(doc.tile(6, 6), Some(0x1F0));
    assert_ne!(doc.tile(7, 6), Some(0x1F0));
    // in place, on a copy
    let copy = scratch("cli-inplace.nw");
    std::fs::copy(&input, &copy).unwrap();
    assert!(run(&["paint", copy.to_str().unwrap(), "0", "0", "1", "1", "77", "--in-place"]).status.success());
    assert_eq!(Document::open(&copy).unwrap().tile(0, 0), Some(77));
    // no temporary file is left behind
    let leftovers = std::fs::read_dir(copy.parent().unwrap()).unwrap().filter_map(Result::ok).filter(|e| e.file_name().to_string_lossy().contains(".tmp")).count();
    assert_eq!(leftovers, 0);
}

#[test]
fn cli_save_of_an_untouched_level_is_identical() {
    let input = starter("new2.nw");
    let out_path = scratch("cli-save.nw");
    let out = run(&["save", input.to_str().unwrap(), "--out", out_path.to_str().unwrap()]);
    assert!(out.status.success());
    assert_eq!(std::fs::read(&out_path).unwrap(), std::fs::read(&input).unwrap());
}

#[test]
fn cli_errors_are_clear_and_exit_non_zero() {
    let f = starter("new1.nw");
    let f = f.to_str().unwrap();
    let o = scratch("cli-never.nw");
    let o = o.to_str().unwrap();
    let cases: [(Vec<&str>, &str); 7] = [
        (vec!["paint", f, "60", "0", "5", "1", "1", "--out", o], "leaves the 64 x 64 board"),
        (vec!["paint", f, "0", "0", "1", "1", "4096", "--out", o], "out of range"),
        (vec!["paint", f, "0", "0", "1", "1", "-3", "--out", o], "whole number"),
        (vec!["paint", f, "0", "0", "1", "1", "1"], "--out FILE or --in-place"),
        (vec!["--version", "2.05", "list", f], "not yet supported"),
        (vec!["--version", "2.31", "list", f], "not yet supported"),
        (vec!["list", "/nonexistent/level.nw"], "cannot read"),
    ];
    for (args, want) in cases {
        let out = run(&args);
        assert!(!out.status.success(), "{args:?}");
        assert!(text(&out.stderr).contains(want), "{args:?}: {}", text(&out.stderr));
    }
    assert!(!Path::new(o).exists(), "a failed paint writes nothing");
    // a supported version works and is named
    assert!(text(&run(&["--version", "2.17.1", "list", f]).stdout).contains("editor version 2.17.1"));
    assert!(text(&run(&["versions"]).stdout).starts_with("2.22\n"));
}
