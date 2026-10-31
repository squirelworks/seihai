//! The `seihai` command: start the client for the chosen version.
//!
//! The classic builds (1.x and 2.x) run in `seihai-mae`; the Torque / 3.x
//! builds run in `seihai-ato`. This program reads `--version` against the
//! profile table in `seihai-core`, then hands every argument, unchanged, to
//! the client binary for that line. With no `--version` (and with no
//! arguments at all, which opens the start window) it is `seihai-mae`, as
//! `seihai` was before the launcher.
//!
//! The client binary is looked for next to this one first, then on `PATH`.
//! `--launcher-dry-run` prints what would be started and exits.
//!
//! Usage:
//!   seihai [ARGS...]                      seihai-mae ARGS...
//!   seihai --version 2.22 [ARGS...]       seihai-mae --version 2.22 ARGS...
//!   seihai --version 3.153 [ARGS...]      seihai-ato --version 3.153 ARGS...

use seihai_core::protocol::{ClientVersion, VersionProfile};
use std::path::PathBuf;
use std::process::Command;

/// The client binary a version runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line {
    /// 1.x and 2.x: `seihai-mae`.
    Mae,
    /// Torque / 3.x: `seihai-ato`.
    Ato,
}

impl Line {
    fn binary(self) -> &'static str {
        match self {
            Line::Mae => "seihai-mae",
            Line::Ato => "seihai-ato",
        }
    }
}

/// Which line a profile belongs to. The token says it: `GNW` for the 2D
/// builds, `G3D` for the 3.x ones. The 4.x, 5.x and 6.x builds send `G3D` tokens
/// too but play in `seihai-mae` (their profile has the Login lobby).
fn line_for(p: &VersionProfile) -> Line {
    if p.token.starts_with("G3D") && p.protocol.lobby.is_none() {
        Line::Ato
    } else {
        Line::Mae
    }
}

/// Modes that never start a client session: the capture viewer and the
/// printing tools. They are `seihai-mae`'s whatever the version, so
/// `seihai --version-chain --version 3.153` keeps working.
const MAE_TOOLS: &[&str] = &["--version-chain", "--check-assets", "--level-info", "--list-only", "--help", "-h"];

/// The value of the last `--version`, as `seihai-mae` reads it.
fn version_arg(args: &[String]) -> Option<&str> {
    args.iter()
        .rposition(|a| a == "--version")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// The line to start for these arguments. An unknown version goes to
/// `seihai-mae`, which says so and falls back to 2.05.
fn pick(args: &[String]) -> (Line, Option<&'static VersionProfile>) {
    let tool = args.first().map(String::as_str) == Some("capture") || args.iter().any(|a| MAE_TOOLS.contains(&a.as_str()));
    let profile = version_arg(args).and_then(ClientVersion::from_label).map(ClientVersion::profile);
    match profile {
        Some(p) if !tool => (line_for(p), Some(p)),
        _ => (Line::Mae, profile),
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

    let (line, profile) = pick(&args);
    if line == Line::Ato {
        let label = profile.map_or("3.x", |p| p.label);
        eprintln!("seihai: {label} is a Torque / 3.x build; seihai-ato draws it with the flat 2D fallback (no Torque 3D yet)");
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
        "seihai: build {} with `cargo build --release` from the seihai folder; it builds seihai, seihai-mae and seihai-ato together",
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

    #[test]
    fn classic_builds_start_mae() {
        assert_eq!(pick(&args("--version 2.22")).0, Line::Mae);
        assert_eq!(pick(&args("--version 1.39 --online h:14923")).0, Line::Mae);
        assert_eq!(pick(&args("")).0, Line::Mae);
        assert_eq!(pick(&args("--assets dir")).0, Line::Mae);
    }

    #[test]
    fn torque_builds_start_ato() {
        assert_eq!(pick(&args("--version 3.153")).0, Line::Ato);
        // The 4.x, 5.x and 6.x tokens start with G3D too, but they play on the flat path.
        assert_eq!(pick(&args("--version 4.04r2")).0, Line::Mae);
        assert_eq!(pick(&args("--version 4.20r6")).0, Line::Mae);
        assert_eq!(pick(&args("--version 5.12")).0, Line::Mae);
        assert_eq!(pick(&args("--version 6.03r7")).0, Line::Mae);
        assert_eq!(pick(&args("--version 3.04 --account a")).0, Line::Ato);
        // The last --version wins, as in seihai-mae.
        assert_eq!(pick(&args("--version 2.22 --version 3.153")).0, Line::Ato);
        assert_eq!(pick(&args("--version 3.153 --version 2.22")).0, Line::Mae);
    }

    #[test]
    fn unknown_versions_and_tools_stay_in_mae() {
        assert_eq!(pick(&args("--version 9.99")).0, Line::Mae);
        assert_eq!(pick(&args("--version")).0, Line::Mae);
        assert_eq!(pick(&args("--version-chain --version 3.153")).0, Line::Mae);
        assert_eq!(pick(&args("--check-assets --version 3.04")).0, Line::Mae);
        assert_eq!(pick(&args("capture show f.pcap --version 3.153")).0, Line::Mae);
    }

    /// Every row is one line or the other, by its label's major version.
    #[test]
    fn every_profile_has_a_line() {
        for p in seihai_core::protocol::PROFILES {
            let want = if p.label.starts_with("3.") { Line::Ato } else { Line::Mae };
            assert_eq!(line_for(p), want, "{}", p.label);
        }
    }

    #[test]
    fn passwords_are_hidden() {
        assert_eq!(shown(&args("--account a --password secret --version 2.22")), "--account a --password **** --version 2.22");
    }
}
