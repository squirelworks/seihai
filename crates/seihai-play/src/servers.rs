//! A server list and its fetch, shared by the start window
//! ([`crate::start`]) and Ato's own server list screen (#469).
//!
//! The listserver login is core's (`Connection::fetch_server_list`). The
//! fetch runs on its own thread so the window keeps drawing; its result is
//! also printed to stdout, so a run with the window open can be read
//! without a screenshot.

use seihai_core::net::{list_failure_text, Connection};
use seihai_core::protocol::{ClientVersion, ListReply, ServerEntry};
use std::io;
use std::sync::mpsc::{self, Receiver, TryRecvError};

/// For a build with no listserver of its own (4.x and 5.x, which reach a
/// game server named `Login` instead): how its list is fetched. Core falls
/// back to an older build's listserver login (`ClientVersion::list_version`);
/// the picked server is then logged in to with the build's own game login,
/// as if the Login server had warped there. The real 4.x and 5.x clients
/// show the list as scripted GUI on the Login server, so this is Seihai's
/// choice, not theirs.
pub fn borrowed_list_note(version: ClientVersion) -> Option<String> {
    let list = version.list_version();
    (list != version).then(|| {
        format!(
            "{} has no listserver; the list comes with the {} login.",
            version.label(),
            list.label()
        )
    })
}

/// The servers a listserver sent, the one picked, and the fetch.
pub struct ServerList {
    /// The listserver asked, `host:port`.
    pub listserver: String,
    pub servers: Vec<ServerEntry>,
    pub selected: Option<usize>,
    pub status: String,
    pending: Option<Receiver<io::Result<ListReply>>>,
}

impl ServerList {
    /// No list yet, with `status` shown.
    pub fn empty(status: impl Into<String>) -> ServerList {
        ServerList {
            listserver: String::new(),
            servers: Vec::new(),
            selected: None,
            status: status.into(),
            pending: None,
        }
    }

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
            let _ = tx.send(Connection::fetch_server_list(
                &addr, version, &account, &password,
            ));
            repaint();
        });
        let mut status = format!("Asking {listserver} for the server list.");
        if let Some(note) = borrowed_list_note(version) {
            status = format!("{note} {status}");
        }
        ServerList {
            pending: Some(rx),
            ..ServerList::empty(status)
        }
        .at(listserver)
    }

    fn at(mut self, listserver: String) -> ServerList {
        self.listserver = listserver;
        self
    }

    /// A list already in hand (tests, and a reply read elsewhere).
    pub fn from_reply(listserver: String, reply: io::Result<ListReply>) -> ServerList {
        let mut l = ServerList::empty("").at(listserver);
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
        let reply = match rx.try_recv() {
            Ok(r) => r,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(io::Error::other("the list fetch stopped")),
        };
        self.pending = None;
        self.take(reply);
    }

    /// A list reply, from the fetch or a test.
    pub fn take(&mut self, reply: io::Result<ListReply>) {
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
        self.print();
    }

    /// The reply on stdout: the status, then one line per server.
    fn print(&self) {
        println!("server list from {}: {}", self.listserver, self.status);
        for s in &self.servers {
            println!("  {}  {}:{}", Self::row(s), s.host, s.port);
        }
    }

    /// The picked server.
    pub fn picked(&self) -> Option<&ServerEntry> {
        self.servers.get(self.selected?)
    }

    /// The picked server's `host:port`, if one is picked.
    pub fn chosen(&self) -> Option<String> {
        self.picked().map(|s| format!("{}:{}", s.host, s.port))
    }

    /// One row's text: kind, name, players and language.
    pub fn row(s: &ServerEntry) -> String {
        let kind = match s.kind.trim() {
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

    /// The rows, in a scroll area `height` high. A click picks a row; a
    /// double click picks it and is returned, to join at once.
    pub fn rows_ui(&mut self, ui: &mut egui::Ui, height: f32) -> Option<usize> {
        let mut joined = None;
        egui::ScrollArea::vertical()
            .max_height(height)
            .show(ui, |ui| {
                for (i, s) in self.servers.iter().enumerate() {
                    let r = ui
                        .selectable_label(self.selected == Some(i), Self::row(s))
                        .on_hover_text(format!("{}:{}", s.host, s.port));
                    if r.clicked() {
                        self.selected = Some(i);
                    }
                    if r.double_clicked() {
                        self.selected = Some(i);
                        joined = Some(i);
                    }
                }
            });
        joined
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
    fn the_lobby_builds_say_whose_list_login_they_borrow() {
        assert_eq!(borrowed_list_note(ClientVersion::V304), None);
        assert_eq!(borrowed_list_note(ClientVersion::V3153), None);
        for v in [
            ClientVersion::V420R8,
            ClientVersion::V500R7,
            ClientVersion::V512,
        ] {
            let n = borrowed_list_note(v).unwrap();
            assert!(
                n.starts_with(&format!("{} has no listserver", v.label())),
                "{n}"
            );
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
        let none = ServerList::from_reply("h:1".into(), Ok(ListReply::default()));
        assert_eq!(none.status, "h:1: no servers in the reply.");
    }
}
