//! Settings and connect window.

use crate::net::Connection;
use crate::protocol::{parse_list_reply, ClientVersion, SessionState};
use crate::settings::Settings;
use eframe::egui;

pub struct Launcher {
    settings: Settings,
    list_host: String,
    list_port: String,
    account: String,
    password: String,
    version: String,
    assets: String,
    status: String,
    servers: Vec<(String, String, u16)>,
    selected: Option<usize>,
    join_addr: Option<String>,
}

impl Launcher {
    pub fn new() -> Self {
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
            settings: s,
            status: "Ready".into(),
            servers: Vec::new(),
            selected: None,
            join_addr: None,
        }
    }

    fn version_enum(&self) -> ClientVersion {
        match self.version.as_str() {
            "2.22" => ClientVersion::V222,
            "2.31" => ClientVersion::V231,
            "2.17" => ClientVersion::V217,
            "1.39" => ClientVersion::V139,
            _ => ClientVersion::V205,
        }
    }

    fn connect_list(&mut self) {
        self.status = "Connecting to listserver...".into();
        let addr = format!("{}:{}", self.list_host, self.list_port);
        match Connection::connect(&addr) {
            Ok(mut c) => {
                let key = 0x40u8;
                if c.login_versioned(self.version_enum(), key, &self.account, &self.password)
                    .is_ok()
                {
                    let mut state = SessionState::default();
                    if let Ok(body) = c.read_frame_into(&mut state) {
                        let reply = parse_list_reply(&body);
                        self.servers = reply
                            .servers
                            .into_iter()
                            .map(|s| (s.name, s.host, s.port))
                            .collect();
                        self.status = format!("{} servers", self.servers.len());
                        self.settings.listserver = Some(addr);
                        self.settings.account = Some(self.account.clone());
                        self.settings.password = Some(self.password.clone());
                        self.settings.version = Some(self.version.clone());
                        self.settings.assets = Some(self.assets.clone().into());
                        self.settings.save();
                    } else {
                        self.status = "no reply".into();
                    }
                } else {
                    self.status = "login failed".into();
                }
            }
            Err(e) => self.status = format!("connect failed: {e}"),
        }
    }
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Seihai");
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
                    for v in ["1.39", "2.05", "2.17", "2.22", "2.31"] {
                        ui.selectable_value(&mut self.version, v.to_string(), v);
                    }
                });
            ui.separator();
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
                    if let Some((_, host, port)) = self.servers.get(i) {
                        let addr = format!("{host}:{port}");
                        self.settings.last_server = Some(addr.clone());
                        self.settings.save();
                        self.join_addr = Some(addr);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        });
    }
}

/// Run the launcher. Returns the chosen gserver address, if any.
pub fn run() -> Option<String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 520.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Seihai",
        options,
        Box::new(|_cc| Ok(Box::new(Launcher::new()))),
    );
    // Join saved the choice before closing the window.
    Settings::load().last_server
}
