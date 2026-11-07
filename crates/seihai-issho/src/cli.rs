//! Command line for the `seihai-issho` binary. No window and no default
//! version: a missing `--version` is an error, not v6.

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
        return ok("seihai-issho --version <6.x profile>\n\
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
    args.iter()
        .rposition(|arg| arg == "--version")
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
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
