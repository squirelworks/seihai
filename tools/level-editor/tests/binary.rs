//! Binary GR-V1 / Z3-V1 open, unchanged round-trip, paint and rewrite.
use level_editor::{Document, LevelProfile, SIZE};
use seihai_core::assets::{encode_binary, tile_code, BinaryLevel, BoardRun, Level};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "seihai-level-editor-binary-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn sample_text() -> String {
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} {SIZE} 0 "));
        for x in 0..SIZE {
            let id = if x == 1 && y == 2 { 7u16 } else { 0 };
            let c = level_editor::tile_code(id);
            s.push(c[0] as char);
            s.push(c[1] as char);
        }
        s.push('\n');
    }
    s.push_str("LINK other.nw 4 5 2 2 playerx playery\n");
    s.push_str("SIGN 8 8\nHi\nSIGNEND\n");
    s.push_str("CHEST 10 10 greenrupee -1\n");
    s.push_str("NPC - 12 12\nsay hi\nNPCEND\n");
    s
}

fn graal_bytes() -> Vec<u8> {
    let level = Level::parse(&sample_text()).unwrap();
    let w = encode_binary(&level, BinaryLevel::Graal);
    assert_eq!(&w.bytes[..8], b"GR-V1.03");
    w.bytes
}

fn graal_105_bytes() -> Vec<u8> {
    let mut level = Level::parse(&sample_text()).unwrap();
    level.boards.push(BoardRun {
        x: 2,
        y: 3,
        width: 2,
        layer: 1,
        tiles: vec![tile_code(100), tile_code(101)],
    });
    level.boards.push(BoardRun {
        x: 0,
        y: 0,
        width: 1,
        layer: 2,
        tiles: vec![tile_code(7)],
    });
    let w = encode_binary(&level, BinaryLevel::Graal);
    assert_eq!(&w.bytes[..8], b"GR-V1.05");
    w.bytes
}

fn zelda_bytes() -> Vec<u8> {
    let level = Level::parse(&sample_text()).unwrap();
    let w = encode_binary(&level, BinaryLevel::Zelda);
    assert_eq!(&w.bytes[..8], b"Z3-V1.04");
    w.bytes
}

#[test]
fn graal_unchanged_round_trips_byte_for_byte() {
    let original = graal_bytes();
    let doc = Document::parse(&original).unwrap();
    assert_eq!(doc.profile(), LevelProfile::Graal);
    assert!(doc.is_binary());
    assert_eq!(doc.tile(1, 2), Some(7));
    assert_eq!(doc.to_bytes(), original);
    let path = tmp("round.graal");
    doc.save(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn zelda_unchanged_round_trips_byte_for_byte() {
    let original = zelda_bytes();
    let doc = Document::parse(&original).unwrap();
    assert_eq!(doc.profile(), LevelProfile::Zelda);
    assert_eq!(doc.to_bytes(), original);
}

#[test]
fn graal_105_reads_layers_and_round_trips_unchanged() {
    let original = graal_105_bytes();
    let doc = Document::parse(&original).unwrap();
    assert_eq!(doc.profile(), LevelProfile::Graal);
    assert_eq!(doc.layer_tile(1, 2, 3), Some(100));
    assert_eq!(doc.layer_tile(1, 3, 3), Some(101));
    assert!(doc.layer_tile(1, 0, 0).is_none());
    assert_eq!(doc.to_bytes(), original);
}

#[test]
fn graal_paint_rewrites_and_core_reads_back() {
    let original = graal_bytes();
    let mut doc = Document::parse(&original).unwrap();
    assert_eq!(doc.paint(1, 2, 1, 1, 42).unwrap(), 1);
    let out = doc.to_bytes();
    assert_ne!(out, original);
    assert_eq!(&out[..8], b"GR-V1.03");
    let again = Document::parse(&out).unwrap();
    assert_eq!(again.tile(1, 2), Some(42));
    let level = Level::from_bytes(&out).unwrap();
    assert_eq!(level.layer0_board()[2 * 64 + 1], 42);
}

#[test]
fn graal_layer1_paint_writes_gr_v105() {
    let original = graal_bytes();
    let mut doc = Document::parse(&original).unwrap();
    assert_eq!(doc.paint_layer(1, 5, 6, 1, 1, Some(99)).unwrap(), 1);
    let out = doc.to_bytes();
    assert_eq!(&out[..8], b"GR-V1.05");
    let again = Document::parse(&out).unwrap();
    assert_eq!(again.layer_tile(1, 5, 6), Some(99));
    assert_eq!(again.tile(1, 2), Some(7));
}

#[test]
fn paint_old_tile_back_keeps_profile_and_is_readable() {
    let original = graal_bytes();
    let mut doc = Document::parse(&original).unwrap();
    doc.paint(1, 2, 1, 1, 42).unwrap();
    doc.paint(1, 2, 1, 1, 7).unwrap();
    let out = doc.to_bytes();
    // Re-encoded (dirty), so not byte-identical to the stock fixture, but
    // the board and objects match what the core reader expects.
    let again = Document::parse(&out).unwrap();
    assert_eq!(again.tile(1, 2), Some(7));
    assert_eq!(again.summary().links, 1);
}

#[test]
fn save_extension_picks_write_profile() {
    let original = graal_bytes();
    let doc = Document::parse(&original).unwrap();
    let nw = tmp("as.nw");
    let zelda = tmp("as.zelda");
    let graal = tmp("as.graal");
    doc.save(&nw).unwrap();
    doc.save(&zelda).unwrap();
    doc.save(&graal).unwrap();
    let nw_bytes = std::fs::read(&nw).unwrap();
    let z_bytes = std::fs::read(&zelda).unwrap();
    let g_bytes = std::fs::read(&graal).unwrap();
    assert!(nw_bytes.starts_with(b"GLEVNW01"));
    assert_eq!(&z_bytes[..8], b"Z3-V1.04");
    // Unchanged Graal → same profile path keeps original bytes.
    assert_eq!(g_bytes, original);
    let _ = std::fs::remove_file(&nw);
    let _ = std::fs::remove_file(&zelda);
    let _ = std::fs::remove_file(&graal);
}

#[test]
fn binary_objects_are_listed_but_not_edited() {
    let doc = Document::parse(&graal_bytes()).unwrap();
    let objs = doc.objects();
    assert!(objs
        .iter()
        .any(|o| o.kind == level_editor::ObjectKind::Link));
    assert!(objs
        .iter()
        .any(|o| o.kind == level_editor::ObjectKind::Sign));
    let err = doc
        .clone()
        .edit_object(0, level_editor::ObjectEdit::Move(1.0, 1.0))
        .unwrap_err();
    assert!(err.to_string().contains("not yet"), "{err}");
}

#[test]
fn profile_from_path() {
    assert_eq!(
        LevelProfile::from_path(std::path::Path::new("a.nw")),
        LevelProfile::Text
    );
    assert_eq!(
        LevelProfile::from_path(std::path::Path::new("a.zelda")),
        LevelProfile::Zelda
    );
    assert_eq!(
        LevelProfile::from_path(std::path::Path::new("a.graal")),
        LevelProfile::Graal
    );
    assert_eq!(
        LevelProfile::from_path(std::path::Path::new("a.editor")),
        LevelProfile::Graal
    );
}
