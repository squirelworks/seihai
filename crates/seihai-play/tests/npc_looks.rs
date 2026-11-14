//! Synthetic state-to-render regressions, not original-client compatibility tests.
use seihai_core::assets::Level;
use seihai_core::offline::OfflineSession;
use seihai_core::protocol::npcprops::{Head, NpcProps, CHARACTER_IMAGE};
use seihai_play::chars::Characters;
use seihai_play::render::{self, NpcImages, Players, Sheet};

#[test]
fn server_npc_looks_reach_the_frame_and_names_and_hide_with_the_npc() {
    let mut s = OfflineSession::default();
    s.current_level = Some(Level::from_board(&[0; 4096]));
    s.player_x = 30.0;
    s.player_y = 30.0;
    s.take_server_npc(
        7,
        &NpcProps {
            image: Some(CHARACTER_IMAGE.into()),
            x: Some(20.0),
            y: Some(25.0),
            gani: Some("idle".into()),
            head: Some(Head::Name("head0.png".into())),
            body: Some("body.png".into()),
            colors: Some([2, 5, 18, 5, 18]),
            nickname: Some("Before".into()),
            ..Default::default()
        },
    );
    let mut chars = Characters::load(None);
    chars.load_pictures(None, &[render::own_figure(&s)]);
    assert!(chars.load_npcs(None, &mut s));
    let draw = |s: &OfflineSession, chars: &Characters| {
        render::draw(
            s,
            &Sheet::load(None).unwrap(),
            &NpcImages::new(),
            Players {
                chars,
                others: &[],
                clock_ms: 0,
                effects: None,
            },
            40,
            30,
        )
    };
    let before = draw(&s, &chars);
    s.current_level.as_mut().unwrap().npcs[0].visible = false;
    let hidden = draw(&s, &chars);
    assert_ne!(
        before.rgba, hidden.rgba,
        "a prepared character is drawn only while visible"
    );
    s.current_level.as_mut().unwrap().npcs[0].visible = true;
    assert_eq!(
        seihai_play::game::npc_names(s.current_level.as_ref().unwrap(), 0, 0),
        vec![("Before".into(), 344.0, 448.0)]
    );
    s.take_server_npc(
        7,
        &NpcProps {
            gani: Some("walk".into()),
            head: Some(Head::Name("different-head.png".into())),
            body: Some("different-body.png".into()),
            colors: Some([0, 0, 0, 0, 0]),
            nickname: Some("After".into()),
            ..Default::default()
        },
    );
    let npc = &s.current_level.as_ref().unwrap().npcs[0];
    assert_eq!(npc.ani.as_deref(), Some("walk"));
    assert_eq!(npc.head.as_deref(), Some("different-head.png"));
    assert_eq!(npc.body.as_deref(), Some("different-body.png"));
    assert_eq!(npc.props.get("#n").map(String::as_str), Some("After"));
    assert_eq!(npc.props.get("#C0").map(String::as_str), Some("0"));
    chars.load_npcs(None, &mut s);
    let after = draw(&s, &chars);
    assert_ne!(
        before.rgba, after.rgba,
        "server appearance changes must reach the pixels"
    );
    let level = s.current_level.as_mut().unwrap();
    level.npcs[0].visible = false;
    assert!(seihai_play::game::npc_names(level, 0, 0).is_empty());
    level.npcs[0].visible = true;
    level.npcs[0].ani = None;
    assert!(
        seihai_play::game::npc_names(level, 0, 0).is_empty(),
        "image NPC has no nickname label"
    );
}
