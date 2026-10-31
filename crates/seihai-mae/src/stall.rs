//! A watchdog for the game loop.
//!
//! The loop marks each part of a pass as it finishes ([`StallWatch::mark`]).
//! A thread of its own checks the time since the last mark, and when it is
//! over the limit it reports the last part that finished, so the stuck part
//! is the one that follows it. It reports again when the loop moves on, with
//! how long it was stuck.
//!
//! Seihai 4b45cd3 on Windows froze after its first frames online (lab,
//! 2026-10-31, `documentation/cross-laptop-2026-10-31.md`): no input, no
//! client bytes, almost no CPU. That points at a blocking call, but the
//! freeze did not happen on Linux. A frozen loop says nothing on its own, so
//! this watch is how the next run shows where it stopped.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long a pass may take before it is reported.
pub const STALL_LIMIT: Duration = Duration::from_secs(3);

/// How often the watch thread looks.
const CHECK_EVERY: Duration = Duration::from_millis(250);

struct Shared {
    start: Instant,
    /// Milliseconds from `start` to the last mark.
    last_ms: AtomicU64,
    /// The part the last mark finished.
    part: Mutex<&'static str>,
    /// The loop is waiting on the user (a modal window such as the F5
    /// level picker): not a stall.
    paused: AtomicBool,
}

/// The loop's side of the watch. Cloning it shares the same watch.
#[derive(Clone)]
pub struct StallWatch {
    shared: Arc<Shared>,
}

impl StallWatch {
    /// Start the watch thread. Each report goes to `report` as one line.
    /// The thread ends when the last `StallWatch` is dropped.
    pub fn spawn(limit: Duration, mut report: impl FnMut(&str) + Send + 'static) -> Self {
        let shared = Arc::new(Shared { start: Instant::now(), last_ms: AtomicU64::new(0), part: Mutex::new("start"), paused: AtomicBool::new(false) });
        let weak = Arc::downgrade(&shared);
        let _ = std::thread::Builder::new().name("seihai-stall-watch".into()).spawn(move || {
            // The mark the current report is about, if one was made.
            let mut reported: Option<u64> = None;
            loop {
                std::thread::sleep(CHECK_EVERY.min(limit));
                let Some(s) = weak.upgrade() else { return };
                let last = s.last_ms.load(Ordering::Acquire);
                let now = s.start.elapsed().as_millis() as u64;
                if let Some(r) = reported.filter(|&r| r != last) {
                    report(&format!("main loop: moved on after {:.1} s", (last.saturating_sub(r)) as f64 / 1000.0));
                    reported = None;
                }
                if s.paused.load(Ordering::Acquire) {
                    continue;
                }
                if reported.is_none() && now.saturating_sub(last) >= limit.as_millis() as u64 {
                    let part = *s.part.lock().unwrap_or_else(|e| e.into_inner());
                    report(&format!(
                        "main loop: stuck for {:.1} s after \"{part}\"",
                        now.saturating_sub(last) as f64 / 1000.0
                    ));
                    reported = Some(last);
                }
            }
        });
        Self { shared }
    }

    /// `part` has just finished.
    pub fn mark(&self, part: &'static str) {
        *self.shared.part.lock().unwrap_or_else(|e| e.into_inner()) = part;
        let ms = self.shared.start.elapsed().as_millis() as u64;
        self.shared.last_ms.store(ms, Ordering::Release);
    }

    /// The loop is about to wait on the user (a modal window). Nothing is
    /// reported until [`resume`](Self::resume).
    pub fn pause(&self) {
        self.shared.paused.store(true, Ordering::Release);
    }

    /// The modal window closed: `part` has just finished, and the watch
    /// times from now.
    pub fn resume(&self, part: &'static str) {
        self.mark(part);
        self.shared.paused.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn watch(limit_ms: u64) -> (StallWatch, std::sync::mpsc::Receiver<String>) {
        let (tx, rx) = channel();
        let w = StallWatch::spawn(Duration::from_millis(limit_ms), move |l| {
            let _ = tx.send(l.to_string());
        });
        (w, rx)
    }

    /// A loop that stops after a part is reported once, naming that part,
    /// and again when it goes on.
    #[test]
    fn a_stuck_pass_names_the_last_part_and_then_its_end() {
        let (w, rx) = watch(100);
        w.mark("look images");
        let line = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(line.starts_with("main loop: stuck for"), "{line}");
        assert!(line.ends_with("after \"look images\""), "{line}");
        // Still stuck: no second report.
        assert!(rx.recv_timeout(Duration::from_millis(400)).is_err());
        w.mark("draw");
        let line = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(line.starts_with("main loop: moved on after"), "{line}");
    }

    /// A loop that keeps marking within the limit is never reported.
    #[test]
    fn a_running_loop_is_quiet() {
        let (w, rx) = watch(300);
        let until = Instant::now() + Duration::from_millis(900);
        while Instant::now() < until {
            w.mark("present");
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(rx.try_recv().is_err());
    }

    /// Time spent paused (a modal window open) is not a stall.
    #[test]
    fn a_paused_loop_is_quiet() {
        let (w, rx) = watch(100);
        w.mark("scripts");
        w.pause();
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
        w.resume("level picker");
        assert!(rx.recv_timeout(Duration::from_millis(60)).is_err());
        drop(w);
    }

    /// Dropping the watch ends its thread: the report sender goes away.
    #[test]
    fn the_thread_ends_with_the_watch() {
        let (w, rx) = watch(50);
        drop(w);
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                _ if Instant::now() > until => panic!("watch thread still running"),
                _ => {}
            }
        }
    }
}
