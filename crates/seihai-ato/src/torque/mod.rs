//! The Torque side of each Graal build: what its `GameConnection` connect
//! request carries and what its connect accept must carry.
//!
//! Data only; nothing here opens a socket. Read statically from the client
//! binaries (Graal_analysis `torque/wire-by-build.md` and
//! `torque/network.md`); only 5.12 was also seen live. Ato runs 3.04 and
//! 3.153; the 4.x and 5.12 rows are here so a later Torque client picks
//! its profile by protocol number in one place.
//!
//! The reimplementation for eras 3–5 is `engine`, compiled unless the
//! `era6` feature is set. This profile table is not that engine.

use seihai_core::protocol::three_d::ThreeD;
use seihai_core::protocol::ClientVersion;

/// Torque reimplementation for eras 3–5.
///
/// The default build compiles it. `--features era6` leaves it out
/// (v6 and later). See `engine/README.md`.
#[cfg(not(feature = "era6"))]
pub mod engine;

/// The Torque UDP link a Graal3D session runs beside its TCP login.
/// Left out of an `era6` build with the engine.
#[cfg(not(feature = "era6"))]
pub mod net;

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
    builds: "4.20r6, 4.20r8 (assumed)",
    game: "Graal3D",
    protocol: 21,
    class_table_types: 2,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: false,
};

