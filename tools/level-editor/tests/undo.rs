//! Undo and redo of board paints: rectangle snapshots, the 128-record cap,
//! groups for a stroke, and a full undo restoring byte-identical bytes.

use level_editor::gui::Editor;
use level_editor::{Document, Version, UNDO_LIMIT};
use std::path::{Path, PathBuf};

fn starter(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/default/levels")
        .join(name)
}

#[test]
fn undo_restores_tiles_and_byte_identical_round_trip() {
    let original = std::fs::read(starter("new1.nw")).unwrap();
    let mut doc = Document::parse(&original).unwrap();
    let old = doc.layer_tile(0, 5, 6).unwrap();
    let new = if old == 0 { 1 } else { 0 };

    assert!(!doc.can_undo());
    assert!(!doc.can_redo());
    assert_eq!(doc.paint_layer(0, 5, 6, 2, 2, Some(new)).unwrap(), 4);
    assert!(doc.is_modified());
    assert!(doc.can_undo());
    assert!(!doc.can_redo());
    assert_ne!(doc.to_bytes(), original);

    assert!(doc.undo());
    assert!(!doc.is_modified());
    assert_eq!(doc.layer_tile(0, 5, 6), Some(old));
    assert_eq!(doc.layer_tile(0, 6, 7), Some(old));
    assert_eq!(
        doc.to_bytes(),
        original,
        "full undo writes the original bytes"
    );
    assert!(!doc.can_undo());
    assert!(doc.can_redo());
}

#[test]
fn redo_reapplies_the_paint() {
    let mut doc = Document::open(&starter("new2.nw")).unwrap();
    let old = doc.layer_tile(0, 1, 1).unwrap();
    let new = (old as u32 + 3) % 4096;
    doc.paint_layer(0, 1, 1, 1, 1, Some(new)).unwrap();
    doc.undo();
    assert_eq!(doc.layer_tile(0, 1, 1), Some(old));
    assert!(doc.redo());
    assert_eq!(doc.layer_tile(0, 1, 1), Some(new as u16));
    assert!(doc.is_modified());
    assert!(doc.can_undo());
    assert!(!doc.can_redo());
}

#[test]
fn a_new_paint_clears_redo() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    doc.paint_layer(0, 0, 0, 1, 1, Some(9)).unwrap();
    doc.undo();
    assert!(doc.can_redo());
    doc.paint_layer(0, 1, 0, 1, 1, Some(8)).unwrap();
    assert!(!doc.can_redo());
    assert_eq!(doc.undo_len(), 1);
}

#[test]
fn layer_1_erase_undoes_to_empty() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    assert_eq!(doc.paint_layer(1, 4, 4, 1, 1, Some(50)).unwrap(), 1);
    assert_eq!(doc.layer_tile(1, 4, 4), Some(50));
    assert_eq!(doc.paint_layer(1, 4, 4, 1, 1, None).unwrap(), 1);
    assert_eq!(doc.layer_tile(1, 4, 4), None);
    assert!(doc.undo());
    assert_eq!(doc.layer_tile(1, 4, 4), Some(50));
    assert!(doc.undo());
    assert_eq!(doc.layer_tile(1, 4, 4), None);
    assert!(!doc.is_modified());
}

#[test]
fn undo_group_is_one_record() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    doc.begin_undo_group();
    for x in 0..5 {
        doc.paint_layer(0, x, 10, 1, 1, Some(11)).unwrap();
    }
    doc.end_undo_group();
    assert_eq!(doc.undo_len(), 1);
    assert!(doc.undo());
    for x in 0..5 {
        assert_ne!(doc.layer_tile(0, x, 10), Some(11));
    }
    assert!(!doc.is_modified());
}

#[test]
fn undo_cap_drops_the_oldest() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    for i in 0..UNDO_LIMIT + 3 {
        let x = i % 64;
        let y = (i / 64) % 64;
        doc.paint_layer(0, x, y, 1, 1, Some(((i % 4000) + 1) as u32))
            .unwrap();
    }
    assert_eq!(doc.undo_len(), UNDO_LIMIT);
}

#[test]
fn editor_stroke_undo_and_keys_path() {
    let mut ed = Editor::new(Version::V222);
    ed.open(&starter("new1.nw")).unwrap();
    let old = ed.doc().unwrap().tile(2, 3).unwrap();
    ed.select(42).unwrap();
    ed.begin_stroke();
    ed.paint_at(2, 3).unwrap();
    ed.paint_at(3, 3).unwrap();
    ed.end_stroke();
    assert_eq!(ed.doc().unwrap().undo_len(), 1);
    assert!(ed.has_unsaved());
    assert!(ed.undo());
    assert_eq!(ed.doc().unwrap().tile(2, 3), Some(old));
    assert_eq!(ed.doc().unwrap().tile(3, 3), Some(old));
    assert!(!ed.has_unsaved());
    assert!(ed.redo());
    assert_eq!(ed.doc().unwrap().tile(2, 3), Some(42));
    assert!(ed.has_unsaved());
}

#[test]
fn painting_the_same_tile_pushes_no_undo() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    let t = doc.layer_tile(0, 0, 0).unwrap() as u32;
    assert_eq!(doc.paint_layer(0, 0, 0, 1, 1, Some(t)).unwrap(), 0);
    assert!(!doc.can_undo());
    assert!(!doc.is_modified());
}
