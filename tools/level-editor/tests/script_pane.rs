//! Tests of the NPC script and sign text pane: the whole body is read and
//! replaced on the document, an unchanged apply keeps the file's bytes, and
//! the pane opens by a double-click on an NPC or sign marker in the window
//! driven headless.

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{EditorApp, Sheet, Tool};
use level_editor::{tile_code, Document, EditError, ObjectEdit, ObjectKind, VersionProfile, SIZE};
use seihai_core::assets::Level;
use std::path::PathBuf;

/// A level with a link, a 25-line NPC script, an NPC with no script and a
/// multi-line sign.
fn level(eol: &str) -> String {
    let code = String::from_utf8(tile_code(5).to_vec()).unwrap();
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code.repeat(64)));
    }
    s.push_str("LINK other.nw 0 20 1 4 playerx 30\n");
    s.push_str("NPC tree.png 12.5 30\n");
    for i in 1..=25 {
        s.push_str(&format!("// line {i}\n"));
    }
    s.push_str("NPCEND\nNPC - 40 40\nNPCEND\nSIGN 3 4\nhello\nsecond line\nSIGNEND\n");
    s.replace('\n', eol)
}

fn script_lines(after: &str, npc: usize) -> Vec<String> {
    Level::parse(after).unwrap().npcs[npc]
        .script
        .lines()
        .map(str::to_string)
        .collect()
}

fn sign_lines(after: &str, sign: usize) -> Vec<String> {
    Level::parse(after).unwrap().signs[sign]
        .text
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn the_pane_reads_the_whole_npc_script() {
    let doc = Document::parse(level("\n").as_bytes()).unwrap();
    let head = doc.script_head(1).unwrap();
    assert_eq!(head.lines.len(), 25);
    assert_eq!(head.total, 25);
    assert_eq!(head.lines[0], "// line 1");
    assert_eq!(head.lines[24], "// line 25");
    let empty = doc.script_head(2).unwrap();
    assert_eq!((empty.lines.len(), empty.total), (0, 0));
    assert!(
        matches!(doc.script_head(0), Err(EditError::Bounds(_))),
        "a link has no script"
    );
    assert!(
        matches!(doc.script_head(3), Err(EditError::Bounds(_))),
        "a sign has no NPC script"
    );
}

#[test]
fn the_pane_reads_every_sign_line() {
    let doc = Document::parse(level("\n").as_bytes()).unwrap();
    let text = doc.sign_text(3).unwrap();
    assert_eq!(text.lines, ["hello", "second line"]);
    assert_eq!(text.total, 2);
    assert!(
        matches!(doc.sign_text(1), Err(EditError::Bounds(_))),
        "an NPC has no sign text"
    );
}

#[test]
fn saving_the_pane_replaces_the_whole_script() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    doc.edit_object(
        1,
        ObjectEdit::Script(vec![
            "if (created) {".into(),
            "  setshape 1,32,32;".into(),
            "}".into(),
        ]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert_eq!(
        script_lines(&after, 0),
        ["if (created) {", "  setshape 1,32,32;", "}",]
    );
    // nothing outside the script changed
    let strip = |t: &str| {
        t.lines()
            .filter(|l| {
                !l.starts_with("// line")
                    && !l.contains("setshape")
                    && *l != "if (created) {"
                    && *l != "}"
            })
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    assert_eq!(strip(&after), strip(&original));
    let npc = &Level::parse(&after).unwrap().npcs[0];
    assert_eq!((npc.x, npc.y, npc.image.as_str()), (12.5, 30.0, "tree.png"));
}

#[test]
fn an_unchanged_full_script_apply_keeps_the_file_bytes() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let lines = doc.script_head(1).unwrap().lines;
    doc.edit_object(1, ObjectEdit::Script(lines)).unwrap();
    assert_eq!(
        String::from_utf8(doc.to_bytes()).unwrap(),
        original,
        "rewriting the same script lines keeps every byte"
    );
}

#[test]
fn an_empty_script_gets_lines_before_npcend_and_can_be_emptied_again() {
    let mut doc = Document::parse(level("\n").as_bytes()).unwrap();
    doc.edit_object(2, ObjectEdit::Script(vec!["// a".into(), "// b".into()]))
        .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("NPC - 40 40\n// a\n// b\nNPCEND\n"));
    doc.edit_object(2, ObjectEdit::Script(Vec::new())).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("NPC - 40 40\nNPCEND\n"));
    assert_eq!(after, level("\n"), "back to the file as it was");
}

#[test]
fn the_pane_refuses_embedded_breaks_and_end_markers() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    assert!(matches!(
        doc.edit_object(1, ObjectEdit::Script(vec!["a\nb".into()])),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(1, ObjectEdit::Script(vec![" NPCEND".into()])),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(3, ObjectEdit::SignText(vec![" SIGNEND".into()])),
        Err(EditError::Bounds(_))
    ));
    assert!(
        matches!(
            doc.edit_object(3, ObjectEdit::Script(vec!["x".into()])),
            Err(EditError::Bounds(_))
        ),
        "signs use SignText, not Script"
    );
    assert!(
        matches!(
            doc.edit_object(1, ObjectEdit::SignText(vec!["x".into()])),
            Err(EditError::Bounds(_))
        ),
        "NPCs use Script, not SignText"
    );
    assert_eq!(
        String::from_utf8(doc.to_bytes()).unwrap(),
        original,
        "nothing was changed"
    );
}

