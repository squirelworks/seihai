//! `seihai-issho`: the 6.x client on its own. The program is
//! [`seihai_issho::app::main`]; the `seihai` program runs the same code
//! with `seihai issho ARGS...` or from its start window.

fn main() -> std::process::ExitCode {
    seihai_issho::app::main(std::env::args().skip(1).collect())
}
