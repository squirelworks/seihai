//! Tests of the second editor slice: layers 0 and 1, object records (links,
//! signs, chests, NPCs), stamps from a GOBJSET01 set, and the gmap grid, on
//! the document, on the editor state, and through the window driven headless.

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{Editor, EditorApp, Sheet, Tool};
use level_editor::{tile_code, Document, EditError, ObjectEdit, ObjectKind, Version, SIZE};
use seihai::assets::Level;
use std::path::{Path, PathBuf};

fn starter(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default/levels").join(name)
}

fn scratch_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seihai-level-editor-slice2-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn scratch(name: &str) -> PathBuf {
    let p = scratch_dir().join(name);
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

/// A level with layer-1 runs, a layer-2 row and objects of every kind.
fn layered() -> String {
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code(5).repeat(64)));
    }
    s.push_str(&format!("BOARD 3 4 2 1 {}{}\n", code(100), code(101)));
    s.push_str(&format!("BOARD 60 63 4 1 {}\n", code(200).repeat(4)));
    s.push_str(&format!("BOARD 10 10 1 2 {}\n", code(300)));
    s.push_str("LINK other.nw 0 20 1 4 playerx 30\n");
    s.push_str("NPC tree.png 12.5 30\nif (created) {\n  setshape 1,32,32;\n}\nNPCEND\n");
    s.push_str("SIGN 3 4\nhello\nSIGNEND\nCHEST 5 6 bombs -1\n");
    s.push_str("BADDY 8 9 0\none\ntwo\nthree\nBADDYEND\n");
    s
}

#[test]
fn layers_are_read_apart_and_the_222_view_is_kept() {
    let doc = Document::parse(layered().as_bytes()).unwrap();
    assert_eq!(doc.layer_tile(0, 3, 4), Some(5));
    assert_eq!(doc.layer_tile(1, 3, 4), Some(100));
    assert_eq!(doc.layer_tile(1, 4, 4), Some(101));
    assert_eq!(doc.layer_tile(1, 5, 4), None, "layer 1 starts empty");
    assert_eq!(doc.layer_tile(1, 63, 63), Some(200));
    assert_eq!(doc.layer1_count(), 6);
    // the 2.22 view: every row on one board
    assert_eq!(doc.tile(3, 4), Some(100));
    assert_eq!(doc.tile(10, 10), Some(300), "the layer 2 row is placed in the 2.22 view");
    assert_eq!(doc.layer_tile(0, 10, 10), Some(5));
    assert_eq!(doc.summary().rows_other_layers_kept, 1);
    assert_eq!(doc.to_bytes(), layered().as_bytes(), "an untouched layered level round trips");
}

#[test]
fn both_layers_are_saved_and_layer_1_is_written_in_runs() {
    let mut doc = Document::parse(layered().as_bytes()).unwrap();
    assert_eq!(doc.paint_layer(1, 20, 7, 3, 1, Some(77)).unwrap(), 3);
    assert_eq!(doc.paint_layer(1, 4, 4, 1, 1, None).unwrap(), 1, "erase one layer 1 cell");
    assert_eq!(doc.paint_layer(1, 4, 4, 1, 1, None).unwrap(), 0);
    assert_eq!(doc.paint_layer(0, 0, 0, 1, 1, Some(9)).unwrap(), 1);
    assert!(matches!(doc.paint_layer(0, 0, 0, 1, 1, None), Err(EditError::Bounds(_))), "layer 0 has no empty cells");
    assert!(matches!(doc.paint_layer(2, 0, 0, 1, 1, Some(1)), Err(EditError::Bounds(_))));
    assert_eq!(doc.tile(20, 7), Some(77), "the 2.22 view follows");
    assert_eq!(doc.tile(4, 4), Some(5), "layer 0 shows through the erased cell");

    let bytes = doc.to_bytes();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(text.contains(&format!("\nBOARD 3 4 1 1 {}\n", code(100))));
    assert!(text.contains(&format!("\nBOARD 20 7 3 1 {}\n", code(77).repeat(3))));
    assert!(text.contains(&format!("\nBOARD 60 63 4 1 {}\n", code(200).repeat(4))));
    assert!(text.contains(&format!("\nBOARD 10 10 1 2 {}\n", code(300))), "the layer 2 row is kept as it was");
    assert!(!text.contains("BOARD 4 4"), "an empty cell is left out");

    let again = Document::parse(&bytes).unwrap();
    for l in 0..2 {
        for y in 0..SIZE {
            for x in 0..SIZE {
                assert_eq!(again.layer_tile(l, x, y), doc.layer_tile(l, x, y), "layer {l} ({x}, {y})");
            }
        }
    }
    assert_eq!(again.tile(10, 10), Some(300));
    // the client reads both layers back
    let level = Level::parse(&text).unwrap();
    assert!(level.boards.iter().any(|b| b.layer == 1 && b.x == 20 && b.y == 7 && b.width == 3));
    assert_eq!(level.layer0_board()[0], 9);
}

