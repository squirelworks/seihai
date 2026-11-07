//! `seihai-issho`: the v6 profile.
//!
//!   seihai-issho --version 6.03r7
//!
//! A 6.x profile prints the shell and exits. This binary does not draw,
//! log in, or open a sound device. With no `--version` it refuses: it does
//! not default to v6. v3–v5 and v7 are refused. v7 is not a profile; Unity
//! is the expected engine.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = seihai_issho::cli::run(&args);
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    let _ = stderr.write_all(outcome.stderr.as_bytes());
    seihai_issho::cli::exit_code(&outcome)
}
