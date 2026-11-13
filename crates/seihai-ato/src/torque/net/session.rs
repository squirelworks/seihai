//! Runs the Torque UDP link of a Graal3D session over a socket.
//!
//! Started at the same moment as the TCP login (§1 of the clean-room
//! note): it keeps repeating the challenge and the connect request until
//! the server accepts (the server waits for the TCP login to finish),
//! then exchanges data packets, answers the mission start, keeps the
//! ghost table and reports the login steps. It ends when the server
//! disconnects or goes silent, or when the caller sets `stop`, in which
//! case it sends a disconnect first.

use super::events::{mission_start_answer, ClassTable, Events};
use super::ghosts::{ClientGame, GameObjectUpdate, GhostChange, GhostState, Ghosts, Transform};
use super::handshake::{self, ConnectRequest, ServerPacket};
use super::link::{Link, Rate};
use crate::torque::TorqueProfile;
use std::io::{ErrorKind, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How often the handshake packets are repeated.
const RESEND: Duration = Duration::from_millis(500);
/// How long the handshake may take before giving up, as the 5.12 client.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// With nothing to say, still send a data packet this often.
const KEEP_ALIVE: Duration = Duration::from_secs(1);
/// Ping when nothing has been acknowledged for this long.
const PING_AFTER: Duration = Duration::from_millis(4600);
/// The link is dead after this long without a packet.
const SILENCE: Duration = Duration::from_secs(30);
/// The largest packet the client sends.
const MAX_PACKET: usize = 450;
/// How long to wait when no datagram is queued.
const IDLE_NAP: Duration = Duration::from_millis(5);

/// Who the client says it is on the UDP link.
#[derive(Debug, Clone)]
pub struct Identity {
    /// The account name; a guest sends `guest`.
    pub account: String,
    pub password: String,
    /// May be empty.
    pub nickname: String,
}

/// How the link ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// The caller stopped it (its TCP side closed).
    Stopped,
    Rejected(String),
    Disconnected(String),
    /// No accept within the connect timeout.
    NoAccept,
    /// The server went silent.
    TimedOut,
}

/// The login steps the 5.12 client reports (`onGraal3DLoginStep`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    Established = 1,
    Phase1 = 2,
    Phase2 = 3,
    Lit = 4,
    Control = 5,
}

/// Runs the link from `socket` (already bound) to `server` until it
/// ends. Progress lines go to `out` with a `3d:` prefix.
pub fn run(
    socket: &UdpSocket,
    server: SocketAddr,
    profile: TorqueProfile,
    who: &Identity,
    stop: &AtomicBool,
    out: &mut impl Write,
) -> Result<Ended, String> {
    run_with_sequence(socket, server, profile, who, stop, out, connect_sequence())
}

