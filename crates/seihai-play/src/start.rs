//! The one start window for every line: pick the version, fill in the
//! account, fetch the server list and pick a server. The `seihai` command
//! opens it when it is started with no arguments.
//!
//! The window does not run a game. Joining a server hands the pick over
//! ([`Handoff`]) to the client of the version's line, which the `seihai`
//! program runs in the same process: Mae for 1.x and 2.x, Ato for 3.x to
//! 5.x, Issho for 6.x and G3D for Graal3D (February 2000). The client is started with
//! `--online` and `--account`, so it logs in at once and its own start
//! window stays closed. For a Mae version, "Play offline" and "Play the
//! tutorial" start Mae offline (`--offline`, `--tutorial`), and "All
//! versions" lists the offline-only builds with Mae's rows. Ato and G3D
//! keep their own start windows, the original clients' screens, and "Open
//! Ato's own window" starts one. Mae's settings window is only the
//! `seihai-mae` development binary's: inside the `seihai` program Mae
//! comes back here instead.
//!
//! The form is saved in `launcher.conf`. Before the first save it starts
//! from the form Mae's window saved in `seihai.conf`, without the
//! password; the per-version asset folders (`assets.<label>`) are the same
//! keys.
//!
//! A server listed with the `3 ` prefix is a 3D server. A 3.x to 5.x build
//! joins it with `--graal3d`, so Ato opens the Torque UDP link next to the
//! TCP login.
//!
//! The password goes over in an environment variable ([`PASSWORD_ENV`]),
//! never on the command line, and is not saved. The rest of the form is
//! saved in `launcher.conf` in the settings folder.
//!
//! This module holds only the window and the hand-over; the server list
//! login is core's (`Connection::fetch_server_list`), through
//! [`crate::servers::ServerList`], which Ato's own list screen shows too.
//!
//! In consumer mode (the default for a new `launcher.conf`) there is no
//! version box: a join picks the version for that server, and a version
//! refusal brings the client back here to join again as the version the
//! server takes ([`crate::consumer`]). Developer mode, the default for an
//! existing `launcher.conf`, keeps the version box.

use crate::consumer::{self, Mode, Next, Pending};
use crate::servers::ServerList;
use seihai_core::protocol::offline_builds::{self, OfflineBuild};
use seihai_core::protocol::{self as proto, ClientVersion, ServerEntry};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The environment variable the password is handed over in
/// (`--password-env`). Every client reads `--password-env`.
pub const PASSWORD_ENV: &str = "SEIHAI_PASSWORD";

/// Set by the `seihai` command for the client it starts: its own path. A
/// client that goes back to a start window (after a lost connection) starts
/// this with `--launcher-error` instead of its own window.
pub const LAUNCHER_ENV: &str = "SEIHAI_LAUNCHER";

/// Why the game the `seihai` start window handed over to ended, for the
/// start window to show when it comes back.
static ENDED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// True when a client runs inside the `seihai` program's start window
/// loop, so a lost connection goes back to that window.
pub fn under_launcher() -> bool {
    std::env::var_os(LAUNCHER_ENV).is_some()
}

/// Note why the game ended (see [`take_ended`]).
pub fn report_ended(why: &str) {
    *ENDED.lock().unwrap_or_else(|e| e.into_inner()) = Some(why.to_string());
}

/// The reason [`report_ended`] noted, once.
pub fn take_ended() -> Option<String> {
    ENDED.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Set by the `seihai` program for every client it runs: its own path.
/// A client that hands over to another line (Mae's version box picking a
/// 3.x build) starts `$SEIHAI_PROGRAM <subcommand> ARGS...`, since the
/// separate client binaries are not shipped.
pub const PROGRAM_ENV: &str = "SEIHAI_PROGRAM";

/// The client lines. Each is a subcommand of the `seihai` program and a
/// development binary of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    /// 1.x and 2.x: `seihai-mae`.
    Mae,
    /// Torque / 3.x to 5.x: `seihai-ato`.
    Ato,
    /// v6 quattro_lib: `seihai-issho`.
    Issho,
    /// Graal3D (February 2000), the standalone 3D client: `seihai-g3d`.
    G3d,
}

impl Line {
    pub fn binary(self) -> &'static str {
        match self {
            Line::Mae => "seihai-mae",
            Line::Ato => "seihai-ato",
            Line::Issho => "seihai-issho",
            Line::G3d => "seihai-g3d",
        }
    }

    /// The `seihai` subcommand that runs this line.
    pub fn subcommand(self) -> &'static str {
        match self {
            Line::Mae => "mae",
            Line::Ato => "ato",
            Line::Issho => "issho",
            Line::G3d => "g3d",
        }
    }

    /// The line a subcommand or a program name names: `ato`, `seihai-ato`,
    /// `seihai-ato.exe`, in any case.
    pub fn from_name(name: &str) -> Option<Line> {
        let n = name.to_ascii_lowercase();
        let n = n.strip_suffix(".exe").unwrap_or(&n);
        let n = n.strip_prefix("seihai-").unwrap_or(n);
        [Line::Mae, Line::Ato, Line::Issho, Line::G3d]
            .into_iter()
            .find(|l| l.subcommand() == n)
    }

    /// The client's name, as the window says it.
    pub fn name(self) -> &'static str {
        match self {
            Line::Mae => "Mae",
            Line::Ato => "Ato",
            Line::Issho => "Issho",
            Line::G3d => "G3D",
        }
    }

    /// The heading the window lists this line's versions under.
    pub fn heading(self) -> &'static str {
        match self {
            Line::Mae => "Mae: 1.x and 2.x",
            Line::Ato => "Ato: 3.x to 5.x",
            Line::Issho => "Issho: 6.x",
            Line::G3d => "Graal3D (February 2000)",
        }
    }
}

