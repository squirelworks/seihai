//! Settings file.
//!
//! Plain key=value text at `seihai.conf` next to the binary, or
//! `~/.config/seihai/seihai.conf`.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub assets: Option<PathBuf>,
    pub account: Option<String>,
    pub password: Option<String>,
    pub version: Option<String>,
    pub last_server: Option<String>,
    pub listserver: Option<String>,
    pub offline: Option<bool>,
    /// Follow a `PLO_SERVERWARP` (178) to the server it names. `None` means
    /// the key is absent, which is the default: follow, as the real client
    /// does. `follow_serverwarp=0` turns it off.
    pub follow_serverwarp: Option<bool>,
    /// Name of the server in `last_server`, for the window title.
    pub last_server_name: Option<String>,
    /// Game server for versions with no listserver (1.33–1.39).
    pub direct_server: Option<String>,
    /// Asset folders for single versions (`assets.1.39=...`), so a
    /// version can use its own install's images.
    pub version_assets: Vec<(String, PathBuf)>,
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        let text = fs::read_to_string(&path).unwrap_or_default();
        Self::parse(&text)
    }

    /// Read settings from the text of a settings file.
    pub fn parse(text: &str) -> Self {
        let mut s = Settings::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let v = v.trim().to_string();
                match k.trim() {
                    "assets" => s.assets = Some(PathBuf::from(v)),
                    "account" => s.account = Some(v),
                    "password" => s.password = Some(v),
                    "version" => s.version = Some(v),
                    "last_server" => s.last_server = Some(v),
                    "listserver" => s.listserver = Some(v),
                    "offline" => s.offline = Some(v == "1"),
                    // An unreadable value leaves the default (follow).
                    "follow_serverwarp" => s.follow_serverwarp = parse_bool(&v),
                    "last_server_name" => s.last_server_name = Some(v),
                    "direct_server" => s.direct_server = Some(v),
                    k => {
                        if let Some(label) = k.strip_prefix("assets.") {
                            s.version_assets.push((label.to_string(), PathBuf::from(v)));
                        }
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&path, self.to_text());
    }

    /// The text of the settings file for these settings.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        if let Some(p) = &self.assets {
            out.push_str(&format!("assets={}\n", p.display()));
        }
        if let Some(a) = &self.account {
            out.push_str(&format!("account={a}\n"));
        }
        if let Some(p) = &self.password {
            out.push_str(&format!("password={p}\n"));
        }
        if let Some(v) = &self.version {
            out.push_str(&format!("version={v}\n"));
        }
        if let Some(s) = &self.last_server {
            out.push_str(&format!("last_server={s}\n"));
        }
        if let Some(l) = &self.listserver {
            out.push_str(&format!("listserver={l}\n"));
        }
        if self.offline == Some(true) {
            out.push_str("offline=1\n");
        }
        if let Some(f) = self.follow_serverwarp {
            out.push_str(&format!("follow_serverwarp={}\n", u8::from(f)));
        }
        if let Some(n) = &self.last_server_name {
            out.push_str(&format!("last_server_name={n}\n"));
        }
        if let Some(d) = &self.direct_server {
            out.push_str(&format!("direct_server={d}\n"));
        }
        for (label, p) in &self.version_assets {
            out.push_str(&format!("assets.{label}={}\n", p.display()));
        }
        out
    }

    /// Whether to follow a server warp. The command line wins over the
    /// settings file, and the file wins over the default (follow).
    /// `cli` is `Some(false)` for `--no-serverwarp` and `Some(true)` for
    /// `--serverwarp`.
    pub fn follow_serverwarp_with(&self, cli: Option<bool>) -> bool {
        cli.or(self.follow_serverwarp).unwrap_or(true)
    }
}

/// `1`/`0`, `true`/`false`, `yes`/`no`, `on`/`off`, in any case.
fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

impl Settings {
    /// The asset folder set for one version, if any.
    pub fn assets_for(&self, label: &str) -> Option<&Path> {
        self.version_assets
            .iter()
            .find(|(l, p)| l == label && !p.as_os_str().is_empty())
            .map(|(_, p)| p.as_path())
    }

    /// Set or clear the asset folder for one version.
    pub fn set_assets_for(&mut self, label: &str, path: &str) {
        self.version_assets.retain(|(l, _)| l != label);
        if !path.trim().is_empty() {
            self.version_assets.push((label.to_string(), PathBuf::from(path.trim())));
        }
    }
}

fn config_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/seihai/seihai.conf")
    } else if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join(".seihai/seihai.conf")
    } else {
        PathBuf::from("seihai.conf")
    }
}

impl AsRef<Path> for Settings {
    fn as_ref(&self) -> &Path {
        Path::new("seihai.conf")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverwarp_is_followed_by_default() {
        let s = Settings::parse("");
        assert_eq!(s.follow_serverwarp, None);
        assert!(s.follow_serverwarp_with(None));
        // A settings file that says nothing about it changes nothing.
        let s = Settings::parse("account=x\nversion=2.31\n");
        assert!(s.follow_serverwarp_with(None));
    }

    #[test]
    fn settings_file_can_turn_it_off() {
        for v in ["0", "false", "no", "off", "FALSE", "Off"] {
            let s = Settings::parse(&format!("follow_serverwarp = {v}\n"));
            assert_eq!(s.follow_serverwarp, Some(false), "{v}");
            assert!(!s.follow_serverwarp_with(None), "{v}");
        }
        for v in ["1", "true", "yes", "on"] {
            let s = Settings::parse(&format!("follow_serverwarp={v}\n"));
            assert!(s.follow_serverwarp_with(None), "{v}");
        }
    }

    #[test]
    fn unreadable_value_keeps_the_default() {
        let s = Settings::parse("follow_serverwarp=maybe\n");
        assert_eq!(s.follow_serverwarp, None);
        assert!(s.follow_serverwarp_with(None));
    }

    #[test]
    fn command_line_overrides_the_file() {
        let off = Settings::parse("follow_serverwarp=0\n");
        let on = Settings::parse("follow_serverwarp=1\n");
        let none = Settings::default();
        // --no-serverwarp always wins.
        assert!(!off.follow_serverwarp_with(Some(false)));
        assert!(!on.follow_serverwarp_with(Some(false)));
        assert!(!none.follow_serverwarp_with(Some(false)));
        // --serverwarp beats a file that says off.
        assert!(off.follow_serverwarp_with(Some(true)));
        assert!(on.follow_serverwarp_with(Some(true)));
        // No flag: the file decides.
        assert!(!off.follow_serverwarp_with(None));
        assert!(on.follow_serverwarp_with(None));
    }

    #[test]
    fn the_option_survives_a_save_and_load() {
        // The launcher loads, edits and saves the file, so a key it does not
        // touch must come back out.
        let s = Settings::parse("account=a\nfollow_serverwarp=0\n");
        let text = s.to_text();
        assert!(text.contains("follow_serverwarp=0\n"), "{text}");
        let back = Settings::parse(&text);
        assert_eq!(back.follow_serverwarp, Some(false));
        assert_eq!(back.account.as_deref(), Some("a"));
        // An unset option stays unset, so the file stays as it was.
        assert!(!Settings::parse("account=a\n").to_text().contains("follow_serverwarp"));
    }
}
