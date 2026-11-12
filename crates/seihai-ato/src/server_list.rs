//! The server list: what Start opens when no game server was given.
//!
//! The list and its fetch are `seihai-play`'s ([`ServerList`]), the same
//! the `seihai` start window shows (#469); this screen is Ato's own
//! layout around it, with Back and Connect. The listserver is the 3.x
//! row's: `listserver.graal.in` on port 14923, the `G3D30123` token and
//! login shape D (id 7 and id 1 in one plain zlib frame, as the real 3.04
//! and 3.153 sent it in the lab). `--listserver host:port` points it at a
//! lab listserver instead. Picking a server and pressing Connect hands
//! back its `host:port`; the login at that game server is the same as
//! with `--online`.

use seihai_core::protocol::ClientVersion;
pub use seihai_play::servers::{borrowed_list_note, ServerList};

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

/// Draw the list and take its input.
pub fn show(list: &mut ServerList, ctx: &egui::Context) -> Option<Pick> {
    list.poll();
    let mut pick = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Servers");
        ui.label(&list.status);
        ui.separator();
        if list.rows_ui(ui, ui.available_height() - 40.0).is_some() {
            pick = list.chosen().map(Pick::Connect);
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                pick = Some(Pick::Back);
            }
            let connect = ui.add_enabled(list.chosen().is_some(), egui::Button::new("Connect"));
            if connect.clicked() {
                pick = list.chosen().map(Pick::Connect);
            }
        });
    });
    if list.waiting() {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
    pick
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_3x_rows_ask_listserver_graal_in() {
        for v in [ClientVersion::V304, ClientVersion::V3153] {
            let d = default_listserver(v);
            assert!(d.starts_with("listserver.graal.in:"), "{d}");
            // 14923: the public listserver sent the list there to this
            // login, and only the account error on 14922.
            assert_eq!(d, "listserver.graal.in:14923");
        }
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
