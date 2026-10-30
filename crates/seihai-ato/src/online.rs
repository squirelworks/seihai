//! Stage 2: log in to a game server as a 3.x build, read what it sends with
//! the core readers, keep the level, and log it. Nothing is drawn.
//!
//! The login is core's (`seihai_core::net::login_gserver`): the 3.04 and
//! 3.153 profiles already carry the login token, the bundled first frame
//! and the props the lab run of the real 3.04 showed. The post-login frame
//! the real client sends once the board is in (file checks and props,
//! `protocol::login::LoginBurst`) is not sent yet.

use crate::level::{Arrival, LevelStore};
use seihai_core::assets::{AssetResolver, AssetRoot, Level};
use seihai_core::net::{login_gserver, random_key};
use seihai_core::protocol::{ClientVersion, SessionState};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

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
    let resolver = assets.map(|d| AssetResolver::new(AssetRoot::new(d), version.profile()));
    let local = |file: &str, mod_time: Option<u32>| -> Option<Level> {
        Level::load(resolver.as_ref()?.level_at_mod_time(file, mod_time)?).ok()
    };
    let mut s = login_gserver(addr, version, random_key(), &login.account, &login.password, false).map_err(|e| e.to_string())?;
    for w in &s.warnings {
        let _ = writeln!(out, "warning: {w}");
    }
    let _ = writeln!(out, "logged in at {addr} as {} ({})", login.account, version.label());
    let frames = s.conn.spawn_reader().map_err(|e| e.to_string())?;
    let mut st = SessionState::for_version(version);
    let mut store = LevelStore::default();
    // A level was stored and has not been logged yet.
    let mut owed = false;
    loop {
        let wait = if owed { SETTLE } else { Duration::from_secs(3600) };
        match frames.recv_timeout(wait) {
            Ok(Ok(body)) => {
                st.apply_body(&body);
                let before = st.current_level_name();
                // A new level ends the one owed: log it as it stood.
                if owed && store.level_name().is_some_and(|n| before.as_deref().is_none_or(|b| !b.eq_ignore_ascii_case(n))) {
                    log_level(&store, &st, out, shown);
                    owed = false;
                }
                owed |= store.update(&mut st, local);
            }
            Ok(Err(e)) => {
                if owed {
                    log_level(&store, &st, out, shown);
                }
                let why = st.disconnect_message.take().unwrap_or_else(|| e.to_string());
                let _ = writeln!(out, "connection closed: {why}");
                return Ok(());
            }
            Err(RecvTimeoutError::Timeout) => {
                if owed {
                    log_level(&store, &st, out, shown);
                    owed = false;
                    if first_level_only {
                        let _ = s.conn.shutdown();
                        return Ok(());
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

fn log_level(store: &LevelStore, st: &SessionState, out: &mut impl Write, shown: &mut dyn FnMut(&Level, &Arrival)) {
    if let (Some(a), Some(level)) = (store.arrival(st), store.level.as_ref()) {
        let _ = writeln!(out, "{}", a.line());
        shown(level, &a);
    }
}
