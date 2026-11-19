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
    fn the_point_is_in_level_tiles() {
        assert_eq!(level_point((320, 224), (-96, 64)), (14.0, 18.0));
    }
}
