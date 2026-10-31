//! Settings and connect window.

use crate::net::Connection;
use crate::protocol::offline_builds::{self, OfflineBuild};
use crate::protocol::{ClientVersion, ServerList};
use crate::settings::Settings;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// The launcher's listserver boxes: a host and a port that follows the
/// selected version's default until the user types a port.
///
/// A port in the settings file (`listserver=host:port`) counts as typed: it
/// is kept and marked user-set. With no port there, the box shows the
/// version's default (14923 for a profile that sends the classic login,
/// 1.41 to 2.17; 14922 for one that sends the gen-5 login, 2.22 and later;
/// never the dead 14921) and changes when the version selector does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEndpoint {
    pub host: String,
    pub port: String,
    /// True once the user typed a port, or the settings file held one.
    pub port_user_set: bool,
}

impl ListEndpoint {
    /// Build the boxes from the settings file's `listserver` value (`host`,
    /// `host:port` or nothing) and the selected version.
    pub fn from_saved(saved: Option<&str>, version: ClientVersion) -> Self {
        let mut it = saved.unwrap_or("").splitn(2, ':');
        let host = it.next().unwrap_or("").trim();
        let port = it.next().unwrap_or("").trim();
        Self {
            host: if host.is_empty() { version.listserver_host().to_string() } else { host.to_string() },
            port: if port.is_empty() { version.listserver_port().to_string() } else { port.to_string() },
            port_user_set: !port.is_empty(),
        }
    }

    /// The version selector changed: an untouched port box takes the new
    /// version's default, a user-set one stays.
    pub fn follow_version(&mut self, version: ClientVersion) {
        if !self.port_user_set {
            self.port = version.listserver_port().to_string();
        }
    }

    /// The user edited the port box. An empty box goes back to following the
    /// version (it is filled in when the version changes or on connect).
    pub fn port_edited(&mut self) {
        self.port_user_set = !self.port.trim().is_empty();
    }

    /// "Use default": drop the user's port and follow the version again.
    pub fn use_default(&mut self, version: ClientVersion) {
        self.port_user_set = false;
        self.port = version.listserver_port().to_string();
    }

    /// `host:port` to dial. An empty port box means the version's default.
    pub fn addr(&self, version: ClientVersion) -> String {
        let port = self.port.trim();
        if port.is_empty() {
            format!("{}:{}", self.host, version.listserver_port())
        } else {
            format!("{}:{port}", self.host)
        }
    }

    /// The `listserver` value to save: with a port only when the user set
    /// one, so a saved default does not stop the box following the version.
    pub fn to_saved(&self) -> String {
        if self.port_user_set {
            format!("{}:{}", self.host, self.port.trim())
        } else {
            self.host.clone()
        }
    }
}

pub struct Launcher {
    list: ListEndpoint,
    account: String,
    password: String,
    /// The nickname the 4.x builds send (empty: the account).
    nickname: String,
    version: String,
    assets: String,
    status: String,
    servers: Vec<(String, String, u16)>,
    selected: Option<usize>,
    show_advanced: bool,
    error: Option<String>,
    /// Game server for versions with no listserver.
    direct: String,
    /// Asset folder for the selected version only.
    version_assets: String,
    /// Set when a 3.x build was picked: the window closes and Ato starts.
    ato: Arc<Mutex<Option<crate::ato::Handoff>>>,
}

impl Launcher {
    pub fn new() -> Self {
        Self::with_error(None)
    }

