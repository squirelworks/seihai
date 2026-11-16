//! Selection, copy, cut, paste, delete and floating paste on the active layer.
//! Selecting alone must not change the file bytes.

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{Editor, EditorApp, Sheet, Tool};
use level_editor::{tile_code, Document, TileRect, VersionProfile, SIZE};
use std::path::{Path, PathBuf};

fn starter(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/default/levels")
        .join(name)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "seihai-level-editor-selection-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    let _ = std::fs::remove_file(&p);
    p
}

fn copy_of(name: &str, as_name: &str) -> PathBuf {
    let p = scratch(as_name);
    std::fs::copy(starter(name), &p).unwrap();
    p
}

fn code(id: u16) -> String {
    String::from_utf8(tile_code(id).to_vec()).unwrap()
}

fn layered() -> String {
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code(5).repeat(64)));
    }
    s.push_str(&format!("BOARD 3 4 2 1 {}{}\n", code(100), code(101)));
    s.push_str(&format!("BOARD 10 10 3 1 {}\n", code(200).repeat(3)));
    s
}

#[test]
fn selecting_alone_keeps_starter_bytes() {
    let original = std::fs::read(starter("new1.nw")).unwrap();
    let mut ed = Editor::new(VersionProfile::TextNw222);
    ed.open(&starter("new1.nw")).unwrap();
    ed.set_selection(TileRect::new(2, 3, 4, 5).unwrap());
    assert!(!ed.has_unsaved());
    assert_eq!(ed.doc().unwrap().to_bytes(), original);
    ed.copy_selection().unwrap();
    assert!(ed.floating.is_some());
    assert!(!ed.has_unsaved(), "copy does not write the board");
    assert_eq!(ed.doc().unwrap().to_bytes(), original);
    ed.cancel_floating();
    assert_eq!(ed.doc().unwrap().to_bytes(), original);
}

#[test]
fn copy_paste_moves_a_block_on_layer_0() {
    let path = copy_of("new1.nw", "paste0.nw");
    let original = std::fs::read(&path).unwrap();
    let mut ed = Editor::new(VersionProfile::TextNw222);
    ed.open(&path).unwrap();
    ed.layer = 0;
    // paint a known pattern, then copy it
    ed.select(111).unwrap();
    for (x, y) in [(10, 10), (11, 10), (10, 11)] {
        ed.paint_at(x, y).unwrap();
    }
    ed.set_selection(TileRect::new(10, 10, 2, 2).unwrap());
    ed.copy_selection().unwrap();
    let clip = ed.clipboard.as_ref().unwrap();
    assert_eq!(clip.w, 2);
    assert_eq!(clip.h, 2);
    assert_eq!(clip.tiles[0], Some(111));
    assert_eq!(clip.tiles[1], Some(111));
    assert_eq!(clip.tiles[2], Some(111), "row-major: (10,11) is index 2");
    // the fourth cell was not painted; it still holds the starter tile
    let old = Document::parse(&original).unwrap().tile(11, 11).unwrap();
    assert_eq!(clip.tiles[3], Some(old));

    ed.paste_at(20, 20).unwrap();
    assert_eq!(ed.drop_floating().unwrap(), 4);
    let doc = ed.doc().unwrap();
    assert_eq!(doc.layer_tile(0, 20, 20), Some(111));
    assert_eq!(doc.layer_tile(0, 21, 20), Some(111));
    assert_eq!(doc.layer_tile(0, 20, 21), Some(111));
    assert_eq!(doc.layer_tile(0, 21, 21), Some(old));
    // source untouched by copy+paste
    assert_eq!(doc.layer_tile(0, 10, 10), Some(111));
}

#[test]
fn cut_clears_source_and_delete_clears_without_clipboard() {
    let doc = Document::parse(layered().as_bytes()).unwrap();
    let rect = TileRect::new(3, 4, 2, 1).unwrap();
    let block = doc.read_layer_block(1, rect).unwrap();
    assert_eq!(block.tiles, vec![Some(100), Some(101)]);

    let mut ed = Editor::new(VersionProfile::TextNw222);
    let path = scratch("cut.nw");
    std::fs::write(&path, layered()).unwrap();
    ed.open(&path).unwrap();
    ed.layer = 1;
    ed.set_selection(rect);
    assert_eq!(ed.cut_selection().unwrap(), 2);
    assert!(ed.has_unsaved());
    assert_eq!(ed.doc().unwrap().layer_tile(1, 3, 4), None);
    assert_eq!(ed.doc().unwrap().layer_tile(1, 4, 4), None);
    assert!(ed.floating.is_some());
    // drop elsewhere
    ed.move_floating(30, 30);
    assert_eq!(ed.drop_floating().unwrap(), 2);
    assert_eq!(ed.doc().unwrap().layer_tile(1, 30, 30), Some(100));
    assert_eq!(ed.doc().unwrap().layer_tile(1, 31, 30), Some(101));

    // delete clears without a clipboard write of the current selection
    ed.set_selection(TileRect::new(10, 10, 3, 1).unwrap());
    let before_clip = ed.clipboard.clone();
    assert_eq!(ed.delete_selection().unwrap(), 3);
    assert_eq!(ed.clipboard, before_clip, "delete leaves the clipboard");
    assert_eq!(ed.doc().unwrap().layer_tile(1, 10, 10), None);
    assert_eq!(ed.doc().unwrap().layer1_count(), 2);
}

