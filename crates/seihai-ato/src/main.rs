//! `seihai-ato`: the 3.x to 5.x client on its own. The program is
//! [`seihai_ato::app::main`]; the `seihai` program runs the same code with
//! `seihai ato ARGS...` or from its start window.

fn main() -> std::process::ExitCode {
    seihai_ato::app::main(std::env::args().skip(1).collect())
}