/// 5.12 Windows (also 5.00r7's protocol number).
pub const GRAAL3D_24: TorqueProfile = TorqueProfile {
    builds: "5.00r7, 5.12",
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

/// The profile of a build Ato runs, from its version profile's `three_d`
/// (seihai-core, mirrored from Graal_analysis `version-profiles/three-d.toml`).
/// `None` for a build with no Torque 3D side (1.x, 2.x, 5.24r4 on).
///
/// The row may be partly assumed (4.20r8's protocol, the Mac twins):
/// [`capabilities`] says which fields were read.
pub fn of(v: ClientVersion) -> Option<TorqueProfile> {
    let t = capabilities(v)?;
    let base = by_protocol(u32::from(t.protocol))?;
    Some(TorqueProfile {
        game: t.game,
        class_table_types: t.class_table.types(),
        class_ids_12bit: t.class_ids_12bit,
        length_prefixes: t.length_prefixes,
        tge_game_layer: t.has_datablocks(),
        ..base
    })
}

/// The build's whole 3D side: terrain, scripts, tools, formats and file
/// rules, with the fields that are assumed rather than read.
pub fn capabilities(v: ClientVersion) -> Option<ThreeD> {
    v.profile().three_d
}

/// One line per capability group, for `--print-profile` and the log.
/// Assumed fields are listed so a reader does not take them as read.
pub fn describe(t: &ThreeD) -> String {
    use seihai_core::protocol::three_d::{ChunkCache, TextureNames};
    let control: Vec<&str> = t.control_objects.iter().map(|c| c.class_name()).collect();
    let terrain = match (t.terrain.ter_file, t.offers_terrain_chunks()) {
        (true, true) => "local .ter and streamed chunks",
        (true, false) => "local .ter only",
        (false, true) => "streamed chunks only",
        (false, false) => "none",
    };
    let cache = match t.terrain.cache {
        ChunkCache::Heights => ", cached in levels3d/heights/<server>/",
        ChunkCache::None => "",
    };
    let names = match t.texture_names {
        TextureNames::StripPng => "bare name (.png stripped)",
        TextureNames::AsNamed => "as named",
    };
    let assumed: Vec<&str> = t.assumed.iter().map(|f| f.key()).collect();
    format!(
        "3d: {} game layer, {} mission phases{}, control {}\n\
         3d: terrain {terrain}{cache}; TorqueScript {}; Terraformer {:?}; TerrainEditor {}\n\
         3d: DTS {}-{}, DIF {}, 3DS {}, MD3 {}; textures by {names}; never downloads {}\n\
         3d: assumed, not read: {}",
        if t.has_datablocks() {
            "TGE"
        } else {
            "GameObject"
        },
        t.mission_phases,
        if t.phases_optional { " (optional)" } else { "" },
        control.join("/"),
        if t.runs_torque_script() {
            "runs"
        } else {
            "none"
        },
        t.terraformer,
        if t.terrain_editor { "yes" } else { "no" },
        t.formats.dts_min,
        t.formats.dts_max,
        t.formats.dif_version,
        if t.formats.three_ds { "yes" } else { "no" },
        t.formats.md3_version,
        if t.download_refused.is_empty() {
            "-".to_string()
        } else {
            t.download_refused.join(" ")
        },
        if assumed.is_empty() {
            "none".to_string()
        } else {
            assumed.join(", ")
        },
    )
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

    /// Every build's Torque profile is the one its version profile's
    /// `three_d` names, and the builds without one have none.
    #[test]
    fn profile_follows_the_version_profile() {
        let want = [
            (ClientVersion::V304, Some(TGE_9)),
            (ClientVersion::V3153, Some(GRAAL3D_19)),
            (ClientVersion::V404R2, Some(GRAAL3D_19)),
            (ClientVersion::V404R2Mac, Some(GRAAL3D_19)),
            (ClientVersion::V420R6, Some(GRAAL3D_21)),
            // Assumed: 4.20r8's protocol was never read.
            (ClientVersion::V420R8, Some(GRAAL3D_21)),
            (ClientVersion::V500R7, Some(GRAAL3D_24)),
            (ClientVersion::V512, Some(GRAAL3D_24)),
            (ClientVersion::V512Mac, Some(GRAAL3D_24)),
            (ClientVersion::V524R4, None),
            (ClientVersion::V603R7, None),
            (ClientVersion::V222, None),
        ];
        for (v, p) in want {
            assert_eq!(of(v).map(|t| t.protocol), p.map(|t| t.protocol), "{v:?}");
            if let (Some(got), Some(p)) = (of(v), p) {
                assert_eq!(got.game, p.game, "{v:?}");
                assert_eq!(got.class_table_types, p.class_table_types, "{v:?}");
                assert_eq!(got.class_ids_12bit, p.class_ids_12bit, "{v:?}");
                assert_eq!(got.length_prefixes, p.length_prefixes, "{v:?}");
                assert_eq!(got.tge_game_layer, p.tge_game_layer, "{v:?}");
            }
        }
        use seihai_core::protocol::three_d::Field;
        assert!(!capabilities(ClientVersion::V420R8)
            .unwrap()
            .confirmed(Field::Protocol));
    }

    #[test]
    fn describe_lists_what_is_assumed() {
        let d = describe(&capabilities(ClientVersion::V512).unwrap());
        assert!(d.contains("streamed chunks only"), "{d}");
        assert!(d.contains("TorqueScript none"), "{d}");
        assert!(d.contains("never downloads .dso .cs .ter"), "{d}");
        assert!(d.contains("assumed, not read: none"), "{d}");
        assert!(d.contains("DTS 17-24, DIF 44"), "{d}");
        let d = describe(&capabilities(ClientVersion::V420R8).unwrap());
        assert!(d.contains("assumed, not read: protocol,"), "{d}");
        let d = describe(&capabilities(ClientVersion::V3153).unwrap());
        assert!(d.contains("local .ter and streamed chunks"), "{d}");
        assert!(d.contains("TorqueScript runs"), "{d}");
    }

    #[test]
    fn three_d_servers() {
        assert!(is_3d_server("3 Seihai Test"));
        assert!(!is_3d_server("Seihai 3"));
        assert_eq!(udp_port(14800), 14801);
    }
}

#[cfg(test)]
mod engine_gate {
    #[cfg(not(feature = "era6"))]
    #[test]
    fn default_build_compiles_the_engine() {
        assert_eq!(super::engine::Era::default(), super::engine::Era::Era3);
        assert!(super::engine::torque_enabled(super::engine::Era::Era3));
        assert!(super::engine::torque_enabled(super::engine::Era::Era4));
        assert!(super::engine::torque_enabled(super::engine::Era::Era5));
        assert!(!super::engine::torque_enabled(super::engine::Era::Era6));
    }

    #[cfg(feature = "era6")]
    #[test]
    fn era6_build_leaves_the_engine_out() {
        // `engine` is not a module in this build. Compiling this test is
        // the check that the rest of the crate still builds without it.
        assert!(cfg!(feature = "era6"));
    }
}
