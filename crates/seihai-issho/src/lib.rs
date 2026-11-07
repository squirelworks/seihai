//! Issho: the v6 client profile.
//!
//! The launcher selects this crate for a 6.x [`seihai_core::protocol::ClientVersion`].
//! Shared protocol facts and file formats stay in `seihai-core`. The Torque
//! line (v3–v5) stays in `seihai-ato`; this crate does not depend on it and
//! does not call it.
//!
//! v7 is named on [`profile::EngineProfile`] and is not selectable. Unity is
//! the expected engine for that era. There is no v7 profile and no Unity code.

pub mod cli;
pub mod profile;
pub mod shell;

pub use profile::{
    is_v6, is_v7_label, select_label, select_version, try_v7, EngineProfile, SelectError,
};
