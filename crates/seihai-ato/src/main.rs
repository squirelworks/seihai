//! Seihai Ato — the Torque / 3.x client line.
//!
//!   seihai-ato --version 3.153 [--account A] [--password-env VAR] [--nick N]
//!   seihai-ato --version 3.04 --print-profile
//!   seihai-ato --version 3.04 --online host:port --account A --password-env VAR
//!              [--assets DIR] [--no-window] [--first-level]
//!
//! With `--online`, Start (or `--no-window`) logs in at that game server and
//! logs each level the server sends: its name, the start position and the
//! NPC count. Nothing is drawn yet.
//!
//! `seihai-launcher` starts this binary for a 3.x `--version` and passes the
//! user's arguments through; flags meant for `seihai-mae` are ignored.

use seihai_ato::{args, online, start_window, torque, wire::Wire};
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
    let login = if a.no_window {
        if a.online.is_none() {
            eprintln!("seihai-ato: --no-window needs --online host:port");
            return ExitCode::from(2);
        }
        online::Login { account, password, nick, language: "English".into() }
    } else {
        let mut w = start_window::StartWindow::new(version, nick, account, password);
        w.online = a.online.clone();
        match start_window::run(w) {
            Ok(Some(l)) => l,
            Ok(None) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("seihai-ato: could not open the start window: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    let Some(addr) = a.online else { return ExitCode::SUCCESS };
    match online::run(&addr, version, &login, a.assets.map(Into::into), a.first_level, &mut std::io::stdout()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("seihai-ato: {e}");
            ExitCode::FAILURE
        }
    }
}
