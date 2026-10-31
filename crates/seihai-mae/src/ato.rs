//! Hand a 3.x pick over to `seihai-ato`.
//!
//! The Torque / 3.x builds run in `seihai-ato`, which has the 3.x start
//! window. Picking 3.04 or 3.153 in the settings window closes it and starts
//! Ato for that build, its start window filled in with the account and
//! password from the form. No server is passed, so the start window is
//! shown and not skipped (Ato skips it only with both `--online` and
//! `--account`).
//!
//! The password goes to Ato in an environment variable, never on its
//! command line. The binary is looked for the way `seihai-launcher` looks
//! for it: next to this one first, then on `PATH`.

use crate::protocol::ClientVersion;
use std::path::PathBuf;
use std::process::Command;

/// The binary 3.x builds run in.
pub const BINARY: &str = "seihai-ato";

/// The environment variable the password is handed over in
/// (`--password-env`).
pub const PASSWORD_ENV: &str = "SEIHAI_ATO_PASSWORD";

/// True for a build that runs in `seihai-ato`. The token says it, as in
/// `seihai-launcher`: `GNW` for the 2D builds, `G3D` for the 3.x ones. The
/// 4.x, 5.x and 6.x builds also send `G3D` tokens, but play on the flat path
/// here (their profile has the Login lobby).
pub fn runs_in_ato(v: ClientVersion) -> bool {
    v.token().starts_with("G3D") && v.profile().protocol.lobby.is_none()
}

/// What Ato is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub version: ClientVersion,
    pub account: String,
    pub password: String,
    /// The asset folder set for this build only, if any.
    pub assets: Option<String>,
}

impl Handoff {
    /// Ato's arguments. Empty fields are left out, so Ato's own defaults
    /// apply.
    pub fn args(&self) -> Vec<String> {
        let mut out = vec!["--version".to_string(), self.version.label().to_string()];
        if !self.account.is_empty() {
            out.extend(["--account".to_string(), self.account.clone()]);
        }
        if !self.password.is_empty() {
            out.extend(["--password-env".to_string(), PASSWORD_ENV.to_string()]);
        }
        if let Some(dir) = self.assets.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
            out.extend(["--assets".to_string(), dir.to_string()]);
        }
        out
    }

    /// The command that starts Ato.
    pub fn command(&self) -> Command {
        let mut cmd = Command::new(find_binary());
        cmd.args(self.args());
        if !self.password.is_empty() {
            cmd.env(PASSWORD_ENV, &self.password);
        }
        cmd
    }

    /// Start Ato in place of this process. Returns only if that failed, with
    /// the text to show.
    pub fn start(&self) -> String {
        let bin = find_binary();
        let err = run(self.command());
        format!("cannot start {}: {err}. Build it with `cargo build --release -p {BINARY}`.", bin.display())
    }
}

/// The Ato binary next to this one, or the bare name for a `PATH` lookup.
fn find_binary() -> PathBuf {
    let file = format!("{BINARY}{}", std::env::consts::EXE_SUFFIX);
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(&file)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(file))
}

/// Replace this process with Ato. Returns only if that failed.
#[cfg(unix)]
fn run(mut cmd: Command) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    cmd.exec()
}

/// Windows has no `exec`: run Ato and leave with its exit code.
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

    #[test]
    fn only_the_3x_builds_run_in_ato() {
        assert!(runs_in_ato(ClientVersion::V304));
        assert!(runs_in_ato(ClientVersion::V3153));
        for v in ClientVersion::all_versions() {
            assert_eq!(runs_in_ato(*v), v.label().starts_with("3."), "{}", v.label());
        }
    }

    #[test]
    fn the_form_fills_the_start_window_without_skipping_it() {
        let h = Handoff {
            version: ClientVersion::V3153,
            account: "guest".into(),
            password: "secret".into(),
            assets: Some(" dir ".into()),
        };
        let args = h.args();
        assert_eq!(args, ["--version", "3.153", "--account", "guest", "--password-env", PASSWORD_ENV, "--assets", "dir"]);
        // The password is in the environment, not on the command line.
        assert!(!args.iter().any(|a| a.contains("secret")));
        let cmd = h.command();
        assert!(cmd.get_envs().any(|(k, v)| k == PASSWORD_ENV && v == Some("secret".as_ref())));
        // No server: Ato shows its start window.
        assert!(!args.iter().any(|a| a == "--online"));
    }

    #[test]
    fn empty_fields_are_left_out() {
        let h = Handoff { version: ClientVersion::V304, account: String::new(), password: String::new(), assets: Some(" ".into()) };
        assert_eq!(h.args(), ["--version", "3.04"]);
        assert_eq!(h.command().get_envs().count(), 0);
    }
}
