//! The command line. `seihai-launcher` passes the user's arguments through
//! unchanged (only `--launcher-dry-run` is removed), so flags meant for
//! `seihai-mae` can arrive here too: unknown ones are ignored, never an
//! error.

use seihai_core::protocol::ClientVersion;

/// What `seihai-ato` was asked to do.
#[derive(Debug, Default, PartialEq)]
pub struct Args {
    /// The last `--version` value, as given.
    pub version: Option<String>,
    pub account: Option<String>,
    /// Name of the environment variable holding the password
    /// (`--password-env VAR`). Passwords are never taken on the command
    /// line.
    pub password_env: Option<String>,
    pub nick: Option<String>,
    /// `--print-profile`: print the wire facts and exit, no window.
    pub print_profile: bool,
    /// The game server to log in to (`--online host:port`, `-o`,
    /// `--server`). Without it Start only shows a note.
    pub online: Option<String>,
    /// An asset folder whose levels stand in for the server's board at the
    /// server's mod time (`--assets DIR`).
    pub assets: Option<String>,
    /// `--no-window`: log in from the command line, without the start
    /// window (needs `--online`).
    pub no_window: bool,
    /// `--first-level`: disconnect once the first level is stored.
    pub first_level: bool,
    /// The listserver Start asks for the server list (`--listserver
    /// host:port`); without it, the version's row (`listserver.graal.in`).
    pub listserver: Option<String>,
    /// `--no-sound`: no sound effects (the device is never opened).
    pub no_sound: bool,
    /// `--level NAME` (`-l`): play this level offline, with no server, as
    /// `seihai-mae --level` does. The name is looked up in `--assets`, or
    /// read as a path when there is no asset folder.
    pub level: Option<String>,
    /// `--graal3d`: the `--online` server is a Graal3D server; also open
    /// the Torque UDP link to its port + 1 (5.x builds; `--no-window`).
    pub graal3d: bool,
}

/// Flags that take a value. Unknown flags are skipped on their own, so a
/// value-taking flag Ato does not use must be listed here, or its value
/// would be read as the next flag.
const VALUE_FLAGS: &[&str] = &[];

pub fn parse<I: IntoIterator<Item = String>>(argv: I) -> Args {
    let mut a = Args::default();
    let mut it = argv.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--version" => a.version = it.next().or(a.version),
            "--account" => a.account = it.next().or(a.account),
            "--password-env" => a.password_env = it.next().or(a.password_env),
            "--nick" | "--nickname" => a.nick = it.next().or(a.nick),
            "--print-profile" => a.print_profile = true,
            "--online" | "-o" | "--server" => a.online = it.next().or(a.online),
            "--assets" => a.assets = it.next().or(a.assets),
            "--no-window" => a.no_window = true,
            "--first-level" => a.first_level = true,
            "--no-sound" => a.no_sound = true,
            "--graal3d" => a.graal3d = true,
            "--listserver" => a.listserver = it.next().or(a.listserver),
            "--level" | "-l" => a.level = it.next().or(a.level),
            f if VALUE_FLAGS.contains(&f) => {
                it.next();
            }
            _ => {}
        }
    }
    a
}

/// True for a build Ato runs: a `G3D` token from 3.x, 4.x or 5.x. The 1.x
/// and 2.x builds are `seihai-mae`'s; 6.x is `seihai-issho`'s (Graal3D was
/// removed from 6.x on).
pub fn runs_in_ato(v: ClientVersion) -> bool {
    v.token().starts_with("G3D") && matches!(v.label().as_bytes().first(), Some(b'3' | b'4' | b'5'))
}

/// The builds Ato runs, in profile order.
pub fn builds() -> Vec<ClientVersion> {
    ClientVersion::all_versions()
        .iter()
        .copied()
        .filter(|v| runs_in_ato(*v))
        .collect()
}

