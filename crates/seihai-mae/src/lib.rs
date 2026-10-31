//! Seihai — open Graal Reborn-compatible client

// The shared modules live in `seihai-core`. They are re-exported under
// their old names so `seihai::protocol`, `crate::offline` and the rest
// still resolve.
pub use seihai_core::{
    arrows, assets, bombs, chatlists, combat, dev, difftest, filecache, gmap, items, life, looks, net, npcart, offline, pose,
    protocol, scripting, servercache, throwing, viewer,
};

pub mod ato;
pub mod gui;
pub mod keys;
pub mod help;
pub mod inventory;
pub mod render;
pub mod settings;
pub mod sound;
pub mod stall;
pub mod ui;
pub mod weaponbar;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
