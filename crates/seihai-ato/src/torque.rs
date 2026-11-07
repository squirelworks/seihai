//! The Torque side of each Graal build: what its `GameConnection` connect
//! request carries and what its connect accept must carry.
//!
//! Data only; nothing here opens a socket. Read statically from the client
//! binaries (Graal_analysis `torque/wire-by-build.md` and
//! `torque/network.md`); only 5.12 was also seen live. Ato runs 3.04 and
//! 3.153; the 4.x and 5.12 rows are here so a later Torque client picks
//! its profile by protocol number in one place.

use seihai_core::protocol::ClientVersion;

/// One Torque connect profile, keyed by the protocol number the client
/// sends twice (current and minimum) and wants back exactly in the accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorqueProfile {
    /// Which builds use it, for messages.
    pub builds: &'static str,
    /// The game string in the connect request.
    pub game: &'static str,
    pub protocol: u32,
    /// Class types listed in Graal's class table at the front of the
    /// accept: 0 for no table (stock Torque), 3 (object, datablock, event)
    /// or 2 (object, event; no datablocks).
    pub class_table_types: u8,
    /// Class ids on the wire are a fixed 12 bits in the server's numbering
    /// (Graal) rather than stock Torque's count-based width.
    pub class_ids_12bit: bool,
    /// Every event and ghost update carries a 14-bit bit length.
    pub length_prefixes: bool,
    /// The Torque game layer (datablocks, `Player`, moves) is present;
    /// `false` for Graal's own `GameObject` model.
    pub tge_game_layer: bool,
}

/// 3.04 Linux: stock TGE 1.x networking.
pub const TGE_9: TorqueProfile = TorqueProfile {
    builds: "3.04",
    game: "Torque Game Engine Demo",
    protocol: 9,
    class_table_types: 0,
    class_ids_12bit: false,
    length_prefixes: false,
    tge_game_layer: true,
};

/// 3.153 Mac and 4.04r2 Windows.
pub const GRAAL3D_19: TorqueProfile = TorqueProfile {
    builds: "3.153, 4.04r2",
    game: "Graal3D",
    protocol: 19,
    class_table_types: 3,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: true,
};

/// 4.20r6 Mac.
pub const GRAAL3D_21: TorqueProfile = TorqueProfile {
    builds: "4.20r6",
    game: "Graal3D",
    protocol: 21,
    class_table_types: 2,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: false,
};

/// 5.12 Windows (also 5.00r7's protocol number).
pub const GRAAL3D_24: TorqueProfile = TorqueProfile {
    builds: "5.12",
    game: "Graal3D",
    protocol: 24,
    class_table_types: 2,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: false,
};

pub const ALL: [TorqueProfile; 4] = [TGE_9, GRAAL3D_19, GRAAL3D_21, GRAAL3D_24];

/// The profile with this protocol number.
pub fn by_protocol(protocol: u32) -> Option<TorqueProfile> {
    ALL.into_iter().find(|p| p.protocol == protocol)
}

/// The profile of a build Ato runs.
pub fn of(v: ClientVersion) -> Option<TorqueProfile> {
    match v {
        ClientVersion::V304 => Some(TGE_9),
        ClientVersion::V3153 => Some(GRAAL3D_19),
        _ => None,
    }
}

/// The Torque UDP port for a 3D server listed on `listed_port`: the
/// listed port + 1, on the same host, beside the Graal TCP connection.
/// (Torque's own default, 28000, only applies to an address with no port.)
pub fn udp_port(listed_port: u16) -> u16 {
    listed_port.wrapping_add(1)
}

/// A server is a 3D server when its listed name starts with `"3 "`.
pub fn is_3d_server(listed_name: &str) -> bool {
    listed_name.starts_with("3 ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocols_are_distinct() {
        for p in ALL {
            assert_eq!(by_protocol(p.protocol), Some(p));
        }
        assert_eq!(by_protocol(12), None);
    }

    #[test]
    fn ato_builds() {
        assert_eq!(of(ClientVersion::V304).unwrap().protocol, 9);
        assert_eq!(of(ClientVersion::V3153).unwrap().protocol, 19);
        assert_eq!(of(ClientVersion::V231), None);
    }

    #[test]
    fn three_d_servers() {
        assert!(is_3d_server("3 Seihai Test"));
        assert!(!is_3d_server("Seihai 3"));
        assert_eq!(udp_port(14800), 14801);
    }
}