/// As [`run`], with the connect sequence given (for tests and replays).
pub fn run_with_sequence(
    socket: &UdpSocket,
    server: SocketAddr,
    profile: TorqueProfile,
    who: &Identity,
    stop: &AtomicBool,
    out: &mut impl Write,
    sequence: u32,
) -> Result<Ended, String> {
    if profile.tge_game_layer {
        return Err(format!(
            "the 3D side of {} (protocol {}) uses the Torque game layer, which Ato does not run yet",
            profile.builds, profile.protocol
        ));
    }
    // Non-blocking, not a read timeout: on Windows a timed-out
    // recv_from can drop a datagram that arrives as it gives up (lab 13b
    // lost 52 of 129 that way).
    socket.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 2048];

    // Challenge, then connect request, each repeated until answered.
    let started = Instant::now();
    let mut digest = None;
    let mut last_send = None::<Instant>;
    let accepted = loop {
        if stop.load(Ordering::Relaxed) {
            return Ok(Ended::Stopped);
        }
        if started.elapsed() > CONNECT_TIMEOUT {
            let _ = writeln!(
                out,
                "3d: The connection to the Graal3D server couldn't be established."
            );
            return Ok(Ended::NoAccept);
        }
        if last_send.is_none_or(|t| t.elapsed() >= RESEND) {
            let packet = match digest {
                None => handshake::challenge_request(sequence),
                Some(digest) => ConnectRequest {
                    sequence,
                    digest,
                    profile,
                    password: who.password.as_bytes(),
                    args: &[who.account.as_bytes(), who.nickname.as_bytes()],
                }
                .encode(),
            };
            socket.send_to(&packet, server).map_err(|e| e.to_string())?;
            last_send = Some(Instant::now());
        }
        let Some(n) = recv(socket, server, &mut buf)? else {
            continue;
        };
        match handshake::read_server_packet(&buf[..n]) {
            Ok(p) if p.sequence() != sequence => {}
            Ok(ServerPacket::ChallengeResponse { digest: d, .. }) => {
                if digest.is_none() {
                    digest = Some(d);
                    last_send = None;
                }
            }
            Ok(ServerPacket::ConnectAccept {
                classes, protocol, ..
            }) => {
                if protocol != profile.protocol {
                    return Ok(Ended::Rejected(format!(
                        "the server runs protocol {protocol}"
                    )));
                }
                break classes;
            }
            Ok(ServerPacket::ConnectReject { reason, .. }) => {
                return Ok(Ended::Rejected(
                    String::from_utf8_lossy(&reason).into_owned(),
                ));
            }
            Ok(ServerPacket::Disconnect { reason, .. }) => {
                return Ok(Ended::Disconnected(
                    String::from_utf8_lossy(&reason).into_owned(),
                ));
            }
            _ => {}
        }
    };
    let _ = writeln!(
        out,
        "3d: Connection established ({} classes)",
        accepted.len()
    );
    let event_type = if profile.class_table_types == 3 { 2 } else { 1 };
    let classes = ClassTable::new(&accepted, event_type);

    let mut link = Link::new(sequence);
    let mut events = Events::new();
    let mut ghosts = Ghosts::new();
    let mut game = ClientGame::before_control();
    let mut max_rate = Some(Rate::CLIENT_512_MAX);
    let mut last_data = None::<Instant>;
    let mut last_heard = Instant::now();
    let mut last_acked = Instant::now();
    let mut step = Step::Established;
    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = socket.send_to(&handshake::disconnect(sequence, b""), server);
            return Ok(Ended::Stopped);
        }
        if last_heard.elapsed() > SILENCE {
            let _ = writeln!(out, "3d: The 3D server connection has timed out");
            return Ok(Ended::TimedOut);
        }
        let due = last_data.is_none_or(|t| t.elapsed() >= KEEP_ALIVE) || events.has_pending();
        if due && !link.window_full() {
            if let Some(mut w) = link.data(None, max_rate.take()) {
                game.write(&mut w);
                if game.update.is_some() {
                    // After the echo, every packet carries the empty update.
                    game.update = Some(GameObjectUpdate::default());
                }
                events.write(&mut w, link.last_sent(), &classes, MAX_PACKET * 8 - 1);
                socket
                    .send_to(w.bytes(), server)
                    .map_err(|e| e.to_string())?;
                last_data = Some(Instant::now());
            }
        } else if last_acked.elapsed() > PING_AFTER {
            socket
                .send_to(&link.ping(), server)
                .map_err(|e| e.to_string())?;
            last_acked = Instant::now();
        }

        let Some(n) = recv(socket, server, &mut buf)? else {
            continue;
        };
        last_heard = Instant::now();
        if !handshake::is_data_packet(&buf[..n]) {
            if let Ok(ServerPacket::Disconnect {
                sequence: s,
                reason,
            }) = handshake::read_server_packet(&buf[..n])
            {
                if s == sequence {
                    let why = String::from_utf8_lossy(&reason).into_owned();
                    let _ = writeln!(out, "3d: disconnected: {why}");
                    return Ok(Ended::Disconnected(why));
                }
            }
            continue;
        }
        let Ok(got) = link.receive(&buf[..n]) else {
            continue;
        };
        if got.notifies.iter().any(|n| n.delivered) {
            last_acked = Instant::now();
        }
        for note in &got.notifies {
            events.notify(*note);
        }
        // As the real 5.12 client: answer pings, and ack every new data
        // packet at once with an ack packet.
        if got.answer_with_ack || got.data.is_some() {
            socket
                .send_to(&link.ack(), server)
                .map_err(|e| e.to_string())?;
        }
        let Some(mut data) = got.data else { continue };
        let server_game = match ghosts.read_game(&mut data.body) {
            Ok(g) => g,
            Err(e) => {
                let _ = writeln!(out, "3d: packet {}: game section: {e:?}", data.sequence);
                continue;
            }
        };
        let delivered = match events.read(&mut data.body, &classes) {
            Ok(evs) => evs,
            Err(e) => {
                let _ = writeln!(out, "3d: packet {}: events: {e:?}", data.sequence);
                continue;
            }
        };
        for ev in &delivered {
            let Some(cmd) = events.command(ev) else {
                continue;
            };
            let name = String::from_utf8_lossy(&cmd.name).into_owned();
            if let Some(answer) = mission_start_answer(&cmd.name) {
                let next = if answer == b"MissionStartPhase1Ack" {
                    Step::Phase1
                } else {
                    Step::Phase2
                };
                let _ = writeln!(out, "3d: {name}");
                events.send_command(answer, &[]);
                step = next;
                if next == Step::Phase2 {
                    // Nothing to light yet: the scene is lit at once.
                    step = Step::Lit;
                    let _ = writeln!(out, "3d: Mission lighting done");
                }
            } else {
                let _ = writeln!(out, "3d: command {name}");
            }
        }
        match ghosts.read_ghosts(&mut data.body, &classes) {
            Ok(changes) => {
                for c in changes {
                    if let GhostChange::Created(i) = c {
                        let _ = writeln!(out, "3d: ghost {i} {}", describe(&ghosts, i));
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(out, "3d: packet {}: ghosts: {e:?}", data.sequence);
            }
        }
        if let Some(i) = server_game.control {
            if step < Step::Control {
                step = Step::Control;
                let _ = writeln!(
                    out,
                    "3d: control object is ghost {i} {}",
                    describe(&ghosts, i)
                );
            }
            if game.control != Some(i) {
                game.control = Some(i);
                if let Some(GhostState::GameObject(me)) = ghosts.ghosts.get(&i) {
                    game.update = Some(control_echo(me));
                    if let Some(t) = me.transform {
                        game.position = t.position;
                    }
                }
            }
        }
    }
}

