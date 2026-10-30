//! Tests of the NPC script pane: the first 20 script lines read and replaced
//! on the document, lines after them kept, and the pane opened by a
//! double-click on an NPC marker in the window driven headless.

use eframe::egui;
use egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use level_editor::gui::{EditorApp, Sheet, Tool};
use level_editor::{tile_code, Document, EditError, ObjectEdit, Version, SCRIPT_PANE_LINES, SIZE};
use seihai_core::assets::Level;
use std::path::PathBuf;

/// A level with a link, a 25-line NPC script, an NPC with no script and a sign.
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
    s.push_str("NPCEND\nNPC - 40 40\nNPCEND\nSIGN 3 4\nhello\nSIGNEND\n");
    s.replace('\n', eol)
}

fn script_lines(after: &str, npc: usize) -> Vec<String> {
    Level::parse(after).unwrap().npcs[npc].script.lines().map(str::to_string).collect()
}

#[test]
fn the_pane_reads_the_first_20_lines_and_the_total() {
    let doc = Document::parse(level("\n").as_bytes()).unwrap();
    let head = doc.script_head(1).unwrap();
    assert_eq!(head.lines.len(), SCRIPT_PANE_LINES);
    assert_eq!(head.lines[0], "// line 1");
    assert_eq!(head.lines[19], "// line 20");
    assert_eq!(head.total, 25);
    let empty = doc.script_head(2).unwrap();
    assert_eq!((empty.lines.len(), empty.total), (0, 0));
    assert!(matches!(doc.script_head(0), Err(EditError::Bounds(_))), "a link has no script");
    assert!(matches!(doc.script_head(3), Err(EditError::Bounds(_))), "a sign has no script");
}

#[test]
fn saving_the_pane_replaces_the_shown_lines_and_keeps_the_rest() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    // 20 shown lines become 3: lines 21 to 25 stay, right after them
    doc.edit_object(1, ObjectEdit::ScriptHead(vec!["if (created) {".into(), "  setshape 1,32,32;".into(), "}".into()])).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    let mut want = vec!["if (created) {".to_string(), "  setshape 1,32,32;".into(), "}".into()];
    want.extend((21..=25).map(|i| format!("// line {i}")));
    assert_eq!(script_lines(&after, 0), want);
    // nothing outside the script changed
    let strip = |t: &str| t.lines().filter(|l| !l.starts_with("// line") && !l.contains("setshape") && *l != "if (created) {" && *l != "}").map(str::to_string).collect::<Vec<_>>();
    assert_eq!(strip(&after), strip(&original));
    let npc = &Level::parse(&after).unwrap().npcs[0];
    assert_eq!((npc.x, npc.y, npc.image.as_str()), (12.5, 30.0, "tree.png"));
}

#[test]
fn an_empty_script_gets_lines_before_npcend_and_can_be_emptied_again() {
    let mut doc = Document::parse(level("\n").as_bytes()).unwrap();
    doc.edit_object(2, ObjectEdit::ScriptHead(vec!["// a".into(), "// b".into()])).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("NPC - 40 40\n// a\n// b\nNPCEND\n"));
    doc.edit_object(2, ObjectEdit::ScriptHead(Vec::new())).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("NPC - 40 40\nNPCEND\n"));
    assert_eq!(after, level("\n"), "back to the file as it was");
}

#[test]
fn the_pane_refuses_more_than_20_lines_breaks_and_npcend() {
    let original = level("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let too_many: Vec<String> = (0..21).map(|i| format!("// {i}")).collect();
    assert!(matches!(doc.edit_object(1, ObjectEdit::ScriptHead(too_many)), Err(EditError::Bounds(_))));
    assert!(matches!(doc.edit_object(1, ObjectEdit::ScriptHead(vec!["a\nb".into()])), Err(EditError::Bounds(_))));
    assert!(matches!(doc.edit_object(1, ObjectEdit::ScriptHead(vec![" NPCEND".into()])), Err(EditError::Bounds(_))));
    assert!(matches!(doc.edit_object(3, ObjectEdit::ScriptHead(vec!["x".into()])), Err(EditError::Bounds(_))), "signs keep the one-line edit");
    assert_eq!(String::from_utf8(doc.to_bytes()).unwrap(), original, "nothing was changed");
}

#[test]
fn a_crlf_file_keeps_crlf_on_script_lines() {
    let mut doc = Document::parse(level("\r\n").as_bytes()).unwrap();
    let mut lines = doc.script_head(1).unwrap().lines;
    lines[4] = "// changed".into();
    lines.remove(0);
    lines.push("// extra".into());
    doc.edit_object(1, ObjectEdit::ScriptHead(lines)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(!after.replace("\r\n", "").contains('\n'));
    assert!(after.contains("\r\n// changed\r\n"));
    let script = script_lines(&after.replace("\r\n", "\n"), 0);
    assert_eq!(script.len(), 25);
    assert_eq!(script[0], "// line 2");
    assert_eq!(script[19], "// extra");
    assert_eq!(script[20], "// line 21");
}

// ---- the window, driven headless ----

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seihai-level-editor-script-{}", std::process::id()));
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
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        self.frame();
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        self.frame();
    }
}

#[test]
fn window_double_click_on_an_npc_opens_the_pane_and_apply_saves() {
    let path = scratch("pane.nw");
    std::fs::write(&path, level("\n")).unwrap();
    let mut h = Harness { ctx: egui::Context::default(), app: EditorApp::new(Version::V222, Sheet::builtin(), Some(&path)), events: Vec::new(), time: 0.0 };
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
    assert_eq!((pane.id, pane.shown, pane.total), (1, 20, 25));
    assert!(!h.app.editor.has_unsaved(), "the double-click did not move it");

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
    assert_eq!(&script[..3], ["if (playertouchsme) {", "  say2 hi;", "}"]);
    assert_eq!(script.len(), 8);

    // more than 20 lines stays in the pane with an error
    h.app.open_script(1);
    h.app.script.as_mut().unwrap().text = vec!["// x"; 21].join("\n");
    assert!(!h.app.apply_script());
    assert!(h.app.script.is_some());
}
