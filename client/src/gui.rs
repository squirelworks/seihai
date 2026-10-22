//! Settings and connect window.

use crate::net::Connection;
use crate::protocol::{ClientVersion, ServerList};
use crate::settings::Settings;
use eframe::egui;

pub struct Launcher {
    list_host: String,
    list_port: String,
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
            list_host: s
                .listserver
                .as_deref()
                .unwrap_or("listserver.graal.in")
                .split(':')
                .next()
                .unwrap_or("listserver.graal.in")
                .to_string(),
            list_port: s
                .listserver
                .as_deref()
                .and_then(|l| l.split(':').nth(1))
                .unwrap_or("14922")
                .to_string(),
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
        let addr = format!("{}:{}", self.list_host, self.list_port);
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
                    settings.listserver = Some(addr);
                    settings.account = Some(self.account.clone());
                    settings.password = Some(self.password.clone());
                    settings.version = Some(self.version.clone());
                    settings.assets = Some(self.assets.clone().into());
                    settings.save();
                }
            }
            Err(e) => self.status = format!("listserver failed: {e}"),
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
                ui.text_edit_singleline(&mut self.list_host);
                ui.label(":");
                ui.text_edit_singleline(&mut self.list_port);
            });
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
                let label = self.version_enum().label();
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
            ui.label("Servers");
            for (i, (name, host, port)) in self.servers.iter().enumerate() {
                if ui
                    .selectable_label(self.selected == Some(i), format!("{name} ({host}:{port})"))
                    .clicked()
                {
                    self.selected = Some(i);
                }
            }
            if let Some(i) = self.selected {
                if ui.button("Join").clicked() {
                    if let Some((name, host, port)) = self.servers.get(i).cloned() {
                        self.join(ctx, &name, format!("{host}:{port}"));
                    }
                }
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
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 520.0]),
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
