//! Named 5.0 differences from the v4 cut.
//!
//! Not a protocol. No byte is read or written. No setter is called.

// Unpacked 5.30 and 5.31 drop graal_set, SimObject and GameConnection.
// That later cut is not these constants. Do not fold it into the 5.0 set.

/// The 5.0 3D protocol check accepts only this value.
pub const PROTOCOL_ACCEPT: u32 = 24;

/// How many `graal_set` names the 5.0.0.7 launcher resolves.
pub const LAUNCHER_SETTERS: u16 = 95;

/// How many `graal_set` names the 5.0.0.7 engine defines.
pub const ENGINE_SETTERS: u16 = 118;
