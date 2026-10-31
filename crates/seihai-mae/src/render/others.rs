//! Other players drawn through a GANI, with their own sword, shield, head
//! and body pictures (props 8, 9, 11, 35). The GANI is the one prop 10
//! names when it is a pose Seihai has loaded, otherwise the pose the sprite
//! prop (17) stands for. See `documentation/other-players-looks.md`.

use crate::assets::Gani;
use crate::offline::{OfflineSession, HIDDEN_STATUS};
use crate::protocol::{HidePose, OtherPlayer};

/// Milliseconds a step of a looping GANI is shown when the step comes from
/// the clock rather than from the sprite prop.
pub const OTHER_STEP_MS: u64 = 50;

/// The dead bit of the STATUS prop (18). The 2.05 client sets it once its
/// death timer reaches 28 ticks and sends no dead sprite.
pub const STATUS_DEAD: u8 = 0x08;

use crate::pose::WALK_S;

/// The GANI prop (10) of another player, lower case, without `.gani`.
fn gani_name(o: &OtherPlayer) -> Option<String> {
    // GServer-v2 adds `.gif` to a name a 2.05 player sends in the bow slot.
    o.looks.gani.as_deref().map(|n| n.to_ascii_lowercase().trim_end_matches(".gani").trim_end_matches(".gif").to_string())
}

/// The hurt blink of another player: this tick only the shadow of the
/// figure is drawn. The real 2.00 blinks a watched player while its GANI is
/// `hurt` or `ridehurt`, and only then: no timer, and the hurt sprite does
/// not start it. One 50 ms tick drawn, one tick shadow only (2.00 trace,
/// 2026-10-30). A pre-2.1 build reads the name from the bow slot, where the
/// server may add `.gif`.
pub fn other_blink_off(o: &OtherPlayer, clock_ms: u64) -> bool {
    let name = o.looks.gani.as_deref().or(o.looks.bow_slot_gani.as_deref()).map(|n| n.to_ascii_lowercase());
    let hurt = name.as_deref().map(|n| n.trim_end_matches(".gani").trim_end_matches(".gif")).is_some_and(|n| n == "hurt" || n == "ridehurt");
    hurt && (clock_ms / OTHER_STEP_MS) % 2 == 1
}

/// Another player is hidden under what they carry, and only the object is
/// drawn, where they stand. The GANI prop says so (`hiddenstill` /
/// `hidden`, as 2.22 sends), or the STATUS hidden bit (0x02) is set on a
/// build that draws the hidden player that way ([`HidePose::Hidden`]).
pub fn other_hidden(session: &OfflineSession, o: &OtherPlayer) -> bool {
    matches!(gani_name(o).as_deref(), Some("hidden" | "hiddenstill"))
        || (o.status.is_some_and(|s| s & HIDDEN_STATUS != 0) && session.mannerisms.hide == HidePose::Hidden)
}

