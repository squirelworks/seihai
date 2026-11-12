//! `seihai-issho`: the v6 profile.
//!
//!   seihai-issho --version 6.03r7
//!
//!   seihai-issho --version 6.03r7 --online host:port --account NAME --password-env VAR
//!                [--assets DIR] [--no-window] [--first-level] [--no-sound]
//!
//!   seihai-issho --version 6.03r7 --level NAME [--assets DIR] [--no-sound]
//!
//! A 6.x profile prints the shell and exits. With `--online` it logs in to
//! that game server (core's client session), opens the game window and
//! walks the level the server sends with the arrow keys; `--no-window`
//! only logs each level. `--level` plays level files with no server. With
//! no `--version` it refuses: it does not default to v6. v3–v5 and v7 are
//! refused. v7 is not a profile; Unity is the expected engine.

use std::io::{self, Write};
use std::process::ExitCode;

fn fail(e: String) -> ExitCode {
    eprintln!("seihai-issho: {e}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = seihai_issho::cli::has;
    let sound_on = !cli(&args, "--no-sound");
    if let Some(o) = seihai_issho::cli::offline(&args) {
        return match o.and_then(|o| play_offline(&o, sound_on)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail(e),
        };
    }
    if let Some(o) = seihai_issho::cli::online(&args, |v| std::env::var(v).ok()) {
        let windowed = !cli(&args, "--no-window");
        let result = o.and_then(|o| {
            if windowed {
                play_online(&o, sound_on)
            } else {
                seihai_issho::cli::run_online(&o, &mut io::stdout())
            }
        });
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail(e),
        };
    }
    let outcome = seihai_issho::cli::run(&args);
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    let _ = stderr.write_all(outcome.stderr.as_bytes());
    seihai_issho::cli::exit_code(&outcome)
}

#[cfg(feature = "window")]
fn play_online(o: &seihai_issho::cli::Online, sound_on: bool) -> Result<(), String> {
    seihai_issho::game::run(o, sound_on)
}

#[cfg(feature = "window")]
fn play_offline(o: &seihai_issho::cli::Offline, sound_on: bool) -> Result<(), String> {
    seihai_issho::game::run_offline(o.version, &o.level, o.assets.clone(), sound_on)
}

#[cfg(not(feature = "window"))]
fn play_online(_: &seihai_issho::cli::Online, _: bool) -> Result<(), String> {
    Err("built without the `window` feature; pass --no-window to only log".into())
}

#[cfg(not(feature = "window"))]
fn play_offline(_: &seihai_issho::cli::Offline, _: bool) -> Result<(), String> {
    Err("built without the `window` feature; --level needs the window".into())
}
