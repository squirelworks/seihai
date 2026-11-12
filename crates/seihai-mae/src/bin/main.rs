//! `seihai-mae`: the 1.x and 2.x client on its own. The program is
//! [`seihai::app::main`]; the `seihai` program runs the same code with
//! `seihai mae ARGS...` or from its start window.

fn main() {
    seihai::app::main(std::env::args().skip(1).collect());
}