    pub fn with_error(error: Option<String>) -> Self {
        let s = Settings::load();
        Self {
            list: ListEndpoint::from_saved(
                s.listserver.as_deref(),
                s.version.as_deref().and_then(ClientVersion::from_label_or_offline).unwrap_or(ClientVersion::V205),
            ),
            account: s.account.clone().unwrap_or_default(),
            password: s.password.clone().unwrap_or_default(),
            nickname: s.nickname.clone().unwrap_or_default(),
            version: s.version.clone().unwrap_or_else(|| "2.05".into()),
            assets: s
                .assets
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            status: "Ready".into(),
            servers: Vec::new(),
            selected: None,
            error,
            show_advanced: s.version.as_deref().is_some_and(needs_all_versions),
            direct: s
                .direct_server
                .clone()
                .unwrap_or_else(|| "localhost:14900".into()),
            version_assets: String::new(),
            ato: Arc::default(),
        }
        .with_version_assets()
    }

    /// Load the per-version asset folder for the selected version.
    fn with_version_assets(mut self) -> Self {
        let label = self.assets_label().to_string();
        self.version_assets = Settings::load()
            .assets_for(&label)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self
    }

    /// Save the form and the chosen server, then close.
    fn join(&self, ctx: &egui::Context, name: &str, addr: String) {
        // An offline-only build never joins a server (its row is only `plays_as`).
        if self.offline_build().is_some() {
            return;
        }
        let mut s = Settings::load();
        s.account = Some(self.account.clone());
        s.password = Some(self.password.clone());
        s.nickname = Some(self.nickname.clone()).filter(|n| !n.trim().is_empty());
        s.version = Some(self.version.clone());
        s.assets = Some(self.assets.clone().into());
        s.set_assets_for(self.version_enum().label(), &self.version_assets);
        s.last_server = Some(addr);
        s.last_server_name = Some(name.to_string());
        s.offline = None;
        s.save();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// A 3.x build was picked: save the form and close, so Ato starts with
    /// it. The saved version stays the last classic one, so the next
    /// settings window does not open on a build it cannot run.
    fn hand_to_ato(&self, ctx: &egui::Context, version: ClientVersion) {
        let mut s = Settings::load();
        s.account = Some(self.account.clone());
        s.password = Some(self.password.clone());
        s.assets = Some(self.assets.clone().into());
        s.set_assets_for(version.label(), &self.version_assets);
        s.last_server = None;
        s.offline = None;
        s.save();
        *self.ato.lock().unwrap() = Some(crate::ato::Handoff {
            version,
            account: self.account.clone(),
            password: self.password.clone(),
            assets: Some(self.version_assets.clone()),
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn version_enum(&self) -> ClientVersion {
        ClientVersion::from_label_or_offline(&self.version).unwrap_or(ClientVersion::V205)
    }

    /// The selected offline-only build, if one is picked.
    fn offline_build(&self) -> Option<&'static OfflineBuild> {
        offline_builds::by_label(&self.version)
    }

    /// The label the per-version asset folder is saved under: the offline
    /// build's own, so it never shares its `plays_as` row's folder.
    fn assets_label(&self) -> &str {
        self.offline_build().map_or(self.version_enum().label(), |b| b.label)
    }

    fn connect_list(&mut self) {
        if let Some(b) = self.offline_build() {
            self.status = b.reason();
            return;
        }
        let version = self.version_enum();
        let list_version = version.list_version();
        if list_version != version {
            self.status = format!("{} has no listserver; fetching as {}", version.label(), list_version.label());
        }
        let addr = self.list.addr(version);
        match Connection::fetch_server_list(&addr, list_version, &self.account, &self.password) {
            Ok(reply) => {
                self.servers = reply
                    .servers
                    .into_iter()
                    .map(|s| (s.name, s.host, s.port))
                    .collect();
                self.status = match reply.error {
                    Some(err) => format!("listserver: {err}"),
                    None if self.servers.is_empty() => "no servers in reply".into(),
                    None => format!("{} servers", self.servers.len()),
                };
                if !self.servers.is_empty() {
                    let mut settings = Settings::load();
                    settings.listserver = Some(self.list.to_saved());
                    settings.account = Some(self.account.clone());
                    settings.password = Some(self.password.clone());
                    settings.version = Some(self.version.clone());
                    settings.assets = Some(self.assets.clone().into());
                    settings.save();
                }
            }
            Err(e) => self.status = crate::net::list_failure_text(&addr, &e),
        }
    }
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Seihai");
            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.separator();
            ui.label("Assets");
            ui.text_edit_singleline(&mut self.assets);
            ui.label("Listserver");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.list.host);
                ui.label(":");
                if ui.text_edit_singleline(&mut self.list.port).changed() {
                    self.list.port_edited();
                }
            });
            {
                let v = self.version_enum();
                let ls = v.listserver();
                let kind = match ls.kind {
                    crate::protocol::ListKind::Legacy => "classic login",
                    crate::protocol::ListKind::Supported => "gen-5 login",
                };
                let status = match ls.status {
                    crate::protocol::PortStatus::Confirmed => "confirmed",
                    crate::protocol::PortStatus::Assumed => "assumed",
                };
                let hint = match ls.status {
                    crate::protocol::PortStatus::Confirmed => {
                        "Seihai's login for this version got a server list on this port from the public listserver. A real client of this version was not captured."
                    }
                    crate::protocol::PortStatus::Assumed => {
                        "The port of this login type. Seihai's login for this version was not run on the public listserver."
                    }
                };
                let hint = format!("{hint} Login shape {}: {}.", ls.shape.letter(), ls.shape_basis.text());
                if self.list.port_user_set {
                    ui.horizontal(|ui| {
                        ui.small(format!("Port set by you. Default for {}: {} ({kind}, {status})", v.label(), ls.port))
                            .on_hover_text(hint);
                        if ui.small_button("Use default").clicked() {
                            self.list.use_default(v);
                        }
                    });
                } else {
                    ui.small(format!("Port follows {}: {} ({kind}, {status})", v.label(), ls.port)).on_hover_text(hint);
                }
            }
            ui.label("Account");
            ui.text_edit_singleline(&mut self.account);
            ui.label("Password");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
            if self.version_enum().profile().protocol.lobby.is_some() {
                ui.label("Nickname");
                ui.add(egui::TextEdit::singleline(&mut self.nickname).hint_text(self.account.as_str()));
            }
            ui.label("Version");
            let before = self.version.clone();
            let shown: &[ClientVersion] = if self.show_advanced {
                ClientVersion::all_versions()
            } else {
                ClientVersion::reborn_versions()
            };
            egui::ComboBox::from_id_salt("version")
                .selected_text(&self.version)
                .show_ui(ui, |ui| {
                    for v in shown {
                        let p = v.profile();
                        let text = match p.basis {
                            crate::protocol::Basis::Binary => format!("{} ({})", p.label, p.token),
                            crate::protocol::Basis::Inferred => format!("{} ({}, inferred)", p.label, p.token),
                        };
                        ui.selectable_value(&mut self.version, p.label.to_string(), text);
                    }
                    if self.show_advanced {
                        ui.separator();
                        for b in offline_builds::OFFLINE_BUILDS {
                            ui.selectable_value(&mut self.version, b.label.to_string(), b.display());
                        }
                    }
                });
            ui.checkbox(&mut self.show_advanced, "All versions");
            if self.version != before {
                let v = self.version_enum();
                self.list.follow_version(v);
                let label = self.assets_label().to_string();
                self.version_assets = Settings::load()
                    .assets_for(&label)
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                if self.offline_build().is_none() && crate::ato::runs_in_ato(v) {
                    self.hand_to_ato(ctx, v);
                }
            }
            let profile = self.version_enum().profile();
            let offline_build = self.offline_build();
            if let Some(b) = offline_build {
                ui.small(format!("{} Reads {}; plays offline with the {} rules.", b.reason(), b.levels, profile.label));
                ui.label(format!("Assets for {} only (point this at its install)", b.label));
            } else {
                ui.small(profile.summary());
                ui.label(format!("Assets for {} only (optional)", profile.label));
            }
            ui.text_edit_singleline(&mut self.version_assets);
            let direct = if offline_build.is_some() { None } else { Some(profile.ui.server_list) };
            if let Some(ServerList::Direct { default }) = direct {
                ui.separator();
                ui.label(format!("{} connects straight to a game server (default was {default})", profile.label));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.direct);
                    if ui.button("Connect").clicked() {
                        let mut s = Settings::load();
                        s.direct_server = Some(self.direct.clone());
                        s.save();
                        let addr = self.direct.clone();
                        self.join(ctx, "", addr);
                    }
                });
            }
            ui.separator();
            if ui.button("Play offline").clicked() {
                let mut s = Settings::load();
                s.last_server = None;
                s.offline = Some(true);
                s.version = Some(self.version.clone());
                s.assets = Some(self.assets.clone().into());
                s.set_assets_for(self.assets_label(), &self.version_assets);
                s.save();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if ui
                .button("Play the tutorial")
                .on_hover_text("Seihai's own short tutorial levels, offline. They use their own art, whatever assets are set.")
                .clicked()
            {
                let mut s = Settings::load();
                s.last_server = None;
                s.offline = Some(true);
                s.version = Some(self.version.clone());
                s.assets = Some(self.assets.clone().into());
                s.set_assets_for(self.assets_label(), &self.version_assets);
                s.save();
                TUTORIAL_PICKED.store(true, std::sync::atomic::Ordering::Relaxed);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if offline_build.is_none() && ui.button("Connect to listserver").clicked() {
                self.connect_list();
            }
            ui.label(&self.status);
        });

        egui::SidePanel::right("servers")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Servers");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, (name, host, port)) in self.servers.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected == Some(i), format!("{name} ({host}:{port})"))
                            .clicked()
                        {
                            self.selected = Some(i);
                        }
                    }
                });
                ui.separator();
                if let Some(i) = self.selected {
                    if ui.button("Join").clicked() {
                        if let Some((name, host, port)) = self.servers.get(i).cloned() {
                            self.join(ctx, &name, format!("{host}:{port}"));
                        }
                    }
                } else {
                    ui.label("Select a server");
                }
            });
    }
}

