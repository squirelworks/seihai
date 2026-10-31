//! A Graal3D session: the TCP link to the server and the UDP socket that
//! carries position and chat straight to the other players.
//!
//! One [`Session::tick`] is one client frame (the real client's timer is
//! 50 ms): read TCP, read UDP, send the changed props, send UDP position to
//! peers when props 1–5 changed.

use crate::codec::{self, Deframer, Reader};
use crate::props::{self, Differ, Player, Prop};
use crate::xfile::XFile;
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The login token of Graal3D beta 2. The server refuses `GR3DV001`.
pub const TOKEN: &[u8; 8] = b"GR3DV002";
/// The shipped server's port.
pub const DEFAULT_PORT: u16 = 14901;
/// The real client's frame timer.
pub const TICK: Duration = Duration::from_millis(50);

// Server → client ids.
const SV_LEVEL: u8 = 0;
const SV_LEVELNAME: u8 = 1;
const SV_OTHERPLPROPS: u8 = 2;
const SV_PLAYERPROPS: u8 = 3;
const SV_LEADER: u8 = 4;
const SV_DISCMESSAGE: u8 = 5;
const SV_SESSION: u8 = 6;
const SV_ADMINMESSAGE: u8 = 8;
const SV_RAWDATA: u8 = 100;

// Client → server ids.
const CL_LOGIN: u8 = 0;
const CL_PROPS: u8 = 1;

// UDP datagram kinds.
const UDP_POSITION: u8 = 0;
const UDP_CHAT: u8 = 1;

/// Something the session saw, for the log or the view.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The server's first message after accept (packet 6).
    Accepted,
    /// Own props arrived (packet 3); the first one carries the player id.
    OwnProps {
        id: u8,
        props: Vec<Prop>,
    },
    /// A level was sent: its name, and the file's size, mesh counts and
    /// the path it was saved to. `None` when the server said "you already
    /// have it".
    Level {
        name: String,
        file: Option<LevelFile>,
    },
    /// Packet 4: first player in the level.
    Leader,
    /// Another player was seen for the first time.
    PlayerJoined {
        id: u8,
    },
    PlayerProps {
        id: u8,
        props: Vec<Prop>,
    },
    PlayerLeft {
        id: u8,
    },
    /// A chat line from another player, over TCP or UDP.
    Chat {
        id: u8,
        nick: String,
        text: String,
        udp: bool,
    },
    /// Position from another player over UDP.
    UdpPosition {
        id: u8,
    },
    AdminMessage(String),
    /// The server's disconnect text (packet 5); the socket closes after.
    DiscMessage(String),
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelFile {
    pub bytes: usize,
    pub path: Option<PathBuf>,
    pub vertices: usize,
    pub faces: usize,
    pub textures: Vec<String>,
    /// Why the mesh could not be read, if it could not.
    pub error: Option<String>,
}

/// What the client logs in with.
#[derive(Debug, Clone, Default)]
pub struct Login {
    pub account: String,
    pub password: String,
}

pub struct Session {
    tcp: TcpStream,
    udp: Option<UdpSocket>,
    deframer: Deframer,
    differ: Differ,
    last_position: Vec<u8>,
    raw_next: bool,
    closed: bool,
    /// Where level files are written; `None` keeps them in memory only.
    level_dir: Option<PathBuf>,
    pub me: Player,
    pub others: BTreeMap<u8, Player>,
    pub level_name: String,
    /// The last level file received (text, `\n` line ends).
    pub level_text: Option<Vec<u8>>,
}

