//! v3 cut. The full TGE surface, plus Graal-only `GuiGraalCtrl`.
//!
//! Every submodule in this directory is on for [`super::Era::Era3`].
//! Stubs only. No socket is opened and no script runs.

use super::Module;

/// Submodules this cut enables, in the order the cut lists them.
pub const MODULES: &[Module] = &[
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
];

pub mod action_map;
pub mod bitmap;
pub mod console;
pub mod dso;
pub mod dts;
pub mod game_connection;
pub mod gui;
pub mod gui_graal_ctrl;
pub mod interiors;
pub mod math;
pub mod player;
pub mod shape_base;
pub mod sim_object;
pub mod terrain;
pub mod vehicles;

/// Calls every stub on this cut. Does no work.
pub fn touch() {
    console::stub();
    dso::stub();
    sim_object::stub();
    action_map::stub();
    gui::stub();
    bitmap::stub();
    math::stub();
    terrain::stub();
    interiors::stub();
    dts::stub();
    shape_base::stub();
    player::stub();
    vehicles::stub();
    game_connection::stub();
    gui_graal_ctrl::stub();
}
