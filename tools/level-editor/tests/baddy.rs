//! Tests of baddy editing: move, type, verses, place and delete, with
//! byte-identical round trip when nothing changes, and only the baddy's
//! own lines rewritten when it does.

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{EditorApp, Sheet, Tool};
use level_editor::{VersionProfile, tile_code, Document, EditError, ObjectEdit, ObjectKind, Version, SIZE};
use seihai_core::assets::Level;

fn board(eol: &str) -> String {
    let code = String::from_utf8(tile_code(5).to_vec()).unwrap();
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code.repeat(64)));
    }
    s.push_str("LINK other.nw 0 20 1 4 playerx 30\n");
    s.push_str("BADDY 8 9 0\none\ntwo\nthree\nBADDYEND\n");
    s.push_str("CHEST 5 6 bombs -1\n");
    s.replace('\n', eol)
}

#[test]
fn an_untouched_level_with_a_baddy_round_trips_byte_for_byte() {
    for eol in ["\n", "\r\n"] {
        let original = board(eol);
        let doc = Document::parse(original.as_bytes()).unwrap();
        assert_eq!(
            doc.to_bytes(),
            original.as_bytes(),
            "eol {:?}",
            eol.as_bytes()
        );
    }
}

#[test]
fn moving_and_editing_a_baddy_rewrites_only_its_lines() {
    let original = board("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let baddy = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Baddy)
        .unwrap();
    assert_eq!(
        (
            baddy.x,
            baddy.y,
            baddy.name.as_str(),
            baddy.verses.as_slice()
        ),
        (
            8.0,
            9.0,
            "0",
            &["one".into(), "two".into(), "three".into()][..]
        )
    );

    doc.edit_object(baddy.id, ObjectEdit::Move(10.4, 11.0))
        .unwrap();
    doc.edit_object(baddy.id, ObjectEdit::BaddyKind("7".into()))
        .unwrap();
    doc.edit_object(
        baddy.id,
        ObjectEdit::BaddyVerses(vec!["saw".into(), "ouch".into(), "grr".into()]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("BADDY 10 11 7\nsaw\nouch\ngrr\nBADDYEND\n"));
    assert!(after.contains("LINK other.nw 0 20 1 4 playerx 30\n"));
    assert!(after.contains("CHEST 5 6 bombs -1\n"));
    let level = Level::parse(&after).unwrap();
    assert_eq!(
        (
            level.baddies[0].x,
            level.baddies[0].y,
            level.baddies[0].kind.as_str(),
            level.baddies[0].verses.as_str()
        ),
        (10.0, 11.0, "7", "saw\nouch\ngrr")
    );
    // the board rows were not rewritten
    assert_eq!(
        after
            .lines()
            .filter(|l| l.starts_with("BOARD"))
            .collect::<Vec<_>>(),
        original
            .lines()
            .filter(|l| l.starts_with("BOARD"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn baddy_edits_are_refused_when_they_would_break_the_record() {
    let mut doc = Document::parse(board("\n").as_bytes()).unwrap();
    let id = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Baddy)
        .unwrap()
        .id;
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::BaddyKind("10".into())),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::BaddyKind("x".into())),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::BaddyVerses(vec!["a\nb".into()])),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::BaddyVerses(vec!["BADDYEND".into()])),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(
            id,
            ObjectEdit::BaddyVerses(vec!["1".into(), "2".into(), "3".into(), "4".into()])
        ),
        Err(EditError::Bounds(_))
    ));
    assert!(
        matches!(
            doc.edit_object(0, ObjectEdit::BaddyKind("1".into())),
            Err(EditError::Bounds(_))
        ),
        "a link is not a baddy"
    );
    // a shorter list replaces the shown verses (like the NPC script pane)
    doc.edit_object(id, ObjectEdit::BaddyVerses(vec!["only".into()]))
        .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("BADDY 8 9 0\nonly\nBADDYEND\n"));
}

#[test]
fn verse_lines_past_the_pane_are_kept() {
    let code = String::from_utf8(tile_code(5).to_vec()).unwrap();
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code.repeat(64)));
    }
    s.push_str("BADDY 1 2 1\na\nb\nc\nkept\nBADDYEND\n");
    let mut doc = Document::parse(s.as_bytes()).unwrap();
    let id = 0;
    assert_eq!(
        doc.object(id).unwrap().verses,
        vec!["a".to_string(), "b".into(), "c".into()]
    );
    doc.edit_object(
        id,
        ObjectEdit::BaddyVerses(vec!["x".into(), "y".into(), "z".into()]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("BADDY 1 2 1\nx\ny\nz\nkept\nBADDYEND\n"));
}

#[test]
fn a_crlf_baddy_keeps_crlf_on_edited_lines() {
    let mut doc = Document::parse(board("\r\n").as_bytes()).unwrap();
    let id = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Baddy)
        .unwrap()
        .id;
    doc.edit_object(id, ObjectEdit::Move(2.0, 3.0)).unwrap();
    doc.edit_object(
        id,
        ObjectEdit::BaddyVerses(vec!["a".into(), "b".into(), "c".into()]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("\r\nBADDY 2 3 0\r\na\r\nb\r\nc\r\nBADDYEND\r\n"));
    assert!(!after.replace("\r\n", "").contains('\n'));
}

struct Harness {
    ctx: egui::Context,
    app: EditorApp,
    events: Vec<Event>,
}

impl Harness {
    fn new(bytes: &[u8]) -> Harness {
        let dir =
            std::env::temp_dir().join(format!("seihai-level-editor-baddy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("baddy.nw");
        std::fs::write(&path, bytes).unwrap();
        let mut h = Harness {
            ctx: egui::Context::default(),
            app: EditorApp::new(VersionProfile::TextNw222, Sheet::builtin(), Some(&path)),
            events: Vec::new(),
        };
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
        self.events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        self.frame();
    }

    fn move_to(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.frame();
    }

    fn release(&mut self, pos: Pos2) {
        self.events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        self.frame();
    }

    fn board_at(&self, x: f32, y: f32) -> Pos2 {
        let r = self.app.board_rect.expect("board drawn");
        r.min + Vec2::new(x * 16.0, y * 16.0)
    }
}

#[test]
fn window_objects_tool_drags_a_baddy() {
    let mut h = Harness::new(board("\n").as_bytes());
    h.app.editor.tool = Tool::Objects;
    h.frame();
    let id = h
        .app
        .editor
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Baddy)
        .unwrap()
        .id;
    let (a, b) = (h.board_at(9.0, 10.0), h.board_at(12.0, 13.0));
    h.press(a);
    assert_eq!(h.app.editor.selected, Some(id));
    h.move_to(b);
    h.release(b);
    let baddy = h.app.editor.doc().unwrap().object(id).unwrap();
    assert_eq!((baddy.x, baddy.y), (11.0, 12.0));
    assert!(h.app.editor.has_unsaved());
}