/// Leading major version of a label (`6` from `6.03r7 Linux`).
fn major(label: &str) -> Option<u8> {
    let digits: String = label.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// The line a profile row runs in. 6.x is Issho. Otherwise the token says
/// it: `GNW` for the 2D builds, `G3D` for the 3.x to 5.x ones.
pub fn line_of(v: ClientVersion) -> Line {
    if major(v.label()) == Some(6) {
        Line::Issho
    } else if v.token().starts_with("G3D") {
        Line::Ato
    } else {
        Line::Mae
    }
}

/// What the version box holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Version(ClientVersion),
    /// An offline-only build (a beta or a test release with no server):
    /// Mae plays it offline with the rules of its `plays_as` row. Listed
    /// under "All versions".
    Offline(&'static str),
    /// Graal3D (February 2000): its own server and wire, not a profile row.
    Graal3D,
}

/// The `--version` value and the version box's text for Graal3D.
pub const GRAAL3D_ARG: &str = "graal3d";
pub const GRAAL3D_LABEL: &str = "Graal3D";

impl Choice {
    pub fn line(self) -> Line {
        match self {
            Choice::Version(v) => line_of(v),
            Choice::Offline(_) => Line::Mae,
            Choice::Graal3D => Line::G3d,
        }
    }

    /// The offline-only build, if this is one.
    pub fn offline_build(self) -> Option<&'static OfflineBuild> {
        match self {
            Choice::Offline(label) => offline_builds::by_label(label),
            _ => None,
        }
    }

    /// True when the version only plays offline.
    pub fn offline_only(self) -> bool {
        matches!(self, Choice::Offline(_))
    }

    pub fn label(self) -> &'static str {
        match self {
            Choice::Version(v) => v.label(),
            Choice::Offline(label) => label,
            Choice::Graal3D => GRAAL3D_LABEL,
        }
    }

    /// The `--version` value the client takes.
    pub fn version_arg(self) -> &'static str {
        match self {
            Choice::Version(v) => v.label(),
            Choice::Offline(label) => label,
            Choice::Graal3D => GRAAL3D_ARG,
        }
    }

    /// From a saved label or a `--version` value.
    pub fn from_label(label: &str) -> Option<Choice> {
        if matches!(
            label.trim().to_ascii_lowercase().as_str(),
            "graal3d" | "g3d" | "gr3dv002"
        ) {
            return Some(Choice::Graal3D);
        }
        ClientVersion::from_label(label.trim())
            .map(Choice::Version)
            .or_else(|| offline_builds::by_label(label.trim()).map(|b| Choice::Offline(b.label)))
    }

    /// True when there is no listserver to ask: Graal3D connects to one
    /// game server typed in the form. (The 1.x rows with no listserver of
    /// their own borrow 2.05's, as in `seihai-mae`.)
    pub fn direct_only(self) -> bool {
        self == Choice::Graal3D
    }

    /// True when the version's own client connects straight to a game
    /// server: the game server box is opened for it.
    pub fn direct_first(self) -> bool {
        match self {
            Choice::Graal3D => true,
            Choice::Version(v) => v.profile().ui.server_list == proto::ServerList::Direct,
            Choice::Offline(_) => false,
        }
    }
}

/// The versions the box lists: every line's, in profile order, then
/// Graal3D. Without `all`, only the default list (`reborn_versions`); with
/// it, every profile row and, after Mae's rows, the offline-only builds.
pub fn choices(all: bool) -> Vec<Choice> {
    let rows = if all {
        ClientVersion::all_versions()
    } else {
        ClientVersion::reborn_versions()
    };
    let rows: Vec<Choice> = rows.iter().copied().map(Choice::Version).collect();
    let mae_end = rows.iter().take_while(|c| c.line() == Line::Mae).count();
    let offline = if all {
        offline_builds::OFFLINE_BUILDS
    } else {
        &[]
    }
    .iter()
    .map(|b| Choice::Offline(b.label));
    rows[..mae_end]
        .iter()
        .copied()
        .chain(offline)
        .chain(rows[mae_end..].iter().copied())
        .chain([Choice::Graal3D])
        .collect()
}

/// How the client starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Start {
    /// Online at the hand-over's server; with no server, the client's own
    /// start window.
    #[default]
    Online,
    /// Mae offline, with no start window (`--offline`).
    Offline,
    /// Mae offline in the built-in tutorial (`--tutorial`).
    Tutorial,
}

/// What the client is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub choice: Choice,
    /// The game server (`host:port`). `None` opens the client's own window.
    pub server: Option<String>,
    pub account: String,
    pub password: String,
    pub nick: String,
    pub assets: Option<String>,
    /// The server is a 3D one (`3 ` in the list): Ato opens the UDP link.
    pub graal3d: bool,
    /// Online, or Mae offline.
    pub start: Start,
}

impl Handoff {
    pub fn line(&self) -> Line {
        self.choice.line()
    }

    /// The client's arguments. Empty fields are left out, so the client's
    /// own defaults apply.
    pub fn args(&self) -> Vec<String> {
        let mut out = vec![
            "--version".to_string(),
            self.choice.version_arg().to_string(),
        ];
        match (self.start, &self.server) {
            (Start::Online, Some(s)) if !s.trim().is_empty() => {
                out.extend(["--online".to_string(), s.trim().to_string()]);
            }
            (Start::Online, _) => {}
            (Start::Offline, _) => out.push("--offline".to_string()),
            (Start::Tutorial, _) => out.push("--tutorial".to_string()),
        }
        let mut push = |flag: &str, value: &str| {
            if !value.trim().is_empty() {
                out.push(flag.to_string());
                out.push(value.trim().to_string());
            }
        };
        push("--account", &self.account);
        if !self.password.is_empty() {
            push("--password-env", PASSWORD_ENV);
        }
        push("--nick", &self.nick);
        if let Some(a) = &self.assets {
            push("--assets", a);
        }
        if self.graal3d
            && self.start == Start::Online
            && self.server.is_some()
            && self.line() == Line::Ato
        {
            out.push("--graal3d".to_string());
        }
        out
    }

    /// The environment the client is started with: the password, if any.
    pub fn env(&self) -> Option<(&'static str, &str)> {
        (!self.password.is_empty()).then_some((PASSWORD_ENV, self.password.as_str()))
    }
}

