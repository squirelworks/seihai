//! Torque reimplementation for client eras 3, 4 and 5.
//!
//! Ato owns this module. The default build compiles it. Era 6 and later
//! leave it out (`--features era6`). The product default
//! follows the selected [`Era`]: on for 3, 4 and 5. [`Era::default`] is
//! [`Era::Era3`], the full surface, until a caller picks another era.
//!
//! v3, v4 and v5 are three cuts. v4 and v5 enable the same subset names
//! and keep separate modules. See `README.md` in this directory.
//! Nothing here opens a socket.

pub mod v3;
pub mod v4;
pub mod v5;

/// Which client era the Torque reimplementation follows.
///
/// The product default is the selected era. A caller who has not picked
/// one gets [`Era3`](Self::Era3), because that variant is the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Era {
    /// Full TGE surface plus `GuiGraalCtrl`.
    #[default]
    Era3,
    /// v4 2D subset and v4 3D subset.
    Era4,
    /// v5.0 2D subset and v5.0 3D subset. Not unpacked 5.30.
    Era5,
    /// v6 and later (`quattro_lib`). The reimplementation stays off.
    Era6,
}

/// One surface submodule. Matching a variant does not run that subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    /// Torque script console.
    Console,
    /// Compiled Torque script (`.dso`).
    Dso,
    /// `SimObject`.
    SimObject,
    /// `ActionMap`.
    ActionMap,
    /// Torque GUI.
    Gui,
    /// `GBitmap`.
    Bitmap,
    /// Torque math.
    Math,
    /// TGE terrain block.
    Terrain,
    /// Interiors.
    Interiors,
    /// DTS shapes.
    Dts,
    /// `ShapeBase`.
    ShapeBase,
    /// Torque `Player`.
    Player,
    /// Torque vehicles.
    Vehicles,
    /// `GameConnection`.
    GameConnection,
    /// Graal-only `GuiGraalCtrl`.
    GuiGraalCtrl,
    /// `GameObject`, a `SceneObject`.
    GameObject,
    /// Terrain chunks on the v4 and v5 3D side.
    TerrainChunks,
}

/// Whether Ato runs the Torque reimplementation for `era`.
///
/// True for eras 3, 4 and 5. False for era 6 and later.
pub fn torque_enabled(era: Era) -> bool {
    matches!(era, Era::Era3 | Era::Era4 | Era::Era5)
}

/// Submodules that are on for `era`, in that cut's order.
///
/// Era 6 returns an empty list. v4 and v5 name the same two subsets and
/// remain different cuts: [`v4`] and [`v5`].
pub fn modules_for(era: Era) -> &'static [Module] {
    match era {
        Era::Era3 => v3::MODULES,
        Era::Era4 => v4::MODULES,
        Era::Era5 => v5::MODULES,
        Era::Era6 => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_era_is_v3_and_only_v6_is_off() {
        assert_eq!(Era::default(), Era::Era3);
        assert!(torque_enabled(Era::default()));
        assert!(torque_enabled(Era::Era3));
        assert!(torque_enabled(Era::Era4));
        assert!(torque_enabled(Era::Era5));
        assert!(!torque_enabled(Era::Era6));
        assert!(modules_for(Era::Era6).is_empty());
    }

    #[test]
    fn v3_enables_the_full_surface() {
        assert_eq!(
            modules_for(Era::Era3),
            &[
                Module::Console,
                Module::Dso,
                Module::SimObject,
                Module::ActionMap,
                Module::Gui,
                Module::Bitmap,
                Module::Math,
                Module::Terrain,
                Module::Interiors,
                Module::Dts,
                Module::ShapeBase,
                Module::Player,
                Module::Vehicles,
                Module::GameConnection,
                Module::GuiGraalCtrl,
            ]
        );
        v3::touch();
    }

    #[test]
    fn v4_enables_its_two_subsets() {
        assert_eq!(
            modules_for(Era::Era4),
            &[
                Module::Gui,
                Module::Bitmap,
                Module::Math,
                Module::GuiGraalCtrl,
                Module::SimObject,
                Module::GameConnection,
                Module::GameObject,
                Module::TerrainChunks,
                Module::Interiors,
            ]
        );
        for off in [
            Module::Console,
            Module::Dso,
            Module::ActionMap,
            Module::ShapeBase,
            Module::Player,
            Module::Vehicles,
            Module::Terrain,
            Module::Dts,
        ] {
            assert!(
                !modules_for(Era::Era4).contains(&off),
                "{off:?} stays off on v4"
            );
        }
        v4::touch();
    }

    #[test]
    fn v5_has_the_same_subset_names_and_its_own_cut() {
        assert_eq!(modules_for(Era::Era5), modules_for(Era::Era4));
        assert_eq!(v5::differences::PROTOCOL_ACCEPT, 24);
        assert_eq!(v5::differences::LAUNCHER_SETTERS, 95);
        assert_eq!(v5::differences::ENGINE_SETTERS, 118);
        // The lists match. The modules do not: v5 owns `differences`.
        v5::touch();
    }
}
