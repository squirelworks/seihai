//! The `seihai` program: every Seihai client in one program, with one
//! start window.
//!
//! Mae (1.x and 2.x), Ato (3.x to 5.x), Issho (6.x) and G3D (Graal3D,
//! February 2000) are linked in as libraries, so the window, renderer,
//! sound and core code they share is in the program once. Each client's
//! whole program is its library's `app::main`; this one calls it in the
//! same process.
//!
//! With no arguments (or only `--launcher-error TEXT`) it opens the start
//! window (`seihai_play::start`): pick any version of any line, fetch the
//! server list and join. The pick runs that line's client, which logs in
//! at once. A lost connection comes back to this window
//! (`SEIHAI_LAUNCHER`), not the client's own.
//!
//! Each client also runs on its own, with its own start window:
//! `seihai mae|ato|issho|g3d ARGS...`, or this program under the name
//! `seihai-mae`, `seihai-ato`, `seihai-issho` or `seihai-g3d` (a copy or a
//! link). The crates' own binaries do the same for development; they are
//! not shipped.
//!
//! With other arguments it reads `--version` against the profile table in
//! `seihai-core` and runs that line's client with every argument,
//! unchanged. Arguments with no `--version` are Mae's. A 7.x label is not
//! a profile and runs nothing. `--launcher-dry-run` prints what would run.
//!
//! Usage:
//!   seihai                                the start window
//!   seihai mae|ato|issho|g3d [ARGS...]    that client, on its own
//!   seihai [ARGS...]                      Mae ARGS...
//!   seihai --version 2.22 [ARGS...]       Mae --version 2.22 ARGS...
//!   seihai --version 3.153 [ARGS...]      Ato --version 3.153 ARGS...
//!   seihai --version 6.03r7 [ARGS...]     Issho --version 6.03r7 ARGS...
//!   seihai --version graal3d [ARGS...]    G3D --version graal3d ARGS...

use seihai_core::protocol::{ClientVersion, VersionProfile};
use seihai_issho::{is_v6, is_v7_label};
use seihai_play::start::{
    Handoff, Line, Saved, StartScreen, LAUNCHER_ENV, PASSWORD_ENV, PROGRAM_ENV,
};
use std::process::ExitCode;

/// Which line a profile belongs to. v6 is Issho. The token says the rest:
/// `GNW` for the 2D builds, `G3D` for the 3.x to 5.x ones, which run in
/// `seihai-ato` (4.x and 5.x with their Login lobby). The start window
/// uses the same rule (`seihai_play::start::line_of`).
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

/// The start window is opened for no arguments, or only the error a
/// client came back with (`--launcher-error TEXT`). Returns that error.
fn window_args(args: &[String]) -> Option<Option<String>> {
    match args {
        [] => Some(None),
        [flag, text] if flag == "--launcher-error" => Some(Some(text.clone())),
        _ => None,
    }
}

/// What the program name or the first argument asks for: a client run on
/// its own, with the arguments left for it.
fn named_line(argv0: &str, args: &[String]) -> Option<(Line, usize)> {
    let name = argv0.rsplit(['/', '\\']).next().unwrap_or("");
    if let Some(l) = Line::from_name(name) {
        return Some((l, 0));
    }
    // `seihai capture ...` stays Mae's capture viewer, not a line.
    let first = args.first()?;
    if first.starts_with("seihai-") {
        return None;
    }
    Line::from_name(first).map(|l| (l, 1))
}

/// Run a client in this process. Returns its exit code.
fn run_line(line: Line, args: Vec<String>) -> ExitCode {
    match line {
        Line::Mae => {
            seihai::app::main(args);
            ExitCode::SUCCESS
        }
        Line::Ato => seihai_ato::app::main(args),
        Line::Issho => seihai_issho::app::main(args),
        Line::G3d => seihai_g3d::app::main(args),
    }
}

fn main() -> ExitCode {
    let mut argv = std::env::args();
    let argv0 = argv.next().unwrap_or_default();
    let mut args: Vec<String> = argv.collect();
    let dry_run = args.iter().any(|a| a == "--launcher-dry-run");
    args.retain(|a| a != "--launcher-dry-run");
    // Every client run from here can start another line in this program.
    if let Ok(me) = std::env::current_exe() {
        std::env::set_var(PROGRAM_ENV, me);
    }

    // `seihai ato ARGS` or a copy named `seihai-ato`: that client, its own
    // start window and all, as its development binary runs it.
    if let Some((line, skip)) = named_line(&argv0, &args) {
        let args = args.split_off(skip);
        if dry_run {
            println!("would run: {} {}", line.subcommand(), shown(&args));
            return ExitCode::SUCCESS;
        }
        return run_line(line, args);
    }

    if let Some(error) = window_args(&args) {
        return start_window(error, dry_run);
    }

    let (line, profile) = match pick(&args) {
        Pick::UnsupportedV7 => {
            eprintln!("seihai: v7 is not a supported profile. Unity is the expected engine; Issho does not select it.");
            return ExitCode::from(2);
        }
        Pick::Start(line, profile) => (line, profile),
    };
    if line == Line::Ato {
        let label = profile.map_or("3.x", |p| p.label);
        eprintln!("seihai: {label} is a Torque (3.x to 5.x) build; Ato draws it with the flat 2D fallback (no Torque 3D yet)");
    }
    if dry_run {
        println!("would run: {} {}", line.subcommand(), shown(&args));
        return ExitCode::SUCCESS;
    }
    run_line(line, args)
}