/// A `3 ` server: a Graal3D world on a 3.x to 5.x listserver.
pub fn is_3d_server(s: &ServerEntry) -> bool {
    s.kind.trim() == "3"
}

/// The form, as saved in `launcher.conf` (no password).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Saved {
    pub version: Option<String>,
    pub account: Option<String>,
    pub nick: Option<String>,
    /// `host:port`, only when the user typed one.
    pub listserver: Option<String>,
    pub direct: Option<String>,
    pub assets: Option<String>,
    /// The asset folder of one version only (`assets.<label>`), as
    /// `seihai-mae` saves it: that build's own install.
    pub assets_for: BTreeMap<String, String>,
    pub all_versions: bool,
    /// `None` in a file saved before consumer mode.
    pub mode: Option<Mode>,
    /// The version to join each server as (`picked.<host:port>`).
    pub picked: BTreeMap<String, String>,
    /// Each server's last `Allowed:` list (`allowed.<host:port>`).
    pub allowed: BTreeMap<String, String>,
    /// The join in progress (`pending`, `pending_3d`, `pending_tried`,
    /// `pending_refused`).
    pub pending: Option<Pending>,
}

impl Saved {
    pub fn path() -> PathBuf {
        seihai_core::config::config_dir().join("launcher.conf")
    }

    pub fn parse(text: &str) -> Saved {
        let mut s = Saved::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = Some(v.trim().to_string()).filter(|v| !v.is_empty());
            match k.trim() {
                "version" => s.version = v,
                "account" => s.account = v,
                "nick" => s.nick = v,
                "listserver" => s.listserver = v,
                "direct" => s.direct = v,
                "assets" => s.assets = v,
                "all_versions" => s.all_versions = v.as_deref() == Some("1"),
                "mode" => s.mode = v.as_deref().and_then(Mode::from_key),
                "pending" => {
                    if let Some(v) = v {
                        s.pending.get_or_insert_with(Pending::default).server = v;
                    }
                }
                "pending_3d" => {
                    s.pending.get_or_insert_with(Pending::default).three_d =
                        v.as_deref() == Some("1")
                }
                "pending_tried" => {
                    s.pending.get_or_insert_with(Pending::default).tried = v.unwrap_or_default()
                }
                "pending_refused" => {
                    s.pending.get_or_insert_with(Pending::default).refused = v
                        .unwrap_or_default()
                        .split(',')
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect()
                }
                k => {
                    let Some(v) = v else { continue };
                    if let Some(label) = k.strip_prefix("assets.") {
                        s.assets_for.insert(label.to_string(), v);
                    } else if let Some(server) = k.strip_prefix("picked.") {
                        s.picked.insert(server.to_string(), v);
                    } else if let Some(server) = k.strip_prefix("allowed.") {
                        s.allowed.insert(server.to_string(), v);
                    }
                }
            }
        }
        // A pending join with no server is a broken file.
        if s.pending.as_ref().is_some_and(|p| p.server.is_empty()) {
            s.pending = None;
        }
        s
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (k, v) in [
            ("version", &self.version),
            ("account", &self.account),
            ("nick", &self.nick),
            ("listserver", &self.listserver),
            ("direct", &self.direct),
            ("assets", &self.assets),
        ] {
            if let Some(v) = v {
                out.push_str(&format!("{k}={v}\n"));
            }
        }
        for (label, dir) in &self.assets_for {
            out.push_str(&format!("assets.{label}={dir}\n"));
        }
        if self.all_versions {
            out.push_str("all_versions=1\n");
        }
        if let Some(m) = self.mode {
            out.push_str(&format!("mode={}\n", m.key()));
        }
        for (server, label) in &self.picked {
            out.push_str(&format!("picked.{server}={label}\n"));
        }
        for (server, list) in &self.allowed {
            out.push_str(&format!("allowed.{server}={list}\n"));
        }
        if let Some(p) = &self.pending {
            out.push_str(&format!("pending={}\n", p.server));
            if p.three_d {
                out.push_str("pending_3d=1\n");
            }
            if !p.tried.is_empty() {
                out.push_str(&format!("pending_tried={}\n", p.tried));
            }
            if !p.refused.is_empty() {
                out.push_str(&format!("pending_refused={}\n", p.refused.join(",")));
            }
        }
        out
    }

    /// `launcher.conf`; before the first save, the form `seihai-mae`'s
    /// own window last saved in `seihai.conf` (account, version, listserver,
    /// game server and asset folders; never its password).
    pub fn load() -> Saved {
        match std::fs::read_to_string(Self::path()) {
            Ok(t) => Saved::parse(&t),
            Err(_) => std::fs::read_to_string(seihai_core::config::config_path())
                .map(|t| Saved::from_mae(&t))
                .unwrap_or_default(),
        }
    }

    /// The fields of `seihai.conf` this window shares with Mae's.
    pub fn from_mae(text: &str) -> Saved {
        let mut s = Saved::parse(text);
        s.nick = None;
        s.direct = None;
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = v.trim();
            match k.trim() {
                "nickname" if !v.is_empty() => s.nick = Some(v.to_string()),
                "direct_server" if !v.is_empty() => s.direct = Some(v.to_string()),
                _ => {}
            }
        }
        // Mae saves `listserver=host` with no port for "follow the version".
        if s.listserver.as_deref().is_some_and(|l| !l.contains(':')) {
            s.listserver = None;
        }
        s.all_versions = s
            .version
            .as_deref()
            .and_then(Choice::from_label)
            .is_some_and(|c| match c {
                Choice::Version(v) => !v.listed_by_default(),
                _ => c.offline_only(),
            });
        s
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(p, self.text());
    }
}

