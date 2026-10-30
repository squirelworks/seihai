# seihai-ato

The Torque / 3.x client line.

The 3.x clients still speak gen-5 framing and use the same board, gmap and
GS1 formats as 2.22 and 2.31, so this crate shares `seihai-core` with the
2.x client (`seihai-mae`). What is 3.x-only (the renderer, the start
window, the tokens and the 3.x-only packet ids) will land here.

For now it does not draw. `cargo run -p seihai-ato` prints the version and
exits 0. It depends on `seihai-core` and nothing else, and has no renderer
code.
