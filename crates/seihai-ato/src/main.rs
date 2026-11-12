//! Seihai Ato — the Torque / 3.x client line.
//!
//!   seihai-ato --version 3.153 [--account A] [--password-env VAR] [--nick N]
//!   seihai-ato --version 3.04 --print-profile
//!   seihai-ato --version 3.04 [--listserver host:port] --account A --password-env VAR
//!   seihai-ato --version 3.04 --online host:port --account A --password-env VAR
//!              [--assets DIR] [--no-window] [--first-level]
//!   seihai-ato --version 5.12 --online host:port --graal3d --no-window ...
//!
//! Start asks the version's listserver (`listserver.graal.in`, or
//! `--listserver host:port`) for its servers and logs in at the one picked.
//! With `--online`, Start logs in at that game server instead, logs each level the
//! server sends (its name, the start position and the NPC count) and draws
//! it flat; the arrow keys walk it. With both `--online` and `--account`
//! the start window is skipped and the login starts at once. `--no-window`
//! only logs. `--graal3d` (with `--no-window`) also opens the Torque UDP
//! link of a Graal3D server at the same moment, to its port + 1, and logs
//! its progress with a `3d:` prefix.
//!
//! `seihai-launcher` starts this binary for a 3.x `--version` and passes the
//! user's arguments through; flags meant for `seihai-mae` are ignored.

use seihai_ato::{args, game, graal3d, online, start_window, torque, wire::Wire};
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
    // `--level` with no server: play the level files offline.
    if let (Some(level), None) = (a.level.as_deref(), a.online.as_deref()) {
        return match game::run_offline(version, level, a.assets.map(Into::into), !a.no_sound) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("seihai-ato: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if !a.no_window {
        let mut w = start_window::StartWindow::new(version, nick, account, password);
        w.online = a.online.clone();
        if let Some(ls) = a.listserver.clone() {
            w.listserver = ls;
        }
        return match game::run(version, w, a.assets.map(Into::into), !a.no_sound) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("seihai-ato: could not open the window: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let Some(addr) = a.online else {
        eprintln!("seihai-ato: --no-window needs --online host:port");
        return ExitCode::from(2);
    };
    let login = online::Login {
        account,
        password,
        nick,
        language: "English".into(),
    };
    let udp = if a.graal3d {
        match graal3d::start(&addr, version, &login) {
            Ok(u) => Some(u),
            Err(e) => {
                eprintln!("seihai-ato: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        None
    };
    let tcp = online::run(
        &addr,
        version,
        &login,
        a.assets.map(Into::into),
        a.first_level,
        &mut std::io::stdout(),
        &mut |_, _| {},
    );
    if let Some(u) = udp {
        let _ = u.stop();
    }
    match tcp {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("seihai-ato: {e}");
            ExitCode::FAILURE
        }
    }
}
