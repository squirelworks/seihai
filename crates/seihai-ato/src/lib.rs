//! Seihai Ato — the Torque / 3.x client line.
//!
//! Stage 1: which 3.x build was asked for ([`args`]), what it says on the
//! Graal wire that 2.31 does not ([`wire`]), its Torque connect profile
//! ([`torque`]) and the start window ([`start_window`]).
//!
//! Stage 2: log in to a game server ([`online`]), read what it sends with
//! the core readers, and keep and log the level ([`level`]) with the
//! board's and props' fields ([`fields`]), for builds with a Torque profile.
//!
//! Start asks the listserver for its servers ([`server_list`]) and logs in
//! at the one picked, unless `--online` names a game server.
//!
//! Stage 3: draw the level flat as `seihai-mae` draws its tiles
//! ([`render`]) and walk it with the arrow keys ([`game`]). The loop and
//! the renderer are `seihai-play`'s, shared with Issho.
//!
//! The Torque reimplementation for eras 3–5 lives in `torque::engine`
//! and is part of the default build. `--features era6` leaves it out.
//! The product default follows the selected era: on for v3, v4 and v5,
//! off for v6 and later. The stub era defaults to v3.

pub mod app;
pub mod args;
pub mod fields;
pub mod game;
pub mod level;
pub mod online;
pub mod server_list;
pub mod start_window;
pub mod torque;
pub mod wire;

/// The flat renderer, shared with Issho in `seihai-play`.
pub use seihai_play::render;
