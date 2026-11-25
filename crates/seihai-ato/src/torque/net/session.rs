//! Runs the Torque UDP link of a Graal3D session over a socket.
//!
//! Started at the same moment as the TCP login (§1 of the clean-room
//! note): it keeps repeating the challenge and the connect request until
//! the server accepts (the server waits for the TCP login to finish),
//! then exchanges data packets, answers the mission start, keeps the
//! ghost table and reports the login steps. Protocol 19 (3.153, 4.04r2)
//! runs the stock Torque game layer from [`super::tge`] instead: three
//! mission phases, datablocks, and a game section with moves. It ends when the server
//! disconnects or goes silent, or when the caller sets `stop`, in which
//! case it sends a disconnect first.

use super::events::{mission_start_answer, BigDataAssembler, ClassTable, Event, Events};
use super::ghosts::{
    ClientGame, GameObjectUpdate, GhostChange, GhostState, Ghosts, Sky, Transform,
};
use super::graal_text::{self, Applied, GraalText};
use super::handshake::{self, ConnectRequest, ServerPacket};
use super::link::{Link, Rate};
use super::scripts3d::{vm_version, Keys3d, Weapons3d};
use super::tge::{self, ClientGame19, Control19, Game19Error, Move, Phase, PhaseAcks};
use crate::torque::TorqueProfile;
use crate::view3d::{Shared3d, SharedInput3d};
use seihai_core::client::FileAsks;
use seihai_core::protocol::three_d::ThreeD;
use seihai_core::protocol::ClientVersion;
use std::collections::{HashMap, VecDeque};
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
/// While the own object moves, a packet at most this often (Torque's
/// default packet rate).
const MOVE_PACKET: Duration = Duration::from_millis(32);
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
    /// The client build. On protocol 19 it picks how the mission phases
    /// are acknowledged: 3.153 echoes the phase sequence, 4.04r2 does not.
    pub version: ClientVersion,
}

/// What the link shares with the rest of the client, each part optional.
#[derive(Clone, Copy, Default)]
pub struct Shared<'a> {
    /// The TCP session's file requests (shapes and Sky textures).
    pub files: Option<&'a FileAsks>,
    /// The world the window draws.
    pub world: Option<&'a Shared3d>,
    /// The keys and chat the window hands over (walk and chat, #462).
    pub input: Option<&'a SharedInput3d>,
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
    /// Every TCP file request the 3D world caused is answered.
    Attached = 6,
}

/// Runs the link from `socket` (already bound) to `server` until it
/// ends. Progress lines go to `out` with a `3d:` prefix. The files the
/// ghosts name are asked for through `files`, and the view's `world` is
/// kept up to date.
#[allow(clippy::too_many_arguments)]
pub fn run(
    socket: &UdpSocket,
    server: SocketAddr,
    profile: TorqueProfile,
    who: &Identity,
    stop: &AtomicBool,
    with: Shared<'_>,
    out: &mut impl Write,
) -> Result<Ended, String> {
    run_with_sequence(
        socket,
        server,
        profile,
        who,
        stop,
        with,
        out,
        connect_sequence(),
    )
}