#[test]
fn objects_are_read_with_the_client_parser() {
    let doc = Document::open(&starter("new1.nw")).unwrap();
    let objs = doc.objects();
    let level = Level::load(starter("new1.nw")).unwrap();
    assert_eq!(objs.iter().filter(|o| o.kind == ObjectKind::Link).count(), level.links.len());
    assert_eq!(objs.iter().filter(|o| o.kind == ObjectKind::Sign).count(), level.signs.len());
    assert_eq!(objs.iter().filter(|o| o.kind == ObjectKind::Chest).count(), level.chests.len());
    let link = &objs[0];
    assert_eq!((link.kind, link.x, link.y, link.w, link.h, link.name.as_str()), (ObjectKind::Link, 41.0, 20.0, 2.0, 2.0, "new2.nw"));
    assert_eq!(link.arrival, ("31".to_string(), "47".to_string()));
    let sign = objs.iter().find(|o| o.kind == ObjectKind::Sign).unwrap();
    assert_eq!(sign.first_line, "Welcome to Seihai!");

    let doc = Document::parse(layered().as_bytes()).unwrap();
    let kinds: Vec<ObjectKind> = doc.objects().iter().map(|o| o.kind).collect();
    assert_eq!(kinds, [ObjectKind::Link, ObjectKind::Npc, ObjectKind::Sign, ObjectKind::Chest, ObjectKind::Baddy]);
    let npc = doc.object(1).unwrap();
    assert_eq!((npc.x, npc.y, npc.name.as_str(), npc.first_line.as_str()), (12.5, 30.0, "tree.png", "if (created) {"));
}

/// The lines of `a` and `b` that differ, as (line number, a, b); same line count assumed.
fn changed_lines(a: &str, b: &str) -> Vec<(usize, String, String)> {
    a.lines().zip(b.lines()).enumerate().filter(|(_, (x, y))| x != y).map(|(i, (x, y))| (i, x.to_string(), y.to_string())).collect()
}

