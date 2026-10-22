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
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        let text = fs::read_to_string(&path).unwrap_or_default();
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
                    _ => {}
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
        let _ = fs::write(&path, out);
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