/// As [`run`], with the connect sequence given (for tests and replays).
#[allow(clippy::too_many_arguments)]
pub fn run_with_sequence(
    socket: &UdpSocket,
    server: SocketAddr,
    profile: TorqueProfile,
    who: &Identity,
    stop: &AtomicBool,
    with: Shared<'_>,
    out: &mut impl Write,
    sequence: u32,
) -> Result<Ended, String> {
    let Shared {
        files,
        world,
        input,
    } = with;
    let tge_layer = profile.tge_game_layer;
    // A build that runs TorqueScript (3.153) acknowledges the mission
    // phases from its scripts; 4.04r2, which has no compiler, natively.
    let three_d = crate::torque::capabilities(who.version);
    let phase_acks = if three_d.is_some_and(|t| t.runs_torque_script()) {
        PhaseAcks::Scripted
    } else {
        PhaseAcks::Native
    };
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
    if tge_layer {
        let _ = writeln!(
            out,
            "3d: protocol {} game layer: static reads only, never run against a real client's server",
            profile.protocol
        );
    }
    let mut ghosts = if tge_layer {
        Ghosts::opaque()
    } else {
        Ghosts::new()
    };
    let mut game = ClientGame::before_control();
    let mut game19 = ClientGame19::before_control();
    let mut datablocks = 0u32;
    // Protocol 19 moves not yet taken by the server, the first numbered
    // `first_move19`.
    let mut moves19: VecDeque<Move> = VecDeque::new();
    let mut first_move19 = 0u32;
    let mut walking19 = false;
    let mut max_rate = Some(Rate::CLIENT_512_MAX);
    let mut last_data = None::<Instant>;
    let mut last_heard = Instant::now();
    let mut last_acked = Instant::now();
    let mut step = Step::Established;
    let mut fetch = Fetch::for_build(three_d);
    let mut big_data = BigDataAssembler::default();
    // The own object as this client moves it, and the sections it changed
    // since the last packet (#462).
    let mut me_at: Option<crate::view3d::Transform> = None;
    let mut own = GameObjectUpdate::default();
    let mut last_move = Instant::now();
    let mut text = GraalText::default();
    // The weapons' scripts, and the id of the next `events` text they send.
    let mut weapons = Weapons3d::new(vm_version(profile.protocol));
    let mut next_transfer: u32 = 0;
    let mut last_tick = Instant::now();
    loop {
        let ms = last_tick.elapsed().as_millis() as u64;
        if ms > 0 {
            last_tick += Duration::from_millis(ms);
            // The scripts' `player` is the own object once it is the
            // control object, and their `keydown2` the window's arrows.
            weapons.me = game.control.and(me_at);
            weapons.keys = match input {
                Some(i) => {
                    let i = i.lock().unwrap_or_else(|e| e.into_inner());
                    Keys3d {
                        forward: i.forward,
                        turn: i.turn,
                    }
                }
                None => Keys3d::default(),
            };
            weapons.tick(ms.min(1000));
            let change = weapons.take_player();
            if let Some(t) = change.transform {
                me_at = Some(t);
                game.position = t.position;
                own.transform = Some(Transform {
                    position: t.position,
                    axis: t.axis,
                    angle: t.angle,
                });
                if let (Some(w), Some(i)) = (world, game.control) {
                    let mut w = w.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(o) = w.objects.get_mut(&i) {
                        o.transform = Some(t);
                    }
                }
            }
            if let Some(line) = change.chat {
                let _ = writeln!(out, "3d: a script says {line}");
                if let (Some(w), Some(i)) = (world, game.control) {
                    w.lock().unwrap_or_else(|e| e.into_inner()).labels_of(
                        i,
                        None,
                        Some(line.as_bytes()),
                    );
                }
                own.chat = Some(line.into_bytes());
            }
        }
        for line in weapons.log.drain(..) {
            let _ = writeln!(out, "3d: {line}");
        }
        if !weapons.events.is_empty() {
            for ev in weapons.events.take(next_transfer) {
                events.send(ev);
            }
            next_transfer = (next_transfer + 1) & 0xFFFF;
        }
        if let Some(f) = files {
            fetch.take_answers(f, world, out);
        }
        if files.is_some() && step == Step::Control && fetch.all_answered() {
            step = Step::Attached;
            let _ = writeln!(
                out,
                "3d: login step 6: every file request answered (files: {}, not on the server: {})",
                fetch.files.len(),
                fetch.files.values().filter(|d| d.is_none()).count()
            );
        }
        if stop.load(Ordering::Relaxed) {
            let _ = socket.send_to(&handshake::disconnect(sequence, b""), server);
            return Ok(Ended::Stopped);
        }
        if last_heard.elapsed() > SILENCE {
            let _ = writeln!(out, "3d: The 3D server connection has timed out");
            return Ok(Ended::TimedOut);
        }
        // The keys and chat from the window move and voice the own object.
        let now = Instant::now();
        let dt = now.duration_since(last_move).as_secs_f32().min(0.1);
        last_move = now;
        if let (Some(inp), Some(_)) = (input, game.control) {
            let (forward, turn, said) = {
                let mut i = inp.lock().unwrap_or_else(|e| e.into_inner());
                (i.forward, i.turn, std::mem::take(&mut i.say))
            };
            // Once a server script walks the player (as on a real 5.12),
            // the keys are the script's, not the client's.
            let walk_here = !weapons.scripted && (forward != 0 || turn != 0);
            if let Some(t) = me_at.filter(|_| walk_here) {
                let t = crate::view3d::step(t, forward, turn, dt);
                me_at = Some(t);
                game.position = t.position;
                own.transform = Some(Transform {
                    position: t.position,
                    axis: t.axis,
                    angle: t.angle,
                });
                if let (Some(w), Some(i)) = (world, game.control) {
                    let mut w = w.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(o) = w.objects.get_mut(&i) {
                        o.transform = Some(t);
                    }
                }
            }
            for line in said {
                let _ = writeln!(out, "3d: say {line}");
                if let (Some(w), Some(i)) = (world, game.control) {
                    w.lock().unwrap_or_else(|e| e.into_inner()).labels_of(
                        i,
                        None,
                        Some(line.as_bytes()),
                    );
                }
                own.chat = Some(line.into_bytes());
            }
        }
        // Protocol 19: the keys become moves of the control object.
        if let (true, Some(inp), Some(_)) = (tge_layer, input, game19.control) {
            let forward = inp.lock().unwrap_or_else(|e| e.into_inner()).forward;
            walking19 = forward != 0;
        }
        let moving = own != GameObjectUpdate::default() || walking19;
        let due = last_data.is_none_or(|t| t.elapsed() >= KEEP_ALIVE)
            || events.has_pending()
            || (moving && last_data.is_none_or(|t| t.elapsed() >= MOVE_PACKET));
        if due && !link.window_full() {
            if let Some(mut w) = link.data(None, max_rate.take()) {
                // After the echo has gone, the own changes ride the update
                // every packet carries.
                if moving && game.update.as_ref() == Some(&GameObjectUpdate::default()) {
                    game.update = Some(std::mem::take(&mut own));
                }
                if tge_layer {
                    if game19.control.is_some() && moves19.len() < tge::MAX_MOVES {
                        let forward = match input {
                            Some(i) => i.lock().unwrap_or_else(|e| e.into_inner()).forward,
                            None => 0,
                        };
                        moves19.push_back(Move::walk(forward, 0));
                    }
                    game19.first_move = first_move19;
                    game19.moves = moves19.iter().copied().collect();
                    game19.write(&mut w);
                } else {
                    game.write(&mut w);
                }
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
        let server_game = if tge_layer {
            let (g, rest_readable) = match tge::read_server_game(&mut data.body, &mut ghosts) {
                Ok(g) => (g, true),
                Err(Game19Error::PacketData { index, class, game }) => {
                    let _ = writeln!(
                        out,
                        "3d: packet {}: ghost {index} ({}) packet data not read yet; rest of the packet skipped",
                        data.sequence,
                        class.as_deref().unwrap_or("unknown class")
                    );
                    (*game, false)
                }
                Err(e) => {
                    let _ = writeln!(out, "3d: packet {}: game section: {e:?}", data.sequence);
                    continue;
                }
            };
            // The ack is the index after the last move the server took.
            while first_move19 < g.last_move_ack && moves19.pop_front().is_some() {
                first_move19 += 1;
            }
            first_move19 = first_move19.max(g.last_move_ack);
            // A Camera or Player control object places the view; a camera
            // ghost's own packet data wins over it.
            let view = g
                .camera_data
                .as_ref()
                .and_then(|d| d.view())
                .or_else(|| g.control_data.as_ref().and_then(|d| d.view()));
            if let Some((position, _)) = g.control_data.as_ref().and_then(|d| d.view()) {
                game19.position = position;
            }
            if let Some((position, yaw)) = view {
                if let Some(w) = world {
                    w.lock().unwrap_or_else(|e| e.into_inner()).camera =
                        Some(crate::view3d::Transform {
                            position,
                            axis: [0.0, 0.0, 1.0],
                            angle: yaw,
                        });
                }
            }
            if let Control19::Object { index, .. } = g.control {
                game19.control = Some(index);
                if step < Step::Control {
                    step = Step::Control;
                    let _ = writeln!(
                        out,
                        "3d: control object is ghost {index} {}",
                        describe(&ghosts, index)
                    );
                }
                if let Some(w) = world {
                    w.lock().unwrap_or_else(|e| e.into_inner()).control = Some(index);
                }
            }
            if !rest_readable {
                continue;
            }
            super::ghosts::ServerGame::default()
        } else {
            match ghosts.read_game(&mut data.body) {
                Ok(g) => g,
                Err(e) => {
                    let _ = writeln!(out, "3d: packet {}: game section: {e:?}", data.sequence);
                    continue;
                }
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
            // Graal text (flags, weapons, scripts) over BigData.
            if let Some(done) = big_data.feed(ev) {
                let unpacked = match done {
                    Ok(d) => Ok(d),
                    Err((d, compression)) => graal_text::unpack(d, compression),
                };
                match unpacked {
                    Ok(d) => {
                        for a in text.apply(&d) {
                            let _ = writeln!(out, "3d: {}", a.describe());
                            let script = match &a {
                                Applied::WeaponScript { weapon, .. } => {
                                    text.weapons.get(weapon).and_then(|w| w.script.as_deref())
                                }
                                _ => None,
                            };
                            weapons.applied(&a, script, &text.flags);
                        }
                    }
                    Err(e) => {
                        let _ = writeln!(out, "3d: skipped {e}");
                    }
                }
                continue;
            }
            if tge_layer {
                if let Event::SimDataBlock {
                    id,
                    class_id,
                    index,
                    total,
                } = ev
                {
                    datablocks += 1;
                    if index + 1 == *total || datablocks == 1 {
                        let _ = writeln!(
                            out,
                            "3d: datablock {}/{total}: {} id {id}",
                            index + 1,
                            classes.datablock_name(*class_id).unwrap_or("unknown class")
                        );
                    }
                    continue;
                }
                if let Some(answer) = tge::connection_answer(ev) {
                    if let (
                        Event::ConnectionMessage {
                            sequence, message, ..
                        },
                        Event::ConnectionMessage { message: reply, .. },
                    ) = (ev, &answer)
                    {
                        let _ = writeln!(
                            out,
                            "3d: connection message {message} (sequence {sequence}), answered {reply}"
                        );
                    }
                    events.send(answer);
                    continue;
                }
            }
            let Some(cmd) = events.command(ev) else {
                continue;
            };
            let name = String::from_utf8_lossy(&cmd.name).into_owned();
            if tge_layer {
                if let Some((phase, ack, args)) = tge::mission_start(&cmd, phase_acks) {
                    let _ = writeln!(out, "3d: {name}: {}", phase.label());
                    if phase == Phase::Lighting {
                        // Nothing to light yet: lit at once, then acked.
                        let _ =
                            writeln!(out, "3d: Mission lighting done ({datablocks} datablocks)");
                    }
                    events.send_command(&ack, &args);
                    step = step.max(match phase {
                        Phase::Datablocks => Step::Phase1,
                        Phase::Ghosts => Step::Phase2,
                        Phase::Lighting => Step::Lit,
                    });
                } else {
                    let _ = writeln!(out, "3d: command {name}");
                }
                continue;
            }
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
                    let i = match c {
                        GhostChange::Created(i) => {
                            let _ = writeln!(out, "3d: ghost {i} {}", describe(&ghosts, i));
                            i
                        }
                        GhostChange::Updated(i) => i,
                        GhostChange::Deleted(i) => {
                            if let Some(w) = world {
                                w.lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .objects
                                    .remove(&i);
                            }
                            continue;
                        }
                    };
                    let Some(g) = ghosts.ghosts.get(&i) else {
                        continue;
                    };
                    if let Some(w) = world {
                        to_world(&mut w.lock().unwrap_or_else(|e| e.into_inner()), i, g);
                    }
                    if let GhostState::Sky(_) = g {
                        if fetch.sky_textures_in && matches!(c, GhostChange::Updated(_)) {
                            let _ = writeln!(out, "3d: Sky read again with its textures");
                        }
                    }
                    if let Some(f) = files {
                        for name in files_named(g) {
                            fetch.want(&name, f, out);
                        }
                    }
                    if let GhostState::Sky(sky) = g {
                        fetch.sky = sky_textures(sky);
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
                if let Some(w) = world {
                    w.lock().unwrap_or_else(|e| e.into_inner()).control = Some(i);
                }
                if let Some(GhostState::GameObject(me)) = ghosts.ghosts.get(&i) {
                    game.update = Some(control_echo(me));
                    if let Some(t) = me.transform {
                        game.position = t.position;
                        me_at = Some(crate::view3d::Transform {
                            position: t.position,
                            axis: t.axis,
                            angle: t.angle,
                        });
                    }
                }
            }
        }
    }
}

/// The TCP files the 3D world asked for (through core's file session), and
/// what came back.
#[derive(Debug, Default)]
struct Fetch {
    /// Asked for and not answered yet, as asked.
    pending: Vec<String>,
    /// Answered, by lower-case name: the bytes, or `None` when the server
    /// had no such file.
    files: HashMap<String, Option<Vec<u8>>>,
    /// The Sky's textures, as last named.
    sky: Vec<String>,
    /// Every Sky texture is answered; a later Sky update is the re-read.
    sky_textures_in: bool,
    /// For a shape texture asked for under one extension, the files to ask
    /// for next if the server has none (`.gif`, then `.bmp`), by the
    /// lower-case name asked.
    fallbacks: HashMap<String, Vec<String>>,
    /// The build's 3D side, for its file rules: the extensions it never
    /// downloads and how it names a texture. `None` asks for everything
    /// as named.
    three_d: Option<ThreeD>,
}

impl Fetch {
    /// A fetch that follows `three_d`'s file rules.
    fn for_build(three_d: Option<ThreeD>) -> Fetch {
        Fetch {
            three_d,
            ..Fetch::default()
        }
    }

    /// Ask for `name` unless it was asked for already, or the build never
    /// asks the gserver for a file of its kind (`.dso`, `.cs` and `.ter`
    /// from 4.04r2 on: Graal_analysis `torque/404r2-resource-filter.md`).
    fn want(&mut self, name: &str, asks: &FileAsks, out: &mut impl Write) {
        if let Some(t) = self.three_d {
            if !t.requests_download(name) {
                let _ = writeln!(
                    out,
                    "3d: not requesting {name}: this build never downloads it"
                );
                return;
            }
        }
        let key = name.to_ascii_lowercase();
        if self.files.contains_key(&key)
            || self.pending.iter().any(|p| p.eq_ignore_ascii_case(name))
        {
            return;
        }
        self.pending.push(name.to_string());
        asks.wanted
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(name.to_string());
        let _ = writeln!(out, "3d: requesting {name} over TCP");
    }

    /// Ask for a shape's texture: the file it names, or for a bare name
    /// the first of [`crate::view3d::texture_files`], then the next one
    /// each time the server has none.
    ///
    /// A build that strips `.png` (5.12) asks for the bare name first, as
    /// the real client does, then for the name as given.
    fn want_texture(&mut self, name: &str, asks: &FileAsks, out: &mut impl Write) {
        let bare = self.three_d.map_or(name, |t| t.texture_request_name(name));
        if bare != name {
            let base = bare.rsplit(['/', '\\']).next().unwrap_or(bare).to_string();
            let mut rest = crate::view3d::texture_files(name);
            rest.retain(|f| !f.eq_ignore_ascii_case(&base));
            if !rest.is_empty() {
                self.fallbacks.insert(base.to_ascii_lowercase(), rest);
            }
            self.want(&base, asks, out);
            return;
        }
        let mut files = crate::view3d::texture_files(name);
        if files.is_empty() {
            return;
        }
        let first = files.remove(0);
        if !files.is_empty() {
            self.fallbacks.insert(first.to_ascii_lowercase(), files);
        }
        self.want(&first, asks, out);
    }

    /// Take the answers core's session has put in.
    fn take_answers(&mut self, asks: &FileAsks, world: Option<&Shared3d>, out: &mut impl Write) {
        let got = std::mem::take(&mut *asks.answers.lock().unwrap_or_else(|e| e.into_inner()));
        for a in got {
            match &a.data {
                Some(d) => {
                    let _ = writeln!(out, "3d: got {} ({} bytes)", a.name, d.len());
                }
                None => {
                    let _ = writeln!(out, "3d: {} is not on the server", a.name);
                }
            }
            let textures = match world {
                Some(w) => w
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .file(&a.name, a.data.as_deref()),
                None => Vec::new(),
            };
            self.pending.retain(|p| !p.eq_ignore_ascii_case(&a.name));
            let key = a.name.to_ascii_lowercase();
            let missing = a.data.is_none();
            self.files.insert(key.clone(), a.data);
            if let Some(mut rest) = self.fallbacks.remove(&key) {
                if missing && !rest.is_empty() {
                    let next = rest.remove(0);
                    if !rest.is_empty() {
                        self.fallbacks.insert(next.to_ascii_lowercase(), rest);
                    }
                    self.want(&next, asks, out);
                }
            }
            for t in textures {
                self.want_texture(&t, asks, out);
            }
        }
        if !self.sky_textures_in
            && !self.sky.is_empty()
            && self
                .sky
                .iter()
                .all(|t| self.files.contains_key(&t.to_ascii_lowercase()))
        {
            self.sky_textures_in = true;
            let _ = writeln!(out, "3d: the Sky's {} textures are in", self.sky.len());
        }
    }

    fn all_answered(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Put ghost `i`'s state into the view's world.
fn to_world(w: &mut crate::view3d::World3d, i: u32, g: &GhostState) {
    let place = |t: Transform| crate::view3d::Transform {
        position: t.position,
        axis: t.axis,
        angle: t.angle,
    };
    match g {
        GhostState::GameObject(u) => {
            w.object(i, u.shape.as_deref(), u.transform.map(place));
            w.labels_of(
                i,
                u.nickname.as_ref().map(|(n, c)| (n.as_slice(), *c)),
                u.chat.as_deref(),
            );
        }
        GhostState::Camera(u) => {
            if let Some(t) = u.transform {
                w.camera = Some(place(t));
            }
        }
        GhostState::Sun {
            direction,
            colour,
            ambient,
            ..
        } => w.sun = Some((*direction, *colour, *ambient)),
        GhostState::Sky(sky) => {
            if sky.solid_colour.is_some() {
                w.sky_colour = sky.solid_colour;
            }
            if sky.fog_colour.is_some() {
                w.fog_colour = sky.fog_colour;
            }
            if sky.visible_distance.is_some() {
                w.visible_distance = sky.visible_distance;
            }
            if sky.textures.is_some() {
                w.sky_textures = sky_textures(sky);
            }
        }
        GhostState::Other(_) => {}
    }
}

/// The Sky's six textures, as its material list names them.
fn sky_textures(sky: &Sky) -> Vec<String> {
    let Some(list) = sky.textures.as_deref() else {
        return Vec::new();
    };
    String::from_utf8_lossy(list)
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// The files a ghost names that the client fetches over TCP: a GameObject's
/// or Camera's shape, and the Sky's textures (asked for without an
/// extension, as the material list gives them).
fn files_named(g: &GhostState) -> Vec<String> {
    match g {
        GhostState::GameObject(u) | GhostState::Camera(u) => u
            .shape
            .as_deref()
            .map(|s| String::from_utf8_lossy(s).trim().to_string())
            .filter(|s| !s.is_empty())
            .into_iter()
            .collect(),
        GhostState::Sky(sky) => sky_textures(sky),
        _ => Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::client::FileAnswer;

    fn asked(asks: &FileAsks) -> Vec<String> {
        std::mem::take(&mut *asks.wanted.lock().unwrap())
    }

    /// The fetch follows the version profile's download filter: 4.04r2 on
    /// never ask the gserver for `.cs`, `.dso` or `.ter`; 3.153, which runs
    /// TorqueScript, may.
    #[test]
    fn refused_extensions_are_not_requested() {
        for v in [
            ClientVersion::V404R2,
            ClientVersion::V420R6,
            ClientVersion::V512,
        ] {
            let asks = FileAsks::default();
            let mut f = Fetch::for_build(crate::torque::capabilities(v));
            let mut out = Vec::new();
            for name in [
                "main.cs",
                "main.cs.dso",
                "world.ter",
                "tree.dts",
                "house.3ds",
            ] {
                f.want(name, &asks, &mut out);
            }
            assert_eq!(asked(&asks), ["tree.dts", "house.3ds"], "{v:?}");
        }
        let asks = FileAsks::default();
        let mut f = Fetch::for_build(crate::torque::capabilities(ClientVersion::V3153));
        f.want("main.cs", &asks, &mut Vec::new());
        assert_eq!(asked(&asks), ["main.cs"]);
    }

    /// 5.12 strips `.png` and asks for the bare name, as the real client
    /// does; when the server has no bare file Ato falls back to the name
    /// as given. 4.04r2 asks for the name as given.
    #[test]
    fn textures_are_named_per_build() {
        let asks = FileAsks::default();
        let mut f = Fetch::for_build(crate::torque::capabilities(ClientVersion::V512));
        let mut out = Vec::new();
        f.want_texture("grass.png", &asks, &mut out);
        assert_eq!(asked(&asks), ["grass"]);
        asks.answers.lock().unwrap().push(FileAnswer {
            name: "grass".into(),
            data: None,
        });
        f.take_answers(&asks, None, &mut out);
        assert_eq!(asked(&asks), ["grass.png"]);
        // A name without .png is asked for as before.
        f.want_texture("rock.jpg", &asks, &mut out);
        assert_eq!(asked(&asks), ["rock.jpg"]);

        let asks = FileAsks::default();
        let mut f = Fetch::for_build(crate::torque::capabilities(ClientVersion::V404R2));
        f.want_texture("grass.png", &asks, &mut out);
        assert_eq!(asked(&asks), ["grass.png"]);
    }

    /// The phase acknowledgement follows the profile: scripted where the
    /// build runs TorqueScript.
    #[test]
    fn scripted_phase_acks_follow_torque_script() {
        let scripted = |v| crate::torque::capabilities(v).is_some_and(|t| t.runs_torque_script());
        assert!(scripted(ClientVersion::V3153));
        assert!(!scripted(ClientVersion::V404R2));
        assert!(!scripted(ClientVersion::V512));
    }
}
