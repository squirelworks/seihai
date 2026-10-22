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
    encode_pos_classic, read_zlib_frame, write_login, write_zlib_frame, ClientVersion,
    EncryptionGen, Gen3, LevelRequest, LoginRequest, PlayerUpdate, SessionState,
};
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
    gen5: Option<crate::protocol::Gen5>,
}

impl Connection {
    /// Connect to `host:port`.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { stream, gen3: None, gen5: None })
    }

    /// Send a raw byte buffer.
    pub fn send_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data)
    }

    /// Send a version-aware login as a zlib frame.
    ///
    /// For gen-5, first sends the listserver handshake: plain zlib of
    /// `GCHAR 7` + `GCHAR key` + `GNW30123` + `newmain`. Every later
    /// frame is then gen-5. The game login follows.
    pub fn login_versioned(
        &mut self,
        version: ClientVersion,
        key: u8,
        account: &str,
        password: &str,
    ) -> io::Result<()> {
        if version.encryption() == EncryptionGen::Gen5 {
            // Listserver handshake for 2.22+.
            let mut hs = Vec::new();
            hs.push(7u8.wrapping_add(0x20));
            hs.push(key.wrapping_add(0x20));
            hs.extend_from_slice(b"GNW30123");
            hs.extend_from_slice(b"newmain");
            hs.push(b'\n');
            let frame = write_zlib_frame(&hs)?;
            self.send_raw(&frame)?;
            self.gen5 = Some(crate::protocol::Gen5::new(key));
        }
        let mut line = Vec::new();
        write_login(&mut line, version, key, account, password)?;
        line.push(b'\n');
        if version.encryption() == EncryptionGen::Gen5 {
            let g5 = self.gen5.as_mut().unwrap();
            let comp = crate::protocol::choose_comp(line.len());
            let compressed = match comp {
                crate::protocol::Gen5Comp::Bz2 => crate::protocol::bz2_compress(&line)?,
                crate::protocol::Gen5Comp::Zlib => crate::protocol::zlib_compress(&line)?,
                crate::protocol::Gen5Comp::None => line.clone(),
            };
            let encoded = g5.encode(compressed, comp);
            let frame = crate::protocol::write_frame(comp, &encoded);
            self.send_raw(&frame)?;
        } else {
            let frame = write_zlib_frame(&line)?;
            self.send_raw(&frame)?;
            if version.encryption() == EncryptionGen::Gen3 {
                self.gen3 = Some(Gen3::new(key));
            }
        }
        self.stream.flush()
    }

    /// Send logical messages in one frame.
    ///
    /// Gen-3: zlib + insertion. Gen-5: ctype + XOR + compress (bz2 if ≥ 0x1000).
    pub fn send_messages(&mut self, msgs: &[Vec<u8>]) -> io::Result<()> {
        let mut body = Vec::new();
        for m in msgs {
            match self.gen3.as_mut() {
                Some(g) => body.extend(g.encode(m)),
                None => body.extend_from_slice(m),
            }
        }
        if let Some(g5) = self.gen5.as_mut() {
            let comp = crate::protocol::choose_comp(body.len());
            let compressed = match comp {
                crate::protocol::Gen5Comp::Bz2 => crate::protocol::bz2_compress(&body)?,
                crate::protocol::Gen5Comp::Zlib => crate::protocol::zlib_compress(&body)?,
                crate::protocol::Gen5Comp::None => body.clone(),
            };
            let encoded = g5.encode(compressed, comp);
            let frame = crate::protocol::write_frame(comp, &encoded);
            self.send_raw(&frame)?;
        } else {
            let frame = write_zlib_frame(&body)?;
            self.send_raw(&frame)?;
        }
        self.stream.flush()
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
    /// `gen5` selects the gen-5 frame reader (ctype + XOR + decompress).
    /// Otherwise plain zlib is used (2.05).
    pub fn spawn_reader(&self, gen5: bool, key: u8) -> io::Result<Receiver<io::Result<Vec<u8>>>> {
        let mut stream = self.stream.try_clone()?;
        stream.set_read_timeout(None)?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut cipher = crate::protocol::Gen5::new(key);
            loop {
                let frame = if gen5 {
                    crate::protocol::read_frame(&mut stream, &mut cipher)
                } else {
                    read_zlib_frame(&mut stream)
                };
                let stop = frame.is_err();
                if tx.send(frame).is_err() || stop {
                    break;
                }
            }
        });
        Ok(rx)
    }

    /// Read one zlib frame and apply its messages to `state`.
    ///
    /// Reads `u16be` length, decompresses the body, then calls
    /// `SessionState::apply_body`. Returns the decompressed body.
    pub fn read_frame_into(&mut self, state: &mut SessionState) -> io::Result<Vec<u8>> {
        let body = read_zlib_frame(&mut self.stream)?;
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

    /// Send a chat message as `PLI_TOALL` (id 6).
    ///
    /// Wire: `GCHAR(6)` + text + `\n`. The text is sent as-is.
    pub fn send_chat(&mut self, text: &str) -> io::Result<()> {
        let mut msg = Vec::new();
        msg.push(6u8.wrapping_add(0x20));
        msg.extend_from_slice(text.as_bytes());
        msg.push(b'\n');
        self.send_raw(&msg)
    }

    /// Request a file from the server (`PLI_WANTFILE`, id 23).
    pub fn send_wantfile(&mut self, name: &str) -> io::Result<()> {
        let mut msg = Vec::new();
        msg.push(23u8.wrapping_add(0x20));
        msg.extend_from_slice(name.as_bytes());
        msg.push(b'\n');
        self.send_messages(&[msg])
    }
    ///
    /// Wire: `GCHAR(28)` + target + `:` + text + `\n`.
    /// Layout is provisional; the id is documented.
    pub fn send_pm(&mut self, target: &str, text: &str) -> io::Result<()> {
        let mut msg = Vec::new();
        msg.push(28u8.wrapping_add(0x20));
        msg.extend_from_slice(target.as_bytes());
        msg.push(b':');
        msg.extend_from_slice(text.as_bytes());
        msg.push(b'\n');
        self.send_raw(&msg)
    }
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
}
