//! v4 cut. A 2D subset and a 3D subset. Not the v3 surface, and not v5.
//!
//! The 2D side keeps Torque GUI and math. It has no `SimObject` console
//! and no `.dso`. The 3D side keeps `SimObject`, `GameConnection`,
//! `GameObject`, terrain chunks and interiors, and drops `ShapeBase` and
//! Torque's `Player`.
//!
//! The launcher is Delphi plus `graal_set` callbacks. That program is not
//! Ato, and it is not compiled here.

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