/// The build to run. No `--version` gives 3.153.
/// A label that is not a 3.x to 5.x build is an error, with the labels Ato
/// knows.
pub fn build(a: &Args) -> Result<ClientVersion, String> {
    let Some(label) = a.version.as_deref() else {
        return Ok(ClientVersion::V3153);
    };
    match ClientVersion::from_label(label) {
        Some(v) if runs_in_ato(v) => Ok(v),
        _ => {
            let known: Vec<&str> = builds().iter().map(|v| v.label()).collect();
            Err(format!(
                "seihai-ato runs the 3.x to 5.x builds ({}); {label:?} is not one of them",
                known.join(", ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Args {
        parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn launcher_style_argv() {
        let a = p("--online 127.0.0.1:14900 --version 3.153 --account guest --password-env GRAAL_PW --assets dir");
        assert_eq!(a.version.as_deref(), Some("3.153"));
        assert_eq!(a.account.as_deref(), Some("guest"));
        assert_eq!(a.password_env.as_deref(), Some("GRAAL_PW"));
        assert_eq!(build(&a), Ok(ClientVersion::V3153));
        assert_eq!(a.online.as_deref(), Some("127.0.0.1:14900"));
        assert_eq!(a.assets.as_deref(), Some("dir"));
        assert!(!a.no_window && !a.first_level);
    }

    #[test]
    fn last_version_wins_and_unknown_flags_are_ignored() {
        let a = p("--version 2.31 --frobnicate --version 3.04");
        assert_eq!(build(&a), Ok(ClientVersion::V304));
    }

    #[test]
    fn default_and_rejection() {
        assert_eq!(build(&p("")), Ok(ClientVersion::V3153));
        assert!(build(&p("--version 2.31")).is_err());
        assert!(build(&p("--version nope")).is_err());
    }

    #[test]
    fn value_flags_do_not_eat_the_next_flag() {
        let a = p("--online --version 3.04");
        // `--online` takes `--version` as its value here, exactly as mae
        // would; the version is then unset.
        assert_eq!(a.version, None);
        let a = p("--assets dir --version 3.04");
        assert_eq!(a.version.as_deref(), Some("3.04"));
    }

    #[test]
    fn ato_runs_3x_to_5x() {
        let labels: Vec<&str> = builds().iter().map(|v| v.label()).collect();
        for l in [
            "3.04", "3.153", "4.04r2", "4.20r8", "5.00r7", "5.12", "5.31r4",
        ] {
            assert!(labels.contains(&l), "{l} in {labels:?}");
        }
        for v in ClientVersion::all_versions() {
            let l = v.label();
            let want = l.starts_with('3') || l.starts_with('4') || l.starts_with('5');
            assert_eq!(runs_in_ato(*v), want, "{l}");
        }
        assert_eq!(build(&p("--version 4.20r8")), Ok(ClientVersion::V420R8));
        assert!(build(&p("--version 2.31")).is_err());
        assert!(build(&p("--version 6.03r7")).is_err());
    }

    #[test]
    fn listserver_is_taken() {
        let a = p("--version 3.04 --listserver 127.0.0.1:14922 --account a");
        assert_eq!(a.listserver.as_deref(), Some("127.0.0.1:14922"));
        assert_eq!(a.account.as_deref(), Some("a"));
    }

    #[test]
    fn graal3d_is_a_switch() {
        assert!(!p("--version 5.12").graal3d);
        assert!(p("--version 5.12 --graal3d --online h:1").graal3d);
    }

    #[test]
    fn no_sound_turns_sound_off() {
        assert!(!p("--version 3.04").no_sound);
        assert!(p("--version 3.04 --no-sound").no_sound);
    }

    #[test]
    fn level_is_taken_for_offline_play() {
        let a = p("--version 3.04 -l start.nw --assets dir");
        assert_eq!(a.level.as_deref(), Some("start.nw"));
        assert_eq!(p("--level other.nw").level.as_deref(), Some("other.nw"));
        assert_eq!(p("--version 3.04").level, None);
    }
}
