//! `seihai-issho`: the v6 profile.
//!
//!   seihai-issho --version 6.03r7
//!
//!   seihai-issho --version 6.03r7 --online host:port --account NAME --password-env VAR
//!
//! A 6.x profile prints the shell and exits. With `--online` it logs in to
//! that game server and logs each level it sends (core's client session).
//! This binary does not draw or open a sound device. With no `--version` it refuses: it does
//! not default to v6. v3–v5 and v7 are refused. v7 is not a profile; Unity
//! is the expected engine.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(o) = seihai_issho::cli::online(&args, |v| std::env::var(v).ok()) {
        let result = o.and_then(|o| seihai_issho::cli::run_online(&o, &mut io::stdout()));
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("seihai-issho: {e}");
                ExitCode::from(2)
            }
        };
    }
    let outcome = seihai_issho::cli::run(&args);
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    let _ = stderr.write_all(outcome.stderr.as_bytes());
    seihai_issho::cli::exit_code(&outcome)
}