/// The start window, then the client it hands over to, in this process.
/// Closing the game comes back to the start window; closing the start
/// window ends the program.
fn start_window(mut error: Option<String>, dry_run: bool) -> ExitCode {
    // A lost connection comes back to this window, not the client's own.
    if let Ok(me) = std::env::current_exe() {
        std::env::set_var(LAUNCHER_ENV, me);
    }
    loop {
        let Some(h) = open_window(error.take()) else {
            return ExitCode::SUCCESS;
        };
        if dry_run {
            println!(
                "would run: {} {}",
                h.line().subcommand(),
                h.args().join(" ")
            );
            return ExitCode::SUCCESS;
        }
        if let Some((var, value)) = h.env() {
            std::env::set_var(var, value);
        }
        // The start window comes back whatever the game's exit code; a
        // client that refused the pick (a 6.x join with no password, say)
        // says so there as well as on the console.
        let line = h.line();
        seihai_play::start::take_ended();
        let code = run_line(line, h.args());
        // A lost connection in Ato or Issho closes the game with its reason.
        if let Some(why) = seihai_play::start::take_ended() {
            error = Some(why);
        } else if code != ExitCode::SUCCESS {
            error = Some(format!(
                "{} stopped with an error (the console has the details).",
                line.name()
            ));
        }
    }
}

/// Open the window and block until it closes. The hand-over, if the
/// player joined a server or opened a client's own window.
fn open_window(error: Option<String>) -> Option<Handoff> {
    struct App {
        screen: StartScreen,
        out: std::rc::Rc<std::cell::RefCell<Option<Handoff>>>,
    }
    impl eframe::App for App {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            if let Some(h) = self.screen.show(ctx) {
                self.screen.saved().save();
                *self.out.borrow_mut() = Some(h);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    let out = std::rc::Rc::new(std::cell::RefCell::new(None));
    let app = App {
        screen: {
            let mut screen = StartScreen::new(Saved::load(), error);
            // The password of this run (and of the run a lost connection
            // restarted from): kept in the environment, never saved.
            screen.password = std::env::var(PASSWORD_ENV).unwrap_or_default();
            screen
        },
        out: out.clone(),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 520.0]),
        ..Default::default()
    };
    if let Err(e) =
        seihai_play::window::run_native("Seihai", options, Box::new(move |_| Ok(Box::new(app))))
    {
        eprintln!("seihai: could not open the start window: {e}");
        eprintln!("seihai: no renderer could draw a window here (tried {}); install the graphics driver, or put Mesa's software opengl32.dll next to seihai.exe", e.tried.iter().map(|(r, _)| *r).collect::<Vec<_>>().join(", then "));
        eprintln!("seihai: run a client with --version and --online instead (see `seihai --help`)");
        std::process::exit(1);
    }
    let h = out.borrow_mut().take();
    h
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
    fn no_arguments_open_the_start_window() {
        assert_eq!(window_args(&[]), Some(None));
        assert_eq!(
            window_args(&args("--launcher-error lost")),
            Some(Some("lost".into()))
        );
        assert_eq!(window_args(&args("--version 2.22")), None);
        assert_eq!(
            window_args(&args("--launcher-error a --version 2.22")),
            None
        );
    }

    /// The router and the start window put every row in the same client.
    #[test]
    fn router_and_window_agree() {
        for p in seihai_core::protocol::PROFILES {
            assert_eq!(
                line_for(p),
                seihai_play::start::line_of(p.version),
                "{}",
                p.label
            );
        }
    }

    #[test]
    fn a_subcommand_or_program_name_runs_that_client() {
        assert_eq!(
            named_line("/usr/bin/seihai", &args("ato --version 3.04")),
            Some((Line::Ato, 1))
        );
        assert_eq!(named_line("seihai", &args("g3d")), Some((Line::G3d, 1)));
        assert_eq!(
            named_line("C:\\x\\seihai-issho.exe", &args("--version 6.03r7")),
            Some((Line::Issho, 0))
        );
        assert_eq!(named_line("seihai-mae", &[]), Some((Line::Mae, 0)));
        assert_eq!(named_line("seihai", &args("--version 3.04")), None);
        assert_eq!(named_line("seihai", &args("capture show f")), None);
        assert_eq!(named_line("seihai", &[]), None);
    }

    #[test]
    fn passwords_are_hidden() {
        assert_eq!(
            shown(&args("--account a --password secret --version 2.22")),
            "--account a --password **** --version 2.22"
        );
    }
}
