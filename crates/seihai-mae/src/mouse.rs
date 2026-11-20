//! Mouse clicks as the real 2.31 sends them (lab key sweep, Graal_analysis
//! `protocol/keysweep/231.tsv`): every left, right and middle click in the
//! game window, on the player, the ground, an NPC or the HUD alike, sends
//! `PLI_TRIGGERACTION` (38) for NPC 0 at the clicked point with
//! `leftmouse`, `rightmouse` or `middlemouse`, and draws nothing. A double
//! click sends `leftmouse`, then `doublemouse` and `leftmouse` again. The
//! wheel sends nothing.

use std::time::{Duration, Instant};

/// The Windows double-click time and box (`GetDoubleClickTime`,
/// `SM_CXDOUBLECLK`) at their defaults.
const DOUBLE_TIME: Duration = Duration::from_millis(500);
const DOUBLE_BOX: i32 = 2;

/// The mouse buttons from one frame to the next.
#[derive(Debug, Default)]
pub struct Clicks {
    was: [bool; 3],
    last_left: Option<(Instant, (i32, i32))>,
}

impl Clicks {
    /// The actions of this frame's new presses of left, right and middle
    /// (`down`), with the pointer at `at` in window pixels.
    pub fn update(&mut self, down: [bool; 3], at: (i32, i32), now: Instant) -> Vec<&'static str> {
        let mut out = Vec::new();
        let pressed = |i: usize| down[i] && !self.was[i];
        if pressed(0) {
            let double = self.last_left.is_some_and(|(t, p)| {
                now.duration_since(t) <= DOUBLE_TIME
                    && (p.0 - at.0).abs() <= DOUBLE_BOX
                    && (p.1 - at.1).abs() <= DOUBLE_BOX
            });
            if double {
                out.push("doublemouse");
                self.last_left = None;
            } else {
                self.last_left = Some((now, at));
            }
            out.push("leftmouse");
        }
        if pressed(1) {
            out.push("rightmouse");
        }
        if pressed(2) {
            out.push("middlemouse");
        }
        self.was = down;
        out
    }
}

/// The level point (in tiles) under window pixel `at`, with the view's
/// top-left corner at `cam` level pixels.
pub fn level_point(at: (i32, i32), cam: (i32, i32)) -> (f32, f32) {
    let t = 16.0;
    ((cam.0 + at.0) as f32 / t, (cam.1 + at.1) as f32 / t)
}

/// The message 2.31 sends before `rightmouse` when the right click lands
/// on the player's own 48-pixel box: `PLI 80` with the account (it opens
/// the "Profile" window; lab bytes `80:?{c224729}` for that run's account).
pub fn own_profile_message(account: &str) -> Vec<u8> {
    let mut m = vec![80 + 0x20];
    m.extend_from_slice(account.as_bytes());
    m.push(b'\n');
    m
}

/// True when window pixel `at` is in the player's box at `origin`.
pub fn on_player(at: (i32, i32), origin: (i32, i32)) -> bool {
    (origin.0..origin.0 + 48).contains(&at.0) && (origin.1..origin.1 + 48).contains(&at.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicks_match_the_real_2_31() {
        let mut c = Clicks::default();
        let t0 = Instant::now();
        let up = [false; 3];
        assert_eq!(c.update([true, false, false], (10, 10), t0), ["leftmouse"]);
        assert!(c.update(up, (10, 10), t0).is_empty());
        let t1 = t0 + Duration::from_millis(200);
        assert_eq!(
            c.update([true, false, false], (11, 10), t1),
            ["doublemouse", "leftmouse"]
        );
        c.update(up, (11, 10), t1);
        // A third quick click starts a new pair.
        assert_eq!(c.update([true, false, false], (11, 10), t1), ["leftmouse"]);
        c.update(up, (11, 10), t1);
        let late = t1 + Duration::from_secs(2);
        assert_eq!(
            c.update([true, false, false], (11, 10), late),
            ["leftmouse"]
        );
        assert_eq!(
            c.update([true, true, true], (0, 0), late),
            ["rightmouse", "middlemouse"]
        );
    }

    #[test]
    fn a_right_click_on_the_player_asks_for_the_profile() {
        assert_eq!(own_profile_message("c224729"), b"pc224729\n");
        assert!(on_player((330, 250), (320, 240)));
        assert!(!on_player((368, 250), (320, 240)));
    }

    #[test]
    fn the_point_is_in_level_tiles() {
        assert_eq!(level_point((320, 224), (-96, 64)), (14.0, 18.0));
    }
}
