//! Networking.
//!
//! Connection and high-level send helpers. Packet formats are provisional
//! and will be adjusted as the decompile reveals the real layouts.

use crate::protocol::{LevelRequest, LoginRequest, PlayerUpdate};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// TCP connection to a listserver or gserver.
pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    /// Connect to `host:port`.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { stream })
    }

    /// Send a raw byte buffer.
    pub fn send_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data)
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

    /// Send a short walk frame for the given update.
    pub fn send_walk(&mut self, upd: &PlayerUpdate) -> io::Result<()> {
        let mut buf = Vec::new();
        upd.write_walk(&mut buf)?;
        self.send_raw(&buf)
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

