//! Asset loading and export (levels, GANIs, images).

mod crc32;
mod gani;
mod level;
mod objects;
mod png;
mod root;

pub use objects::{ObjectSet, Stamp};
pub use root::AssetRoot;

pub use crc32::crc32;
pub use gani::{Gani, GaniFrame, GaniPlacement, GaniPlayer, GaniSprite};
pub use level::{
    decode_sign_record, decode_sign_text, tile_code, tile_id, tile_src_rect, Baddy, BoardRun, Chest, Level,
    Link, Npc, Sign, Tile, TileTypes,
};
pub use png::{read_png_rgb, read_png_rgba, write_png_indexed, write_png_rgb};
