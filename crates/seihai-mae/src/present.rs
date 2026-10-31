//! Sending frames to the window only when they change.
//!
//! minifb has no damage tracking: each `update_with_buffer` scales the
//! whole buffer to the window in software and sends every pixel to the
//! X server with `XPutImage`. At 60 passes a second that was most of a
//! client's CPU (the stress test of 2026-10-31 measured 25–30 % of a core
//! per client). Most passes draw the same picture as the one before:
//! the scene is only redrawn when something changes, the walk moves every
//! 64 ms and the 2.x game frame is 50 ms. [`Presenter`] keeps the last
//! frame sent and says when a new one differs, so the loop sends only
//! changed frames and otherwise just polls input.

use std::time::{Duration, Instant};

/// Decides which passes send their frame to the window.
#[derive(Default)]
pub struct Presenter {
    /// The RGB pixels of the last frame sent.
    last: Vec<u8>,
    /// The frame and window sizes it was sent at.
    last_size: (usize, usize, usize, usize),
    last_at: Option<Instant>,
    /// Frames sent and passes skipped, for the frame log.
    pub sent: u64,
    pub skipped: u64,
}

impl Presenter {
    /// An unchanged frame is still sent this often. The window has no
    /// backing store, so a part of it that was covered stays blank until
    /// the next frame is sent.
    pub const REFRESH: Duration = Duration::from_millis(250);

    /// Whether the frame with these RGB `pixels` (`width` × `height`) has
    /// to be sent to a window now `window` big. It does when it differs
    /// from the last frame sent, when either size changed, or when the last
    /// one was sent [`Self::REFRESH`] or more ago. A `true` records the
    /// frame as sent.
    pub fn should_send(&mut self, pixels: &[u8], width: usize, height: usize, window: (usize, usize), now: Instant) -> bool {
        let size = (width, height, window.0, window.1);
        let stale = self.last_at.map_or(true, |t| now.saturating_duration_since(t) >= Self::REFRESH);
        if !stale && size == self.last_size && self.last == pixels {
            self.skipped += 1;
            return false;
        }
        self.last.clear();
        self.last.extend_from_slice(pixels);
        self.last_size = size;
        self.last_at = Some(now);
        self.sent += 1;
        true
    }
}

/// Packs RGB bytes into minifb's `0RGB` words, reusing `out`.
pub fn rgb_to_0rgb(pixels: &[u8], out: &mut Vec<u32>) {
    out.clear();
    out.extend(pixels.chunks_exact(3).map(|c| (u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2])));
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: (usize, usize) = (640, 480);

    #[test]
    fn first_frame_is_sent() {
        let mut p = Presenter::default();
        assert!(p.should_send(&[1, 2, 3], 1, 1, W, Instant::now()));
    }

    #[test]
    fn same_frame_is_skipped_until_the_refresh() {
        let mut p = Presenter::default();
        let t = Instant::now();
        let px = vec![7u8; 4 * 3];
        assert!(p.should_send(&px, 2, 2, W, t));
        assert!(!p.should_send(&px, 2, 2, W, t + Duration::from_millis(16)));
        assert!(!p.should_send(&px, 2, 2, W, t + Duration::from_millis(249)));
        assert!(p.should_send(&px, 2, 2, W, t + Presenter::REFRESH));
        assert_eq!((p.sent, p.skipped), (2, 2));
    }

    #[test]
    fn a_changed_pixel_is_sent() {
        let mut p = Presenter::default();
        let t = Instant::now();
        let mut px = vec![0u8; 4 * 3];
        assert!(p.should_send(&px, 2, 2, W, t));
        px[11] = 1;
        assert!(p.should_send(&px, 2, 2, W, t + Duration::from_millis(16)));
        // And that frame is now the one compared against.
        assert!(!p.should_send(&px, 2, 2, W, t + Duration::from_millis(32)));
    }

    #[test]
    fn a_resized_window_or_frame_is_sent() {
        let mut p = Presenter::default();
        let t = Instant::now();
        let px = vec![0u8; 4 * 3];
        assert!(p.should_send(&px, 2, 2, W, t));
        assert!(p.should_send(&px, 2, 2, (800, 600), t));
        // Same bytes, other shape.
        assert!(p.should_send(&px, 4, 1, (800, 600), t));
        assert!(!p.should_send(&px, 4, 1, (800, 600), t));
    }

    #[test]
    fn packs_rgb_into_words() {
        let mut out = vec![99];
        rgb_to_0rgb(&[0x12, 0x34, 0x56, 0xFF, 0x00, 0x80], &mut out);
        assert_eq!(out, vec![0x123456, 0xFF0080]);
    }
}
