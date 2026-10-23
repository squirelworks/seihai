//! Networking.
//!
//! Connection and high-level send helpers. Packet formats are provisional
//! where noted; login and framing follow the cleaned 2.05/2.22 observations.

mod udp;

pub use udp::{
    decode_udp, encode_udp, encode_udp_move, udp_key, PeerChannel, UDP_ID_BOMBADD, UDP_ID_CHAT,
    UDP_ID_ENTERLEVEL, UDP_ID_HIT, UDP_ID_ITEMDEL, UDP_ID_LEVELNAME, UDP_ID_MOVE, UDP_ID_SHOWIMG,
    UDP_ID_TRIGGER, UDP_PORT,
};

use crate::protocol::{
    bz2_compress, choose_comp, encode_pos_classic, read_zlib_frame, write_frame, write_gstring,
    write_login, write_zlib_frame, zlib_compress, ClientVersion, EncryptionGen, Gen3, Gen5,
    Gen5Comp, IdSet, LevelRequest, ListReply, LoginRequest, PlayerUpdate, SessionState,
};
use crate::protocol::parse_list_reply;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// TCP connection to a listserver or gserver.
pub struct Connection {
    stream: TcpStream,
    /// Gen-3 insertion state for client→server messages, set by a
    /// versioned 2.05-style login.
    gen3: Option<Gen3>,
    /// Gen-5 XOR state for client→server frames, set by a 2.22 login.
    gen5: Option<Gen5>,
    /// Gen-5 XOR state for server→client frames. Each direction has its
    /// own iterator, both seeded at login.
    gen5_in: Option<Gen5>,
    /// Client→server ids the logged-in version can send, set by
    /// [`Self::login_versioned`]. Others are refused.
    pli: Option<IdSet>,
}

