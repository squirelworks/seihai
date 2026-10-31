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

/// Scales RGB `src` (`sw` × `sh`) to `dst` (`dw` × `dh`, `0RGB` words)
/// by nearest neighbour, picking the same source pixels as minifb's
/// `image_resize_linear` (10-bit fixed-point steps), so a frame looks the
/// same through either path. A destination row that repeats the source
/// row of the row above (every second row at 2x) is copied, not rebuilt.
pub fn scale_rgb_into(src: &[u8], sw: usize, sh: usize, dst: &mut [u32], dw: usize, dh: usize) {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 || src.len() < sw * sh * 3 || dst.len() < dw * dh {
        return;
    }
    let step_x = ((sw as f32 / dw as f32) * 1024.0) as usize;
    let step_y = ((sh as f32 / dh as f32) * 1024.0) as usize;
    let xs: Vec<usize> = (0..dw).map(|j| ((j * step_x) >> 10).min(sw - 1) * 3).collect();
    let mut prev: Option<usize> = None;
    for i in 0..dh {
        let sy = ((i * step_y) >> 10).min(sh - 1);
        let (done, rest) = dst.split_at_mut(i * dw);
        let row = &mut rest[..dw];
        if prev == Some(sy) {
            row.copy_from_slice(&done[(i - 1) * dw..]);
            continue;
        }
        let line = &src[sy * sw * 3..(sy + 1) * sw * 3];
        for (d, &x) in row.iter_mut().zip(&xs) {
            *d = (u32::from(line[x]) << 16) | (u32::from(line[x + 1]) << 8) | u32::from(line[x + 2]);
        }
        prev = Some(sy);
    }
}

/// Frames sent through MIT-SHM shared memory (softbuffer's X11 backend)
/// into the window minifb opened, instead of minifb's `XPutImage` over the
/// socket. Only for a minifb window on X11: on Wayland minifb already hands
/// frames over in `wl_shm` buffers. softbuffer falls back to a plain
/// `PutImage` by itself when the server has no MIT-SHM.
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
pub struct ShmWindow {
    surface: softbuffer::Surface<Handles, Handles>,
    // Kept alive for the surface.
    _context: softbuffer::Context<Handles>,
}

/// The raw X11 handles of minifb's window. The window has to outlive the
/// [`ShmWindow`] made from them: drop that first.
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
#[derive(Clone, Copy)]
pub struct Handles {
    window: raw_window_handle::RawWindowHandle,
    display: raw_window_handle::RawDisplayHandle,
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
impl raw_window_handle::HasWindowHandle for Handles {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: `ShmWindow::new` documents that the window outlives it.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.window) })
    }
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
impl raw_window_handle::HasDisplayHandle for Handles {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: as above.
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(self.display) })
    }
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]
impl ShmWindow {
    /// The shared-memory path for `window`, if it is an X11 window and
    /// `SEIHAI_NO_SHM` is not set. `window` must outlive the result.
    pub fn new(window: &minifb::Window) -> Option<Self> {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
        if std::env::var_os("SEIHAI_NO_SHM").is_some() {
            return None;
        }
        let handles = Handles { window: window.window_handle().ok()?.as_raw(), display: window.display_handle().ok()?.as_raw() };
        if !matches!(handles.window, RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) {
            return None;
        }
        let context = softbuffer::Context::new(handles).map_err(|e| eprintln!("shared-memory window: {e}; using minifb's")).ok()?;
        let surface = softbuffer::Surface::new(&context, handles).map_err(|e| eprintln!("shared-memory window: {e}; using minifb's")).ok()?;
        Some(Self { surface, _context: context })
    }

    /// Scales the RGB frame (`width` × `height`) to the window's `size` and
    /// shows it.
    pub fn present(&mut self, pixels: &[u8], width: usize, height: usize, size: (usize, usize)) -> Result<(), softbuffer::SoftBufferError> {
        use std::num::NonZeroU32;
        let (Some(w), Some(h)) = (NonZeroU32::new(size.0 as u32), NonZeroU32::new(size.1 as u32)) else {
            return Ok(());
        };
        self.surface.resize(w, h)?;
        let mut buf = self.surface.buffer_mut()?;
        scale_rgb_into(pixels, width, height, &mut buf, size.0, size.1);
        buf.present()
    }
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

    /// minifb's `image_resize_linear`, word for word.
    fn minifb_resize(src: &[u32], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u32> {
        let step_x = ((sw as f32 / dw as f32) * 1024.0) as i32;
        let step_y = ((sh as f32 / dh as f32) * 1024.0) as i32;
        let mut out = Vec::new();
        let mut fy = 0i32;
        for _ in 0..dh {
            let y = (fy >> 10) as usize * sw;
            let mut fx = 0i32;
            for _ in 0..dw {
                out.push(src[y + (fx >> 10) as usize]);
                fx += step_x;
            }
            fy += step_y;
        }
        out
    }

    #[test]
    fn scaling_picks_the_pixels_minifb_picks() {
        let (sw, sh) = (37, 23);
        let rgb: Vec<u8> = (0..sw * sh * 3).map(|i| (i * 7 % 251) as u8).collect();
        let mut words = Vec::new();
        rgb_to_0rgb(&rgb, &mut words);
        for (dw, dh) in [(74, 46), (37, 23), (100, 51), (50, 90), (20, 11), (111, 69)] {
            let mut dst = vec![0u32; dw * dh];
            scale_rgb_into(&rgb, sw, sh, &mut dst, dw, dh);
            assert_eq!(dst, minifb_resize(&words, sw, sh, dw, dh), "{dw}x{dh}");
        }
    }

    #[test]
    fn scaling_ignores_a_short_buffer() {
        let mut dst = vec![5u32; 4];
        scale_rgb_into(&[1, 2, 3], 2, 2, &mut dst, 2, 2);
        assert_eq!(dst, vec![5; 4]);
    }
}