#[test]
fn paste_clips_at_the_board_edge() {
    let mut ed = Editor::new(VersionProfile::TextNw222);
    ed.open(&starter("new2.nw")).unwrap();
    ed.layer = 0;
    ed.select(7).unwrap();
    ed.paint_at(0, 0).unwrap();
    ed.paint_at(1, 0).unwrap();
    ed.paint_at(0, 1).unwrap();
    ed.paint_at(1, 1).unwrap();
    ed.set_selection(TileRect::new(0, 0, 2, 2).unwrap());
    ed.copy_selection().unwrap();
    ed.cancel_floating();
    ed.paste_at(63, 63).unwrap();
    let f = ed.floating.as_ref().unwrap();
    assert_eq!((f.x, f.y), (62, 62), "clamped so a 2 x 2 fits");
    let n = ed.drop_floating().unwrap();
    assert!(n >= 1);
    assert_eq!(ed.doc().unwrap().layer_tile(0, 62, 62), Some(7));
}

#[test]
fn cut_then_undo_by_paste_back_restores_starter_bytes() {
    // starter is canonical; cut a 1x1 and paste it back in place
    let path = copy_of("new1.nw", "roundtrip.nw");
    let original = std::fs::read(&path).unwrap();
    let mut ed = Editor::new(VersionProfile::TextNw222);
    ed.open(&path).unwrap();
    let old = ed.doc().unwrap().tile(8, 8).unwrap();
    ed.set_selection(TileRect::new(8, 8, 1, 1).unwrap());
    ed.cut_selection().unwrap();
    assert_eq!(ed.doc().unwrap().layer_tile(0, 8, 8), Some(0));
    ed.move_floating(8, 8);
    ed.drop_floating().unwrap();
    assert_eq!(ed.doc().unwrap().layer_tile(0, 8, 8), Some(old));
    // after the cut the board was rewritten to canonical rows; putting the
    // old tile back on a starter yields the starter bytes again
    assert_eq!(ed.doc().unwrap().to_bytes(), original);
}

#[test]
fn rectangle_helpers_refuse_empty_and_off_board() {
    assert!(TileRect::new(0, 0, 0, 1).is_err());
    assert!(TileRect::new(60, 0, 5, 1).is_err());
    assert_eq!(
        TileRect::from_corners((5, 7), (3, 4)).unwrap(),
        TileRect::new(3, 4, 3, 4).unwrap()
    );
}

// ---- window ----

struct Harness {
    ctx: egui::Context,
    app: EditorApp,
    events: Vec<Event>,
}

impl Harness {
    fn new(file: Option<&Path>) -> Harness {
        let mut h = Harness {
            ctx: egui::Context::default(),
            app: EditorApp::new(VersionProfile::TextNw222, Sheet::builtin(), file),
            events: Vec::new(),
        };
        h.frame();
        h.frame();
        h
    }

    fn frame(&mut self) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1100.0, 860.0))),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        };
        let app = &mut self.app;
        let _ = self.ctx.run(input, |ctx| app.ui(ctx));
    }

    fn press(&mut self, pos: Pos2, button: PointerButton) {
        self.events.push(Event::PointerMoved(pos));
        self.events.push(Event::PointerButton {
            pos,
            button,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        self.frame();
    }

    fn move_to(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.frame();
    }

    fn release(&mut self, pos: Pos2, button: PointerButton) {
        self.events.push(Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        self.frame();
    }

    fn board_at(&self, x: usize, y: usize) -> Pos2 {
        let r = self.app.board_rect.expect("board drawn");
        r.min + Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
    }

    fn key(&mut self, key: egui::Key, modifiers: Modifiers) {
        self.events.push(Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        self.frame();
    }
}

#[test]
fn window_select_drag_copy_paste_with_keys() {
    let path = copy_of("new1.nw", "win-sel.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.tool = Tool::Select;
    h.app.editor.select(42).unwrap();
    // paint two cells with the paint API, then select them in the window
    h.app.editor.tool = Tool::Paint;
    h.app.editor.paint_at(4, 5).unwrap();
    h.app.editor.paint_at(5, 5).unwrap();
    h.app.editor.tool = Tool::Select;

    let a = h.board_at(4, 5);
    let b = h.board_at(5, 5);
    h.press(a, PointerButton::Primary);
    h.move_to(b);
    h.release(b, PointerButton::Primary);
    let sel = h.app.editor.selection.expect("selection after drag");
    assert_eq!((sel.x, sel.y, sel.w, sel.h), (4, 5, 2, 1));

    h.key(egui::Key::C, Modifiers::COMMAND);
    assert!(h.app.editor.floating.is_some());
    assert!(h.app.editor.clipboard.is_some());

    // move the float and drop with a click
    let dest = h.board_at(14, 15);
    h.move_to(dest);
    h.press(dest, PointerButton::Primary);
    h.release(dest, PointerButton::Primary);
    assert!(h.app.editor.floating.is_none());
    assert_eq!(h.app.editor.doc().unwrap().layer_tile(0, 14, 15), Some(42));
    assert_eq!(h.app.editor.doc().unwrap().layer_tile(0, 15, 15), Some(42));
}
