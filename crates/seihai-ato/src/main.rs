//! Seihai Ato — the Torque / 3.x client line.
//!
//!   seihai-ato --version 3.153 [--account A] [--password-env VAR] [--nick N]
//!   seihai-ato --version 3.04 --print-profile
//!
//! `seihai-launcher` starts this binary for a 3.x `--version` and passes the
//! user's arguments through; flags meant for `seihai-mae` are ignored.

use seihai_ato::{args, start_window, torque, wire::Wire};
use std::process::ExitCode;

fn main() -> ExitCode {
    let a = args::parse(std::env::args().skip(1));
    let version = match args::build(&a) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    if a.print_profile {
        print!("{}", Wire::of(version).report());
        if let Some(t) = torque::of(version) {
            println!(
                "torque: game {:?}, protocol {}, class table types {}, 12-bit class ids {}, 14-bit lengths {}",
                t.game, t.protocol, t.class_table_types, t.class_ids_12bit, t.length_prefixes
            );
        }
        return ExitCode::SUCCESS;
    }
    let password = match a.password_env.as_deref() {
        Some(var) => std::env::var(var).unwrap_or_default(),
        None => String::new(),
    };
    let account = a.account.unwrap_or_default();
    let nick = a.nick.unwrap_or_else(|| account.clone());
    match start_window::run(start_window::StartWindow::new(version, nick, account, password)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("seihai-ato: could not open the start window: {e}");
            ExitCode::FAILURE
        }
    }
}
