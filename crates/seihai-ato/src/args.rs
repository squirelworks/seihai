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
}

/// Flags that take a value. Unknown flags are skipped on their own, so a
/// value-taking flag Ato does not use must be listed here, or its value
/// would be read as the next flag.
const VALUE_FLAGS: &[&str] = &["--online", "-o", "--assets", "--server", "--level", "--listserver"];

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
            f if VALUE_FLAGS.contains(&f) => {
                it.next();
            }
            _ => {}
        }
    }
    a
}

/// The 3.x builds Ato runs.
pub const BUILDS: [ClientVersion; 2] = [ClientVersion::V304, ClientVersion::V3153];

/// The build to run. No `--version` gives 3.153, the newer of the two.
/// A label that is not a 3.x build is an error, with the labels Ato knows.
pub fn build(a: &Args) -> Result<ClientVersion, String> {
    let Some(label) = a.version.as_deref() else { return Ok(ClientVersion::V3153) };
    match ClientVersion::from_label(label) {
        Some(v) if BUILDS.contains(&v) => Ok(v),
        _ => {
            let known: Vec<&str> = BUILDS.iter().map(|v| v.label()).collect();
            Err(format!("seihai-ato runs the 3.x builds ({}); {label:?} is not one of them", known.join(", ")))
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
}