/// Set when "Play the tutorial" closed the start window.
static TUTORIAL_PICKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the start window was closed with "Play the tutorial" (and clear it).
pub fn take_tutorial_pick() -> bool {
    TUTORIAL_PICKED.swap(false, std::sync::atomic::Ordering::Relaxed)
}

pub fn run() -> Option<String> {
    run_inner(None)
}

pub fn run_with_error(error: String) -> Option<String> {
    run_inner(Some(error))
}

fn run_inner(mut error: Option<String>) -> Option<String> {
    loop {
        // Clear any previous choice so closing the window does not reconnect.
        let mut s = Settings::load();
        s.last_server = None;
        s.save();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 520.0]),
            ..Default::default()
        };
        let ato: Arc<Mutex<Option<crate::ato::Handoff>>> = Arc::default();
        let ato2 = ato.clone();
        let _ = eframe::run_native(
            "Seihai",
            options,
            Box::new(move |_cc| {
                let mut l = Launcher::with_error(error);
                l.ato = ato2;
                Ok(Box::new(l))
            }),
        );
        // A 3.x build was picked: Ato takes over. If it cannot start, the
        // window comes back with the reason.
        let Some(handoff) = ato.lock().unwrap().take() else { return Settings::load().last_server };
        let why = handoff.start();
        eprintln!("seihai-mae: {why}");
        error = Some(why);
    }
}

