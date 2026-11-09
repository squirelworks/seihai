//! Hand a 3.x pick over to `seihai-ato`, and a Graal3D pick to `seihai-g3d`.
//!
//! The Torque / 3.x builds run in `seihai-ato`, which has the 3.x start
//! window. Picking 3.04 or 3.153 in the settings window closes it and starts
//! Ato for that build, its start window filled in with the account and
//! password from the form. No server is passed, so the start window is
//! shown and not skipped (Ato skips it only with both `--online` and
//! `--account`).
//!
//! Graal3D (February 2000) is not a version of this client at all: it has
//! its own server and wire. Picking "Graal3D" starts `seihai-g3d` the same
//! way, with no server, so its connect form is shown, filled with the
//! account and password.
//!
//! The password goes over in an environment variable, never on the
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

/// The binary Graal3D runs in, and its password variable.
pub const G3D_BINARY: &str = "seihai-g3d";
pub const G3D_PASSWORD_ENV: &str = "SEIHAI_G3D_PASSWORD";

/// The version-list label for Graal3D. Not a [`ClientVersion`]: it is
/// shown after them, and only ever hands over to `seihai-g3d`.
pub const GRAAL3D_LABEL: &str = "Graal3D";

/// Which client a pick starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A 3.x build, in `seihai-ato`.
    Ato(ClientVersion),
    /// Graal3D (February 2000), in `seihai-g3d`.
    Graal3D,
}

impl Target {
    pub fn binary(self) -> &'static str {
        match self {
            Target::Ato(_) => BINARY,
            Target::Graal3D => G3D_BINARY,
        }
    }

    fn password_env(self) -> &'static str {
        match self {
            Target::Ato(_) => PASSWORD_ENV,
            Target::Graal3D => G3D_PASSWORD_ENV,
        }
    }

    /// The `--version` value. `seihai-g3d` and `seihai-launcher` both take
    /// `graal3d`.
    fn version_arg(self) -> &'static str {
        match self {
            Target::Ato(v) => v.label(),
            Target::Graal3D => "graal3d",
        }
    }
}

/// True for a build that runs in `seihai-ato`. The token says it, as in
/// `seihai-launcher`: `GNW` for the 2D builds, `G3D` for the 3.x ones. The
/// 4.x, 5.x and 6.x builds also send `G3D` tokens, but play on the flat path
/// here (their profile has the Login lobby).
pub fn runs_in_ato(v: ClientVersion) -> bool {
    v.token().starts_with("G3D") && v.profile().protocol.lobby.is_none()
}

/// What Ato or G3D is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub target: Target,
    pub account: String,
    pub password: String,
    /// The asset folder set for this build only, if any.
    pub assets: Option<String>,
}

impl Handoff {
    /// The arguments. Empty fields are left out, so the client's own
    /// defaults apply.
    pub fn args(&self) -> Vec<String> {
        let mut out = vec![
            "--version".to_string(),
            self.target.version_arg().to_string(),
        ];
        if !self.account.is_empty() {
            out.extend(["--account".to_string(), self.account.clone()]);
        }
        if !self.password.is_empty() {
            out.extend([
                "--password-env".to_string(),
                self.target.password_env().to_string(),
            ]);
        }
        if let Some(dir) = self
            .assets
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
        {
            out.extend(["--assets".to_string(), dir.to_string()]);
        }
        out
    }

    /// The command that starts the client.
    pub fn command(&self) -> Command {
        let mut cmd = Command::new(find_binary(self.target.binary()));
        cmd.args(self.args());
        if !self.password.is_empty() {
            cmd.env(self.target.password_env(), &self.password);
        }
        cmd
    }

    /// Start the client in place of this process. Returns only if that
    /// failed, with the text to show.
    pub fn start(&self) -> String {
        let name = self.target.binary();
        let bin = find_binary(name);
        let err = run(self.command());
        format!(
            "cannot start {}: {err}. Build it with `cargo build --release -p {name}`.",
            bin.display()
        )
    }
}

/// The binary next to this one, or the bare name for a `PATH` lookup.
fn find_binary(name: &str) -> PathBuf {
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(&file)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(file))
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

    #[test]
    fn only_the_3x_builds_run_in_ato() {
        assert!(runs_in_ato(ClientVersion::V304));
        assert!(runs_in_ato(ClientVersion::V3153));
        for v in ClientVersion::all_versions() {
            assert_eq!(
                runs_in_ato(*v),
                v.label().starts_with("3."),
                "{}",
                v.label()
            );
        }
    }

    #[test]
    fn the_form_fills_the_start_window_without_skipping_it() {
        let h = Handoff {
            target: Target::Ato(ClientVersion::V3153),
            account: "guest".into(),
            password: "secret".into(),
            assets: Some(" dir ".into()),
        };
        let args = h.args();
        assert_eq!(
            args,
            [
                "--version",
                "3.153",
                "--account",
                "guest",
                "--password-env",
                PASSWORD_ENV,
                "--assets",
                "dir"
            ]
        );
        // The password is in the environment, not on the command line.
        assert!(!args.iter().any(|a| a.contains("secret")));
        let cmd = h.command();
        assert!(cmd
            .get_envs()
            .any(|(k, v)| k == PASSWORD_ENV && v == Some("secret".as_ref())));
        // No server: Ato shows its start window.
        assert!(!args.iter().any(|a| a == "--online"));
    }

    #[test]
    fn empty_fields_are_left_out() {
        let h = Handoff {
            target: Target::Ato(ClientVersion::V304),
            account: String::new(),
            password: String::new(),
            assets: Some(" ".into()),
        };
        assert_eq!(h.args(), ["--version", "3.04"]);
        assert_eq!(h.command().get_envs().count(), 0);
    }

    #[test]
    fn graal3d_starts_g3d_with_its_own_password_variable() {
        let h = Handoff {
            target: Target::Graal3D,
            account: "lab".into(),
            password: "secret".into(),
            assets: Some("g3d".into()),
        };
        assert_eq!(
            h.args(),
            [
                "--version",
                "graal3d",
                "--account",
                "lab",
                "--password-env",
                G3D_PASSWORD_ENV,
                "--assets",
                "g3d"
            ]
        );
        assert!(h
            .command()
            .get_program()
            .to_string_lossy()
            .contains(G3D_BINARY));
        assert!(h
            .command()
            .get_envs()
            .any(|(k, v)| k == G3D_PASSWORD_ENV && v == Some("secret".as_ref())));
        assert!(!h
            .args()
            .iter()
            .any(|a| a == "--online" || a.contains("secret")));
    }
}
