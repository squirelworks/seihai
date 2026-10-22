//! Seihai — open Graal Reborn-compatible client

pub mod assets;
pub mod gui;
pub mod net;
pub mod offline;
pub mod protocol;
pub mod render;
pub mod scripting;
pub mod settings;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
