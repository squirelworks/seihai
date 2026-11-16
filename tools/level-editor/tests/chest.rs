//! Tests of the chest item picker: item name and sign index edits rewrite
//! only the chest line, refuse bad values, and leave an unchanged level
//! byte-identical.

use level_editor::{tile_code, Document, EditError, ObjectEdit, ObjectKind, SIZE};
use seihai_core::assets::{Level, CHEST_ITEMS};

fn board(eol: &str) -> String {
    let code = String::from_utf8(tile_code(5).to_vec()).unwrap();
    let mut s = String::from("GLEVNW01\n");
    for y in 0..SIZE {
        s.push_str(&format!("BOARD 0 {y} 64 0 {}\n", code.repeat(64)));
    }
    s.push_str("LINK other.nw 0 20 1 4 playerx 30\n");
    s.push_str("CHEST 5 6 bombs -1\n");
    s.push_str("SIGN 3 4\nhello\nSIGNEND\n");
    s.replace('\n', eol)
}

#[test]
fn an_untouched_level_with_a_chest_round_trips_byte_for_byte() {
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
fn changing_item_and_sign_rewrites_only_the_chest_line() {
    let original = board("\n");
    let mut doc = Document::parse(original.as_bytes()).unwrap();
    let chest = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Chest)
        .unwrap();
    assert_eq!((chest.name.as_str(), chest.sign_index), ("bombs", -1));
    assert!(CHEST_ITEMS.contains(&"bombs"));

    doc.edit_object(chest.id, ObjectEdit::ChestItem("heart".into()))
        .unwrap();
    doc.edit_object(chest.id, ObjectEdit::ChestSign(0)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("CHEST 5 6 heart 0\n"));
    assert!(after.contains("LINK other.nw 0 20 1 4 playerx 30\n"));
    assert!(after.contains("SIGN 3 4\nhello\nSIGNEND\n"));
    let level = Level::parse(&after).unwrap();
    assert_eq!(
        (level.chests[0].item.as_str(), level.chests[0].sign_index),
        ("heart", 0)
    );
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
fn chest_edits_are_refused_when_they_would_break_the_line() {
    let mut doc = Document::parse(board("\n").as_bytes()).unwrap();
    let id = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Chest)
        .unwrap()
        .id;
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::ChestItem("notanitem".into())),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::ChestItem("two words".into())),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::ChestItem("".into())),
        Err(EditError::Bounds(_))
    ));
    assert!(matches!(
        doc.edit_object(id, ObjectEdit::ChestSign(-2)),
        Err(EditError::Bounds(_))
    ));
    assert!(
        matches!(
            doc.edit_object(0, ObjectEdit::ChestItem("bombs".into())),
            Err(EditError::Bounds(_))
        ),
        "a link is not a chest"
    );
    // every classic name is accepted
    for name in CHEST_ITEMS {
        doc.edit_object(id, ObjectEdit::ChestItem(name.to_string()))
            .unwrap();
    }
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains(&format!(
        "CHEST 5 6 {} -1\n",
        CHEST_ITEMS[CHEST_ITEMS.len() - 1]
    )));
}

#[test]
fn a_crlf_chest_keeps_crlf_on_the_edited_line() {
    let mut doc = Document::parse(board("\r\n").as_bytes()).unwrap();
    let id = doc
        .objects()
        .into_iter()
        .find(|o| o.kind == ObjectKind::Chest)
        .unwrap()
        .id;
    doc.edit_object(id, ObjectEdit::ChestItem("goldrupee".into()))
        .unwrap();
    doc.edit_object(id, ObjectEdit::ChestSign(2)).unwrap();
    let after = String::from_utf8(doc.to_bytes()).unwrap();
    assert!(after.contains("\r\nCHEST 5 6 goldrupee 2\r\n"));
    assert!(!after.replace("\r\n", "").contains('\n'));
}