/// The GANI and step to draw another player with, if Seihai has that pose
/// loaded. `clock_ms` steps the GANIs prop 10 names.
pub fn other_pose<'a>(session: &'a OfflineSession, o: &OtherPlayer, clock_ms: u64) -> Option<(&'a Gani, usize)> {
    let p = &session.poses;
    let walk = session.walk_gani.as_ref().map(|w| &w.gani);
    let looped = |g: Option<&'a Gani>| g.filter(|g| g.frame_count() > 0).map(|g| (g, (clock_ms / OTHER_STEP_MS) as usize % g.frame_count()));
    let first = |g: Option<&'a Gani>, i: usize| g.filter(|g| g.frame_count() > 0).map(|g| (g, i.min(g.frame_count() - 1)));
    // A dead player: the dead bit is all the wire carries. By the time it is
    // set the player's own death GANI has run 28 of its 30 ticks, so the last
    // step is shown (inferred).
    if o.status.is_some_and(|s| s & STATUS_DEAD != 0) {
        if let Some(dead) = first(p.dead.as_ref(), usize::MAX) {
            return Some(dead);
        }
    }
    // Hidden: `hidden` while the GANI prop walks, else `hiddenstill`. The
    // stock files draw only the shadow; without them none, and the caller
    // draws nothing of the figure (see `other_hidden`).
    if other_hidden(session, o) {
        return match gani_name(o).as_deref() {
            Some("hidden") => looped(p.hidden.as_ref()),
            _ => first(p.hiddenstill.as_ref(), 0),
        };
    }
    if let Some(name) = gani_name(o) {
        let named = match name.as_str() {
            "idle" => first(p.idle.as_ref(), 0),
            "walk" => looped(walk),
            "sword" => looped(p.sword.as_ref()),
            "hurt" => looped(p.hurt.as_ref()),
            "sit" => first(p.sit.as_ref(), 0),
            "sleep" => first(p.sleep.as_ref(), 0),
            "carrystill" => first(p.carrystill.as_ref(), 0),
            // Picking up: without `lift.gani`, the carry pose it gives way
            // to, as for the local player.
            "lift" => looped(p.lift.as_ref()).or_else(|| first(p.carrystill.as_ref(), 0)),
            "carry" => looped(p.carry.as_ref()),
            "dead" => first(p.dead.as_ref(), usize::MAX),
            // Reading the M map (2.01+): `maps1` goes on to the looping
            // `maps2`; `maps3` when the map closes. Standing without them.
            "maps1" | "maps2" => looped(p.maps2.as_ref()).or_else(|| first(p.maps1.as_ref(), 0)).or_else(|| first(p.idle.as_ref(), 0)),
            "maps3" => first(p.maps3.as_ref(), 0).or_else(|| first(p.idle.as_ref(), 0)),
            _ => None,
        };
        if named.is_some() {
            return named;
        }
    }
    let s = o.sprite.unwrap_or(0) / 4;
    match s {
        1..=5 => first(walk, WALK_S.iter().position(|&w| w == s).unwrap_or(0)),
        6..=10 => first(p.sword.as_ref(), usize::from(s - 6)),
        18 => first(p.carrystill.as_ref(), 0),
        19 | 20 => first(p.carry.as_ref(), usize::from(s - 19)),
        28 => first(p.sit.as_ref(), 0),
        32 => first(p.hurt.as_ref(), 0),
        _ => None,
    }
    .or_else(|| first(p.idle.as_ref(), 0))
}