/// The server's level, for the F5 window when online.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevel {
    /// The server's name, else its address.
    pub server: String,
    /// The level the server has us in, if it said yet.
    pub level: Option<String>,
    /// Its copy in this server's download cache, if one was saved.
    pub cached: Option<std::path::PathBuf>,
}

impl ServerLevel {
    /// The line naming the server's level.
    pub fn line(&self) -> String {
        match &self.level {
            Some(l) => format!("On {}: {l}", self.server),
            None => format!("On {}: no level yet", self.server),
        }
    }

    /// The text by the "open offline" button.
    pub fn cache_line(&self) -> String {
        match (&self.level, &self.cached) {
            (Some(_), Some(p)) => format!("Saved copy: {}", p.display()),
            (Some(l), None) => format!("{l} is not in the download cache (the server drew it without sending the file)"),
            (None, _) => String::new(),
        }
    }
}

/// What the F5 window shows.
#[derive(Debug, Clone)]
pub struct LevelWindow {
    /// Online: the server and its level.
    pub server: Option<ServerLevel>,
    /// The folder the list starts in.
    pub start: std::path::PathBuf,
    /// Folders to jump to (label, folder): the levels folder, the
    /// download cache, the built-in levels.
    pub places: Vec<(String, std::path::PathBuf)>,
    /// The level extensions this version opens (".nw", ".graal", ...).
    pub exts: Vec<String>,
}

