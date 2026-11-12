//! `seihai-g3d`: the Graal3D client on its own. The program is
//! [`seihai_g3d::app::main`]; the `seihai` program runs the same code with
//! `seihai g3d ARGS...` or from its start window.

fn main() -> std::process::ExitCode {
    seihai_g3d::app::main(std::env::args().skip(1).collect())
}
