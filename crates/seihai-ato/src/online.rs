//! Stage 2: log in to a game server as a 3.x to 5.x build, read what it
//! sends with the core readers, keep the level, and log it.
//!
//! The session is core's (`seihai_core::client`), shared with Issho. Ato
//! adds the fields its Torque profiles read under each level line
//! ([`crate::fields`]); only builds with a Torque profile log them.

use crate::level::Arrival;
use seihai_core::assets::Level;
use seihai_core::protocol::ClientVersion;
use std::io::Write;
use std::path::PathBuf;

pub use seihai_core::client::{
    FileAnswer, FileAsks, Hooks, LinkAsk, Live, Login, NpcChange, Other, Pose, ServerJoin,
    SharedBoardMods, SharedNpcChanges, SharedOthers, SharedPose, WorldView,
};

/// Log in at `addr` and log every level the server sends to `out`, with
/// Ato's field lines, until the server closes the connection or, with
/// `first_level_only`, the first level is stored (core's
/// `client::run_with`).
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

/// As [`run`], with the session [`Hooks`]: the Graal3D side is told of
/// each server before its login and fetches its files through the session.
#[allow(clippy::too_many_arguments)]
pub fn run_hooked(
    addr: &str,
    version: ClientVersion,
    login: &Login,
    assets: Option<PathBuf>,
    first_level_only: bool,
    out: &mut impl Write,
    shown: &mut dyn FnMut(&Level, &Arrival),
    hooks: &Hooks,
) -> Result<(), String> {
    seihai_core::client::run_hooked(
        addr,
        version,
        login,
        assets,
        first_level_only,
        out,
        shown,
        None,
        &|level, name, st| crate::fields::lines(version, level, name, st),
        hooks,
    )
}

/// As [`run`], with the game view's [`Live`] state: the walk is sent to the
/// server and the other players come back.
#[allow(clippy::too_many_arguments)]
pub fn run_with_pose(
    addr: &str,
    version: ClientVersion,
    login: &Login,
    assets: Option<PathBuf>,
    first_level_only: bool,
    out: &mut impl Write,
    shown: &mut dyn FnMut(&Level, &Arrival),
    live: Option<Live>,
) -> Result<(), String> {
    seihai_core::client::run_with(
        addr,
        version,
        login,
        assets,
        first_level_only,
        out,
        shown,
        live,
        &|level, name, st| crate::fields::lines(version, level, name, st),
    )
}
