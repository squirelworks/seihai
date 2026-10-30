//! Tests of the level editor window: the editor state on its own, the sheet
//! mapping, and the real window code driven with synthetic mouse and key
//! input on a headless egui context (no display needed).

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{board_image, line_cells, sheet_cell, sheet_tile, Editor, EditorApp, Sheet, SHEET_COLS, SHEET_ROWS};
use level_editor::{Document, EditError, Version, TILE_LIMIT};
use std::path::{Path, PathBuf};

fn starter(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/default/levels").join(name)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seihai-level-editor-gui-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    let _ = std::fs::remove_file(&p);
    p
}

/// A copy of a starter level in a scratch folder, so the test can save over it.
fn copy_of(name: &str, as_name: &str) -> PathBuf {
    let p = scratch(as_name);
    std::fs::copy(starter(name), &p).unwrap();
    p
}

#[test]
fn sheet_mapping_covers_every_text_id_once() {
    let mut seen = vec![false; (SHEET_COLS * SHEET_ROWS) as usize];
    for id in 0..TILE_LIMIT {
        let (c, r) = sheet_cell(id);
        assert!(c < SHEET_COLS && r < SHEET_ROWS, "id {id}");
        assert_eq!(sheet_tile(c, r), Some(id));
        let k = (r * SHEET_COLS + c) as usize;
        assert!(!seen[k], "two ids on one cell");
        seen[k] = true;
    }
    assert_eq!(sheet_tile(SHEET_COLS, 0), None);
    assert_eq!(sheet_tile(0, SHEET_ROWS), None);
    // the renderer's examples: 513 is block 1, column 1, row 0
    assert_eq!(sheet_cell(513), (17, 0));
}

#[test]
fn line_cells_has_no_gaps() {
    for (a, b) in [((0, 0), (5, 2)), ((10, 3), (2, 9)), ((4, 4), (4, 4)), ((63, 0), (0, 63))] {
        let cells = line_cells(a, b);
        assert_eq!(cells.first(), Some(&a));
        assert_eq!(cells.last(), Some(&b));
        for w in cells.windows(2) {
            let (dx, dy) = (w[0].0.abs_diff(w[1].0), w[0].1.abs_diff(w[1].1));
            assert!(dx <= 1 && dy <= 1 && dx + dy > 0, "{w:?}");
        }
    }
}

#[test]
fn editor_paint_pick_and_save() {
    let path = copy_of("new1.nw", "state.nw");
    let original = std::fs::read(&path).unwrap();
    let mut ed = Editor::new(Version::V222);
    assert_eq!(ed.version, Version::V222);
    assert!(matches!(ed.paint_at(0, 0), Err(EditError::Io(_))), "no level open");
    ed.open(&path).unwrap();
    assert!(!ed.has_unsaved());

    let old = ed.doc().unwrap().tile(5, 6).unwrap();
    let new = if old == 0x200 { 0x201 } else { 0x200 };
    ed.select(new as u32).unwrap();
    assert_eq!(ed.paint_at(5, 6).unwrap(), 1);
    assert_eq!(ed.paint_at(5, 6).unwrap(), 0, "the same tile again changes nothing");
    assert!(ed.has_unsaved());
    assert!(matches!(ed.paint_at(64, 0), Err(EditError::Bounds(_))));
    assert!(matches!(ed.select(4096), Err(EditError::TileRange(4096))));

    ed.select(0).unwrap();
    assert_eq!(ed.pick_at(5, 6), Some(new));
    assert_eq!(ed.tile, new);

    ed.save().unwrap();
    assert!(!ed.has_unsaved());
    let saved = Document::open(&path).unwrap();
    assert_eq!(saved.tile(5, 6), Some(new));

    // painting the old tile back gives the starter's bytes again (canonical rows)
    ed.select(old as u32).unwrap();
    ed.paint_at(5, 6).unwrap();
    let back = scratch("state-back.nw");
    ed.save_as(&back).unwrap();
    assert_eq!(std::fs::read(&back).unwrap(), original);
    assert_eq!(ed.path(), Some(back.as_path()), "Save as becomes the Save target");
}

#[test]
fn open_error_keeps_the_current_level() {
    let path = copy_of("new2.nw", "keep.nw");
    let mut ed = Editor::new(Version::V222);
    ed.open(&path).unwrap();
    let bad = scratch("bad.nw");
    std::fs::write(&bad, b"GR-V1.03 binary").unwrap();
    assert!(matches!(ed.open(&bad), Err(EditError::NotText(_))));
    assert_eq!(ed.path(), Some(path.as_path()));
}