#[test]
fn moving_and_editing_rewrites_only_that_record() {
    let original = std::fs::read_to_string(starter("new1.nw")).unwrap();
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    doc.edit_object(0, ObjectEdit::Move(40.4, 22.0)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert_eq!(changed_lines(&original, &after), vec![(65, "LINK new2.nw 41 20 2 2 31 47".into(), "LINK new2.nw 40 22 2 2 31 47".into())], "whole tiles for a link");
    assert!(doc.is_modified());

    doc.edit_object(0, ObjectEdit::LinkArrival("playerx".into(), "playery+10".into())).unwrap();
    doc.edit_object(0, ObjectEdit::LinkDest("cave.nw".into())).unwrap();
    doc.edit_object(0, ObjectEdit::LinkSize(3, 2)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert_eq!(after.lines().nth(65), Some("LINK cave.nw 40 22 3 2 playerx playery+10"));
    let link = &Level::parse(&after).unwrap().links[0];
    assert_eq!((link.new_x, link.new_y, link.y_offset), (None, None, 10.0), "the client reads the new arrival");

    assert!(matches!(doc.edit_object(0, ObjectEdit::Move(63.0, 0.0)), Err(EditError::Bounds(_))), "a 3-wide link at x 63 leaves the board");
    assert!(matches!(doc.edit_object(0, ObjectEdit::LinkArrival("".into(), "1".into())), Err(EditError::Bounds(_))));
    assert!(matches!(doc.edit_object(0, ObjectEdit::LinkDest("two words".into())), Err(EditError::Bounds(_))));
    assert!(matches!(doc.edit_object(0, ObjectEdit::FirstLine("x".into())), Err(EditError::Bounds(_))), "links have no text");

    // the sign's first line; the other lines stay
    let sign = doc.objects().iter().find(|o| o.kind == ObjectKind::Sign).unwrap().id;
    doc.edit_object(sign, ObjectEdit::FirstLine("Hello again!".into())).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("SIGN 26 28\nHello again!\nArrow keys walk."));
    assert!(matches!(doc.edit_object(sign, ObjectEdit::FirstLine("a\nb".into())), Err(EditError::Bounds(_))));
    // the board was not rewritten for any of this
    assert_eq!(after.lines().filter(|l| l.starts_with("BOARD")).count(), original.lines().filter(|l| l.starts_with("BOARD")).count());
    assert_eq!(after.lines().filter(|l| l.starts_with("BOARD")).collect::<Vec<_>>(), original.lines().filter(|l| l.starts_with("BOARD")).collect::<Vec<_>>());
}

#[test]
fn npc_moves_keep_their_fraction_and_the_first_script_line_is_edited() {
    let mut doc = Document::parse(layered().as_bytes()).unwrap();
    doc.edit_object(1, ObjectEdit::Move(14.5, 31.0)).unwrap();
    doc.edit_object(1, ObjectEdit::FirstLine("if (playerenters) {".into())).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("NPC tree.png 14.5 31\nif (playerenters) {\n  setshape 1,32,32;\n}\nNPCEND\n"));
    let npc = &Level::parse(&after).unwrap().npcs[0];
    assert_eq!((npc.x, npc.y), (14.5, 31.0));
    assert!(matches!(doc.edit_object(4, ObjectEdit::Move(1.0, 1.0)), Err(EditError::Bounds(_))), "baddies are not edited");
    // a CRLF file keeps CRLF on the edited line
    let crlf = layered().replace('\n', "\r\n");
    let mut doc = Document::parse(crlf.as_bytes()).unwrap();
    doc.edit_object(3, ObjectEdit::Move(6.0, 7.0)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("\r\nCHEST 6 7 bombs -1\r\n"));
    assert!(!after.replace("\r\n", "").contains('\n'));
}

#[test]
fn objects_are_added_and_deleted_and_the_client_reads_them() {
    let mut doc = Document::open(&starter("new2.nw")).unwrap();
    let before = doc.objects().len();
    let l = doc.add_object(ObjectKind::Link, 63, 63).unwrap();
    let s = doc.add_object(ObjectKind::Sign, 10, 11).unwrap();
    let c = doc.add_object(ObjectKind::Chest, 12, 13).unwrap();
    let n = doc.add_object(ObjectKind::Npc, 14, 15).unwrap();
    assert_eq!((l, s, c, n), (before, before + 1, before + 2, before + 3));
    assert!(matches!(doc.add_object(ObjectKind::Baddy, 1, 1), Err(EditError::Bounds(_))));
    assert!(matches!(doc.add_object(ObjectKind::Chest, 64, 1), Err(EditError::Bounds(_))));
    doc.edit_object(n, ObjectEdit::FirstLine("setimg tree.png;".into())).unwrap();
    let text = String::from_utf8(doc.to_bytes()).unwrap();
    let level = Level::parse(&text).unwrap();
    let orig = Level::load(starter("new2.nw")).unwrap();
    assert_eq!(level.links.len(), orig.links.len() + 1);
    assert_eq!(level.signs.len(), orig.signs.len() + 1);
    assert_eq!(level.chests.len(), orig.chests.len() + 1);
    assert_eq!(level.npcs.len(), 1);
    assert_eq!(level.npcs[0].script, "setimg tree.png;");
    let new_link = level.links.last().unwrap();
    assert_eq!((new_link.x, new_link.y, new_link.width, new_link.height), (63, 63, 1, 1), "clipped to the board");

    doc.delete_object(c).unwrap();
    doc.delete_object(l).unwrap();
    let text = String::from_utf8(doc.to_bytes()).unwrap();
    let level = Level::parse(&text).unwrap();
    assert_eq!((level.links.len(), level.chests.len()), (orig.links.len(), orig.chests.len()));
    assert!(text.starts_with(&std::fs::read_to_string(starter("new2.nw")).unwrap()), "the original lines are untouched; additions go at the end");
}