impl Session {
    /// Connect, bind UDP (unless `udp` is false) and send the login with
    /// the first props line in one frame, as the real client does.
    pub fn connect(
        addr: &str,
        login: &Login,
        mut me: Player,
        udp: bool,
        level_dir: Option<PathBuf>,
    ) -> io::Result<Session> {
        let target = addr.to_socket_addrs()?.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("no address for {addr}"))
        })?;
        let tcp = TcpStream::connect_timeout(&target, Duration::from_secs(10))?;
        tcp.set_nodelay(true)?;
        tcp.set_read_timeout(Some(TICK))?;
        let udp = if udp {
            let bind: SocketAddr = match target.ip() {
                IpAddr::V4(_) => (Ipv4Addr::UNSPECIFIED, 0).into(),
                IpAddr::V6(_) => (std::net::Ipv6Addr::UNSPECIFIED, 0).into(),
            };
            let s = UdpSocket::bind(bind)?;
            s.set_nonblocking(true)?;
            me.udp_port = s.local_addr()?.port() as u32;
            Some(s)
        } else {
            me.udp_port = 0;
            None
        };
        let mut s = Session {
            tcp,
            udp,
            deframer: Deframer::default(),
            differ: Differ::new(),
            last_position: me.position_values(),
            raw_next: false,
            closed: false,
            level_dir,
            me,
            others: BTreeMap::new(),
            level_name: String::new(),
            level_text: None,
        };
        let mut login_line = TOKEN.to_vec();
        codec::put_str(&mut login_line, login.account.as_bytes());
        codec::put_str(&mut login_line, login.password.as_bytes());
        let mut msgs = vec![codec::message(CL_LOGIN, &login_line)];
        if let Some(d) = s.differ.diff(&s.me) {
            msgs.push(codec::message(CL_PROPS, &d));
        }
        s.tcp.write_all(&codec::frame(&msgs)?)?;
        Ok(s)
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Set a chat line: it goes to UDP peers at once and to the server in
    /// the next props diff.
    pub fn say(&mut self, text: &str) -> io::Result<()> {
        self.me.chat = text.to_string();
        let mut d = vec![codec::c(UDP_CHAT)];
        d.extend_from_slice(&props::cut_text(text));
        self.send_udp(&d)
    }

    /// The other players this client sends UDP to: in the same level, with
    /// a UDP port and an address.
    pub fn peers(&self) -> Vec<SocketAddr> {
        if self.udp.is_none() {
            return Vec::new();
        }
        self.others
            .values()
            .filter(|p| p.in_level && p.udp_port != 0)
            .filter_map(|p| Some(SocketAddr::from((p.address?, p.udp_port as u16))))
            .collect()
    }

    fn send_udp(&self, d: &[u8]) -> io::Result<()> {
        if let Some(u) = &self.udp {
            for peer in self.peers() {
                match u.send_to(d, peer) {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// One frame: receive, then send what changed.
    pub fn tick(&mut self) -> io::Result<Vec<Event>> {
        let mut ev = Vec::new();
        if self.closed {
            return Ok(ev);
        }
        let mut buf = [0u8; 16384];
        loop {
            match self.tcp.read(&mut buf) {
                Ok(0) => {
                    self.closed = true;
                    break;
                }
                Ok(n) => {
                    self.deframer.push(&buf[..n]);
                    if n < buf.len() {
                        break;
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break
                }
                Err(e) => return Err(e),
            }
        }
        for (id, payload) in self.deframer.messages()? {
            self.handle(id, &payload, &mut ev);
        }
        self.read_udp(&mut ev);
        if self.closed {
            ev.push(Event::Closed);
            return Ok(ev);
        }
        if let Some(d) = self.differ.diff(&self.me) {
            self.tcp
                .write_all(&codec::frame(&[codec::message(CL_PROPS, &d)])?)?;
        }
        let pos = self.me.position_values();
        if pos != self.last_position {
            let mut d = vec![codec::c(UDP_POSITION)];
            d.extend_from_slice(&pos);
            self.send_udp(&d)?;
            self.last_position = pos;
        }
        Ok(ev)
    }

    fn handle(&mut self, id: u8, payload: &[u8], ev: &mut Vec<Event>) {
        if self.raw_next {
            // The client flags the line after RAWDATA as raw and drops it.
            self.raw_next = false;
            return;
        }
        match id {
            SV_SESSION => ev.push(Event::Accepted),
            SV_PLAYERPROPS => {
                let p = props::read_all(payload);
                p.iter().for_each(|x| self.me.apply(x));
                // Assumed: the client doesn't send back what the server
                // just set (the lab client's movement routine was patched
                // out, so this is not captured).
                self.differ.mark_sent(&self.me, p.iter().map(Prop::id));
                if p.iter().any(|x| (1..=5).contains(&x.id())) {
                    self.last_position = self.me.position_values();
                }
                ev.push(Event::OwnProps {
                    id: self.me.id,
                    props: p,
                });
            }
            SV_LEVELNAME => self.level_name = String::from_utf8_lossy(payload).into_owned(),
            SV_LEVEL => {
                let name = self.level_name.clone();
                if payload.is_empty() {
                    ev.push(Event::Level { name, file: None });
                } else {
                    let text = codec::unescape_lines(payload);
                    let file = self.store_level(&name, &text);
                    self.level_text = Some(text);
                    ev.push(Event::Level {
                        name,
                        file: Some(file),
                    });
                }
            }
            SV_OTHERPLPROPS => {
                let mut r = Reader::new(payload);
                let Some(pid) = r.c() else { return };
                let p = props::read_all(r.rest());
                if p.contains(&Prop::Left) {
                    self.others.remove(&pid);
                    ev.push(Event::PlayerLeft { id: pid });
                    return;
                }
                let player = self.others.entry(pid).or_insert_with(|| {
                    ev.push(Event::PlayerJoined { id: pid });
                    Player {
                        id: pid,
                        ..Player::default()
                    }
                });
                for x in &p {
                    player.apply(x);
                    if let Prop::Chat(t) = x {
                        if !t.is_empty() {
                            ev.push(Event::Chat {
                                id: pid,
                                nick: player.nick.clone(),
                                text: t.clone(),
                                udp: false,
                            });
                        }
                    }
                }
                ev.push(Event::PlayerProps { id: pid, props: p });
            }
            SV_LEADER => ev.push(Event::Leader),
            SV_DISCMESSAGE => ev.push(Event::DiscMessage(latin1(payload))),
            SV_ADMINMESSAGE => {
                ev.push(Event::AdminMessage(latin1(&codec::unescape_lines(payload))))
            }
            SV_RAWDATA => self.raw_next = true,
            _ => {}
        }
    }

    fn read_udp(&mut self, ev: &mut Vec<Event>) {
        let Some(u) = &self.udp else { return };
        let mut buf = [0u8; 512];
        while let Ok((n, from)) = u.recv_from(&mut buf) {
            let IpAddr::V4(ip) = from.ip() else { continue };
            // Only from a known player's address and port.
            let Some(p) = self
                .others
                .values_mut()
                .find(|p| p.address == Some(ip.octets()) && p.udp_port == from.port() as u32)
            else {
                continue;
            };
            let d = &buf[..n];
            match d.first().map(|&b| codec::uc(b)) {
                Some(UDP_POSITION) => {
                    if p.apply_position_values(&d[1..]) {
                        ev.push(Event::UdpPosition { id: p.id });
                    }
                }
                Some(UDP_CHAT) => {
                    let text = latin1(&d[1..d.len().min(201)]);
                    p.chat = text.clone();
                    ev.push(Event::Chat {
                        id: p.id,
                        nick: p.nick.clone(),
                        text,
                        udp: true,
                    });
                }
                _ => {}
            }
        }
    }

    fn store_level(&self, name: &str, text: &[u8]) -> LevelFile {
        let mut f = LevelFile {
            bytes: text.len(),
            path: None,
            vertices: 0,
            faces: 0,
            textures: Vec::new(),
            error: None,
        };
        match XFile::parse(text) {
            Ok(x) => {
                f.vertices = x.vertex_count();
                f.faces = x.face_count();
                f.textures = x.textures();
            }
            Err(e) => f.error = Some(e.to_string()),
        }
        if let Some(dir) = &self.level_dir {
            let path = dir.join(safe_file_name(name));
            match std::fs::create_dir_all(dir).and_then(|_| std::fs::write(&path, text)) {
                Ok(()) => f.path = Some(path),
                Err(e) => f.error = Some(format!("could not save: {e}")),
            }
        }
        f
    }

    /// The saved level, if one was written.
    pub fn level_path(&self) -> Option<PathBuf> {
        self.level_dir
            .as_ref()
            .map(|d| d.join(safe_file_name(&self.level_name)))
    }
}

fn latin1(b: &[u8]) -> String {
    b.iter().map(|&x| x as char).collect()
}

/// A level name from the server as a plain file name in the level folder:
/// no folders, no leading dot. The real client wrote whatever it got next
/// to its exe; Seihai doesn't trust the server with paths.
pub fn safe_file_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    let clean = clean.trim_start_matches('.');
    if clean.is_empty() {
        "level.x".into()
    } else {
        clean.to_string()
    }
}

/// `host` or `host:port`, with the shipped port when none is given.
pub fn with_default_port(addr: &str) -> String {
    if addr
        .rsplit_once(':')
        .is_some_and(|(_, p)| p.parse::<u16>().is_ok())
    {
        addr.to_string()
    } else {
        format!("{addr}:{DEFAULT_PORT}")
    }
}

/// Where levels are saved: `levels/` in the asset folder, else a folder in
/// the system temp directory.
pub fn default_level_dir(assets: Option<&Path>) -> PathBuf {
    match assets {
        Some(a) => a.join("levels"),
        None => std::env::temp_dir().join("seihai-g3d-levels"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn file_names() {
        assert_eq!(safe_file_name("startlevel.x"), "startlevel.x");
        assert_eq!(safe_file_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_file_name("..\\x\\.hidden"), "hidden");
        assert_eq!(safe_file_name(""), "level.x");
        assert_eq!(with_default_port("example.test"), "example.test:14901");
        assert_eq!(
            with_default_port("example.test:15000"),
            "example.test:15000"
        );
    }

    /// A scripted server that replays the original server's login replies
    /// (from the lab capture) and checks what the client sends.
    #[test]
    fn login_against_a_scripted_server() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap().to_string();
        let server = std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            c.write_all(&codec::frame(&[codec::message(6, b" ")]).unwrap())
                .unwrap();
            let mut d = Deframer::default();
            let mut got = Vec::new();
            let mut buf = [0u8; 4096];
            while got.len() < 2 {
                let n = c.read(&mut buf).unwrap();
                d.push(&buf[..n]);
                got.extend(d.messages().unwrap());
            }
            let level = b"xof 0302txt 0064\xa7Mesh obj {\xa7 3;\xa7 0.0;0.0;0.0;,\xa7 1.0;0.0;0.0;,\xa7 0.0;0.0;1.0;;\xa7 1;\xa7 3;0,1,2;;\xa7}\xa7";
            let replies = [
                codec::message(3, b" !!^`\"^`#^`$\xa7%  "),
                codec::message(1, b"startlevel.x"),
                codec::message(0, level),
                codec::message(4, b""),
                codec::message(2, b"\"&&body.x,#Bob/   0 (  \x9fR!"),
                codec::message(2, b"\"-%hi A!"),
                codec::message(2, b"\"S"),
            ];
            c.write_all(&codec::frame(&replies).unwrap()).unwrap();
            got
        });
        let me = Player {
            nick: "Alice".into(),
            ..Player::default()
        };
        let login = Login {
            account: "alice".into(),
            password: "pw1".into(),
        };
        let mut s = Session::connect(&addr, &login, me, false, None).unwrap();
        let mut ev = Vec::new();
        for _ in 0..40 {
            ev.extend(s.tick().unwrap());
            if ev.contains(&Event::PlayerLeft { id: 2 }) {
                break;
            }
        }
        let got = server.join().unwrap();
        assert_eq!(got[0], (0, b"GR3DV002%alice#pw1".to_vec()));
        assert_eq!(
            got[1],
            (
                1,
                b"&&body.x'&head.x((body.bmp)'man.bmp*_P  +@   ,%Alice/   ".to_vec()
            )
        );
        assert_eq!(ev[0], Event::Accepted);
        assert_eq!(s.me.id, 1);
        assert_eq!(s.level_name, "startlevel.x");
        let lv = ev.iter().find_map(|e| match e {
            Event::Level { file: Some(f), .. } => Some(f.clone()),
            _ => None,
        });
        let lv = lv.unwrap();
        assert_eq!((lv.vertices, lv.faces, lv.error), (3, 1, None));
        assert!(ev.contains(&Event::Leader));
        assert!(ev.contains(&Event::PlayerJoined { id: 2 }));
        assert!(ev.contains(&Event::Chat {
            id: 2,
            nick: "Bob".into(),
            text: "hi A!".into(),
            udp: false
        }));
        assert!(ev.contains(&Event::PlayerLeft { id: 2 }));
    }
}
