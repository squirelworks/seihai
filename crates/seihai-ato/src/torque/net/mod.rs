//! The Torque UDP link of a Graal3D session, from the client side.
//!
//! Built from the clean-room note "The Torque UDP session, start to first
//! ghost" (Graal_analysis `torque/udp-session.md`) and the open Torque net
//! layer (MIT). No decompiler output and no binary addresses are used.
//!
//! So far this is the bit stream every UDP packet is written in
//! ([`bits`]) and its string coding ([`huffman`]). Nothing here opens a
//! socket yet.

pub mod bits;
pub mod events;
pub mod handshake;
pub mod huffman;
pub mod link;