#[test]
fn paint_then_edit_an_object_keeps_both() {
    let mut doc = Document::open(&starter("new1.nw")).unwrap();
    doc.paint_layer(1, 1, 1, 2, 2, Some(42)).unwrap();
    doc.edit_object(0, ObjectEdit::Move(10.0, 10.0)).unwrap();
    assert_eq!(doc.layer_tile(1, 2, 2), Some(42));
    let again = Document::parse(&doc.to_bytes()).unwrap();
    assert_eq!(again.layer_tile(1, 2, 2), Some(42));
    assert_eq!((again.object(0).unwrap().x, again.object(0).unwrap().y), (10.0, 10.0));
}

fn stamp_set() -> PathBuf {
    let p = scratch("stamps.txt");
    let row = |a: u16, b: u16, c: u16| format!("{}{}{}\n", code(a), code(b), code(c));
    let text = format!("GOBJSET01\n\nOBJECT 3 2 Rock\n{}{}OBJECTEND\nOBJECT 1 1 Dot\n{}OBJECTEND\n", row(1, 2, 3), row(4, 5, 6), code(9) + "\n");
    std::fs::write(&p, text).unwrap();
    p
}

#[test]
fn stamps_load_and_drop_on_the_active_layer_clipped_to_the_board() {
    let path = copy_of("new1.nw", "stamp.nw");
    let mut ed = Editor::new(Version::V222);
    ed.open(&path).unwrap();
    assert!(ed.place_stamp(0, 0).is_err(), "no set loaded");
    assert_eq!(ed.load_stamps(&stamp_set()).unwrap(), 2);
    assert_eq!(ed.current_stamp().unwrap().name, "Rock");
    ed.layer = 1;
    assert_eq!(ed.place_stamp(10, 20).unwrap(), 6);
    let doc = ed.doc().unwrap();
    assert_eq!(doc.layer_tile(1, 10, 20), Some(1));
    assert_eq!(doc.layer_tile(1, 12, 21), Some(6));
    assert_eq!(doc.layer_tile(1, 13, 20), None);
    // at the corner only the cell on the board is placed
    assert_eq!(ed.place_stamp(63, 63).unwrap(), 1);
    assert_eq!(ed.doc().unwrap().layer_tile(1, 63, 63), Some(1));
    ed.layer = 0;
    ed.stamp = Some(1);
    assert_eq!(ed.place_stamp(0, 0).unwrap(), 1);
    assert_eq!(ed.doc().unwrap().layer_tile(0, 0, 0), Some(9));
    assert!(ed.has_unsaved());
    let bad = scratch("bad-stamps.txt");
    std::fs::write(&bad, "nope").unwrap();
    assert!(matches!(ed.load_stamps(&bad), Err(EditError::NotText(_))));
}

