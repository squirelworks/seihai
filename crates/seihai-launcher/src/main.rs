//! The `seihai` command: start the client for the chosen version.
//!
//! The classic builds (1.x and 2.x) run in `seihai-mae`; the Torque / 3.x
//! to 5.x builds run in `seihai-ato`; a 6.x profile runs in `seihai-issho`.
//! This program reads `--version` against the
//! profile table in `seihai-core`, then hands every argument, unchanged, to
//! the client binary for that line. With no `--version` (and with no
//! arguments at all, which opens the start window) it is `seihai-mae`, as
//! `seihai` was before the launcher. A 7.x label is not a profile and starts
//! nothing.
//!
//! The client binary is looked for next to this one first, then on `PATH`.
//! `--launcher-dry-run` prints what would be started and exits.
//!
//! Usage:
//!   seihai [ARGS...]                      seihai-mae ARGS...
//!   seihai --version 2.22 [ARGS...]       seihai-mae --version 2.22 ARGS...
//!   seihai --version 3.153 [ARGS...]      seihai-ato --version 3.153 ARGS...
//!   seihai --version 6.03r7 [ARGS...]     seihai-issho --version 6.03r7 ARGS...
//!   seihai --version graal3d [ARGS...]    seihai-g3d --version graal3d ARGS...

use seihai_core::protocol::{ClientVersion, VersionProfile};
use seihai_issho::{is_v6, is_v7_label};
use std::path::PathBuf;
use std::process::Command;

/// The client binary a version runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line {
    /// 1.x and 2.x: `seihai-mae`.
    Mae,
    /// Torque / 3.x to 5.x: `seihai-ato`.
    Ato,
    /// v6 quattro_lib: `seihai-issho`. Not started unless the profile is 6.x.
    Issho,
    /// Graal3D (February 2000), the standalone 3D client: `seihai-g3d`.
    G3d,
}

impl Line {
    fn binary(self) -> &'static str {
        match self {
            Line::Mae => "seihai-mae",
            Line::Ato => "seihai-ato",
            Line::Issho => "seihai-issho",
            Line::G3d => "seihai-g3d",
        }
    }
}

/// Which line a profile belongs to. v6 is Issho. The token says the rest:
/// `GNW` for the 2D builds, `G3D` for the 3.x to 5.x ones, which run in
/// `seihai-ato` (4.x and 5.x with their Login lobby).
fn line_for(p: &VersionProfile) -> Line {
    if is_v6(p.version) {
        Line::Issho
    } else if p.token.starts_with("G3D") {
        Line::Ato
    } else {
        Line::Mae
    }
}

/// Modes that never start a client session: the capture viewer and the
/// printing tools. They are `seihai-mae`'s whatever the version, so
/// `seihai --version-chain --version 3.153` keeps working.
const MAE_TOOLS: &[&str] = &[
    "--version-chain",
    "--check-assets",
    "--level-info",
    "--list-only",
    "--help",
    "-h",
];

/// The value of the last `--version`, as `seihai-mae` reads it.
fn version_arg(args: &[String]) -> Option<&str> {
    args.iter()
        .rposition(|a| a == "--version")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// `--version` values for the February 2000 Graal3D. It has its own
/// server and wire, so it is not a row of the profile table.
fn is_graal3d(v: &str) -> bool {
    matches!(
        v.to_ascii_lowercase().as_str(),
        "graal3d" | "g3d" | "gr3dv002"
    )
}

/// What `pick` decided. v7 is not a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pick {
    Start(Line, Option<&'static VersionProfile>),
    /// Unity is the expected engine. No process is started.
    UnsupportedV7,
}

/// The line to start for these arguments. An unknown version goes to
/// `seihai-mae`, which says so and falls back to 2.05. A v7 label does not.
fn pick(args: &[String]) -> Pick {
    if version_arg(args).is_some_and(is_v7_label) {
        return Pick::UnsupportedV7;
    }
    if version_arg(args).is_some_and(is_graal3d) {
        return Pick::Start(Line::G3d, None);
    }
    let tool = args.first().map(String::as_str) == Some("capture")
        || args.iter().any(|a| MAE_TOOLS.contains(&a.as_str()));
    let profile = version_arg(args)
        .and_then(ClientVersion::from_label)
        .map(ClientVersion::profile);
    match profile {
        Some(p) if !tool => Pick::Start(line_for(p), Some(p)),
        _ => Pick::Start(Line::Mae, profile),
    }
}

/// The client binary next to this one, or the bare name for a `PATH` lookup.
fn find_binary(line: Line) -> PathBuf {
    let file = format!("{}{}", line.binary(), std::env::consts::EXE_SUFFIX);
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(&file)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(file))
}

/// The arguments for a printout, with any `--password` value hidden.
fn shown(args: &[String]) -> String {
    let mut out = Vec::with_capacity(args.len());
    let mut hide = false;
    for a in args {
        out.push(if hide { "****" } else { a.as_str() });
        hide = a == "--password";
    }
    out.join(" ")
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let dry_run = args.iter().any(|a| a == "--launcher-dry-run");
    args.retain(|a| a != "--launcher-dry-run");

    let (line, profile) = match pick(&args) {
        Pick::UnsupportedV7 => {
            eprintln!("seihai: v7 is not a supported profile. Unity is the expected engine; Issho does not select it.");
            std::process::exit(2);
        }
        Pick::Start(line, profile) => (line, profile),
    };
    if line == Line::Ato {
        let label = profile.map_or("3.x", |p| p.label);
        eprintln!("seihai: {label} is a Torque (3.x to 5.x) build; seihai-ato draws it with the flat 2D fallback (no Torque 3D yet)");
    }
    if line == Line::Issho {
        let label = profile.map_or("6.x", |p| p.label);
        eprintln!("seihai: {label} is a v6 build; seihai-issho is the quattro_lib profile (no playfield yet)");
    }
    let bin = find_binary(line);
    if dry_run {
        println!("would start: {} {}", bin.display(), shown(&args));
        return;
    }
    let mut cmd = Command::new(&bin);
    cmd.args(&args);
    let err = run(cmd);
    eprintln!("seihai: cannot start {}: {err}", bin.display());
    eprintln!(
        "seihai: build {} with `cargo build --release` from the seihai folder; it builds seihai, seihai-mae, seihai-ato, seihai-issho and seihai-g3d together",
        line.binary()
    );
    std::process::exit(1);
}

