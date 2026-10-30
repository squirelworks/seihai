//! Seihai — open Graal Reborn-compatible client

pub mod assets;
pub mod bombs;
pub mod chatlists;
pub mod combat;
pub mod dev;
pub mod filecache;
pub mod difftest;
pub mod gmap;
pub mod gui;
pub mod keys;
pub mod help;
pub mod inventory;
pub mod items;
pub mod life;
pub mod looks;
pub mod net;
pub mod npcart;
pub mod offline;
pub mod pose;
pub mod protocol;
pub mod render;
pub mod scripting;
pub mod servercache;
pub mod settings;
pub mod sound;
pub mod stall;
pub mod throwing;
pub mod ui;
pub mod viewer;
pub mod weaponbar;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
