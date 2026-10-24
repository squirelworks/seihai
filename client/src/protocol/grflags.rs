//! The `gr.x` and `gr.y` flags a real 2.17 or 2.22 client sets.
//!
//! Source: the lab, 2026-10-24 and 2026-10-25, real clients on a private
//! loopback server (`documentation/gr-flags.md`). The flags carry the
//! player's position as text and go to the server as `PLI_FLAGSET` (18) with
//! `name=value`, so server scripts can read them:
//!
//! * 2.17 sets both once at login, in the post-login frame. A walk was not
//!   captured; nothing more is claimed.
//! * 2.22 sets both at login, and from then on every 50 ms tick in which the
//!   position differs from the last flag, only the axis that changed (`gr.x`
//!   before `gr.y`), in the frame ahead of the walk props. After a server warp
//!   or a level warp the next tick sets the axes that moved.
//! * 2.31 sends no `gr` flags.
//!
//! The value is the position in tiles as text: a whole number has no decimals
//! (`30`), a half has `.5` (`30.5`). On a gmap it is the position in the whole
//! map (`cell * 64 + local`), so it runs on across a crossing: `63.5`, `64`,
//! `64.5`.

/// `PLI_FLAGSET`.
pub const PLI_FLAGSET: u8 = 18;

/// How a build sets the flags. A profile without a mode sets none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrFlagsMode {
    /// Both flags once, at login (2.17).
    LoginOnly,
    /// At login, then whenever the position changes (2.22).
    Tracking,
}

/// A position in tiles as the clients write it into the flag.
pub fn tile_text(v: f32) -> String {
    let half = (v * 2.0).round() / 2.0;
    format!("{half}")
}

/// `PLI_FLAGSET` for one flag.
pub fn flag_message(name: &str, value: &str) -> Vec<u8> {
    let mut m = vec![PLI_FLAGSET + 0x20];
    m.extend(name.bytes());
    m.push(b'=');
    m.extend(value.bytes());
    m.push(b'\n');
    m
}

/// What was last sent, and when to send the flags again.
#[derive(Debug, Clone)]
pub struct GrFlags {
    mode: GrFlagsMode,
    x: Option<String>,
    y: Option<String>,
}

impl GrFlags {
    pub fn new(mode: GrFlagsMode) -> Self {
        Self { mode, x: None, y: None }
    }

    /// The messages for a player now at `(x, y)` in the whole map: the flags
    /// that differ from the ones last sent (all of them the first time).
    /// A [`GrFlagsMode::LoginOnly`] tracker sends only that first time.
    pub fn sync(&mut self, x: f32, y: f32) -> Vec<Vec<u8>> {
        if self.mode == GrFlagsMode::LoginOnly && self.x.is_some() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (name, last, v) in [("gr.x", &mut self.x, x), ("gr.y", &mut self.y, y)] {
            let t = tile_text(v);
            if last.as_deref() != Some(t.as_str()) {
                out.push(flag_message(name, &t));
                *last = Some(t);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(msgs: &[Vec<u8>]) -> Vec<String> {
        msgs.iter().map(|m| String::from_utf8_lossy(&m[1..m.len() - 1]).into_owned()).collect()
    }

    #[test]
    fn values_are_written_as_the_clients_do() {
        assert_eq!(tile_text(30.0), "30");
        assert_eq!(tile_text(30.5), "30.5");
        assert_eq!(tile_text(0.5), "0.5");
        assert_eq!(tile_text(64.5), "64.5");
        assert_eq!(tile_text(58.0), "58");
    }

    #[test]
    fn the_message_is_flagset_with_name_equals_value() {
        // The bytes of the real 2.22 client's `gr.x=30`.
        assert_eq!(flag_message("gr.x", "30"), [18 + 0x20, 0x67, 0x72, 0x2e, 0x78, 0x3d, 0x33, 0x30, b'\n']);
    }

    #[test]
    fn login_sets_both_then_only_what_moved() {
        let mut g = GrFlags::new(GrFlagsMode::Tracking);
        assert_eq!(texts(&g.sync(30.0, 30.5)), ["gr.x=30", "gr.y=30.5"]);
        assert!(g.sync(30.0, 30.5).is_empty());
        assert_eq!(texts(&g.sync(29.5, 30.5)), ["gr.x=29.5"]);
        assert_eq!(texts(&g.sync(29.0, 30.5)), ["gr.x=29"]);
        assert_eq!(texts(&g.sync(29.0, 31.0)), ["gr.y=31"]);
        // a warp that moves both: x first
        assert_eq!(texts(&g.sync(60.0, 30.0)), ["gr.x=60", "gr.y=30"]);
    }

    #[test]
    fn a_gmap_crossing_runs_on_in_whole_map_coordinates() {
        let mut g = GrFlags::new(GrFlagsMode::Tracking);
        g.sync(63.0, 10.0);
        assert_eq!(texts(&g.sync(63.5, 10.0)), ["gr.x=63.5"]);
        assert_eq!(texts(&g.sync(64.0, 10.0)), ["gr.x=64"]);
        assert_eq!(texts(&g.sync(64.5, 10.0)), ["gr.x=64.5"]);
    }

    #[test]
    fn login_only_sends_once() {
        let mut g = GrFlags::new(GrFlagsMode::LoginOnly);
        assert_eq!(g.sync(30.0, 30.5).len(), 2);
        assert!(g.sync(31.0, 30.5).is_empty());
        assert!(g.sync(60.0, 10.0).is_empty());
    }
}