/// The client's first update of a new control object, as the real 5.12
/// sends it: its shape, transform and flags back, with a zero rotation
/// written as axis (1, 0, 0), angle 0.
pub fn control_echo(me: &GameObjectUpdate) -> GameObjectUpdate {
    let transform = me.transform.map(|t| {
        if t.angle == 0.0 {
            Transform {
                axis: [1.0, 0.0, 0.0],
                ..t
            }
        } else {
            t
        }
    });
    GameObjectUpdate {
        shape: me.shape.clone(),
        transform,
        flags: me.flags,
        ..Default::default()
    }
}

/// The class and name of ghost `i`, for the log.
fn describe(ghosts: &Ghosts, i: u32) -> String {
    match ghosts.ghosts.get(&i) {
        Some(GhostState::GameObject(g)) => {
            format!(
                "GameObject {}",
                String::from_utf8_lossy(g.name.as_deref().unwrap_or_default())
            )
        }
        Some(GhostState::Camera(_)) => "Camera".into(),
        Some(GhostState::Sun { .. }) => "Sun".into(),
        Some(GhostState::Sky(_)) => "Sky".into(),
        Some(GhostState::Other(name)) => name.clone().unwrap_or_else(|| "(unknown class)".into()),
        None => "(gone)".into(),
    }
}

/// One packet from `server`, or `None` when none is queued (after a short
/// nap, so the loop does not spin). Packets from any other address are
/// ignored.
fn recv(socket: &UdpSocket, server: SocketAddr, buf: &mut [u8]) -> Result<Option<usize>, String> {
    match socket.recv_from(buf) {
        Ok((n, from)) if from == server => Ok(Some(n)),
        Ok(_) => Ok(None),
        Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            std::thread::sleep(IDLE_NAP);
            Ok(None)
        }
        // A refused send (no server yet) shows up here on some systems.
        Err(e)
            if e.kind() == ErrorKind::ConnectionReset
                || e.kind() == ErrorKind::ConnectionRefused =>
        {
            Ok(None)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// A fresh connect sequence, even or odd, from the clock.
fn connect_sequence() -> u32 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (t.subsec_nanos() ^ t.as_secs() as u32) | 1 << 10
}