impl Connection {
    /// Connect to `host:port`.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { stream, gen3: None, gen5: None, gen5_in: None, pli: None })
    }

    /// Send a raw byte buffer.
    pub fn send_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data)
    }

    /// Send a version-aware game-server login.
    ///
    /// The login frame is plain zlib for every generation: no gen-3 byte,
    /// no gen-5 compression byte or XOR, since the server needs the key
    /// inside it. Every later client→server frame is ciphered. For gen 5
    /// the iterator is still at the seed for the first frame after the
    /// login (lab 2.22 capture). The `GNW30123` id 7 handshake belongs to
    /// the listserver only; see [`Self::login_listserver`].
    pub fn login_versioned(
        &mut self,
        version: ClientVersion,
        key: u8,
        account: &str,
        password: &str,
    ) -> io::Result<()> {
        let mut line = Vec::new();
        write_login(&mut line, version, key, account, password)?;
        line.push(b'\n');
        self.send_raw(&write_zlib_frame(&line)?)?;
        self.pli = Some(version.profile().protocol.pli);
        match version.encryption() {
            EncryptionGen::Gen3 => self.gen3 = Some(Gen3::new(key)),
            EncryptionGen::Gen5 => {
                self.gen5 = Some(Gen5::new(key));
                self.gen5_in = Some(Gen5::new(key));
            }
            _ => {}
        }
        self.stream.flush()
    }

    /// Log in to a listserver and ask for the server list.
    ///
    /// - 2.22+: a plain zlib frame with id 7 = `GCHAR key` + `GNW30123`
    ///   + `newmain`, then a gen-5 frame with id 1 = account and password.
    ///   Replies are gen-5 frames keyed with the same key.
    /// - Older: one plain zlib frame with id 0 `newmain` and id 1. There
    ///   is no cipher on this link.
    pub fn login_listserver(
        &mut self,
        version: ClientVersion,
        key: u8,
        account: &str,
        password: &str,
    ) -> io::Result<()> {
        let mut serverlist = vec![1 + 0x20];
        write_gstring(&mut serverlist, account)?;
        write_gstring(&mut serverlist, password)?;
        serverlist.push(b'\n');
        if version.encryption() == EncryptionGen::Gen5 {
            let mut hs = vec![7 + 0x20, (key & 0x7F) + 0x20];
            hs.extend_from_slice(version.listserver_token().as_bytes());
            hs.extend_from_slice(b"newmain\n");
            self.send_raw(&write_zlib_frame(&hs)?)?;
            self.gen5 = Some(Gen5::new(key));
            self.gen5_in = Some(Gen5::new(key));
            self.send_messages(&[serverlist])
        } else {
            let mut body = b" newmain\n".to_vec();
            body.extend_from_slice(&serverlist);
            self.send_raw(&write_zlib_frame(&body)?)?;
            self.stream.flush()
        }
    }

    /// Log in to a listserver and collect its reply.
    ///
    /// Reads frames until one carries servers or an error, the server
    /// closes, or five frames have gone by. 1.33–1.39 have no listserver,
    /// so they fetch the list as 2.05.
    pub fn fetch_server_list(
        addr: &str,
        version: ClientVersion,
        account: &str,
        password: &str,
    ) -> io::Result<ListReply> {
        let version = version.list_version();
        let mut conn = Self::connect(addr)?;
        conn.login_listserver(version, random_key(), account, password)?;
        let mut reply = ListReply::default();
        for _ in 0..5 {
            let body = match conn.read_body() {
                Ok(b) => b,
                Err(e) if reply.status.is_some() => {
                    let _ = e;
                    break;
                }
                Err(e) => return Err(e),
            };
            let part = parse_list_reply(&body);
            reply.servers.extend(part.servers);
            reply.status = part.status.or(reply.status);
            reply.site_url = part.site_url.or(reply.site_url);
            reply.error = part.error.or(reply.error);
            reply.upgrade_url = part.upgrade_url.or(reply.upgrade_url);
            if !reply.servers.is_empty() || reply.error.is_some() {
                break;
            }
        }
        Ok(reply)
    }

    /// Read one server frame and return its decompressed body: gen-5
    /// framing after a gen-5 login, plain zlib otherwise.
    pub fn read_body(&mut self) -> io::Result<Vec<u8>> {
        match self.gen5_in.as_mut() {
            Some(g) => crate::protocol::read_frame(&mut self.stream, g),
            None => read_zlib_frame(&mut self.stream),
        }
    }

    /// Send logical messages in one frame.
    ///
    /// Gen-3: zlib + insertion. Gen-5: ctype + XOR + compress (bz2 if ≥ 0x1000).
    pub fn send_messages(&mut self, msgs: &[Vec<u8>]) -> io::Result<()> {
        if let Some(pli) = &self.pli {
            if let Some(id) = msgs.iter().filter_map(|m| m.first()).map(|b| b.wrapping_sub(0x20)).find(|&id| !pli.contains(id)) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("this client version cannot send message {id}"),
                ));
            }
        }
        let mut body = Vec::new();
        for m in msgs {
            match self.gen3.as_mut() {
                Some(g) => body.extend(g.encode(m)),
                None => body.extend_from_slice(m),
            }
        }
        if let Some(g5) = self.gen5.as_mut() {
            let comp = choose_comp(body.len());
            let compressed = match comp {
                Gen5Comp::Bz2 => bz2_compress(&body)?,
                Gen5Comp::Zlib => zlib_compress(&body)?,
                Gen5Comp::None => body,
            };
            let encoded = g5.encode(compressed, comp);
            let frame = write_frame(comp, &encoded);
            self.send_raw(&frame)?;
        } else {
            let frame = write_zlib_frame(&body)?;
            self.send_raw(&frame)?;
        }
        self.stream.flush()
    }

    /// Send what `version` sends straight after its login line:
    /// `PLI_LANGUAGE` (37) for 2.05 and later. 2.17+ would follow it with
    /// `PLI_TAMPERCHECK` (39), which Seihai cannot fill yet.
    pub fn send_post_login(&mut self, version: ClientVersion, language: &str) -> io::Result<()> {
        if !version.profile().protocol.sends_language {
            return Ok(());
        }
        let mut msg = vec![37 + 0x20];
        msg.extend_from_slice(&to_latin1(language));
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

    /// Send `PLI_PLAYERPROPS` (2) with X (15), Y (16) and SPRITE (17).
    ///
    /// This is the 2.05 "walk" message: X/Y in half-tiles, SPRITE with
    /// the direction in its low two bits.
    pub fn send_position(&mut self, x: f32, y: f32, dir: u8) -> io::Result<()> {
        let msg = vec![
            2 + 0x20,
            15 + 0x20,
            encode_pos_classic(x).wrapping_add(0x20),
            16 + 0x20,
            encode_pos_classic(y).wrapping_add(0x20),
            17 + 0x20,
            (dir % 4) + 0x20,
            b'\n',
        ];
        self.send_messages(&[msg])
    }

    /// Read server frames on a background thread.
    ///
    /// After a gen-5 login the reader uses gen-5 framing (ctype + XOR +
    /// decompress) and takes over the server→client iterator. Otherwise
    /// it reads plain zlib frames.
    pub fn spawn_reader(&mut self) -> io::Result<Receiver<io::Result<Vec<u8>>>> {
        let mut stream = self.stream.try_clone()?;
        stream.set_read_timeout(None)?;
        let mut cipher = self.gen5_in.take();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || loop {
            let frame = match cipher.as_mut() {
                Some(g) => crate::protocol::read_frame(&mut stream, g),
                None => read_zlib_frame(&mut stream),
            };
            let stop = frame.is_err();
            if tx.send(frame).is_err() || stop {
                break;
            }
        });
        Ok(rx)
    }

    /// Read one server frame and apply its messages to `state`.
    ///
    /// Uses [`Self::read_body`], then `SessionState::apply_body`. Returns
    /// the decompressed body.
    pub fn read_frame_into(&mut self, state: &mut SessionState) -> io::Result<Vec<u8>> {
        let body = self.read_body()?;
        state.apply_body(&body);
        Ok(body)
    }

    /// Send the locked classic login line.
    pub fn login_classic(
        &mut self,
        username: &str,
        password: &str,
        version: &str,
    ) -> io::Result<()> {
        let pkt = LoginRequest::new(username, password).with_version(version);
        let mut buf = Vec::new();
        pkt.write_classic(&mut buf)?;
        self.send_raw(&buf)?;
        self.stream.write_all(b"\n")?;
        self.stream.flush()
    }

    /// Read lines from the server until the connection closes or times out.
    /// Returns the collected text. No decryption.
    pub fn read_lines(&mut self) -> io::Result<String> {
        let mut reader = BufReader::new(&self.stream);
        let mut out = String::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => out.push_str(&line),
                Err(e) if e.kind() == io::ErrorKind::TimedOut => break,
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    /// Connect to a listserver, send the classic login, return the reply text.
    pub fn listserver_session(
        addr: &str,
        username: &str,
        password: &str,
        version: &str,
    ) -> io::Result<String> {
        let mut conn = Self::connect(addr)?;
        conn.login_classic(username, password, version)?;
        conn.read_lines()
    }

    /// Send a login request (provisional encoding).
    pub fn login(&mut self, username: &str, password: &str) -> io::Result<()> {
        let pkt = LoginRequest::new(username, password);
        let mut buf = Vec::new();
        pkt.write_to(&mut buf)?;
        self.send_raw(&buf)
    }

    /// Request a level by name.
    pub fn request_level(&mut self, name: &str) -> io::Result<()> {
        let pkt = LevelRequest::new(name);
        let mut buf = Vec::new();
        pkt.write_to(&mut buf)?;
        self.send_raw(&buf)
    }

    /// Send a player position update.
    pub fn send_player_update(&mut self, x: f32, y: f32, dir: u8) -> io::Result<()> {
        let pkt = PlayerUpdate { x, y, dir };
        let mut buf = Vec::new();
        pkt.write_to(&mut buf)?;
        self.send_raw(&buf)
    }

    /// Send a walk update for the given position (see [`Self::send_position`]).
    pub fn send_walk(&mut self, upd: &PlayerUpdate) -> io::Result<()> {
        self.send_position(upd.x, upd.y, upd.dir)
    }

    /// Say `text` in the level: `PLI_PLAYERPROPS` (2) with CURCHAT (12)
    /// as a GSTRING. The server shows it as the chat bubble and runs chat
    /// commands such as `setnick` from it. An empty text clears the bubble.
    pub fn send_chat(&mut self, text: &str) -> io::Result<()> {
        let mut msg = vec![2 + 0x20, 12 + 0x20];
        push_gstring_latin1(&mut msg, text);
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

    /// Send a toall: `PLI_TOALL` (6) with a GSTRING message.
    pub fn send_toall(&mut self, text: &str) -> io::Result<()> {
        let mut msg = vec![6 + 0x20];
        push_gstring_latin1(&mut msg, text);
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

    /// Send a private message: `PLI_PRIVATEMESSAGE` (28) with a GSHORT
    /// count, a GSHORT per player id, then the message gtokenized.
    pub fn send_pm(&mut self, ids: &[u16], text: &str) -> io::Result<()> {
        let mut msg = vec![28 + 0x20];
        msg.extend_from_slice(&gshort(ids.len() as u16));
        for &id in ids {
            msg.extend_from_slice(&gshort(id));
        }
        msg.extend(to_latin1(&gtokenize(text)));
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

    /// Open a chest: `PLI_OPENCHEST` (20), `c x`, `c y` in whole tiles.
    pub fn send_open_chest(&mut self, x: i32, y: i32) -> io::Result<()> {
        let msg = vec![20 + 0x20, (x as u8) + 0x20, (y as u8) + 0x20, b'\n'];
        self.send_messages(&[msg])
    }

    /// Warp to `level` at `(x, y)` in tiles (`PLI_LEVELWARP`, id 0):
    /// `GCHAR x*2`, `GCHAR y*2`, level name. The server answers with the
    /// level, as at login.
    pub fn send_levelwarp(&mut self, x: f32, y: f32, level: &str) -> io::Result<()> {
        let mut msg = vec![0x20];
        msg.push(((x * 2.0).clamp(0.0, 127.0) as u8) + 0x20);
        msg.push(((y * 2.0).clamp(0.0, 127.0) as u8) + 0x20);
        msg.extend_from_slice(&to_latin1(level));
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

    /// Request a file from the server (`PLI_WANTFILE`, id 23).
    pub fn send_wantfile(&mut self, name: &str) -> io::Result<()> {
        let mut msg = Vec::new();
        msg.push(23u8.wrapping_add(0x20));
        msg.extend_from_slice(name.as_bytes());
        msg.push(b'\n');
        self.send_messages(&[msg])
    }

}

/// A GSHORT: two GCHARs, high 7 bits first.
fn gshort(v: u16) -> [u8; 2] {
    [((v >> 7) & 0x7F) as u8 + 0x20, (v & 0x7F) as u8 + 0x20]
}

/// Graal text is single-byte: Latin-1 as is, anything else as `?`.
fn to_latin1(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' })
        .collect()
}

/// A GSTRING of `text` in Latin-1, cut to the 223 bytes a GCHAR length
/// can hold.
fn push_gstring_latin1(out: &mut Vec<u8>, text: &str) {
    let mut b = to_latin1(text);
    b.truncate(223);
    out.push(b.len() as u8 + 0x20);
    out.extend(b);
}

/// GServer's gtokenize: one quoted token per line, `"` doubled, joined
/// with commas.
pub fn gtokenize(text: &str) -> String {
    text.lines()
        .map(|l| format!("\"{}\"", l.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(",")
}

/// A login key like the real client's `rand & 0x7F`.
///
/// A fixed even key would always put the gen-3 byte at index 0 of short
/// lines.
pub fn random_key() -> u8 {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(1)
        & 0x7F) as u8
}

/// Extract server-list-like lines from a listserver reply.
///
/// Provisional: returns non-empty lines that do not start with the
/// version token. Real list framing is still to be locked.
pub fn parse_server_list(reply: &str) -> Vec<String> {
    reply
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("GNW"))
        .map(|l| l.to_string())
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{parse_player_props, zlib_decompress};
    use std::io::Read;
    use std::net::TcpListener;

    fn read_frame(s: &mut TcpStream) -> Vec<u8> {
        let mut len = [0u8; 2];
        s.read_exact(&mut len).unwrap();
        let mut body = vec![0u8; u16::from_be_bytes(len) as usize];
        s.read_exact(&mut body).unwrap();
        zlib_decompress(&body).unwrap()
    }

    #[test]
    fn walk_after_login_is_gen3_framed() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let mut conn = Connection::connect(&addr).unwrap();
        let (mut srv, _) = listener.accept().unwrap();

        conn.login_versioned(ClientVersion::V205, 0x42, "ab", "cd").unwrap();
        conn.send_position(30.5, 12.0, 2).unwrap();

        let login = read_frame(&mut srv);
        assert_eq!(&login[..2], &[0x20, 0x42 + 0x20]);

        // Server side: undo the gen-3 byte, as GServer does.
        let body = read_frame(&mut srv);
        let line = body.strip_suffix(b"\n").unwrap();
        let mut g = Gen3::new(0x42);
        let mut enc = line.to_vec();
        enc.push(b'\n');
        let msg = g.decode(&enc).unwrap();
        assert_eq!(msg[0], 2 + 0x20);
        let props = parse_player_props(&msg[1..msg.len() - 1]);
        let vals: Vec<(u8, Option<u8>)> = props.iter().map(|p| (p.id, p.gchar())).collect();
        assert_eq!(vals, vec![(15, Some(61)), (16, Some(24)), (17, Some(2))]);
    }

    fn read_raw(s: &mut TcpStream) -> Vec<u8> {
        let mut len = [0u8; 2];
        s.read_exact(&mut len).unwrap();
        let mut body = vec![0u8; u16::from_be_bytes(len) as usize];
        s.read_exact(&mut body).unwrap();
        body
    }

    fn pair() -> (Connection, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let conn = Connection::connect(&addr).unwrap();
        let (srv, _) = listener.accept().unwrap();
        (conn, srv)
    }

    /// GServer's view of a 2.22 login: the first frame is plain zlib and
    /// is the login line itself (type 5, key, token), and the next frame
    /// is gen-5 with the iterator at the seed.
    #[test]
    fn gen5_login_is_plain_then_seeded() {
        let (mut conn, mut srv) = pair();
        conn.login_versioned(ClientVersion::V222, 0x2B, "ab", "cd").unwrap();
        conn.send_messages(&[b"EEnglish\n".to_vec()]).unwrap();

        let login = read_frame(&mut srv);
        assert_eq!(&login[..10], b"%KGNW03014");
        assert_eq!(&login[10..], b"\"ab\"cd\n");

        let raw = read_raw(&mut srv);
        let mut g = Gen5::new(0x2B);
        let body = crate::protocol::read_frame(
            &mut [&(raw.len() as u16).to_be_bytes()[..], &raw].concat().as_slice(),
            &mut g,
        )
        .unwrap();
        assert_eq!(body, b"EEnglish\n");
    }

    /// 2.22 listserver login: plain zlib id 7 handshake, then a gen-5
    /// frame with id 1, then a gen-5 reply read back with the same key.
    #[test]
    fn gen5_listserver_login() {
        let (mut conn, mut srv) = pair();
        conn.login_listserver(ClientVersion::V222, 0x66, "ab", "cd").unwrap();
        assert_eq!(read_frame(&mut srv), b"'\x86GNW30123newmain\n");
        let raw = read_raw(&mut srv);
        let mut g = Gen5::new(0x66);
        assert_eq!(raw[0], Gen5Comp::None as u8);
        let mut body = raw[1..].to_vec();
        g.xor_body(&mut body, Gen5Comp::None);
        assert_eq!(body, b"!\"ab\"cd\n");

        // Reply: id 2 status, gen-5 zlib, server iterator from the seed.
        let mut out = Gen5::new(0x66);
        let z = zlib_compress(b"\"Lab listserver\n").unwrap();
        let frame = write_frame(Gen5Comp::Zlib, &out.encode(z, Gen5Comp::Zlib));
        srv.write_all(&frame).unwrap();
        let reply = parse_list_reply(&conn.read_body().unwrap());
        assert_eq!(reply.status.as_deref(), Some("Lab listserver"));
    }

    /// 2.05 listserver login: one plain zlib frame, id 0 then id 1.
    #[test]
    fn classic_listserver_login() {
        let (mut conn, mut srv) = pair();
        conn.login_listserver(ClientVersion::V205, 0x66, "ab", "cd").unwrap();
        assert_eq!(read_frame(&mut srv), b" newmain\n!\"ab\"cd\n");
    }

    /// GServer's gen-2 test is "type 0, then a known token": 1.39 sends
    /// no key.
    #[test]
    fn gen2_login_has_no_key() {
        let (mut conn, mut srv) = pair();
        conn.login_versioned(ClientVersion::V139, 0x42, "ab", "cd").unwrap();
        assert_eq!(read_frame(&mut srv), b" GNW07080\"ab\"cd\n");
    }

    /// A gen-5 server that refuses the login: the reader hands over the
    /// disconnect text before the close.
    #[test]
    fn gen5_disconnect_message_reaches_state() {
        let (mut conn, mut srv) = pair();
        conn.login_versioned(ClientVersion::V222, 0x11, "ab", "cd").unwrap();
        let _ = read_frame(&mut srv);
        let mut out = Gen5::new(0x11);
        let msg = b"0Your client version is not allowed on this server.\n".to_vec();
        let frame = write_frame(Gen5Comp::None, &out.encode(msg, Gen5Comp::None));
        srv.write_all(&frame).unwrap();
        drop(srv);

        let rx = conn.spawn_reader().unwrap();
        let mut state = SessionState::default();
        let mut closed = false;
        while let Ok(item) = rx.recv() {
            match item {
                Ok(body) => state.apply_body(&body),
                Err(_) => {
                    closed = true;
                    break;
                }
            }
        }
        assert!(closed);
        assert_eq!(
            state.disconnect_message.as_deref(),
            Some("Your client version is not allowed on this server.")
        );
    }

    #[test]
    fn chat_toall_and_pm_wire() {
        let (mut conn, mut srv) = pair();
        conn.login_versioned(ClientVersion::V139, 0, "ab", "cd").unwrap();
        let _ = read_frame(&mut srv);
        conn.send_chat("hi there").unwrap();
        conn.send_toall("all").unwrap();
        conn.send_pm(&[200, 3], "a \"b\"").unwrap();
        assert_eq!(read_frame(&mut srv), b"\",(hi there\n");
        assert_eq!(read_frame(&mut srv), b"&#all\n");
        // 28, count 2, ids 200 and 3 as GSHORTs, then gtokenized text.
        assert_eq!(read_frame(&mut srv), b"< \"!h #\"a \"\"b\"\"\"\n");
    }

    /// 1.39 cannot send PLI_LANGUAGE (37); 2.05 sends it after login.
    #[test]
    fn sends_follow_the_version() {
        let (mut conn, mut server) = pair();
        conn.login_versioned(ClientVersion::V139, 0, "ab", "cd").unwrap();
        let _ = read_frame(&mut server);
        conn.send_post_login(ClientVersion::V139, "English").unwrap();
        let mut lang = vec![37 + 0x20];
        lang.extend_from_slice(b"English\n");
        assert!(conn.send_messages(&[lang]).is_err());

        let (mut conn, mut server) = pair();
        conn.login_versioned(ClientVersion::V205, 0x42, "ab", "cd").unwrap();
        let _ = read_frame(&mut server);
        conn.send_post_login(ClientVersion::V205, "English").unwrap();
        let body = read_frame(&mut server);
        assert_eq!(body[0], 37 + 0x20);
    }
}
