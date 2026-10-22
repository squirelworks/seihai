//! Settings and connect window.

use crate::net::Connection;
use crate::protocol::ClientVersion;
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
        }
    }

    fn version_enum(&self) -> ClientVersion {
        for v in ClientVersion::all_versions() {
            if v.label() == self.version {
                return *v;
            }
        }
        ClientVersion::V205
    }

    fn connect_list(&mut self) {
        let list_version = if self.version == "1.39" {
            self.status = "1.39 has no listserver; fetching as 2.05".into();
            ClientVersion::V205
        } else {
            self.version_enum()
        };
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
            egui::ComboBox::from_label("")
                .selected_text(&self.version)
                .show_ui(ui, |ui| {
                    for v in ClientVersion::main_versions() {
                        ui.selectable_value(&mut self.version, v.label().to_string(), v.label());
                    }
                });
            ui.checkbox(&mut self.show_advanced, "Advanced versions");
            if self.show_advanced {
                egui::ComboBox::from_label("all")
                    .selected_text(&self.version)
                    .show_ui(ui, |ui| {
                        for v in ClientVersion::all_versions() {
                            ui.selectable_value(
                                &mut self.version,
                                v.label().to_string(),
                                format!("{} ({})", v.label(), v.token()),
                            );
                        }
                    });
            }
            ui.separator();
            if ui.button("Play offline").clicked() {
                let mut s = Settings::load();
                s.last_server = None;
                s.offline = Some(true);
                s.save();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
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
                    if let Some((_, host, port)) = self.servers.get(i) {
                        let addr = format!("{host}:{port}");
                        let mut s = Settings::load();
                        s.last_server = Some(addr);
                        s.save();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
