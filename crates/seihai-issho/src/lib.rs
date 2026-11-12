//! Issho: the v6 client profile.
//!
//! The launcher selects this crate for a 6.x [`seihai_core::protocol::ClientVersion`].
//! Shared protocol facts and file formats stay in `seihai-core`. The Torque
//! line (v3–v5) stays in `seihai-ato`; this crate does not depend on it and
//! does not call it.
//!
//! With the `window` feature (on by default) [`game`] runs the game loop
//! `seihai-ato` runs, from `seihai-play`: it logs in, draws the level
//! flat, walks it with the arrows and plays the walk's sounds.
//!
//! v7 is named on [`profile::EngineProfile`] and is not selectable. Unity is
//! the expected engine for that era. There is no v7 profile and no Unity code.

pub mod app;
pub mod cli;
#[cfg(feature = "window")]
pub mod game;
pub mod profile;
pub mod shell;

pub use profile::{
    is_v6, is_v7_label, select_label, select_version, try_v7, EngineProfile, SelectError,
};
