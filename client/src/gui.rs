//! Settings and connect menu drawn in a minifb window.
//!
//! Keyboard: Up/Down move, Left/Right change, Enter confirms,
//! Escape cancels.

use crate::net::Connection;
use crate::protocol::{parse_list_reply, ClientVersion, SessionState};
use crate::settings::Settings;
use minifb::{Key, Window, WindowOptions};

pub struct Launcher {
    list_host: String,
    list_port: String,
    account: String,
    password: String,
    version: String,
    assets: String,
    status: String,
    servers: Vec<(String, String, u16)>,
    selected: usize,
    field: usize,
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
            selected: 0,
            field: 0,
            error,
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
        // 1.39 has no listserver. Fetch the list with 2.05, then join as 1.39.
        let list_version = if self.version == "1.39" {
            println!("1.39 has no listserver; fetching the list as 2.05");
            ClientVersion::V205
        } else {
            self.version_enum()
        };
        self.status = "Connecting...".into();
        let addr = format!("{}:{}", self.list_host, self.list_port);
        match Connection::connect(&addr) {
            Ok(mut c) => {
                let key = 0x40u8;
                if c.login_versioned(list_version, key, &self.account, &self.password)
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
                        let mut settings = Settings::load();
                        settings.listserver = Some(addr);
                        settings.account = Some(self.account.clone());
                        settings.password = Some(self.password.clone());
                        settings.version = Some(self.version.clone());
                        settings.assets = Some(self.assets.clone().into());
                        settings.save();
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

    fn draw(&self, buf: &mut [u32]) {
        // Clear.
        for p in buf.iter_mut() {
            *p = 0x1a1a1a;
        }
        // Simple text via filled blocks is not practical here.
        // The menu is driven by the terminal; this window is a placeholder
        // that shows status. The real input is stdin in run().
        let _ = buf;
    }
}

/// Run the launcher. Returns the chosen gserver address, if any.
pub fn run() -> Option<String> {
    run_inner(None)
}

/// Run with an error from a failed connection.
pub fn run_with_error(error: String) -> Option<String> {
    run_inner(Some(error))
}

fn run_inner(error: Option<String>) -> Option<String> {
    let mut launcher = Launcher::with_error(error);
    // Terminal menu: the minifb window is open so the user sees something,
    // but input is from stdin so we do not need a font.
    let mut window = match Window::new(
        "Seihai",
        420,
        200,
        WindowOptions::default(),
    ) {
        Ok(w) => w,
        Err(_) => return terminal_menu(&mut launcher),
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));
    let mut buf = vec![0x1a1a1au32; 420 * 200];
    // Print the menu to the terminal and read choices.
    let result = terminal_menu(&mut launcher);
    let _ = window.update_with_buffer(&buf, 420, 200);
    result
}

fn terminal_menu(l: &mut Launcher) -> Option<String> {
    if let Some(err) = &l.error {
        println!("Error: {err}");
    }
    println!("Seihai");
    println!("1) Connect");
    println!("2) Settings");
    println!("q) Quit");
    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            return None;
        }
        match line.trim() {
            "1" => {
                l.connect_list();
                println!("{}", l.status);
                if !l.servers.is_empty() {
                    for (i, (name, host, port)) in l.servers.iter().enumerate() {
                        println!("  {i}) {name} ({host}:{port})");
                    }
                    println!("pick a number:");
                    let mut pick = String::new();
                    let _ = std::io::stdin().read_line(&mut pick);
                    if let Ok(n) = pick.trim().parse::<usize>() {
                        if let Some((_, host, port)) = l.servers.get(n) {
                            let addr = format!("{host}:{port}");
                            let mut s = Settings::load();
                            s.last_server = Some(addr.clone());
                            s.save();
                            return Some(addr);
                        }
                    }
                }
            }
            "2" => {
                println!("Settings");
                println!("1) assets: {}", l.assets);
                println!("2) listserver: {}:{}", l.list_host, l.list_port);
                println!("3) account: {}", l.account);
                println!("4) password: {}", if l.password.is_empty() { "(empty)" } else { "****" });
                println!("5) version: {}", l.version);
                println!("b) back");
                loop {
                    let mut sline = String::new();
                    if std::io::stdin().read_line(&mut sline).is_err() {
                        break;
                    }
                    match sline.trim() {
                        "1" => {
                            println!("assets path:");
                            let mut p = String::new();
                            let _ = std::io::stdin().read_line(&mut p);
                            l.assets = p.trim().to_string();
                        }
                        "2" => {
                            println!("host:");
                            let mut h = String::new();
                            let _ = std::io::stdin().read_line(&mut h);
                            l.list_host = h.trim().to_string();
                            println!("port:");
                            let mut p = String::new();
                            let _ = std::io::stdin().read_line(&mut p);
                            l.list_port = p.trim().to_string();
                        }
                        "3" => {
                            println!("account:");
                            let mut a = String::new();
                            let _ = std::io::stdin().read_line(&mut a);
                            l.account = a.trim().to_string();
                        }
                        "4" => {
                            println!("password:");
                            let mut p = String::new();
                            let _ = std::io::stdin().read_line(&mut p);
                            l.password = p.trim().to_string();
                        }
                        "5" => {
                            println!("version (1.39 / 2.05 / 2.17 / 2.22 / 2.31):");
                            let mut v = String::new();
                            let _ = std::io::stdin().read_line(&mut v);
                            l.version = v.trim().to_string();
                        }
                        "b" => break,
                        _ => {}
                    }
                }
            }
            "q" => return None,
            _ => {}
        }
    }
}
