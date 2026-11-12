//! Command line for the `seihai-issho` binary. No window and no default
//! version: a missing `--version` is an error, not v6.
//!
//! `--online host:port` (`-o`, `--server`) logs in to that game server as
//! the 6.x build with core's client session (`seihai_core::client`, the
//! loop `seihai-ato` runs) and logs each level it sends, as
//! `seihai-ato --no-window` does. The password is read from the variable
//! `--password-env` names, never from the command line.

use crate::profile::{select_label, EngineProfile, SelectError};
use crate::shell::Session;
use std::process::ExitCode;

/// What the binary would print and return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub code: u8,
    pub stdout: String,
    pub stderr: String,
}

/// Run one invocation. `args` is what follows the program name.
pub fn run(args: &[String]) -> Outcome {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return ok("seihai-issho --version <6.x profile> [--online host:port \
--account NAME --password-env VAR [--assets DIR] [--first-level]]\n\
v6 profiles: 6.00r2, 6.015, 6.034, 6.03r7, 6.03r7 Linux\n\
v7 is not a supported profile.\n");
    }
    let Some(label) = version_arg(args) else {
        return err(
            "seihai-issho: pass --version with a v6 profile. Issho does not default to v6.\n",
        );
    };
    match select_label(label) {
        Ok(EngineProfile::V6) => {
            let version = seihai_core::protocol::ClientVersion::from_label(label)
                .expect("v6 label has a profile");
            match Session::open(version) {
                Ok(session) => ok(format!("{}\n", session.summary())),
                Err(error) => err(format!("seihai-issho: {error}\n")),
            }
        }
        // Not constructed. Unity is the expected engine. Any later variant
        // is refused the same way.
        Ok(EngineProfile::V7(_)) => err(format!("seihai-issho: {}\n", SelectError::UnsupportedV7)),
        Err(error) => err(format!("seihai-issho: {label}: {error}\n")),
    }
}

pub fn exit_code(outcome: &Outcome) -> ExitCode {
    ExitCode::from(outcome.code)
}

