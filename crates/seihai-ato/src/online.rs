//! Stage 2: log in to a game server as a 3.x to 5.x build, read what it sends with
//! the core readers, keep the level, and log it. Nothing is drawn.
//!
//! The login is core's (`seihai_core::net::login_gserver`): the 3.04 and
//! 3.153 profiles already carry the login token, the bundled first frame
//! and the props the lab run of the real 3.04 showed. Once the server's
//! login burst is in, Ato sends the frame the real client answers with,
//! as `seihai-mae` does: core's `protocol::login::LoginBurst` (3.x: the
//! file checks and props once the board is in) or, for a 4.x build,
//! `protocol::lobby::LobbyBurst` (idle.gani, the PM and lister setup, 46,
//! the window list and the own props with the nickname). File replies are
//! kept by core's `FileSession`.
//!
//! A server NPC's GS2 script (`PLO_NPCBYTECODE`, 131, which 3.153 handles
//! and 3.04 does not) is kept by core's session state, read with
//! `scripting::gs2::Script::parse` when the level is logged, and logged
//! with its opcodes. It is not run.
//!
//! A server warp (`PLO_SERVERWARP`, 178) is followed as the real clients
//! do: leave this server and log in again at the host and port it names,
//! with a new key and the same account. The 4.x and 5.x builds need it:
//! they log in first to a game server named `Login` and play only after
//! its warp (`seihai_core::protocol::lobby`).
//!
//! Under the level line come the fields the server sent with it (board,
//! links, chests, signs, the player's props and each server NPC's props,
//! [`crate::fields`]). Only builds with a Torque profile log them.

use crate::level::{Arrival, LevelStore};
use seihai_core::assets::{AssetResolver, AssetRoot, Level};
use seihai_core::filecache::FileSession;
use seihai_core::net::{login_gserver_with, random_key, walk_message_for, wantfile_message};
use seihai_core::protocol::lobby::LobbyBurst;
use seihai_core::protocol::login::LoginBurst;
use seihai_core::protocol::walkcadence::WalkCadence;
use seihai_core::protocol::{ClientVersion, SessionState};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How often the loop looks while a post-login frame is owed, so a frame
/// due 240 ms after the level goes out on time.
const BURST_TICK: Duration = Duration::from_millis(20);

/// What the start window (or the command line) hands over for a login.
/// The nickname and language are not sent yet: the real 3.04 sends them in
/// its post-login props, which Stage 2 leaves out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Login {
    pub account: String,
    pub password: String,
    pub nick: String,
    pub language: String,
}

/// How long the server must stay quiet after a level is stored before it
/// is logged: its NPCs and the player's props can come in later frames.
const SETTLE: Duration = Duration::from_millis(300);

/// Log in at `addr` and log every level the server sends to `out`, until
/// the server closes the connection or, with `first_level_only`, the first
/// level is stored. `assets` is an asset folder whose levels may stand in
/// for the board at the server's mod time. `shown` gets each level as it
/// is logged (the game view draws it).
pub fn run(
    addr: &str,
    version: ClientVersion,
    login: &Login,
    assets: Option<PathBuf>,
    first_level_only: bool,
    out: &mut impl Write,
    shown: &mut dyn FnMut(&Level, &Arrival),
) -> Result<(), String> {
    run_with_pose(
        addr,
        version,
        login,
        assets,
        first_level_only,
        out,
        shown,
        None,
    )
}

/// Where the player stands and faces, as the game view last set it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    /// 0 up, 1 left, 2 down, 3 right.
    pub dir: u8,
    pub walking: bool,
}

/// The game view writes the pose here; the network thread reads it each
/// 50 ms tick and sends the walk props.
pub type SharedPose = Arc<Mutex<Option<Pose>>>;

/// One walk tick, as the real clients run it.
const WALK_TICK: Duration = Duration::from_millis(50);

/// Sends the player's walk props on the build's own terms: core's
/// [`WalkCadence`] for the builds whose walk was captured (3.04 and the 4.x
/// builds with recorded after-login messages: changed props only, GANI
/// `walk`/`idle`), otherwise the whole walk message
/// ([`walk_message_for`]: X2/Y2 with precise positions, X/Y otherwise) at
/// most once a tick when the pose changed, as `seihai-mae` does for those
/// builds.
struct WalkSender {
    version: ClientVersion,
    cadence: Option<WalkCadence>,
    last: Option<Pose>,
    ganis: bool,
}

impl WalkSender {
    fn new(version: ClientVersion) -> Self {
        WalkSender {
            version,
            cadence: WalkCadence::for_version(version),
            last: None,
            ganis: version.profile().ui.ganis,
        }
    }