/// The window.
pub struct StartScreen {
    pub choice: Choice,
    pub account: String,
    pub password: String,
    pub nick: String,
    /// The asset folder box: the folder for every version (`assets`).
    pub assets: String,
    /// The folder for the picked version only, which wins over `assets`.
    pub version_assets: String,
    assets_for: BTreeMap<String, String>,
    /// The listserver box. Follows the version's default until typed in.
    pub listserver: String,
    pub listserver_typed: bool,
    /// The game server for a version with no listserver, and for Graal3D.
    pub direct: String,
    pub all_versions: bool,
    pub error: Option<String>,
    /// The server list and its status line (`seihai_play::servers`, the
    /// same list Ato's own screen shows).
    pub list: ServerList,
    pub mode: Mode,
    /// Consumer mode: the version to join each server as.
    pub picked: BTreeMap<String, String>,
    /// Consumer mode: each server's last `Allowed:` list.
    pub allowed: BTreeMap<String, String>,
    /// Consumer mode: the join in progress.
    pub pending: Option<Pending>,
    /// Consumer mode: the question for the player, one version per era.
    pub ask: Option<Vec<ClientVersion>>,
    /// Consumer mode: remember the answer to [`Self::ask`] for the server.
    pub remember: bool,
    /// Consumer mode: the join to make at once (after a refusal).
    pub auto_join: Option<Handoff>,
}

/// The version consumer mode fetches the server list as. The public
/// listserver answers by login shape, not by version, and 2.05's classic
/// login on 14923 is the one it answered (seihai-core `list_failure_text`).
pub const CONSUMER_LIST_VERSION: ClientVersion = ClientVersion::V205;

/// The default listserver box for a pick.
fn default_listserver(c: Choice) -> String {
    match c {
        Choice::Version(v) => format!("{}:{}", v.listserver_host(), v.listserver_port()),
        Choice::Offline(_) | Choice::Graal3D => String::new(),
    }
}

/// The default game server box for a pick: Graal3D's own port.
fn default_direct(c: Choice) -> String {
    match c {
        Choice::Graal3D => "127.0.0.1:14901".into(),
        Choice::Version(_) | Choice::Offline(_) => "localhost:14900".into(),
    }
}

impl StartScreen {
    pub fn new(saved: Saved, error: Option<String>) -> StartScreen {
        let choice = saved
            .version
            .as_deref()
            .and_then(Choice::from_label)
            .unwrap_or(Choice::Version(ClientVersion::V205));
        let listserver_typed = saved.listserver.is_some();
        // A file saved before consumer mode belongs to a developer who
        // picked versions; a new one starts in consumer mode.
        let mode = saved.mode.unwrap_or(if saved.version.is_some() {
            Mode::Developer
        } else {
            Mode::Consumer
        });
        let list_choice = match mode {
            Mode::Consumer => Choice::Version(CONSUMER_LIST_VERSION),
            Mode::Developer => choice,
        };
        let mut screen = StartScreen {
            choice,
            account: saved.account.unwrap_or_default(),
            password: String::new(),
            nick: saved.nick.unwrap_or_default(),
            assets: saved.assets.unwrap_or_default(),
            version_assets: saved
                .assets_for
                .get(choice.label())
                .cloned()
                .unwrap_or_default(),
            assets_for: saved.assets_for,
            listserver: saved
                .listserver
                .unwrap_or_else(|| default_listserver(list_choice)),
            listserver_typed,
            direct: saved.direct.unwrap_or_else(|| default_direct(choice)),
            all_versions: saved.all_versions
                || matches!(choice, Choice::Version(v) if !v.listed_by_default())
                || choice.offline_only(),
            error,
            list: ServerList::empty("Ready."),
            mode,
            picked: saved.picked,
            allowed: saved.allowed,
            pending: saved.pending,
            ask: None,
            remember: true,
            auto_join: None,
        };
        screen.came_back();
        screen
    }

    /// A join came back with [`Self::error`]: in consumer mode, a version
    /// refusal joins again or asks; anything else ends the join.
    fn came_back(&mut self) {
        let Some(mut p) = self.pending.take() else {
            return;
        };
        if self.mode != Mode::Consumer {
            return;
        }
        let Some(msg) = self.error.clone() else {
            return;
        };
        if let Some(list) = consumer::allowed_text(&msg) {
            self.allowed.insert(p.server.clone(), list);
        }
        match consumer::after_refusal(&mut p, &msg) {
            Next::Done => {}
            Next::Join(v) => {
                let (server, three_d) = (p.server.clone(), p.three_d);
                self.pending = Some(p);
                self.auto_join = Some(self.join_as(v, server, three_d));
            }
            Next::Ask(options) => {
                self.error = Some(format!("{} takes more than one kind of client.", p.server));
                self.pending = Some(p);
                self.ask = Some(options);
            }
            Next::Stop(why) => self.error = Some(why),
        }
    }

    /// The version consumer mode would join `server` as now.
    pub fn planned(&self, server: &str, three_d: bool) -> ClientVersion {
        consumer::first_version(self.picked.get(server).map(String::as_str), three_d)
    }

    /// Consumer mode: join `server` as `v`, and note the join so a refusal
    /// can pick again.
    pub fn join_as(&mut self, v: ClientVersion, server: String, three_d: bool) -> Handoff {
        let refused = self
            .pending
            .take()
            .filter(|p| p.server == server)
            .map(|p| p.refused)
            .unwrap_or_default();
        if self.remember {
            self.picked.insert(server.clone(), v.label().to_string());
        }
        self.pending = Some(Pending {
            server: server.clone(),
            three_d,
            tried: v.label().to_string(),
            refused,
        });
        self.ask = None;
        let some = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        Handoff {
            choice: Choice::Version(v),
            server: Some(server),
            account: self.account.trim().to_string(),
            password: self.password.clone(),
            nick: self.nick.trim().to_string(),
            assets: self
                .assets_for
                .get(v.label())
                .and_then(|a| some(a))
                .or_else(|| some(&self.assets)),
            graal3d: three_d,
            start: Start::Online,
        }
    }

    /// The version the server list is fetched as.
    pub fn list_choice(&self) -> Choice {
        match self.mode {
            Mode::Consumer => Choice::Version(CONSUMER_LIST_VERSION),
            Mode::Developer => self.choice,
        }
    }