#[test]
fn board_image_uses_the_sheet() {
    let sheet = Sheet::builtin();
    assert_eq!((sheet.width, sheet.height), (2048, 512));
    let doc = Document::open(&starter("new1.nw")).unwrap();
    let fb = board_image(&doc, &sheet, [true, true]);
    assert_eq!((fb.width, fb.height), (1024, 1024));
    // the top-left pixel of the board is the top-left pixel of tile (0, 0) on the sheet
    let (c, r) = sheet_cell(doc.tile(0, 0).unwrap());
    let s = ((r * 16 * sheet.width + c * 16) * 3) as usize;
    assert_eq!(&fb.pixels[0..3], &sheet.rgb[s..s + 3]);
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
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1100.0, 860.0))),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        };
        let app = &mut self.app;
        let _ = self.ctx.run(input, |ctx| app.ui(ctx));
    }

    fn press(&mut self, pos: Pos2, button: PointerButton) {
        self.events.push(Event::PointerMoved(pos));
        self.events.push(Event::PointerButton { pos, button, pressed: true, modifiers: Modifiers::NONE });
        self.frame();
    }

    fn move_to(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.frame();
    }

    fn release(&mut self, pos: Pos2, button: PointerButton) {
        self.events.push(Event::PointerButton { pos, button, pressed: false, modifiers: Modifiers::NONE });
        self.frame();
    }

    fn click(&mut self, pos: Pos2, button: PointerButton) {
        self.press(pos, button);
        self.release(pos, button);
        self.frame();
    }

    /// Screen position of the middle of board cell (x, y) (zoom 1, not scrolled).
    fn board_at(&self, x: usize, y: usize) -> Pos2 {
        let r = self.app.board_rect.expect("board drawn");
        r.min + Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0)
    }

    fn palette_at(&self, id: u16) -> Pos2 {
        let r = self.app.palette_rect.expect("palette drawn");
        let (c, row) = sheet_cell(id);
        r.min + Vec2::new(c as f32 * 16.0 + 8.0, row as f32 * 16.0 + 8.0)
    }
}

#[test]
fn window_palette_click_then_board_click_paints_and_ctrl_s_saves() {
    let path = copy_of("new1.nw", "window.nw");
    let mut h = Harness::new(Some(&path));
    assert_eq!(h.app.editor.version, Version::V222, "the selector defaults to 2.22");
    let doc = h.app.editor.doc().unwrap();
    let old = doc.tile(3, 4).unwrap();
    // a tile in the palette's visible top-left area that differs from the board's
    let want: u16 = if old == 18 { 19 } else { 18 };

    let p = h.palette_at(want);
    h.click(p, PointerButton::Primary);
    assert_eq!(h.app.editor.tile, want, "palette click selects");

    let b = h.board_at(3, 4);
    h.click(b, PointerButton::Primary);
    assert_eq!(h.app.editor.doc().unwrap().tile(3, 4), Some(want), "board click paints");
    assert!(h.app.editor.has_unsaved());

    h.events.push(Event::Key { key: egui::Key::S, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::COMMAND });
    h.frame();
    assert!(!h.app.editor.has_unsaved(), "Ctrl+S saved");
    assert_eq!(Document::open(&path).unwrap().tile(3, 4), Some(want));
}

#[test]
fn window_drag_paints_a_stroke_and_right_click_picks() {
    let path = copy_of("new2.nw", "drag.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.select(7).unwrap();
    let (a, b) = (h.board_at(2, 10), h.board_at(12, 10));
    h.press(a, PointerButton::Primary);
    h.move_to(b); // one jump of ten cells in one frame
    h.release(b, PointerButton::Primary);
    let doc = h.app.editor.doc().unwrap();
    for x in 2..=12 {
        assert_eq!(doc.tile(x, 10), Some(7), "x {x}");
    }
    assert_ne!(doc.tile(13, 10), Some(7));

    let other = doc.tile(30, 30).unwrap();
    let r = h.board_at(30, 30);
    h.click(r, PointerButton::Secondary);
    assert_eq!(h.app.editor.tile, other, "right click picks the tile under the mouse");
}

#[test]
fn window_close_with_unsaved_changes_asks_first() {
    let path = copy_of("new1.nw", "close.nw");
    let mut h = Harness::new(Some(&path));
    h.app.editor.select(9).unwrap();
    let b = h.board_at(0, 0);
    h.click(b, PointerButton::Primary);
    assert!(h.app.editor.has_unsaved());
    // the window manager asks to close
    h.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1100.0, 860.0))),
        viewports: {
            let mut v = egui::ViewportIdMap::default();
            v.insert(egui::ViewportId::ROOT, egui::ViewportInfo { events: vec![egui::ViewportEvent::Close], ..Default::default() });
            v
        },
        ..Default::default()
    };
    let app = &mut h.app;
    let _ = h.ctx.run(input, |ctx| app.ui(ctx));
    assert!(!h.app.closing, "unsaved changes hold the close for the prompt");
}

#[test]
fn window_without_a_file_shows_no_board() {
    let h = Harness::new(None);
    assert!(h.app.editor.doc().is_none());
    assert!(h.app.board_rect.is_none());
    assert!(h.app.palette_rect.is_some());
}