    /// The message for this tick, if any.
    fn tick(&mut self, pose: Pose) -> Option<Vec<u8>> {
        if let Some(wc) = self.cadence.as_mut() {
            let gani = self
                .ganis
                .then_some(if pose.walking { "walk" } else { "idle" });
            let st = wc.state(pose.x, pose.y, pose.dir, gani);
            return wc.tick(st);
        }
        let first = self.last.is_none();
        let same = self
            .last
            .is_some_and(|l| (l.x, l.y, l.dir) == (pose.x, pose.y, pose.dir));
        self.last = Some(pose);
        // The first pose is where the server put the player: nothing to say.
        if first || same {
            return None;
        }
        Some(walk_message_for(self.version, pose.x, pose.y, pose.dir))
    }
}

/// As [`run`], with the game view's pose: the walk is sent to the server.
#[allow(clippy::too_many_arguments)]
pub fn run_with_pose(
    addr: &str,
    version: ClientVersion,
    login: &Login,
    assets: Option<PathBuf>,
    first_level_only: bool,
    out: &mut impl Write,
    shown: &mut dyn FnMut(&Level, &Arrival),
    pose: Option<SharedPose>,
) -> Result<(), String> {
    let resolver = assets.map(|d| AssetResolver::new(AssetRoot::new(d), version.profile()));
    let local = |file: &str, mod_time: Option<u32>| -> Option<Level> {
        Level::load(resolver.as_ref()?.level_at_mod_time(file, mod_time)?).ok()
    };
    let file_resolver = resolver
        .clone()
        .unwrap_or_else(|| AssetResolver::empty(version.profile()));
    let mut files = FileSession::new(file_resolver, false);
    let mut addr = addr.to_string();
    let mut first_session = true;
    let mut server_name: Option<String> = None;
    'servers: loop {
        let mut s = login_gserver_with(
            &addr,
            version,
            random_key(),
            &login.account,
            &login.password,
            false,
            first_session,
            wantfile_message,
        )
        .map_err(|e| e.to_string())?;
        first_session = false;
        for w in &s.warnings {
            let _ = writeln!(out, "warning: {w}");
        }
        let _ = writeln!(
            out,
            "logged in at {addr} as {} ({})",
            login.account,
            version.label()
        );
        let frames = s.conn.spawn_reader().map_err(|e| e.to_string())?;
        files.new_connection();
        let mut login_burst = LoginBurst::for_version(version);
        let mut lobby_burst = LobbyBurst::for_version(version);
        let title = version.profile().window_title(server_name.as_deref());
        let mut st = SessionState::for_version(version);
        let mut walker = WalkSender::new(version);
        let mut walk_clock = Instant::now();
        let mut store = LevelStore::default();
        // A level was stored and has not been logged yet.
        let mut owed = false;
        let mut owed_at: Option<Instant> = None;
        loop {
            let burst_owed = login_burst.as_ref().is_some_and(|b| b.pending())
                || lobby_burst.as_ref().is_some_and(|b| b.pending());
            let mut wait = match (owed, burst_owed) {
                (_, true) => BURST_TICK,
                (true, false) => SETTLE,
                (false, false) => Duration::from_secs(3600),
            };
            if pose.is_some() {
                wait = wait.min(WALK_TICK);
            }
            // The time `owed` started, so the settle wait is not cut short
            // by the burst tick.
            let settle_until = owed_at.map(|t| t + SETTLE);
            match frames.recv_timeout(wait) {
                Ok(Ok(body)) => {
                    st.apply_body(&body);
                    if let Some(w) = st.server_warp.take() {
                        if owed {
                            log_level(version, &store, &st, out, shown);
                        }
                        let _ = s.conn.shutdown();
                        let name = w.display_name();
                        let Some(to) = w.address() else {
                            return Err(format!(
                                "server warp to {name:?} has no usable host and port"
                            ));
                        };
                        let _ = writeln!(out, "server warp to {name:?}: {to}");
                        addr = to;
                        server_name = Some(name);
                        continue 'servers;
                    }
                    let before = st.current_level_name();
                    // A new level ends the one owed: log it as it stood.
                    if owed
                        && store.level_name().is_some_and(|n| {
                            before.as_deref().is_none_or(|b| !b.eq_ignore_ascii_case(n))
                        })
                    {
                        log_level(version, &store, &st, out, shown);
                        owed = false;
                    }
                    if store.update(&mut st, local) {
                        owed = true;
                        owed_at = Some(Instant::now());
                    }
                    if st.current_level_name().is_some() {
                        let now = Instant::now();
                        if let Some(b) = login_burst.as_mut() {
                            b.note_level(now);
                        }
                        if let Some(b) = lobby_burst.as_mut() {
                            b.note_level(now);
                        }
                    }
                    let got = files.process(&mut st);
                    if !got.send.is_empty() {
                        let _ = s.conn.send_messages(&got.send);
                    }
                }
                Ok(Err(e)) => {
                    if owed {
                        log_level(version, &store, &st, out, shown);
                    }
                    let why = st
                        .disconnect_message
                        .take()
                        .unwrap_or_else(|| e.to_string());
                    let _ = writeln!(out, "connection closed: {why}");
                    return Ok(());
                }
                Err(RecvTimeoutError::Timeout) => {
                    if owed && settle_until.is_some_and(|t| Instant::now() >= t) {
                        log_level(version, &store, &st, out, shown);
                        owed = false;
                        owed_at = None;
                        if first_level_only {
                            let _ = s.conn.shutdown();
                            return Ok(());
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
            // The walk, once a tick, after the post-login frame has gone
            // (the real clients send nothing before it).
            let burst_owed = login_burst.as_ref().is_some_and(|b| b.pending())
                || lobby_burst.as_ref().is_some_and(|b| b.pending());
            if let Some(p) = pose.as_ref() {
                if !burst_owed && walk_clock.elapsed() >= WALK_TICK {
                    walk_clock = Instant::now();
                    let current = *p.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(m) = current.and_then(|c| walker.tick(c)) {
                        let _ = s.conn.send_messages(&[m]);
                    }
                }
            }
            // The post-login frame, once it is due (core decides when).
            let now = Instant::now();
            if let Some(b) = login_burst.as_mut() {
                let el = &mut s.enter_level;
                let conn = &mut s.conn;
                let msgs = b.take(
                    &st,
                    &login.account,
                    now,
                    |n| files.need(n),
                    || el.take_login(),
                    Vec::new,
                );
                if !msgs.is_empty() {
                    let _ = conn.send_messages(&msgs);
                    let _ = writeln!(out, "sent the post-login frame ({} messages)", msgs.len());
                }
            }
            if let Some(b) = lobby_burst.as_mut() {
                let el = &mut s.enter_level;
                let frames = b.take(
                    &st,
                    if login.nick.is_empty() {
                        &login.account
                    } else {
                        &login.nick
                    },
                    now,
                    |n| files.need(n),
                    || el.take_login(),
                    &[title.as_str()],
                );
                if !frames.is_empty() {
                    for f in &frames {
                        let _ = s.conn.send_messages(f);
                    }
                    let _ = writeln!(out, "sent the post-login frames ({})", frames.len());
                }
            }
        }
    }
}

fn log_level(
    version: ClientVersion,
    store: &LevelStore,
    st: &SessionState,
    out: &mut impl Write,
    shown: &mut dyn FnMut(&Level, &Arrival),
) {
    if let (Some(a), Some(level)) = (store.arrival(st), store.level.as_ref()) {
        let _ = writeln!(out, "{}", a.line());
        for l in crate::fields::lines(version, level, &a.name, st) {
            let _ = writeln!(out, "{l}");
        }
        for l in a.script_lines() {
            let _ = writeln!(out, "{l}");
        }
        shown(level, &a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(x: f32, dir: u8, walking: bool) -> Pose {
        Pose {
            x,
            y: 30.0,
            dir,
            walking,
        }
    }

    /// 3.153 has X2/Y2 and no captured cadence: the whole walk message,
    /// once the pose changes from where the server put the player.
    #[test]
    fn a_build_without_a_cadence_sends_its_walk_message_on_change() {
        let mut w = WalkSender::new(ClientVersion::V3153);
        assert!(w.cadence.is_none());
        assert_eq!(w.tick(pose(30.0, 2, false)), None);
        assert_eq!(w.tick(pose(30.0, 2, false)), None);
        let m = w.tick(pose(30.5, 3, true)).unwrap();
        assert_eq!(m, walk_message_for(ClientVersion::V3153, 30.5, 30.0, 3));
        assert_eq!(w.tick(pose(30.5, 3, true)), None);
    }

    /// 3.04 and 4.04r2 walk on core's cadence: nothing on the first tick,
    /// then the changed props.
    #[test]
    fn the_captured_builds_walk_on_cores_cadence() {
        for v in [ClientVersion::V304, ClientVersion::V404R2] {
            let mut w = WalkSender::new(v);
            assert!(w.cadence.is_some(), "{}", v.label());
            assert_eq!(w.tick(pose(30.0, 2, false)), None, "{}", v.label());
            let mut sent = 0;
            for i in 1..=4 {
                sent += w.tick(pose(30.0 + 0.5 * i as f32, 3, true)).is_some() as u32;
            }
            assert!(sent >= 1, "{}", v.label());
        }
    }
}