    /// Switch between consumer and developer mode.
    pub fn set_mode(&mut self, mode: Mode) {
        if mode == self.mode {
            return;
        }
        self.mode = mode;
        self.ask = None;
        if !self.listserver_typed {
            self.listserver = default_listserver(self.list_choice());
        }
    }

    /// The form as saved.
    pub fn saved(&self) -> Saved {
        let some = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        Saved {
            version: Some(self.choice.label().to_string()),
            account: some(&self.account),
            nick: some(&self.nick),
            listserver: self
                .listserver_typed
                .then(|| some(&self.listserver))
                .flatten(),
            direct: some(&self.direct),
            assets: some(&self.assets),
            assets_for: self.assets_for_saved(),
            all_versions: self.all_versions,
            mode: Some(self.mode),
            picked: self.picked.clone(),
            allowed: self.allowed.clone(),
            pending: self.pending.clone(),
        }
    }

    /// The per-version folders, with the box's for the picked version.
    fn assets_for_saved(&self) -> BTreeMap<String, String> {
        let mut m = self.assets_for.clone();
        match self.version_assets.trim() {
            "" => m.remove(self.choice.label()),
            d => m.insert(self.choice.label().to_string(), d.to_string()),
        };
        m
    }

    /// The version box changed.
    pub fn pick(&mut self, c: Choice) {
        if c == self.choice {
            return;
        }
        let old_direct = default_direct(self.choice);
        self.assets_for = self.assets_for_saved();
        self.version_assets = self.assets_for.get(c.label()).cloned().unwrap_or_default();
        self.choice = c;
        if !self.listserver_typed {
            self.listserver = default_listserver(c);
        }
        if self.direct == old_direct {
            self.direct = default_direct(c);
        }
        self.list = ServerList::empty(format!("{}: {}", c.label(), c.line().heading()));
    }

    /// The hand-over for `server` (or the client's own window with `None`).
    pub fn handoff(&self, server: Option<String>, graal3d: bool) -> Handoff {
        let some = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        Handoff {
            choice: self.choice,
            server,
            account: self.account.trim().to_string(),
            password: self.password.clone(),
            nick: self.nick.trim().to_string(),
            assets: some(&self.version_assets).or_else(|| some(&self.assets)),
            graal3d,
            start: Start::Online,
        }
    }

    /// Mae offline (`Start::Offline` or `Start::Tutorial`).
    pub fn offline(&self, start: Start) -> Handoff {
        let mut h = Handoff {
            start,
            ..self.handoff(None, false)
        };
        // Consumer mode has no version box: Mae plays offline as the
        // remembered version if it is Mae's, else as 2.05.
        if self.mode == Mode::Consumer && h.choice.line() != Line::Mae {
            h.choice = Choice::Version(CONSUMER_LIST_VERSION);
            h.assets = self
                .assets_for
                .get(h.choice.label())
                .cloned()
                .filter(|a| !a.trim().is_empty())
                .or_else(|| Some(self.assets.trim().to_string()).filter(|a| !a.is_empty()));
        }
        h
    }

    /// Join the selected server.
    pub fn join_selected(&mut self) -> Option<Handoff> {
        let s = self.list.picked()?;
        let (server, three_d) = (format!("{}:{}", s.host, s.port), is_3d_server(s));
        Some(match self.mode {
            Mode::Developer => self.handoff(Some(server), three_d),
            Mode::Consumer => {
                let v = self.planned(&server, three_d);
                self.join_as(v, server, three_d)
            }
        })
    }

    /// Connect to the game server typed in the form.
    pub fn join_direct(&mut self) -> Handoff {
        let server = self.direct.trim().to_string();
        match self.mode {
            Mode::Developer => self.handoff(Some(server), false),
            Mode::Consumer => {
                let v = self.planned(&server, false);
                self.join_as(v, server, false)
            }
        }
    }

    /// Ask the listserver for its servers, on a thread.
    pub fn fetch(&mut self, repaint: impl FnOnce() + Send + 'static) {
        let Choice::Version(v) = self.list_choice() else {
            return;
        };
        let addr = v.complete_list_addr(self.listserver.trim());
        let (account, password) = (self.account.clone(), self.password.clone());
        self.list = ServerList::fetch(addr, v, account, password, repaint);
    }

    pub fn waiting(&self) -> bool {
        self.list.waiting()
    }

    /// Take the reply if it has come in.
    pub fn poll(&mut self) {
        self.list.poll();
    }