#[test]
fn a_crlf_file_keeps_crlf_on_script_and_sign_lines() {
    let mut doc = Document::parse(level("\r\n").as_bytes()).unwrap();
    let mut lines = doc.script_head(1).unwrap().lines;
    lines[4] = "// changed".into();
    lines.remove(0);
    lines.push("// extra".into());
    doc.edit_object(1, ObjectEdit::Script(lines)).unwrap();
    doc.edit_object(
        3,
        ObjectEdit::SignText(vec!["first".into(), "second".into(), "third".into()]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(!after.replace("\r\n", "").contains('\n'));
    assert!(after.contains("\r\n// changed\r\n"));
    assert!(after.contains("\r\nfirst\r\nsecond\r\nthird\r\nSIGNEND\r\n"));
    let script = script_lines(&after.replace("\r\n", "\n"), 0);
    assert_eq!(script.len(), 25);
    assert_eq!(script[0], "// line 2");
    assert_eq!(script[24], "// extra");
}

#[test]
fn full_sign_text_is_replaced_and_unchanged_apply_keeps_bytes() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let same = doc.sign_text(3).unwrap().lines;
    doc.edit_object(3, ObjectEdit::SignText(same)).unwrap();
    assert_eq!(
        String::from_utf8(doc.to_bytes()).unwrap(),
        original,
        "rewriting the same sign lines keeps every byte"
    );
    doc.edit_object(
        3,
        ObjectEdit::SignText(vec![
            "line one".into(),
            "line two".into(),
            "line three".into(),
        ]),
    )
    .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert_eq!(
        sign_lines(&after, 0),
        ["line one", "line two", "line three"]
    );
    assert!(after.contains("SIGN 3 4\nline one\nline two\nline three\nSIGNEND\n"));
    // a long script (more than the old 20-line cap) is accepted
    let many: Vec<String> = (1..=30).map(|i| format!("// long {i}")).collect();
    doc.edit_object(1, ObjectEdit::Script(many.clone()))
        .unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert_eq!(script_lines(&after, 0), many);
}

// ---- the window, driven headless ----

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("seihai-level-editor-script-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

struct Harness {
    ctx: egui::Context,
    app: EditorApp,
    events: Vec<Event>,
    time: f64,
}

impl Harness {
    fn frame(&mut self) {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 1100.0))),
            events: std::mem::take(&mut self.events),
            time: Some(self.time),
            ..Default::default()
        };
        let app = &mut self.app;
        let _ = self.ctx.run(input, |ctx| app.ui(ctx));
    }

    fn click(&mut self, pos: Pos2) {
        self.events.push(Event::PointerMoved(pos));
        self.events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        });
        self.frame();
        self.events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        self.frame();
    }
}

#[test]
fn window_double_click_on_an_npc_opens_the_pane_and_apply_saves() {
    let path = scratch("pane.nw");
    std::fs::write(&path, level("\n")).unwrap();
    let mut h = Harness {
        ctx: egui::Context::default(),
        app: EditorApp::new(VersionProfile::TextNw222, Sheet::builtin(), Some(&path)),
        events: Vec::new(),
        time: 0.0,
    };
    h.frame();
    h.app.editor.tool = Tool::Objects;
    h.frame();
    let r = h.app.board_rect.expect("board drawn");
    // the NPC's marker is 2 x 2 from (12.5, 30)
    let p = r.min + Vec2::new(13.5 * 16.0, 31.0 * 16.0);
    h.click(p);
    assert!(h.app.script.is_none(), "one click only selects");
    h.click(p);
    h.frame();
    let pane = h.app.script.clone().expect("a double-click opens the pane");
    assert_eq!((pane.id, pane.kind, pane.total), (1, ObjectKind::Npc, 25));
    assert!(
        !h.app.editor.has_unsaved(),
        "the double-click did not move it"
    );

    // a double-click on the link does not open a pane
    h.app.script = None;
    let l = r.min + Vec2::new(0.5 * 16.0, 21.0 * 16.0);
    h.click(l);
    h.click(l);
    h.frame();
    assert!(h.app.script.is_none());

    h.app.open_script(1);
    h.app.script.as_mut().unwrap().text = "if (playertouchsme) {\n  say2 hi;\n}".into();
    assert!(h.app.apply_script());
    assert!(h.app.script.is_none());
    h.app.editor.save().unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    let script = script_lines(&saved, 0);
    assert_eq!(&script[..], ["if (playertouchsme) {", "  say2 hi;", "}"]);

    // more than 20 lines is accepted
    h.app.open_script(1);
    h.app.script.as_mut().unwrap().text = (1..=21)
        .map(|i| format!("// x{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(h.app.apply_script());
    assert!(h.app.script.is_none());
    h.app.editor.save().unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(script_lines(&saved, 0).len(), 21);
}

#[test]
fn window_double_click_on_a_sign_opens_the_text_pane() {
    let path = scratch("sign-pane.nw");
    std::fs::write(&path, level("\n")).unwrap();
    let mut h = Harness {
        ctx: egui::Context::default(),
        app: EditorApp::new(VersionProfile::TextNw222, Sheet::builtin(), Some(&path)),
        events: Vec::new(),
        time: 0.0,
    };
    h.frame();
    h.app.editor.tool = Tool::Objects;
    h.frame();
    let r = h.app.board_rect.expect("board drawn");
    // the sign's marker is 2 x 1 from (3, 4)
    let p = r.min + Vec2::new(3.5 * 16.0, 4.0 * 16.0);
    h.click(p);
    h.click(p);
    h.frame();
    let pane = h.app.script.clone().expect("a double-click opens the pane");
    assert_eq!((pane.id, pane.kind, pane.total), (3, ObjectKind::Sign, 2));
    assert_eq!(pane.text, "hello\nsecond line");
    h.app.script.as_mut().unwrap().text = "a\nb\nc".into();
    assert!(h.app.apply_script());
    h.app.editor.save().unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(sign_lines(&saved, 0), ["a", "b", "c"]);
}