/// What was picked in the F5 window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelChoice {
    /// Play this level file offline (online: leave the server first).
    Open(std::path::PathBuf),
    /// Leave the server, or the offline game, for the server list.
    ServerList,
}

/// The entries of `dir` for the F5 list: its folders (with a trailing `/`)
/// and then its files with one of `exts` (any case), each part sorted.
pub fn level_entries(dir: &std::path::Path, exts: &[String]) -> Vec<String> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.path().is_dir() {
                dirs.push(format!("{name}/"));
            } else {
                let lower = name.to_ascii_lowercase();
                if exts.iter().any(|x| lower.ends_with(&x.to_ascii_lowercase())) {
                    files.push(name);
                }
            }
        }
    }
    // Natural order: level2 before level10.
    let key = |s: &String| {
        let mut k = String::new();
        let mut digits = String::new();
        for c in s.to_ascii_lowercase().chars().chain(std::iter::once('\0')) {
            if c.is_ascii_digit() {
                digits.push(c);
                continue;
            }
            if !digits.is_empty() {
                k.push_str(&format!("{digits:0>12}"));
                digits.clear();
            }
            if c != '\0' {
                k.push(c);
            }
        }
        k
    };
    dirs.sort_by_key(key);
    files.sort_by_key(key);
    dirs.extend(files);
    dirs
}

/// The F5 window. Blocks until it closes; `None` when it was closed with
/// nothing picked.
pub fn level_window(w: LevelWindow) -> Option<LevelChoice> {
    struct Picker {
        w: LevelWindow,
        dir: std::path::PathBuf,
        entries: Vec<String>,
        out: Arc<Mutex<Option<LevelChoice>>>,
    }
    impl Picker {
        fn go(&mut self, dir: std::path::PathBuf) {
            self.dir = dir;
            self.entries = level_entries(&self.dir, &self.w.exts);
        }
        fn pick(&self, ctx: &egui::Context, c: LevelChoice) {
            *self.out.lock().unwrap() = Some(c);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    impl eframe::App for Picker {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            egui::TopBottomPanel::top("server").show(ctx, |ui| {
                ui.add_space(4.0);
                match self.w.server.clone() {
                    Some(s) => {
                        ui.heading(s.line());
                        if let Some(level) = &s.level {
                            ui.horizontal_wrapped(|ui| {
                                let open = ui.add_enabled(s.cached.is_some(), egui::Button::new(format!("Open {level} offline")));
                                if open.clicked() {
                                    if let Some(p) = s.cached.clone() {
                                        self.pick(ctx, LevelChoice::Open(p));
                                    }
                                }
                                ui.label(s.cache_line());
                            });
                        }
                        ui.label("Opening a level here leaves the server and plays it offline.");
                    }
                    None => {
                        ui.heading("Open a level (offline)");
                    }
                }
                ui.add_space(4.0);
            });
            egui::TopBottomPanel::bottom("buttons").show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Return to server list").clicked() {
                        self.pick(ctx, LevelChoice::ServerList);
                    }
                    if ui.button("Cancel").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(4.0);
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                if !self.w.places.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for (label, dir) in self.w.places.clone() {
                            if ui.selectable_label(self.dir == dir, label).clicked() {
                                self.go(dir);
                            }
                        }
                    });
                }
                ui.horizontal(|ui| {
                    if ui.button("..").clicked() {
                        if let Some(p) = self.dir.parent() {
                            self.go(p.to_path_buf());
                        }
                    }
                    ui.label(self.dir.display().to_string());
                });
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    if self.entries.is_empty() {
                        ui.label("No levels in this folder.");
                    }
                    for name in self.entries.clone() {
                        if ui.button(&name).clicked() {
                            let path = self.dir.join(name.trim_end_matches('/'));
                            if path.is_dir() {
                                self.go(path);
                            } else {
                                self.pick(ctx, LevelChoice::Open(path));
                            }
                        }
                    }
                });
            });
        }
    }
    let chosen: Arc<Mutex<Option<LevelChoice>>> = Arc::default();
    let mut picker = Picker { dir: w.start.clone(), w, entries: Vec::new(), out: chosen.clone() };
    picker.go(picker.dir.clone());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 520.0]),
        ..Default::default()
    };
    let _ = eframe::run_native("Seihai: levels", options, Box::new(|_| Ok(Box::new(picker))));
    let picked = chosen.lock().unwrap().take();
    picked
}