/// The image names another player's looks need, with the folder each is
/// looked up in first. Empty names (no sword, no shield) are left out.
pub fn look_images(o: &OtherPlayer) -> impl Iterator<Item = (&'static str, &str)> {
    let l = &o.looks;
    [("swords", &l.sword), ("shields", &l.shield), ("heads", &l.head), ("bodies", &l.body)]
        .into_iter()
        .filter_map(|(dir, n)| n.as_deref().filter(|n| !n.is_empty()).map(|n| (dir, n)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::GaniPlayer;
    use crate::pose::PoseKind;

    /// A single-direction GANI with `n` steps.
    fn gani(n: usize) -> Gani {
        let mut text = String::from("SINGLEDIRECTION\nANI\n");
        for _ in 0..n {
            text.push_str("1 0 0\n\n");
        }
        text.push_str("ANIEND\n");
        Gani::parse(&text).unwrap()
    }

    fn other(sprite: u8, gani: Option<&str>) -> OtherPlayer {
        let mut o = OtherPlayer {
            id: 1,
            x: 0.0,
            y: 0.0,
            dir: 2,
            nick: None,
            chat: None,
            chat_timer: Default::default(),
            account: None,
            cell: None,
            sprite: None,
            status: None,
            ap: None,
            sword_power: None,
            carry: None,
            looks: Default::default(),
        };
        o.sprite = Some(sprite);
        o.looks.gani = gani.map(str::to_string);
        o
    }

    fn session() -> OfflineSession {
        let mut s = OfflineSession::default();
        s.set_pose_gani(PoseKind::Idle, gani(1));
        s.set_pose_gani(PoseKind::Sword, gani(5));
        s.set_pose_gani(PoseKind::Hurt, gani(2));
        s.set_walk_gani(GaniPlayer::new(gani(8)));
        s
    }

    fn is(got: Option<(&Gani, usize)>, want: &Gani, frame: usize) -> bool {
        got.is_some_and(|(g, f)| std::ptr::eq(g, want) && f == frame)
    }

    #[test]
    fn the_sprite_number_picks_the_pose_and_step() {
        let s = session();
        let p = &s.poses;
        // dir 2 + 4 * S.
        assert!(is(other_pose(&s, &other(2, None), 0), p.idle.as_ref().unwrap(), 0));
        assert!(is(other_pose(&s, &other(2 + 4 * 8, None), 0), p.sword.as_ref().unwrap(), 2), "S 8: third swing step");
        assert!(is(other_pose(&s, &other(2 + 4 * 32, None), 0), p.hurt.as_ref().unwrap(), 0));
        let walk = &s.walk_gani.as_ref().unwrap().gani;
        assert!(is(other_pose(&s, &other(2 + 4 * 5, None), 0), walk, 1), "S 5: walk frame 1");
        // A number Seihai has no pose for: idle.
        assert!(is(other_pose(&s, &other(2 + 4 * 24, None), 0), p.idle.as_ref().unwrap(), 0));
    }

    #[test]
    fn a_gani_name_wins_and_steps_with_the_clock() {
        let s = session();
        let walk = &s.walk_gani.as_ref().unwrap().gani;
        assert!(is(other_pose(&s, &other(2, Some("walk")), 3 * OTHER_STEP_MS), walk, 3));
        // An unknown name falls back to the sprite number.
        assert!(is(other_pose(&s, &other(2 + 4 * 32, Some("dance")), 0), s.poses.hurt.as_ref().unwrap(), 0));
        // Nothing loaded: none.
        assert!(other_pose(&OfflineSession::default(), &other(2, None), 0).is_none());
    }

    #[test]
    fn a_player_reading_the_map_shows_maps2_then_maps3() {
        let mut s = session();
        // Without the files: standing.
        assert!(is(other_pose(&s, &other(2, Some("maps1")), 0), s.poses.idle.as_ref().unwrap(), 0));
        s.set_pose_gani(PoseKind::Maps1, gani(2));
        s.set_pose_gani(PoseKind::Maps2, gani(3));
        s.set_pose_gani(PoseKind::Maps3, gani(2));
        assert!(is(other_pose(&s, &other(2, Some("maps1")), 4 * OTHER_STEP_MS), s.poses.maps2.as_ref().unwrap(), 1));
        assert!(is(other_pose(&s, &other(2, Some("maps3")), 0), s.poses.maps3.as_ref().unwrap(), 0));
    }

    #[test]
    fn a_hurt_gani_blinks_every_other_tick_and_nothing_else_does() {
        let mut o = other(2 + 4 * 32, Some("hurt"));
        let blinks: Vec<bool> = (0..4).map(|t| other_blink_off(&o, t * OTHER_STEP_MS)).collect();
        assert_eq!(blinks, [false, true, false, true]);
        // The bow slot of a pre-2.1 build, with the `.gif` GServer adds.
        o.looks.gani = None;
        o.looks.bow_slot_gani = Some("hurt.gif".into());
        assert!(other_blink_off(&o, OTHER_STEP_MS));
        o.looks.bow_slot_gani = Some("ridehurt".into());
        assert!(other_blink_off(&o, OTHER_STEP_MS));
        // The hurt sprite alone, or another GANI: no blink.
        o.looks.bow_slot_gani = Some("idle".into());
        assert!(!other_blink_off(&o, OTHER_STEP_MS));
        assert!(!other_blink_off(&other(2 + 4 * 32, None), OTHER_STEP_MS));
    }

    #[test]
    fn the_dead_bit_shows_the_last_dead_step() {
        let mut s = session();
        s.set_pose_gani(PoseKind::Dead, gani(3));
        let mut o = other(2 + 4 * 5, Some("walk"));
        o.status = Some(STATUS_DEAD | 0x01);
        assert!(is(other_pose(&s, &o, 0), s.poses.dead.as_ref().unwrap(), 2), "the dead bit wins over the walk");
        o.status = Some(0x01);
        assert!(!is(other_pose(&s, &o, 0), s.poses.dead.as_ref().unwrap(), 2));
        // No dead GANI loaded: the old pose.
        let mut o = other(2, None);
        o.status = Some(STATUS_DEAD);
        let plain = session();
        assert!(is(other_pose(&plain, &o, 0), plain.poses.idle.as_ref().unwrap(), 0));
    }

    #[test]
    fn lift_plays_lift_or_falls_back_to_carrystill() {
        let mut s = session();
        s.set_pose_gani(PoseKind::CarryStill, gani(1));
        assert!(is(other_pose(&s, &other(2, Some("lift")), 0), s.poses.carrystill.as_ref().unwrap(), 0), "no lift.gani: carrystill");
        s.set_pose_gani(PoseKind::Lift, gani(3));
        assert!(is(other_pose(&s, &other(2, Some("lift.gani")), 2 * OTHER_STEP_MS), s.poses.lift.as_ref().unwrap(), 2));
    }

    #[test]
    fn a_hidden_player_draws_the_hidden_poses() {
        let mut s = session();
        s.set_pose_gani(PoseKind::CarryStill, gani(1));
        s.set_pose_gani(PoseKind::Carry, gani(2));
        s.set_pose_gani(PoseKind::HiddenStill, gani(1));
        s.set_pose_gani(PoseKind::Hidden, gani(4));
        // What 2.22 sends: hiddenstill with STATUS 22, S 18 (carry still).
        let mut o = other(2 + 4 * 18, Some("hiddenstill"));
        o.status = Some(22);
        assert!(other_hidden(&s, &o));
        assert!(is(other_pose(&s, &o, 0), s.poses.hiddenstill.as_ref().unwrap(), 0));
        // Walking hidden: S 19, the hidden walk on the clock.
        let mut o = other(2 + 4 * 19, Some("hidden"));
        o.status = Some(22);
        assert!(is(other_pose(&s, &o, 3 * OTHER_STEP_MS), s.poses.hidden.as_ref().unwrap(), 3));
        // The bit alone, with a carry pose named: still hidden.
        let mut o = other(2 + 4 * 18, Some("carrystill"));
        o.status = Some(HIDDEN_STATUS);
        assert!(is(other_pose(&s, &o, 0), s.poses.hiddenstill.as_ref().unwrap(), 0));
        // Out again: the bit is clear, the carry pose is back.
        o.status = Some(0x10);
        assert!(!other_hidden(&s, &o));
        assert!(is(other_pose(&s, &o, 0), s.poses.carrystill.as_ref().unwrap(), 0));
    }

    #[test]
    fn hidden_without_its_ganis_draws_no_figure() {
        let mut s = session();
        s.set_pose_gani(PoseKind::CarryStill, gani(1));
        let mut o = other(2 + 4 * 18, Some("hiddenstill"));
        o.status = Some(22);
        assert!(other_hidden(&s, &o));
        assert!(other_pose(&s, &o, 0).is_none(), "not carrystill, not idle");
    }

    #[test]
    fn the_bit_alone_follows_the_builds_hide_pose() {
        let mut s = session();
        s.set_pose_gani(PoseKind::CarryStill, gani(1));
        let mut o = other(2 + 4 * 18, None);
        o.status = Some(HIDDEN_STATUS);
        s.set_mannerisms(crate::protocol::MANNER_139);
        assert!(!other_hidden(&s, &o), "1.39 keeps the carry poses");
        assert!(is(other_pose(&s, &o, 0), s.poses.carrystill.as_ref().unwrap(), 0));
        s.set_mannerisms(crate::protocol::MANNER_222);
        assert!(other_hidden(&s, &o));
    }

    #[test]
    fn the_dead_bit_wins_over_hiding() {
        let mut s = session();
        s.set_pose_gani(PoseKind::Dead, gani(3));
        s.set_pose_gani(PoseKind::HiddenStill, gani(1));
        let mut o = other(2, Some("hiddenstill"));
        o.status = Some(STATUS_DEAD | HIDDEN_STATUS);
        assert!(is(other_pose(&s, &o, 0), s.poses.dead.as_ref().unwrap(), 2));
    }
}
