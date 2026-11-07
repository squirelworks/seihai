//! v5.0 cut. The same two subset names as v4, not the same cut.
//!
//! The launcher resolves 95 `graal_set` names and the engine defines 118.
//! The 3D protocol check accepts only 24. See [`differences`]. None of
//! that is implemented here. The launcher is Delphi plus those callbacks.
//! It is not this module.
//!
// Unpacked 5.30 and 5.31 are a later cut. They drop graal_set, SimObject
// and GameConnection, so the 3D Torque link is gone. Do not fold that cut
// into this 5.0 set.

use super::Module;

/// Submodules this cut enables, in the order the cut lists them.
pub const MODULES: &[Module] = &[
    Module::Gui,
    Module::Bitmap,
    Module::Math,
    Module::GuiGraalCtrl,
    Module::SimObject,
    Module::GameConnection,
    Module::GameObject,
    Module::TerrainChunks,
    Module::Interiors,
];

pub mod bitmap;
pub mod game_connection;
pub mod game_object;
pub mod gui;
pub mod gui_graal_ctrl;
pub mod interiors;
pub mod math;
pub mod sim_object;
pub mod terrain_chunks;

pub mod differences;

/// Calls every stub on this cut. Does no work.
pub fn touch() {
    gui::stub();
    bitmap::stub();
    math::stub();
    gui_graal_ctrl::stub();
    sim_object::stub();
    game_connection::stub();
    game_object::stub();
    terrain_chunks::stub();
    interiors::stub();
}
