//! The server list: what Start opens when no game server was given.
//!
//! The listserver login is core's (`Connection::fetch_server_list`), with
//! the 3.x row's listserver: `listserver.graal.in` on the row's port, the
//! `G3D30123` token and login shape D (id 7 and id 1 in one plain zlib
//! frame, as the real 3.04 and 3.153 sent it in the lab). `--listserver
//! host:port` points it at a lab listserver instead.
//!
//! The fetch runs on its own thread so the window keeps drawing. Picking a
//! server and pressing Connect hands back its `host:port`; the login at
//! that game server is the same as with `--online`.

use seihai_core::net::{list_failure_text, Connection};
use seihai_core::protocol::{ClientVersion, ListReply, ServerEntry};
use std::io;
use std::sync::mpsc::{self, Receiver, TryRecvError};

/// The listserver Start asks by default for `version`: the row's host and
/// port.
pub fn default_listserver(version: ClientVersion) -> String {
    format!(
        "{}:{}",
        version.listserver_host(),
        version.listserver_port()
    )
}

/// What the list screen asks the app to do.
#[derive(Debug, PartialEq)]
pub enum Pick {
    /// Log in at this game server (`host:port`).
    Connect(String),
    /// Back to the start window.
    Back,
}

/// The server list screen and its fetch.
pub struct ServerList {
    pub listserver: String,
    pub servers: Vec<ServerEntry>,
    pub selected: Option<usize>,
    pub status: String,
    pending: Option<Receiver<io::Result<ListReply>>>,
}

impl ServerList {
    /// Start fetching the list from `listserver` for `version`. `repaint`
    /// is called once the reply is in.
    pub fn fetch(
        listserver: String,
        version: ClientVersion,
        account: String,
        password: String,
        repaint: impl FnOnce() + Send + 'static,
    ) -> ServerList {
        let (tx, rx) = mpsc::channel();
        let addr = listserver.clone();
        std::thread::spawn(move || {
            let r = Connection::fetch_server_list(&addr, version, &account, &password);
            let _ = tx.send(r);
            repaint();
        });
        ServerList {
            status: format!("Asking {listserver} for the server list."),
            listserver,
            servers: Vec::new(),
            selected: None,
            pending: Some(rx),
        }
    }

    /// A list already in hand (tests, and a reply read elsewhere).
    pub fn from_reply(listserver: String, reply: io::Result<ListReply>) -> ServerList {
        let mut l = ServerList {
            listserver,
            servers: Vec::new(),
            selected: None,
            status: String::new(),
            pending: None,
        };
        l.take(reply);
        l
    }

    /// True while the reply has not come in.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    /// Take the reply if it has come in.
    pub fn poll(&mut self) {
        let Some(rx) = &self.pending else { return };
        match rx.try_recv() {
            Ok(r) => {
                self.pending = None;
                self.take(r);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.status = format!("{}: the list fetch stopped.", self.listserver);
            }
        }
    }

    fn take(&mut self, reply: io::Result<ListReply>) {
        match reply {
            Ok(reply) => {
                self.servers = reply.servers;
                self.selected = None;
                self.status = match (reply.error, self.servers.len()) {
                    (Some(err), _) => format!("listserver: {err}"),
                    (None, 0) => format!("{}: no servers in the reply.", self.listserver),
                    (None, n) => {
                        let s = if n == 1 { "" } else { "s" };
                        match reply.status {
                            Some(st) if !st.trim().is_empty() => {
                                format!("{n} server{s}. {}", st.trim())
                            }
                            _ => format!("{n} server{s}."),
                        }
                    }
                };
            }
            Err(e) => self.status = list_failure_text(&self.listserver, &e),
        }
    }

    /// The picked server's `host:port`, if one is picked.
    pub fn chosen(&self) -> Option<String> {
        let s = self.servers.get(self.selected?)?;
        Some(format!("{}:{}", s.host, s.port))
    }

    /// One row's text: kind, name, players and language.
    pub fn row(s: &ServerEntry) -> String {
        let kind = match s.kind.as_str() {
            "" => String::new(),
            k => format!("[{k}] "),
        };
        let players = s.players.map_or_else(|| "?".into(), |n| n.to_string());
        let lang = if s.language.is_empty() {
            String::new()
        } else {
            format!("  ({})", s.language)
        };
        format!("{kind}{}  {players} players{lang}", s.name)
    }

