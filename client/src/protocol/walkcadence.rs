//! How the real 2.22 client sends its own walk and sword props.
//!
//! Captures (lab `gmap_cross` and `door_warp`, and the live Graal the
//! Adventure run in Graal_analysis `live-runs/2gta-222-2026-10-30/`):
//!
//! * at most one `PLI_PLAYERPROPS` per 100 ms, carrying only the props that
//!   changed since the last one, in ascending id order;
//! * the first step: `GANI` (`walk`), the axis that moved, and `SPRITE` when
//!   the direction changed; the next steps: the moving axis alone, so one
//!   message per tile at the 0.5 tile per 50 ms walk;
//! * a stop on a half tile: that position in the next 100 ms slot, then
//!   `GANI` (`idle`) in the one after;
//! * a sword swing: `GANI` (`sword`) with no `SPRITE`, in the same frame as
//!   the `PLI_BOARDMODIFY` of the bush it cuts.
//!
//! The 2.31 client sends `X2`/`Y2` every 50 ms instead, and is not handled
//! here.

use super::props::encode_pos_classic;
use super::ClientVersion;

/// The 2.22 client's shortest gap between two of its own props messages,
/// in 50 ms ticks.
pub const INTERVAL_TICKS: u32 = 2;

/// Ticks the GANI stays `walk` after the last step before `idle` is sent:
/// in both captures `idle` went out two ticks after the last step.
pub const IDLE_AFTER_TICKS: u32 = 2;

/// What the player shows, as the props would carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalkState {
    /// `X` and `Y` in half tiles.
    pub x: u8,
    pub y: u8,
    pub sprite: u8,
    /// The GANI name, when the pose is one this sender names (`walk`,
    /// `idle`, `sword`). `None` leaves the GANI prop alone.
    pub gani: Option<&'static str>,
}

impl WalkState {
    pub fn new(x: f32, y: f32, sprite: u8, gani: Option<&'static str>) -> Self {
        Self { x: encode_pos_classic(x), y: encode_pos_classic(y), sprite, gani }
    }
}

/// The 2.22 props sender. [`tick`](Self::tick) is called once per 50 ms
/// tick; it holds what was last sent and the tick counts it needs.
#[derive(Debug, Clone)]
pub struct WalkCadence {
    sent: Option<WalkState>,
    tick: u32,
    sent_tick: Option<u32>,
    pos: (u8, u8),
    moved_tick: u32,
}

impl WalkCadence {
    /// The builds this sender is for: 2.22 only.
    pub fn for_version(v: ClientVersion) -> Option<Self> {
        (v == ClientVersion::V222).then_some(Self { sent: None, tick: 0, sent_tick: None, pos: (0, 0), moved_tick: 0 })
    }