#[test]
fn erase_and_pick_follow_the_active_layer() {
    let path = copy_of("new2.nw", "erase.nw");
    let mut ed = Editor::new(Version::V222);
    ed.open(&path).unwrap();
    assert!(matches!(ed.erase_at(0, 0), Err(EditError::Bounds(_))), "layer 0 cannot be erased");
    ed.layer = 1;
    ed.select(321).unwrap();
    ed.paint_at(5, 5).unwrap();
    assert_eq!(ed.doc().unwrap().layer_tile(0, 5, 5), Document::open(&path).unwrap().layer_tile(0, 5, 5), "layer 0 is untouched");
    ed.select(0).unwrap();
    assert_eq!(ed.pick_at(5, 5), Some(321));
    assert_eq!(ed.pick_at(6, 5), None, "an empty layer 1 cell picks nothing");
    assert_eq!(ed.erase_at(5, 5).unwrap(), 1);
    assert_eq!(ed.doc().unwrap().layer_tile(1, 5, 5), None);
}

/// A 2 x 2 gmap next to copies of the starter levels; one cell names a missing file.
fn gmap() -> PathBuf {
    let dir = scratch_dir().join("world");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(starter("new1.nw"), dir.join("w-a.nw")).unwrap();
    std::fs::copy(starter("new2.nw"), dir.join("w-b.nw")).unwrap();
    let p = dir.join("world.gmap");
    std::fs::write(&p, "GRMAP001\nWIDTH 2\nHEIGHT 2\nLEVELNAMES\n\"w-a.nw\",\"w-b.nw\"\n\"missing.nw\",\"\"\nLEVELNAMESEND\n").unwrap();
    p
}

#[test]
fn gmap_cells_name_files_next_to_the_gmap() {
    let mut ed = Editor::new(Version::V222);
    let bad = scratch("bad.gmap");
    std::fs::write(&bad, "not a gmap").unwrap();
    assert!(matches!(ed.open_gmap(&bad), Err(EditError::NotText(_))));
    let g = gmap();
    ed.open_gmap(&g).unwrap();
    let view = ed.gmap.as_ref().unwrap();
    assert_eq!((view.map.width, view.map.height), (2, 2));
    assert_eq!(view.level_path(1, 0), Some(g.parent().unwrap().join("w-b.nw")));
    assert_eq!(view.level_path(1, 1), None, "a hole");
    ed.open(&view.level_path(0, 0).unwrap()).unwrap();
    assert!(ed.gmap.is_some(), "opening a level keeps the gmap");
}

// ---- the window, driven headless ----

struct Harness {
    ctx: egui::Context,
    app: EditorApp,
    events: Vec<Event>,
}

impl Harness {
    fn new(file: Option<&Path>) -> Harness {
        let mut h = Harness { ctx: egui::Context::default(), app: EditorApp::new(Version::V222, Sheet::builtin(), file), events: Vec::new() };
        h.frame();
        h.frame();
        h
    }

    fn frame(&mut self) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 1100.0))),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        };
        let app = &mut self.app;
        let _ = self.ctx.run(input, |ctx| app.ui(ctx));
    }

    fn press(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        self.frame();
    }

    fn move_to(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.frame();
    }

    fn release(&mut self, pos: Pos2) {
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        self.frame();
    }

    fn click(&mut self, pos: Pos2) {
        self.press(pos);
        self.release(pos);
        self.frame();
    }

    /// Screen position of board point (x, y) in tiles (zoom 1, not scrolled).
    fn board_at(&self, x: f32, y: f32) -> Pos2 {
        let r = self.app.board_rect.expect("board drawn");
        r.min + Vec2::new(x * 16.0, y * 16.0)
    }
}

