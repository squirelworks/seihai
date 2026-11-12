//! The game loop the windowed Seihai clients share.
//!
//! [`game::Game`] is the loop with no window: it takes the levels core's
//! client session sends ([`game::spawn_online`]), walks the player over
//! them, crosses gmap edges and links, plays the walk's sounds through
//! core's sound engine and asks the server for the missing ones.
//! [`render`] draws the level flat, and [`view::View`] puts that frame in
//! an egui window with the status line, the position and the names.
//!
//! `seihai-ato` (3.x to 5.x) and `seihai-issho` (6.x) both run it. Each
//! keeps its own start, profile and window size.

pub mod game;
pub mod render;
pub mod view;