    /// Draw the list and take its input.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<Pick> {
        self.poll();
        let mut pick = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Servers");
            ui.label(&self.status);
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 40.0)
                .show(ui, |ui| {
                    for (i, s) in self.servers.iter().enumerate() {
                        let r = ui.selectable_label(self.selected == Some(i), Self::row(s));
                        if r.clicked() {
                            self.selected = Some(i);
                        }
                        if r.double_clicked() {
                            pick = Some(Pick::Connect(format!("{}:{}", s.host, s.port)));
                        }
                    }
                });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Back").clicked() {
                    pick = Some(Pick::Back);
                }
                let connect = ui.add_enabled(self.chosen().is_some(), egui::Button::new("Connect"));
                if connect.clicked() {
                    pick = self.chosen().map(Pick::Connect);
                }
            });
        });
        if self.waiting() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        pick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: &str, name: &str, port: u16) -> ServerEntry {
        ServerEntry {
            kind: kind.into(),
            name: name.into(),
            language: "English".into(),
            description: String::new(),
            url: String::new(),
            version: String::new(),
            host: "127.0.0.1".into(),
            port,
            players: Some(3),
        }
    }

    #[test]
    fn the_3x_rows_ask_listserver_graal_in() {
        for v in [ClientVersion::V304, ClientVersion::V3153] {
            let d = default_listserver(v);
            assert!(d.starts_with("listserver.graal.in:"), "{d}");
            assert_eq!(d, format!("listserver.graal.in:{}", v.listserver_port()));
        }
    }

    #[test]
    fn a_reply_fills_the_list_and_a_pick_gives_its_address() {
        let reply = ListReply {
            servers: vec![entry("", "Lab", 14900), entry("3", "Lab 3D", 14910)],
            status: Some("Welcome\n".into()),
            ..Default::default()
        };
        let mut l = ServerList::from_reply("127.0.0.1:14922".into(), Ok(reply));
        assert_eq!(l.status, "2 servers. Welcome");
        assert_eq!(l.chosen(), None);
        l.selected = Some(1);
        assert_eq!(l.chosen().as_deref(), Some("127.0.0.1:14910"));
        assert_eq!(
            ServerList::row(&l.servers[1]),
            "[3] Lab 3D  3 players  (English)"
        );
    }

    #[test]
    fn an_error_text_is_shown() {
        let reply = ListReply {
            error: Some("Account not found".into()),
            ..Default::default()
        };
        let l = ServerList::from_reply("127.0.0.1:14922".into(), Ok(reply));
        assert_eq!(l.status, "listserver: Account not found");
        assert!(l.servers.is_empty());
    }

    /// The whole path on loopback: a stand-in listserver reads the 3.04
    /// login and answers with one server, and the fetch thread hands it over.
    #[test]
    fn fetch_reads_a_lab_listserver() {
        use seihai_core::protocol::{write_frame, Gen5, Gen5Comp};
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut len = [0u8; 2];
            s.read_exact(&mut len).unwrap();
            let mut body = vec![0u8; u16::from_be_bytes(len) as usize];
            s.read_exact(&mut body).unwrap();
            let login = seihai_core::protocol::zlib_decompress(&body).unwrap();
            assert!(login.windows(8).any(|w| w == b"G3D30123"), "3.x list token");
            // One server: count, then name, language, description, url,
            // version, players, host, port as GSTRINGs (id 0, SVRLIST).
            let gs = |t: &str| [&[(t.len() + 32) as u8][..], t.as_bytes()].concat();
            let mut list = vec![32u8, 32 + 1, 32 + 8];
            for f in ["Lab", "English", "", "", "1.0", "1", "127.0.0.1", "14900"] {
                list.extend(gs(f));
            }
            list.push(b'\n');
            list.extend(b"\"Lab listserver\n");
            let z = seihai_core::protocol::zlib_compress(&list).unwrap();
            let mut out = Gen5::new(login_key(&login));
            s.write_all(&write_frame(Gen5Comp::Zlib, &out.encode(z, Gen5Comp::Zlib)))
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(300));
        });
        let mut l = ServerList::fetch(addr, ClientVersion::V304, "ab".into(), "cd".into(), || {});
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while l.waiting() && std::time::Instant::now() < until {
            l.poll();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        server.join().unwrap();
        assert_eq!(l.servers.len(), 1, "{}", l.status);
        assert_eq!(l.servers[0].name, "Lab");
        l.selected = Some(0);
        assert_eq!(l.chosen().as_deref(), Some("127.0.0.1:14900"));
    }

    /// The encryption key the 3.x login frame carries: the byte after id 7.
    fn login_key(login: &[u8]) -> u8 {
        login[1].wrapping_sub(32)
    }
}
