//! Seihai Ato — the Torque / 3.x client line.
//!
//! Stage 1: which 3.x build was asked for ([`args`]), what it says on the
//! Graal wire that 2.31 does not ([`wire`]), its Torque connect profile
//! ([`torque`]) and the start window ([`start_window`]).
//!
//! Stage 2: log in to a game server ([`online`]), open the Torque UDP socket
//! beside that login ([`torque`]), read what the server sends with the core
//! readers, and keep and log the level ([`level`]) with the board's and
//! props' fields ([`fields`]), for builds with a Torque profile.
//!
//! Stage 3: draw the level flat as `seihai-mae` draws its tiles
//! ([`render`]) and walk it with the arrow keys ([`game`]).
//!
//! The real Torque lineage engine sources live in `vendor/Torque3D` (MIT).
//! [`TORQUE_GAME_ENGINE_VERSION`] is read from that tree at build time.

mod torque_engine_version {
    include!(concat!(env!("OUT_DIR"), "/torque_engine_version.rs"));
}

pub use torque_engine_version::TORQUE_GAME_ENGINE_VERSION;

pub mod args;
pub mod fields;
pub mod game;
pub mod level;
pub mod online;
pub mod render;
pub mod start_window;
pub mod torque;
pub mod wire;

#[cfg(test)]
mod torque_engine_version_tests {
    use super::TORQUE_GAME_ENGINE_VERSION;

    #[test]
    fn submodule_version_is_nonempty() {
        assert!(
            !TORQUE_GAME_ENGINE_VERSION.is_empty(),
            "TORQUE_GAME_ENGINE_VERSION from vendor/Torque3D/Engine/source/app/version.h"
        );
    }
}
