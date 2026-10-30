//! Seihai Ato — the Torque / 3.x client line.
//!
//! Stage 1: which 3.x build was asked for ([`args`]), what it says on the
//! Graal wire that 2.31 does not ([`wire`]), its Torque connect profile
//! ([`torque`]) and the start window ([`start_window`]).
//!
//! Stage 2: log in to a game server ([`online`]), read what it sends with
//! the core readers, and keep and log the level ([`level`]). Nothing is
//! drawn yet.

pub mod args;
pub mod level;
pub mod online;
pub mod start_window;
pub mod torque;
pub mod wire;
