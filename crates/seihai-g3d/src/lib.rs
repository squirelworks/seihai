//! Seihai G3D: a client for Graal3D, the February 2000 3D side project.
//!
//! Graal3D has its own server and its own wire, unrelated to the 2D
//! gservers and to the Torque 3D of the 3.x–5.x clients. The facts come
//! from Graal_analysis `graal3d/` (`protocol.md`, `formats.md`,
//! `client-behaviour.md`), which were checked against the real client and
//! server in a loopback lab.
//!
//! - [`codec`]: the `+32` text codec and the zlib frames.
//! - [`props`]: the player property table.
//! - [`session`]: the TCP session and the peer-to-peer UDP socket.
//! - [`xfile`]: the DirectX `.x` text meshes that levels and models use.

pub mod codec;
pub mod props;
pub mod session;
pub mod xfile;