    /// One 50 ms tick: the props message to send in this tick's frame, if
    /// anything changed since the last one and two ticks have passed since
    /// it. The first call only records the state: the client sends nothing
    /// until the player does something.
    pub fn tick(&mut self, cur: WalkState) -> Option<Vec<u8>> {
        self.tick = self.tick.wrapping_add(1);
        let n = self.tick;
        if (cur.x, cur.y) != self.pos {
            self.pos = (cur.x, cur.y);
            self.moved_tick = n;
        }
        let Some(last) = self.sent else {
            self.sent = Some(cur);
            return None;
        };
        // A stop keeps `walk` for two ticks after the last step.
        let settling = last.gani == Some("walk") && n.wrapping_sub(self.moved_tick) < IDLE_AFTER_TICKS;
        let gani = cur.gani.filter(|g| last.gani != Some(*g) && !(settling && *g == "idle"));
        if gani.is_none() && cur.x == last.x && cur.y == last.y && cur.sprite == last.sprite {
            return None;
        }
        if self.sent_tick.is_some_and(|t| n.wrapping_sub(t) < INTERVAL_TICKS) {
            return None;
        }
        let mut m = vec![2 + 0x20];
        if let Some(g) = gani {
            m.push(10 + 0x20);
            m.push(g.len() as u8 + 0x20);
            m.extend(g.as_bytes());
        }
        if cur.x != last.x {
            m.extend([15 + 0x20, cur.x.wrapping_add(0x20)]);
        }
        if cur.y != last.y {
            m.extend([16 + 0x20, cur.y.wrapping_add(0x20)]);
        }
        if cur.sprite != last.sprite {
            m.extend([17 + 0x20, cur.sprite.wrapping_add(0x20)]);
        }
        m.push(b'\n');
        self.sent = Some(WalkState { gani: gani.or(last.gani), ..cur });
        self.sent_tick = Some(n);
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(m: &[u8]) -> String {
        String::from_utf8_lossy(&m[1..m.len() - 1]).into_owned()
    }

    fn hx(v: f32) -> char {
        ((v * 2.0) as u8 + 0x20) as char
    }

    /// Run `ticks` (position, gani) through a sender that starts idle at
    /// `start`, facing down; the result is (tick, props) of each message.
    fn run(start: f32, ticks: &[(f32, &'static str)], sprite: u8) -> Vec<(usize, String)> {
        let mut c = WalkCadence::for_version(ClientVersion::V222).unwrap();
        assert_eq!(c.tick(WalkState::new(start, 30.0, 2, Some("idle"))), None);
        // Long idle first, as in both captures.
        for _ in 0..10 {
            assert_eq!(c.tick(WalkState::new(start, 30.0, 2, Some("idle"))), None);
        }
        ticks.iter().enumerate().filter_map(|(i, &(x, g))| c.tick(WalkState::new(x, 30.0, sprite, Some(g))).map(|m| (i, props(&m)))).collect()
    }

    /// Lab 2026-10-30, real 2.22, right from 30 for eleven steps: walk +
    /// X + SPRITE at once, then X every second tick, idle two ticks after
    /// the last step.
    #[test]
    fn the_lab_walk_right() {
        let mut t: Vec<(f32, &str)> = (1..=11).map(|i| (30.0 + 0.5 * i as f32, "walk")).collect();
        t.extend([(35.5, "idle"), (35.5, "idle"), (35.5, "idle")]);
        assert_eq!(
            run(30.0, &t, 3),
            vec![
                (0, format!("*$walk/{}1#", hx(30.5))),
                (2, format!("/{}", hx(31.5))),
                (4, format!("/{}", hx(32.5))),
                (6, format!("/{}", hx(33.5))),
                (8, format!("/{}", hx(34.5))),
                (10, format!("/{}", hx(35.5))),
                (12, "*$idle".to_string()),
            ]
        );
    }

    /// Graal the Adventure: the last step on a tick between two sends; the
    /// position goes in the next slot, idle in the one after.
    #[test]
    fn the_gta_walk_stops_between_two_sends() {
        let mut t: Vec<(f32, &str)> = (0..8).map(|i| (42.0 + 0.5 * i as f32, "walk")).collect();
        t.extend([(45.5, "idle"), (45.5, "idle"), (45.5, "idle"), (45.5, "idle")]);
        assert_eq!(
            run(41.5, &t, 3),
            vec![
                (0, format!("*$walk/{}1#", hx(42.0))),
                (2, format!("/{}", hx(43.0))),
                (4, format!("/{}", hx(44.0))),
                (6, format!("/{}", hx(45.0))),
                (8, format!("/{}", hx(45.5))),
                (10, "*$idle".to_string()),
            ]
        );
    }

    #[test]
    fn a_sword_swing_sends_its_gani_without_a_sprite() {
        assert_eq!(run(30.0, &[(30.0, "sword"), (30.0, "sword")], 2), vec![(0, "*%sword".to_string())]);
        assert!(WalkCadence::for_version(ClientVersion::V231).is_none());
    }
}