    /// Draw the window and take its input. Returns the hand-over once the
    /// player joins a server or opens a client's own window.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<Handoff> {
        if let Some(h) = self.auto_join.take() {
            return Some(h);
        }
        self.poll();
        let mut out = None;
        egui::SidePanel::right("servers")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Servers");
                ui.label(&self.list.status);
                ui.separator();
                if self
                    .list
                    .rows_ui(ui, ui.available_height() - 40.0)
                    .is_some()
                {
                    out = self.join_selected();
                }
                ui.separator();
                ui.horizontal(|ui| {
                    let join =
                        ui.add_enabled(self.list.selected.is_some(), egui::Button::new("Join"));
                    if join.clicked() {
                        out = self.join_selected();
                    }
                    if self.mode == Mode::Consumer {
                        if let Some(s) = self.list.picked() {
                            let server = format!("{}:{}", s.host, s.port);
                            let v = self.planned(&server, is_3d_server(s));
                            ui.small(format!("Joins as {}", v.label()));
                        }
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Seihai");
            if let Some(e) = &self.error {
                ui.colored_label(egui::Color32::RED, e);
            }
            if let Some(options) = self.ask.clone() {
                ui.group(|ui| {
                    ui.label("Which kind of client do you want to play on this server?");
                    for v in options {
                        if ui.button(consumer::option_text(v)).clicked() {
                            if let Some(p) = self.pending.clone() {
                                out = Some(self.join_as(v, p.server, p.three_d));
                            }
                        }
                    }
                    ui.checkbox(&mut self.remember, "Remember for this server");
                });
            }
            let mut developer = self.mode == Mode::Developer;
            if ui
                .checkbox(&mut developer, "Developer mode")
                .on_hover_text("Pick the client version yourself. Otherwise Seihai picks the version each server takes.")
                .changed()
            {
                self.set_mode(if developer { Mode::Developer } else { Mode::Consumer });
            }
            ui.separator();
            if self.mode == Mode::Developer {
            ui.label("Version");
            let mut picked = self.choice;
            let mut line = None;
            egui::ComboBox::from_id_salt("version")
                .selected_text(self.choice.label())
                .width(260.0)
                .show_ui(ui, |ui| {
                    for c in choices(self.all_versions) {
                        if line != Some(c.line()) {
                            if line.is_some() {
                                ui.separator();
                            }
                            line = Some(c.line());
                            ui.label(egui::RichText::new(c.line().heading()).strong());
                        }
                        ui.selectable_value(&mut picked, c, c.label());
                    }
                });
            if ui.checkbox(&mut self.all_versions, "All versions").changed() {
                ui.ctx().request_repaint();
            }
            self.pick(picked);
            ui.small(format!(
                "Plays in {}.",
                self.choice.line().heading()
            ));
            } else {
                ui.small("Seihai picks the version each server takes.");
            }

            ui.label("Account");
            ui.text_edit_singleline(&mut self.account);
            ui.label("Password");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
            ui.label("Nickname");
            ui.add(egui::TextEdit::singleline(&mut self.nick).hint_text(self.account.as_str()));
            ui.label("Assets folder (optional)");
            ui.text_edit_singleline(&mut self.assets);
            if self.mode == Mode::Developer {
                ui.label(format!("Assets for {} only (optional; its own install)", self.choice.label()));
                ui.text_edit_singleline(&mut self.version_assets);
            }
            ui.separator();

            let developer = self.mode == Mode::Developer;
            if let Some(b) = self.choice.offline_build().filter(|_| developer) {
                ui.label(b.reason());
                ui.small(format!("Reads {}; plays offline with the {} rules.", b.levels, b.plays_as.label()));
            } else if developer && self.choice.direct_only() {
                ui.label("Game server (host:port)");
                ui.text_edit_singleline(&mut self.direct);
                if ui.button("Connect").clicked() {
                    out = Some(self.handoff(Some(self.direct.trim().to_string()), false));
                }
            } else {
                ui.label("Listserver (host:port)");
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.listserver).changed() {
                        self.listserver_typed = !self.listserver.trim().is_empty();
                    }
                    if self.listserver_typed && ui.small_button("Default").clicked() {
                        self.listserver_typed = false;
                        self.listserver = default_listserver(self.list_choice());
                    }
                });
                let fetch = ui.add_enabled(!self.waiting(), egui::Button::new("Server list"));
                if fetch.clicked() {
                    let repaint = ctx.clone();
                    self.fetch(move || repaint.request_repaint());
                }
                egui::CollapsingHeader::new("Or connect to one game server")
                    .id_salt(("direct", self.choice.label()))
                    .default_open(developer && self.choice.direct_first())
                    .show(ui, |ui| {
                    ui.text_edit_singleline(&mut self.direct);
                    if ui.button("Connect").clicked() {
                        out = Some(self.join_direct());
                    }
                });
            }
            ui.separator();
            let line = if developer { self.choice.line() } else { Line::Mae };
            match line {
                // Mae's offline play and tutorial; its own window is the
                // development binary's.
                Line::Mae => {
                    ui.horizontal(|ui| {
                        if ui.button("Play offline").clicked() {
                            out = Some(self.offline(Start::Offline));
                        }
                        if ui
                            .button("Play the tutorial")
                            .on_hover_text("Seihai's own short tutorial levels, offline. They use their own art, whatever assets are set.")
                            .clicked()
                        {
                            out = Some(self.offline(Start::Tutorial));
                        }
                    });
                }
                // Issho has no start window of its own.
                Line::Issho => {}
                line => {
                    if ui
                        .button(format!("Open {}'s own window", line.name()))
                        .on_hover_text("Starts the client with no server: its own start window, as the original client's.")
                        .clicked()
                    {
                        out = Some(self.handoff(None, false));
                    }
                }
            }
        });
        if self.waiting() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ended_reason_is_taken_once() {
        report_ended("The server closed the connection.");
        assert_eq!(
            take_ended().as_deref(),
            Some("The server closed the connection.")
        );
        assert_eq!(take_ended(), None);
    }

    #[test]
    fn every_row_has_its_line() {
        for v in ClientVersion::all_versions() {
            let l = v.label();
            let want = match major(l) {
                Some(6) => Line::Issho,
                Some(3..=5) => Line::Ato,
                _ => Line::Mae,
            };
            assert_eq!(line_of(*v), want, "{l}");
        }
        assert_eq!(Choice::Graal3D.line(), Line::G3d);
    }

    #[test]
    fn lines_are_named_by_subcommand_or_program() {
        for l in [Line::Mae, Line::Ato, Line::Issho, Line::G3d] {
            assert_eq!(Line::from_name(l.subcommand()), Some(l));
            assert_eq!(Line::from_name(l.binary()), Some(l));
            assert_eq!(Line::from_name(&format!("{}.EXE", l.binary())), Some(l));
        }
        assert_eq!(Line::from_name("seihai"), None);
        assert_eq!(Line::from_name("capture"), None);
    }

    #[test]
    fn the_box_lists_every_line_once_each_in_order() {
        for all in [false, true] {
            let lines: Vec<Line> = choices(all).iter().map(|c| c.line()).collect();
            let mut seen = lines.clone();
            seen.dedup();
            assert_eq!(
                seen,
                [Line::Mae, Line::Ato, Line::Issho, Line::G3d],
                "all={all}"
            );
        }
    }

    /// The window in developer mode, as a `launcher.conf` from before
    /// consumer mode opens it.
    fn screen() -> StartScreen {
        StartScreen::new(
            Saved {
                mode: Some(Mode::Developer),
                ..Saved::default()
            },
            None,
        )
    }

    fn one_server(s: &mut StartScreen, kind: &str) {
        s.list.take(Ok(proto::ListReply {
            servers: vec![ServerEntry {
                kind: kind.into(),
                name: "Lab".into(),
                host: "h".into(),
                port: 14900,
                ..Default::default()
            }],
            ..Default::default()
        }));
        s.list.selected = Some(0);
    }

    const REFUSAL: &str =
        "disconnected: Your client version is not allowed on this server.\nAllowed: ";

    #[test]
    fn a_new_file_is_consumer_mode_and_an_old_one_developer_mode() {
        assert_eq!(
            StartScreen::new(Saved::default(), None).mode,
            Mode::Consumer
        );
        let old = Saved::parse("version=2.22\naccount=a\n");
        assert_eq!(StartScreen::new(old, None).mode, Mode::Developer);
        let s = StartScreen::new(Saved::parse("version=2.22\nmode=consumer\n"), None);
        assert_eq!(s.mode, Mode::Consumer);
        // The list is fetched as 2.05 whatever the saved version.
        assert_eq!(s.list_choice(), Choice::Version(CONSUMER_LIST_VERSION));
        assert_eq!(s.listserver, default_listserver(s.list_choice()));
    }

    /// A refusal brings the client back; the window joins again as the
    /// version the server takes, and remembers it.
    #[test]
    fn consumer_mode_joins_again_after_a_refusal() {
        let mut s = StartScreen::new(Saved::default(), None);
        s.account = "a".into();
        one_server(&mut s, "");
        let h = s.join_selected().unwrap();
        assert_eq!(h.choice, Choice::Version(ClientVersion::V603R7));
        assert_eq!(h.line(), Line::Issho);
        // As the program saves it before handing over, then starts again
        // with the refusal.
        let text = s.saved().text();
        assert!(text.contains("pending=h:14900"), "{text}");
        assert!(text.contains("picked.h:14900=6.03r7"), "{text}");
        let back = StartScreen::new(Saved::parse(&text), Some(format!("{REFUSAL}2.22, 5.07")));
        let h = back.auto_join.clone().expect("joins again at once");
        assert_eq!(h.choice, Choice::Version(ClientVersion::V222));
        assert_eq!(h.server.as_deref(), Some("h:14900"));
        assert_eq!(h.args()[..4], ["--version", "2.22", "--online", "h:14900"]);
        assert_eq!(back.picked["h:14900"], "2.22");
        assert_eq!(back.allowed["h:14900"], "2.22, 5.07");
        assert_eq!(back.pending.as_ref().unwrap().refused, ["6.03r7"]);
        // The next join of that server starts at 2.22.
        let text = back.saved().text();
        let mut again = StartScreen::new(
            Saved::parse(&text),
            Some("The server closed the connection.".into()),
        );
        assert!(again.auto_join.is_none());
        assert!(
            again.pending.is_none(),
            "a join that did not come back refused is over"
        );
        one_server(&mut again, "");
        assert_eq!(
            again.join_selected().unwrap().choice,
            Choice::Version(ClientVersion::V222)
        );
    }

    #[test]
    fn two_eras_with_a_gap_ask_and_the_answer_joins() {
        let mut s = StartScreen::new(Saved::default(), None);
        one_server(&mut s, "");
        s.join_selected().unwrap();
        let text = s.saved().text();
        let mut back = StartScreen::new(
            Saved::parse(&text),
            Some(format!("{REFUSAL}1.41r1, 5.12 - 6.037")),
        );
        // 6.03r7 was refused already, so 5.31r4 stands for Modern.
        assert_eq!(
            back.ask,
            Some(vec![ClientVersion::V531R4, ClientVersion::V141])
        );
        assert!(back.auto_join.is_none());
        let p = back.pending.clone().unwrap();
        let h = back.join_as(ClientVersion::V141, p.server, p.three_d);
        assert_eq!(h.line(), Line::Mae);
        assert_eq!(back.ask, None);
        assert_eq!(back.picked["h:14900"], "1.41");
    }

    #[test]
    fn developer_mode_only_shows_the_refusal() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V205));
        one_server(&mut s, "");
        let h = s.join_selected().unwrap();
        assert_eq!(h.choice, Choice::Version(ClientVersion::V205));
        let mut saved = s.saved();
        saved.pending = Some(Pending {
            server: "h:14900".into(),
            tried: "2.05".into(),
            ..Pending::default()
        });
        let back = StartScreen::new(saved, Some(format!("{REFUSAL}2.22")));
        assert!(back.auto_join.is_none() && back.ask.is_none());
        assert!(back.error.unwrap().contains("Allowed: 2.22"));
    }

    #[test]
    fn a_3d_server_in_consumer_mode_joins_as_a_3d_build() {
        let mut s = StartScreen::new(Saved::default(), None);
        one_server(&mut s, "3");
        let h = s.join_selected().unwrap();
        assert_eq!(h.line(), Line::Ato);
        assert!(h.args().ends_with(&["--graal3d".to_string()]));
    }

    #[test]
    fn consumer_offline_play_is_mae() {
        let mut s = StartScreen::new(Saved::default(), None);
        s.choice = Choice::Version(ClientVersion::V603R7);
        let h = s.offline(Start::Offline);
        assert_eq!(h.line(), Line::Mae);
        assert_eq!(h.args()[..3], ["--version", "2.05", "--offline"]);
    }

    #[test]
    fn join_hands_over_with_the_password_in_the_environment() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V222));
        s.account = "acc".into();
        s.password = "pw".into();
        let h = s.handoff(Some("h:14900".into()), false);
        assert_eq!(h.line(), Line::Mae);
        assert_eq!(
            h.args(),
            [
                "--version",
                "2.22",
                "--online",
                "h:14900",
                "--account",
                "acc",
                "--password-env",
                PASSWORD_ENV
            ]
        );
        assert_eq!(h.env(), Some((PASSWORD_ENV, "pw")));
        assert!(!h.args().iter().any(|a| a == "pw"));
    }

    #[test]
    fn a_3d_server_adds_graal3d_for_ato_only() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V512));
        s.list.take(Ok(proto::ListReply {
            servers: vec![ServerEntry {
                kind: "3".into(),
                name: "Lab 3D".into(),
                host: "127.0.0.1".into(),
                port: 14910,
                ..Default::default()
            }],
            ..Default::default()
        }));
        s.list.selected = Some(0);
        let h = s.join_selected().unwrap();
        assert_eq!(h.line(), Line::Ato);
        assert!(h.args().ends_with(&["--graal3d".to_string()]));
        s.pick(Choice::Version(ClientVersion::V222));
        let h = s.handoff(Some("h:1".into()), true);
        assert!(!h.args().contains(&"--graal3d".to_string()));
    }

    #[test]
    fn six_x_goes_to_issho_and_graal3d_to_g3d() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V603R7));
        assert_eq!(s.handoff(None, false).line(), Line::Issho);
        assert_eq!(s.handoff(None, false).args(), ["--version", "6.03r7"]);
        s.pick(Choice::Graal3D);
        assert!(s.choice.direct_only());
        assert_eq!(s.direct, "127.0.0.1:14901");
        let h = s.handoff(Some(s.direct.clone()), false);
        assert_eq!(h.line(), Line::G3d);
        assert_eq!(
            h.args()[..4],
            ["--version", "graal3d", "--online", "127.0.0.1:14901"]
        );
    }

    #[test]
    fn the_listserver_box_follows_the_version_until_typed() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V139));
        let classic = s.listserver.clone();
        s.pick(Choice::Version(ClientVersion::V222));
        assert_eq!(
            s.listserver,
            default_listserver(Choice::Version(ClientVersion::V222))
        );
        assert!(classic.starts_with("listserver.graal.in:"));
        s.listserver = "lab:14922".into();
        s.listserver_typed = true;
        s.pick(Choice::Version(ClientVersion::V304));
        assert_eq!(s.listserver, "lab:14922");
    }

    #[test]
    fn offline_builds_are_mae_rows_under_all_versions_only() {
        let short = choices(false);
        assert!(!short.iter().any(|c| c.offline_only()));
        let all = choices(true);
        let first = all.iter().position(|c| c.offline_only()).unwrap();
        assert!(all[..first].iter().all(|c| c.line() == Line::Mae));
        assert_eq!(all[first - 1].line(), Line::Mae);
        for b in offline_builds::OFFLINE_BUILDS {
            let c = Choice::from_label(b.label).unwrap();
            assert_eq!(c, Choice::Offline(b.label));
            assert!(all.contains(&c), "{}", b.label);
            assert_eq!(c.line(), Line::Mae);
        }
    }

    #[test]
    fn play_offline_and_the_tutorial_start_mae_with_no_server() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V222));
        s.account = "a".into();
        let h = s.offline(Start::Offline);
        assert_eq!(h.line(), Line::Mae);
        assert_eq!(
            h.args(),
            ["--version", "2.22", "--offline", "--account", "a"]
        );
        let h = s.offline(Start::Tutorial);
        assert_eq!(h.args()[2], "--tutorial");
        assert!(!h.args().contains(&"--online".to_string()));
        let b = &offline_builds::OFFLINE_BUILDS[0];
        s.pick(Choice::Offline(b.label));
        assert!(s.choice.offline_only());
        assert_eq!(
            s.offline(Start::Offline).args()[..3],
            ["--version", b.label, "--offline"]
        );
    }

    #[test]
    fn the_version_folder_follows_the_pick_and_wins_over_the_shared_one() {
        let mut s = screen();
        s.assets = "/all".into();
        s.pick(Choice::Version(ClientVersion::V222));
        s.version_assets = "/g222".into();
        assert_eq!(s.handoff(None, false).assets.as_deref(), Some("/g222"));
        s.pick(Choice::Version(ClientVersion::V139));
        assert_eq!(s.version_assets, "");
        assert_eq!(s.handoff(None, false).assets.as_deref(), Some("/all"));
        s.pick(Choice::Version(ClientVersion::V222));
        assert_eq!(s.version_assets, "/g222");
        let text = s.saved().text();
        assert!(text.contains("assets.2.22=/g222"), "{text}");
        assert_eq!(Saved::parse(&text).assets_for["2.22"], "/g222");
    }

    #[test]
    fn the_first_start_takes_mae_s_saved_form_but_not_its_password() {
        let mae = "account=acc\npassword=pw\nversion=1.39\nlistserver=listserver.graal.in\nnickname=Nick\ndirect_server=h:1\nassets.2.22=/g\n";
        let s = Saved::from_mae(mae);
        assert_eq!(s.account.as_deref(), Some("acc"));
        assert_eq!(s.version.as_deref(), Some("1.39"));
        assert_eq!(s.listserver, None, "a host alone follows the version");
        assert_eq!(s.nick.as_deref(), Some("Nick"));
        assert_eq!(s.direct.as_deref(), Some("h:1"));
        assert_eq!(s.assets_for["2.22"], "/g");
        assert!(!s.text().contains("pw"));
        let w = StartScreen::new(s, None);
        assert_eq!(w.password, "");
        assert_eq!(w.choice, Choice::Version(ClientVersion::V139));
    }

    #[test]
    fn saved_form_round_trips_without_the_password() {
        let mut s = screen();
        s.pick(Choice::Version(ClientVersion::V420R8));
        s.account = "a".into();
        s.password = "secret".into();
        s.nick = "n".into();
        let text = s.saved().text();
        assert!(!text.contains("secret"));
        let back = StartScreen::new(Saved::parse(&text), None);
        assert_eq!(back.choice, Choice::Version(ClientVersion::V420R8));
        assert_eq!((back.account.as_str(), back.nick.as_str()), ("a", "n"));
        assert_eq!(back.password, "");
        assert_eq!(Choice::from_label("G3D"), Some(Choice::Graal3D));
    }
}