#[test]
fn window_paints_on_layer_1_and_saves_both_layers() {
    let path = copy_of("new1.nw", "win-layer.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.layer = 1;
    h.app.editor.select(555).unwrap();
    h.frame();
    let p = h.board_at(7.5, 8.5);
    h.click(p);
    let doc = h.app.editor.doc().unwrap();
    assert_eq!(doc.layer_tile(1, 7, 8), Some(555));
    assert_eq!(doc.layer_tile(0, 7, 8), Document::open(&starter("new1.nw")).unwrap().layer_tile(0, 7, 8));
    h.app.editor.save().unwrap();
    let saved = Document::open(&path).unwrap();
    assert_eq!(saved.layer_tile(1, 7, 8), Some(555));
    assert!(std::fs::read_to_string(&path).unwrap().contains(&format!("BOARD 7 8 1 1 {}", code(555))));
}

#[test]
fn window_stamp_tool_drops_the_stamp_where_clicked() {
    let path = copy_of("new1.nw", "win-stamp.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.load_stamps(&stamp_set()).unwrap();
    h.app.editor.tool = Tool::Stamp;
    h.frame();
    let p = h.board_at(20.5, 25.5);
    h.click(p);
    let doc = h.app.editor.doc().unwrap();
    assert_eq!(doc.layer_tile(0, 20, 25), Some(1));
    assert_eq!(doc.layer_tile(0, 22, 26), Some(6));
}

#[test]
fn window_objects_tool_drags_a_link_and_adds_at_the_clicked_cell() {
    let path = copy_of("new1.nw", "win-objects.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.tool = Tool::Objects;
    h.frame();
    // the first link is 2 x 2 at (41, 20); grab it in its middle and drag 3 right, 2 down
    let (a, b) = (h.board_at(42.0, 21.0), h.board_at(45.0, 23.0));
    h.press(a);
    assert_eq!(h.app.editor.selected, Some(0));
    h.move_to(b);
    h.release(b);
    let link = h.app.editor.doc().unwrap().object(0).unwrap();
    assert_eq!((link.x, link.y), (44.0, 22.0));
    assert!(h.app.editor.has_unsaved());
    // a click on an empty cell deselects and sets where new objects go
    let e = h.board_at(5.5, 6.5);
    h.click(e);
    assert_eq!(h.app.editor.selected, None);
    let id = h.app.editor.add_object(ObjectKind::Npc, 5, 6).unwrap();
    assert_eq!(h.app.editor.doc().unwrap().object(id).unwrap().kind, ObjectKind::Npc);
}

#[test]
fn window_gmap_grid_opens_the_clicked_level() {
    let g = gmap();
    let mut h = Harness::new(Some(&g));
    assert!(h.app.map_view);
    assert!(h.app.editor.doc().is_none());
    let (_, rect) = *h.app.gmap_cells.iter().find(|(c, _)| *c == (1, 0)).expect("cell drawn");
    h.click(rect.center());
    assert_eq!(h.app.editor.path(), Some(g.parent().unwrap().join("w-b.nw").as_path()));
    assert!(!h.app.map_view, "the opened level is shown");
    // the missing cell is drawn but does nothing
    h.app.map_view = true;
    h.frame();
    h.frame();
    let (_, rect) = *h.app.gmap_cells.iter().find(|(c, _)| *c == (0, 1)).expect("cell drawn");
    h.click(rect.center());
    assert_eq!(h.app.editor.path(), Some(g.parent().unwrap().join("w-b.nw").as_path()));
}

#[test]
fn cli_paints_layer_1_and_lists_it() {
    let input = copy_of("new1.nw", "cli-layer.nw");
    let out = scratch("cli-layer-out.nw");
    let exe = env!("CARGO_BIN_EXE_level-editor");
    let r = std::process::Command::new(exe)
        .args(["paint", input.to_str().unwrap(), "2", "3", "4", "1", "77", "--layer", "1", "--out", out.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(String::from_utf8_lossy(&r.stdout).contains("on layer 1"));
    let doc = Document::open(&out).unwrap();
    assert_eq!(doc.layer_tile(1, 3, 3), Some(77));
    assert_eq!(doc.layer_tile(0, 3, 3), Document::open(&input).unwrap().layer_tile(0, 3, 3));
    let r = std::process::Command::new(exe).args(["list", out.to_str().unwrap()]).output().unwrap();
    assert!(String::from_utf8_lossy(&r.stdout).contains("layer 1 cells filled 4"));
    let r = std::process::Command::new(exe).args(["paint", input.to_str().unwrap(), "0", "0", "1", "1", "1", "--layer", "2", "--in-place"]).output().unwrap();
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("--layer must be 0 or 1"));
}