/// A saved version the default list does not show (a minor or
/// offline-only build): the launcher opens with "All versions" ticked so
/// the pick is still in the list.
fn needs_all_versions(label: &str) -> bool {
    ClientVersion::from_label(label).is_none_or(|v| !v.listed_by_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A saved Reborn version opens the short list; a minor, offline-only or
    /// unknown one opens "All versions".
    #[test]
    fn saved_minor_version_opens_all_versions() {
        for l in ["1.28 beta 2", "1.3 test", "1.31", "1.39", "2.05", "2.22", "3.04", "5.00r7", "6.03r7"] {
            assert!(!needs_all_versions(l), "{l}");
        }
        for l in ["1.21", "1.32", "2.00", "2.1", "3.153", "5.24r4", "6.03r7 Linux", "no such build"] {
            assert!(needs_all_versions(l), "{l}");
        }
        if let Some(b) = offline_builds::OFFLINE_BUILDS.first() {
            assert!(needs_all_versions(b.label), "{}", b.label);
        }
    }

    const V200: ClientVersion = ClientVersion::V200;
    const V205: ClientVersion = ClientVersion::V205;
    const V222: ClientVersion = ClientVersion::V222;

    /// With nothing saved the port box shows the version's default and
    /// changes when the version selector does (classic login 14923, gen-5
    /// login 14922).
    #[test]
    fn port_box_follows_the_selected_version() {
        let mut l = ListEndpoint::from_saved(None, V205);
        assert_eq!((l.host.as_str(), l.port.as_str(), l.port_user_set), ("listserver.graal.in", "14923", false));
        l.follow_version(V200);
        assert_eq!(l.port, "14923");
        l.follow_version(V222);
        assert_eq!(l.port, "14922");
        // 2.1 to 2.17 send the classic login: their box shows 14923.
        for v in [ClientVersion::V210, ClientVersion::V212, ClientVersion::V216, ClientVersion::V217] {
            l.follow_version(v);
            assert_eq!(l.port, "14923", "{v:?}");
            l.follow_version(V222);
            assert_eq!(l.port, "14922");
        }
        l.follow_version(ClientVersion::V201);
        assert_eq!(l.port, "14923");
        assert_eq!(l.addr(ClientVersion::V201), "listserver.graal.in:14923");
    }

    /// A port the user typed stays when the version changes, and is marked
    /// user-set; "Use default" hands it back to the version.
    #[test]
    fn typed_port_is_kept_and_marked_user_set() {
        let mut l = ListEndpoint::from_saved(None, V205);
        l.port = "14999".into();
        l.port_edited();
        assert!(l.port_user_set);
        l.follow_version(V200);
        assert_eq!(l.port, "14999");
        assert_eq!(l.addr(V200), "listserver.graal.in:14999");
        l.use_default(V200);
        assert!(!l.port_user_set);
        assert_eq!(l.port, "14923");
        l.follow_version(V222);
        assert_eq!(l.port, "14922");
    }

    /// A port in the settings file counts as typed; a host with no port does not.
    #[test]
    fn saved_port_is_user_set_and_saved_host_alone_follows() {
        let l = ListEndpoint::from_saved(Some("example.org:14925"), V200);
        assert_eq!((l.host.as_str(), l.port.as_str(), l.port_user_set), ("example.org", "14925", true));
        let mut l = ListEndpoint::from_saved(Some("example.org"), V200);
        assert_eq!((l.host.as_str(), l.port.as_str(), l.port_user_set), ("example.org", "14923", false));
        l.follow_version(V222);
        assert_eq!(l.port, "14922");
        // What gets saved: a port only when the user set one, so saving the
        // form after a list fetch does not freeze the default.
        assert_eq!(l.to_saved(), "example.org");
        l.port = "14925".into();
        l.port_edited();
        assert_eq!(l.to_saved(), "example.org:14925");
        // Round trip.
        assert_eq!(ListEndpoint::from_saved(Some(&l.to_saved()), V222), l);
    }

    /// Clearing the port box goes back to following the version, and an
    /// empty box dials the version's default.
    #[test]
    fn empty_port_box_means_the_version_default() {
        let mut l = ListEndpoint::from_saved(Some("listserver.graal.in:14999"), V200);
        l.port.clear();
        l.port_edited();
        assert!(!l.port_user_set);
        assert_eq!(l.addr(V200), "listserver.graal.in:14923");
        assert_eq!(l.addr(V222), "listserver.graal.in:14922");
    }

    /// The launcher has one "Connect to listserver" button. A second copy
    /// under the status line went unnoticed for a while (2026-10-31).
    #[test]
    fn one_connect_to_listserver_button() {
        let src = include_str!("gui.rs");
        let label = concat!("button(\"Connect to ", "listserver\")");
        assert_eq!(src.matches(label).count(), 1);
    }

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("seihai-gui-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Folders first, then the levels this version opens, in any case.
    #[test]
    fn level_list_has_folders_then_levels() {
        let d = tmp("entries");
        std::fs::create_dir_all(d.join("bodies")).unwrap();
        for f in ["zeta.nw", "Alpha.NW", "readme.txt", "old.graal", "mid.nw"] {
            std::fs::write(d.join(f), b"x").unwrap();
        }
        let exts = vec![".nw".to_string()];
        assert_eq!(level_entries(&d, &exts), ["bodies/", "Alpha.NW", "mid.nw", "zeta.nw"]);
        assert!(level_entries(&d.join("missing"), &exts).is_empty());
        let n = tmp("natural");
        for f in ["level10.nw", "level2.nw", "level1.nw"] {
            std::fs::write(n.join(f), b"x").unwrap();
        }
        assert_eq!(level_entries(&n, &exts), ["level1.nw", "level2.nw", "level10.nw"]);
    }

    #[test]
    fn server_level_lines() {
        let mut s = ServerLevel { server: "Classic".into(), level: Some("onlinestartlocal.nw".into()), cached: None };
        assert_eq!(s.line(), "On Classic: onlinestartlocal.nw");
        assert!(s.cache_line().contains("not in the download cache"));
        s.cached = Some("/c/levels/onlinestartlocal.nw".into());
        assert!(s.cache_line().starts_with("Saved copy: "));
        s.level = None;
        assert_eq!(s.line(), "On Classic: no level yet");
        assert_eq!(s.cache_line(), "");
    }
}
