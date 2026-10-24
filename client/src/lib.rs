//! Seihai — open Graal Reborn-compatible client

pub mod assets;
pub mod dev;
pub mod difftest;
pub mod gmap;
pub mod gui;
pub mod help;
pub mod life;
pub mod net;
pub mod offline;
pub mod protocol;
pub mod render;
pub mod scripting;
pub mod settings;
pub mod ui;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
