//! The Torque UDP link of a Graal3D session, from the client side.
//!
//! Built from the clean-room note "The Torque UDP session, start to first
//! ghost" (Graal_analysis `torque/udp-session.md`) and the open Torque net
//! layer (MIT). No decompiler output and no binary addresses are used.
//!
//! [`bits`] and [`huffman`] are the bit stream and string coding every
//! packet uses, [`handshake`] opens the link, [`link`] numbers and acks
//! data packets, [`events`] and [`ghosts`] read and write their contents,
//! [`graal_text`] applies the Graal text sent over BigData,
//! [`scripts3d`] runs the weapon scripts it brings on core's GS2 VM,
//! [`tge`] is the protocol 19 game layer (3.153, 4.04r2),
//! and [`session`] runs it all over a UDP socket (Ato's `--graal3d`).

pub mod bits;
pub mod events;
pub mod ghosts;
pub mod graal_text;
pub mod handshake;
pub mod huffman;
pub mod link;
pub mod scripts3d;
pub mod session;
pub mod tge;
