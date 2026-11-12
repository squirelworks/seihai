//! The game loop the windowed Seihai clients share.
//!
//! [`game::Game`] is the loop with no window: it takes the levels core's
//! client session sends ([`game::spawn_online`]), walks the player over
//! them, crosses gmap edges and links, plays the walk's sounds through
//! core's sound engine and asks the server for the missing ones.
//! [`render`] draws the level flat through `seihai-draw`, the players from
//! their GANIs ([`chars`]), and [`view::View`] puts that frame in
//! an egui window with the status line, the position and the names.
//!
//! `seihai-ato` (3.x to 5.x) and `seihai-issho` (6.x) both run it. Each
//! keeps its own start, profile and window size.
//!
//! [`start`] is the one start window for every line, the `seihai`
//! command's: pick the version and the server, and hand over to that
//! line's client.
//!
//! [`window::run_native`] opens every Seihai window: OpenGL, else wgpu
//! (Direct3D 12 and its software WARP on Windows).

pub mod chars;
pub mod game;
pub mod render;
pub mod servers;
pub mod start;
pub mod view;
pub mod window;
