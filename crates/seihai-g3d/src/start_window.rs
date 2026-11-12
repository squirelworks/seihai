//! The start window: the client's connect form (`TConnectFrame`, Graal_analysis
//! `graal3d/README.md`) with nickname, host, port, account and password.
//!
//! The fields and their defaults are the client's: nickname `Stefan`, the
//! loopback host, port 14901, account and password `guest`. The layout and
//! the button names are Seihai's, since the form's positions were not read.
//! The server address in `graal3dconfig.txt` is never offered here, so a
//! start from the window never reaches a host the player did not type.

use std::sync::{Arc, Mutex};

/// The client's defaults for the form.
pub const DEFAULT_NICK: &str = "Stefan";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 14901;
pub const DEFAULT_ACCOUNT: &str = "guest";
pub const DEFAULT_PASSWORD: &str = "guest";

/// What the form holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectForm {
    pub nick: String,
    pub host: String,
    pub port: String,
    pub account: String,
    pub password: String,
}

impl Default for ConnectForm {
    fn default() -> Self {
        ConnectForm {
            nick: DEFAULT_NICK.into(),
            host: DEFAULT_HOST.into(),
            port: DEFAULT_PORT.to_string(),
            account: DEFAULT_ACCOUNT.into(),
            password: DEFAULT_PASSWORD.into(),
        }
    }
}

impl ConnectForm {
    /// `host:port`, or what is wrong with the form.
    pub fn addr(&self) -> Result<String, String> {
        let host = self.host.trim();
        if host.is_empty() {
            return Err("Enter a host.".into());
        }
        if host.contains(char::is_whitespace) {
            return Err("The host has a space in it.".into());
        }
        let port: u16 = self
            .port
            .trim()
            .parse()
            .ok()
            .filter(|&p| p > 0)
            .ok_or("The port is a number from 1 to 65535.")?;
        Ok(if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]:{port}")
        } else {
            format!("{host}:{port}")
        })
    }

    /// The nickname to log in with: the typed one, or the account.
    pub fn nick_or_account(&self) -> String {
        let n = self.nick.trim();
        if n.is_empty() {
            self.account.trim().to_string()
        } else {
            n.to_string()
        }
    }
}

struct App {
    form: ConnectForm,
    status: String,
    chosen: Arc<Mutex<Option<ConnectForm>>>,
}

impl App {
    fn connect(&mut self, ctx: &egui::Context) {
        match self.form.addr() {
            Ok(_) => {
                *self.chosen.lock().unwrap_or_else(|e| e.into_inner()) = Some(self.form.clone());
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.status = e,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Graal3D");
            ui.add_space(8.0);
            egui::Grid::new("connect")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    for (label, value, secret) in [
                        ("Nickname", &mut self.form.nick, false),
                        ("Host", &mut self.form.host, false),
                        ("Port", &mut self.form.port, false),
                        ("Account", &mut self.form.account, false),
                        ("Password", &mut self.form.password, true),
                    ] {
                        ui.label(label);
                        ui.add(
                            egui::TextEdit::singleline(value)
                                .password(secret)
                                .desired_width(200.0),
                        );
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("Connect").clicked() || enter {
                    self.connect(ctx);
                }
                if ui.button("Quit").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            if !self.status.is_empty() {
                ui.add_space(6.0);
                ui.colored_label(egui::Color32::LIGHT_RED, &self.status);
            }
        });
    }
}

/// Show the form until Connect (the filled form) or Quit / close (`None`).
pub fn run(form: ConnectForm) -> Result<Option<ConnectForm>, String> {
    let chosen = Arc::new(Mutex::new(None));
    let app = App {
        form,
        status: String::new(),
        chosen: chosen.clone(),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([340.0, 240.0])
            .with_resizable(false),
        ..Default::default()
    };
    seihai_play::window::run_native("Graal3D", options, Box::new(move |_cc| Ok(Box::new(app))))
        .map_err(|e| e.to_string())?;
    let out = chosen.lock().unwrap_or_else(|e| e.into_inner()).take();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_clients() {
        let f = ConnectForm::default();
        assert_eq!(
            (f.nick.as_str(), f.account.as_str(), f.password.as_str()),
            ("Stefan", "guest", "guest")
        );
        assert_eq!(f.addr().unwrap(), "127.0.0.1:14901");
    }

    #[test]
    fn the_address_is_checked() {
        let with = |host: &str, port: &str| {
            ConnectForm {
                host: host.into(),
                port: port.into(),
                ..ConnectForm::default()
            }
            .addr()
        };
        assert_eq!(with(" lab.local ", " 14902 ").unwrap(), "lab.local:14902");
        assert_eq!(with("::1", "14901").unwrap(), "[::1]:14901");
        assert!(with("", "14901").is_err());
        assert!(with("a b", "14901").is_err());
        assert!(with("h", "0").is_err());
        assert!(with("h", "70000").is_err());
        assert!(with("h", "x").is_err());
    }

    #[test]
    fn an_empty_nickname_falls_back_to_the_account() {
        let f = ConnectForm {
            nick: "  ".into(),
            account: "lab".into(),
            ..ConnectForm::default()
        };
        assert_eq!(f.nick_or_account(), "lab");
        assert_eq!(ConnectForm::default().nick_or_account(), "Stefan");
    }
}
