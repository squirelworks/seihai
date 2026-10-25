//! Settings and connect window.

use crate::net::Connection;
use crate::protocol::{ClientVersion, ServerList};
use crate::settings::Settings;
use eframe::egui;

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
                s.version.as_deref().and_then(ClientVersion::from_label).unwrap_or(ClientVersion::V205),
            ),
            account: s.account.clone().unwrap_or_default(),
            password: s.password.clone().unwrap_or_default(),
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
            show_advanced: false,
            direct: s
                .direct_server
                .clone()
                .unwrap_or_else(|| "localhost:14900".into()),
            version_assets: String::new(),
        }
        .with_version_assets()
    }

    /// Load the per-version asset folder for the selected version.
    fn with_version_assets(mut self) -> Self {
        let label = self.version_enum().label();
        self.version_assets = Settings::load()
            .assets_for(label)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self
    }

    /// Save the form and the chosen server, then close.
    fn join(&self, ctx: &egui::Context, name: &str, addr: String) {
        let mut s = Settings::load();
        s.account = Some(self.account.clone());
        s.password = Some(self.password.clone());
        s.version = Some(self.version.clone());
        s.assets = Some(self.assets.clone().into());
        s.set_assets_for(self.version_enum().label(), &self.version_assets);
        s.last_server = Some(addr);
        s.last_server_name = Some(name.to_string());
        s.offline = None;
        s.save();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn version_enum(&self) -> ClientVersion {
        ClientVersion::from_label(&self.version).unwrap_or(ClientVersion::V205)
    }

    fn connect_list(&mut self) {
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
            ui.label("Version");
            let before = self.version.clone();
            let shown: &[ClientVersion] = if self.show_advanced {
                ClientVersion::all_versions()
            } else {
                ClientVersion::main_versions()
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
                });
            ui.checkbox(&mut self.show_advanced, "All versions");
            if self.version != before {
                let v = self.version_enum();
                self.list.follow_version(v);
                let label = v.label();
                self.version_assets = Settings::load()
                    .assets_for(label)
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
            }
            let profile = self.version_enum().profile();
            ui.small(profile.summary());
            ui.label(format!("Assets for {} only (optional)", profile.label));
            ui.text_edit_singleline(&mut self.version_assets);
            if let ServerList::Direct { default } = profile.ui.server_list {
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
                s.set_assets_for(self.version_enum().label(), &self.version_assets);
                s.save();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if ui.button("Connect to listserver").clicked() {
                self.connect_list();
            }
            ui.label(&self.status);
            ui.separator();
            if ui.button("Connect to listserver").clicked() {
                self.connect_list();
            }
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

pub fn run() -> Option<String> {
    run_inner(None)
}

pub fn run_with_error(error: String) -> Option<String> {
    run_inner(Some(error))
}

fn run_inner(error: Option<String>) -> Option<String> {
    // Clear any previous choice so closing the window does not reconnect.
    let mut s = Settings::load();
    s.last_server = None;
    s.save();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 520.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Seihai",
        options,
        Box::new(move |_cc| Ok(Box::new(Launcher::with_error(error)))),
    );
    Settings::load().last_server
}

/// Pick a level file with one of `exts` (e.g. `.nw`). Returns the chosen
/// path, if any.
pub fn pick_level(start: &std::path::Path, exts: &[&str]) -> Option<std::path::PathBuf> {
    use std::sync::{Arc, Mutex};
    let chosen = Arc::new(Mutex::new(None));
    let chosen2 = chosen.clone();
    struct Picker {
        dir: std::path::PathBuf,
        entries: Vec<String>,
        out: Arc<Mutex<Option<std::path::PathBuf>>>,
        exts: Vec<String>,
    }
    impl Picker {
        fn refresh(&mut self) {
            self.entries.clear();
            if let Ok(rd) = std::fs::read_dir(&self.dir) {
                let mut dirs = Vec::new();
                let mut files = Vec::new();
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if e.path().is_dir() {
                        dirs.push(format!("{name}/"));
                    } else if self.exts.iter().any(|x| name.ends_with(x.as_str())) {
                        files.push(name);
                    }
                }
                dirs.sort();
                files.sort();
                self.entries.extend(dirs);
                self.entries.extend(files);
            }
        }
    }
    impl eframe::App for Picker {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Pick a level");
                ui.label(self.dir.display().to_string());
                if ui.button("..").clicked() {
                    if let Some(p) = self.dir.parent() {
                        self.dir = p.to_path_buf();
                        self.refresh();
                    }
                }
                for name in self.entries.clone() {
                    if ui.button(&name).clicked() {
                        let path = self.dir.join(name.trim_end_matches('/'));
                        if path.is_dir() {
                            self.dir = path;
                            self.refresh();
                        } else {
                            *self.out.lock().unwrap() = Some(path);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            });
        }
    }
    let mut picker = Picker {
        dir: start.to_path_buf(),
        entries: Vec::new(),
        out: chosen2,
        exts: exts.iter().map(|x| x.to_string()).collect(),
    };
    picker.refresh();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 480.0]),
        ..Default::default()
    };
    let _ = eframe::run_native("Pick a level", options, Box::new(|_| Ok(Box::new(picker))));
    let picked = chosen.lock().unwrap().take();
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