/// Replace this process with the client. Returns only if that failed.
#[cfg(unix)]
fn run(mut cmd: Command) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    cmd.exec()
}

/// Windows has no `exec`: run the client and leave with its exit code.
#[cfg(not(unix))]
fn run(mut cmd: Command) -> std::io::Error {
    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    fn line(s: &str) -> Line {
        match pick(&args(s)) {
            Pick::Start(line, _) => line,
            Pick::UnsupportedV7 => panic!("v7 is not a client line"),
        }
    }

    #[test]
    fn classic_builds_start_mae() {
        assert_eq!(line("--version 2.22"), Line::Mae);
        assert_eq!(line("--version 1.39 --online h:14923"), Line::Mae);
        assert_eq!(line(""), Line::Mae);
        assert_eq!(line("--assets dir"), Line::Mae);
    }

    #[test]
    fn torque_builds_start_ato() {
        assert_eq!(line("--version 3.153"), Line::Ato);
        // 4.x and 5.x are Ato's too, with their Login lobby.
        assert_eq!(line("--version 4.04r2"), Line::Ato);
        assert_eq!(line("--version 4.20r8"), Line::Ato);
        assert_eq!(line("--version 5.00r7"), Line::Ato);
        assert_eq!(line("--version 5.12"), Line::Ato);
        assert_eq!(line("--version 3.04 --account a"), Line::Ato);
        // The last --version wins, as in seihai-mae.
        assert_eq!(line("--version 2.22 --version 3.153"), Line::Ato);
        assert_eq!(line("--version 3.153 --version 2.22"), Line::Mae);
    }

    #[test]
    fn v6_starts_issho_and_v3_to_v5_do_not() {
        assert_eq!(line("--version 6.00r2"), Line::Issho);
        assert_eq!(line("--version 6.015"), Line::Issho);
        assert_eq!(line("--version 6.034"), Line::Issho);
        assert_eq!(line("--version 6.03r7"), Line::Issho);
        assert_eq!(
            pick(&[String::from("--version"), String::from("6.03r7 Linux")]),
            Pick::Start(Line::Issho, Some(ClientVersion::V603R7Linux.profile()))
        );
        assert_eq!(line("--version 3.04"), Line::Ato);
        assert_eq!(line("--version 3.153"), Line::Ato);
        assert_eq!(line("--version 4.20r8"), Line::Ato);
        assert_eq!(line("--version 5.00r7"), Line::Ato);
        assert_eq!(line("--version 5.31r4"), Line::Ato);
        // Tools stay in mae even for a 6.x label: they never start a session.
        assert_eq!(line("--version-chain --version 6.03r7"), Line::Mae);
        assert_eq!(line("--help --version 6.03r7"), Line::Mae);
    }

    #[test]
    fn v7_is_not_started() {
        assert_eq!(pick(&args("--version 7")), Pick::UnsupportedV7);
        assert_eq!(pick(&args("--version 7.0")), Pick::UnsupportedV7);
        assert_eq!(pick(&args("--version V7.2")), Pick::UnsupportedV7);
        assert_eq!(pick(&args("--version 7.0 --online h")), Pick::UnsupportedV7);
        // A later --version that is a real profile still wins.
        assert_eq!(line("--version 7.0 --version 2.22"), Line::Mae);
        assert_eq!(
            pick(&args("--version 2.22 --version 7")),
            Pick::UnsupportedV7
        );
    }

    #[test]
    fn graal3d_starts_g3d() {
        assert_eq!(line("--version graal3d --online h"), Line::G3d);
        assert_eq!(line("--version G3D"), Line::G3d);
        assert_eq!(line("--version graal3d --version 2.22"), Line::Mae);
    }

    #[test]
    fn unknown_versions_and_tools_stay_in_mae() {
        assert_eq!(line("--version 9.99"), Line::Mae);
        assert_eq!(line("--version"), Line::Mae);
        assert_eq!(line("--version-chain --version 3.153"), Line::Mae);
        assert_eq!(line("--check-assets --version 3.04"), Line::Mae);
        assert_eq!(line("capture show f.pcap --version 3.153"), Line::Mae);
    }

    /// Every row is one line, by its major version. v6 is Issho; 3.x is Ato.
    #[test]
    fn every_profile_has_a_line() {
        for p in seihai_core::protocol::PROFILES {
            let want = if is_v6(p.version) {
                Line::Issho
            } else if ["3.", "4.", "5."].iter().any(|m| p.label.starts_with(m)) {
                Line::Ato
            } else {
                Line::Mae
            };
            assert_eq!(line_for(p), want, "{}", p.label);
        }
    }

    #[test]
    fn passwords_are_hidden() {
        assert_eq!(
            shown(&args("--account a --password secret --version 2.22")),
            "--account a --password **** --version 2.22"
        );
    }
}