fn ok(stdout: impl Into<String>) -> Outcome {
    Outcome {
        code: 0,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

fn err(stderr: impl Into<String>) -> Outcome {
    Outcome {
        code: 2,
        stdout: String::new(),
        stderr: stderr.into(),
    }
}

/// The value of the last `--version`, as the launcher passes it.
fn version_arg(args: &[String]) -> Option<&str> {
    value(args, &["--version"])
}

/// The value after the last of `flags`.
fn value<'a>(args: &'a [String], flags: &[&str]) -> Option<&'a str> {
    args.iter()
        .rposition(|arg| flags.contains(&arg.as_str()))
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

/// A login asked for on the command line (`--online`).
#[derive(Debug, Clone, PartialEq)]
pub struct Online {
    pub addr: String,
    pub version: seihai_core::protocol::ClientVersion,
    pub login: seihai_core::client::Login,
    pub assets: Option<std::path::PathBuf>,
    pub first_level: bool,
}

/// The login `args` ask for: `None` without `--online`, an error message
/// when the version is not a v6 profile or the account or password is
/// missing. `env` reads the variable `--password-env` names.
pub fn online(
    args: &[String],
    env: impl Fn(&str) -> Option<String>,
) -> Option<Result<Online, String>> {
    let addr = value(args, &["--online", "-o", "--server"])?;
    Some(online_login(args, addr, env))
}

fn online_login(
    args: &[String],
    addr: &str,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Online, String> {
    let label = version_arg(args)
        .ok_or("pass --version with a v6 profile. Issho does not default to v6.")?;
    let version = match select_label(label) {
        Ok(EngineProfile::V6) => {
            seihai_core::protocol::ClientVersion::from_label(label).expect("v6 label has a profile")
        }
        Ok(EngineProfile::V7(_)) => return Err(SelectError::UnsupportedV7.to_string()),
        Err(error) => return Err(format!("{label}: {error}")),
    };
    let account = value(args, &["--account"]).ok_or("--online needs --account")?;
    let var = value(args, &["--password-env"]).ok_or(
        "--online needs --password-env VAR (the password is never taken on the command line)",
    )?;
    let password = env(var).ok_or_else(|| format!("--password-env: {var} is not set"))?;
    let nick = value(args, &["--nick", "--nickname"]).unwrap_or(account);
    Ok(Online {
        addr: addr.to_string(),
        version,
        login: seihai_core::client::Login {
            account: account.to_string(),
            password,
            nick: nick.to_string(),
            language: "English".into(),
        },
        assets: value(args, &["--assets"]).map(std::path::PathBuf::from),
        first_level: args.iter().any(|a| a == "--first-level"),
    })
}

/// Log in as `o` asks and log each level to `out` (core's client session).
pub fn run_online(o: &Online, out: &mut impl std::io::Write) -> Result<(), String> {
    seihai_core::client::run(
        &o.addr,
        o.version,
        &o.login,
        o.assets.clone(),
        o.first_level,
        out,
        &mut |_, _| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().copied().map(str::to_string).collect()
    }

    #[test]
    fn v6_selects_the_shell() {
        let out = run(&args(&["--version", "6.03r7"]));
        assert_eq!(out.code, 0);
        assert!(out.stdout.contains("6.03r7"), "{}", out.stdout);
        assert!(out.stdout.contains("quattro_lib"), "{}", out.stdout);
        assert!(out.stdout.contains("TSocket"), "{}", out.stdout);
        assert!(out.stderr.is_empty());
    }

    #[test]
    fn linux_v6_is_one_label() {
        let out = run(&args(&["--version", "6.03r7 Linux"]));
        assert_eq!(out.code, 0);
        assert!(out.stdout.contains("6.03r7 Linux"), "{}", out.stdout);
    }

    #[test]
    fn v3_to_v5_are_rejected() {
        for label in ["3.04", "3.153", "4.20r8", "5.12", "5.31r4"] {
            let out = run(&args(&["--version", label]));
            assert_eq!(out.code, 2, "{label}");
            assert!(
                out.stderr.contains("does not select") || out.stderr.contains("not a v6"),
                "{label}: {}",
                out.stderr
            );
        }
    }

    #[test]
    fn v7_is_rejected() {
        let out = run(&args(&["--version", "7.0"]));
        assert_eq!(out.code, 2);
        assert!(out.stderr.contains("Unity"), "{}", out.stderr);
        assert!(
            out.stderr.contains("not a supported profile"),
            "{}",
            out.stderr
        );
    }

    fn env(name: &str) -> Option<String> {
        (name == "PW").then(|| "secret".to_string())
    }

    #[test]
    fn online_takes_the_login() {
        let a = args(&[
            "--version",
            "6.03r7",
            "--online",
            "127.0.0.1:14900",
            "--account",
            "guest",
            "--password-env",
            "PW",
            "--first-level",
        ]);
        let o = online(&a, env).unwrap().unwrap();
        assert_eq!(o.addr, "127.0.0.1:14900");
        assert_eq!(o.version, seihai_core::protocol::ClientVersion::V603R7);
        assert_eq!(
            (o.login.account.as_str(), o.login.password.as_str()),
            ("guest", "secret")
        );
        assert_eq!(o.login.nick, "guest");
        assert!(o.first_level && o.assets.is_none());
        assert!(online(&args(&["--version", "6.03r7"]), env).is_none());
    }

    #[test]
    fn online_refuses_a_missing_password_or_a_non_v6_build() {
        let base = ["--online", "h:1", "--account", "a"];
        let mut a = args(&base);
        a.extend(args(&["--version", "6.03r7"]));
        assert!(online(&a, env)
            .unwrap()
            .unwrap_err()
            .contains("--password-env"));
        a.extend(args(&["--password-env", "NOPE"]));
        assert!(online(&a, env)
            .unwrap()
            .unwrap_err()
            .contains("NOPE is not set"));
        let mut a = args(&base);
        a.extend(args(&["--version", "3.04", "--password-env", "PW"]));
        assert!(online(&a, env).unwrap().is_err());
    }

    #[test]
    fn no_version_does_not_default_to_v6() {
        let out = run(&args(&[]));
        assert_eq!(out.code, 2);
        assert!(
            out.stderr.contains("does not default to v6"),
            "{}",
            out.stderr
        );
    }
}
