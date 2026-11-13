//! Ato's protocol 19 game layer (3.153, 4.04r2) against a synthetic
//! loopback server.
//!
//! No real protocol 19 server or client has been traced past the
//! handshake, so this server is written from the static reads (§10 of
//! Graal_analysis `torque/udp-session.md`) and the stock Torque mission
//! download: phase 1 and its ack, the datablocks and `DataBlocksDone`,
//! phase 2 with the ghost-always objects, phase 3 (lighting) and its ack,
//! then the control object. The client must answer every step in order,
//! with 3.153's acks echoing the phase sequence.

#![cfg(not(feature = "era6"))]

use seihai_ato::torque::net::bits::{BitReader, BitWriter};
use seihai_ato::torque::net::events::{Arg, ClassTable, Event, Events};
use seihai_ato::torque::net::handshake::{self, ClassEntry};
use seihai_ato::torque::net::link::Link;
use seihai_ato::torque::net::session::{self, Ended, Identity};
use seihai_ato::torque::net::tge::{self, message, ClientGame19, Control19};
use seihai_ato::torque::GRAAL3D_19;
use seihai_core::protocol::ClientVersion;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const SEQ: u32 = 0x400;

/// What the server got: the commands with their arguments, the
/// connection messages, and the client's log.
type Seen = (Vec<(Vec<u8>, Vec<Arg>)>, Vec<Event>, String);
const MISSION: i32 = 7;

/// Object, datablock and event classes, in the server's numbering.
const CLASSES: &[(u32, u32, &str)] = &[
    (0, 0, "Player"),
    (0, 1, "Item"),
    (0, 2, "Sky"),
    (1, 0, "PlayerData"),
    (1, 1, "ItemData"),
    (2, 0, "NetStringEvent"),
    (2, 1, "RemoteCommandEvent"),
    (2, 2, "ConnectionMessageEvent"),
    (2, 3, "SimDataBlockEvent"),
];

fn accept() -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(handshake::kind::CONNECT_ACCEPT);
    w.u32(SEQ);
    for (t, id, name) in CLASSES {
        w.flag(true);
        w.int(*t, 12);
        w.int(*id, 12);
        w.string(name.as_bytes());
    }
    w.flag(false);
    w.u32(19);
    w.into_bytes()
}

fn table() -> ClassTable {
    let entries: Vec<ClassEntry> = CLASSES
        .iter()
        .map(|(t, id, name)| ClassEntry {
            class_type: *t,
            server_id: *id,
            name: name.as_bytes().to_vec(),
        })
        .collect();
    ClassTable::new(&entries, 2)
}

/// A ghost section creating `(index, class id, payload bits)`, payloads
/// being bits this client does not decode.
fn write_ghosts(w: &mut BitWriter, creates: &[(u32, u32, u32)]) {
    if creates.is_empty() {
        w.flag(false);
        return;
    }
    w.flag(true);
    w.int(2, 4);
    for &(index, class, bits) in creates {
        w.flag(true);
        w.int(index, 2);
        w.flag(false);
        w.int(class, 12);
        w.int(bits, 14);
        for i in 0..bits {
            w.flag(i % 3 == 0);
        }
    }
    w.flag(false);
}

fn command(name_tag: u32, name: &str, events: &mut Events) {
    events.send(Event::NetString {
        id: name_tag,
        text: name.as_bytes().to_vec(),
    });
    events.send(Event::RemoteCommand(vec![
        Arg::Tag(name_tag),
        Arg::Int(MISSION),
        Arg::Text(b"levels/lab.mis".to_vec()),
    ]));
}

fn run_against_the_server(version: ClientVersion) -> Seen {
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    server
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let addr = server.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let client = std::thread::spawn(move || {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let who = Identity {
            account: "lab".into(),
            password: "pw".into(),
            nickname: "Lab".into(),
            version,
        };
        let mut out = Vec::new();
        let ended = session::run_with_sequence(
            &socket,
            addr,
            GRAAL3D_19,
            &who,
            &flag,
            session::Shared::default(),
            &mut out,
            SEQ,
        );
        (ended, String::from_utf8(out).unwrap())
    });

    let mut buf = [0u8; 2048];
    let deadline = Instant::now() + Duration::from_secs(20);
    // Handshake: answer the challenge, then accept the connect request.
    let peer = loop {
        assert!(Instant::now() < deadline, "no connect request");
        let Ok((n, from)) = server.recv_from(&mut buf) else {
            continue;
        };
        match buf[0] {
            handshake::kind::CHALLENGE_REQUEST => {
                let mut resp = vec![handshake::kind::CHALLENGE_RESPONSE];
                resp.extend(SEQ.to_le_bytes());
                resp.extend(0u8..16);
                server.send_to(&resp, from).unwrap();
            }
            handshake::kind::CONNECT_REQUEST => {
                let mut r = BitReader::new(&buf[21..n]);
                assert_eq!(r.string().unwrap(), b"GameConnection");
                r.u32().unwrap();
                r.u32().unwrap();
                assert_eq!(r.string().unwrap(), b"Graal3D");
                assert_eq!((r.u32().unwrap(), r.u32().unwrap()), (19, 19));
                server.send_to(&accept(), from).unwrap();
                break from;
            }
            _ => {}
        }
    };

    let classes = table();
    let mut link = Link::new(SEQ);
    let mut events = Events::new();
    let mut commands = Vec::new();
    let mut messages = Vec::new();
    let mut stage = 0;
    let mut control_sent = false;
    let mut moves_seen = Vec::new();
    while !control_sent {
        assert!(Instant::now() < deadline, "stuck at stage {stage}");
        let Ok((n, _)) = server.recv_from(&mut buf) else {
            continue;
        };
        if !handshake::is_data_packet(&buf[..n]) {
            continue;
        }
        let Ok(got) = link.receive(&buf[..n]) else {
            continue;
        };
        for note in &got.notifies {
            events.notify(*note);
        }
        let Some(mut data) = got.data else { continue };
        let (game, moves) = ClientGame19::read(&mut data.body).unwrap();
        moves_seen.push((game.first_move, moves));
        for ev in events.read(&mut data.body, &classes).unwrap() {
            if let Some(c) = events.command(&ev) {
                commands.push((c.name.clone(), c.args.clone()));
            } else if let Event::ConnectionMessage { .. } = ev {
                messages.push(ev);
            }
        }
        let has = |name: &[u8]| commands.iter().any(|(n, _)| n == name);
        let said = |m: u32| {
            messages
                .iter()
                .any(|e| matches!(e, Event::ConnectionMessage { message, .. } if *message == m))
        };
        let mut ghosts = Vec::new();
        let mut control = Control19::Unchanged;
        match stage {
            0 => {
                command(0, "MissionStartPhase1", &mut events);
                stage = 1;
            }
            1 if has(b"MissionStartPhase1Ack") => {
                for (index, class_id) in [(0, 0), (1, 1)] {
                    events.send(Event::SimDataBlock {
                        id: 33 + index,
                        class_id,
                        index,
                        total: 2,
                    });
                }
                events.send(Event::ConnectionMessage {
                    sequence: MISSION as u32,
                    message: message::DATA_BLOCKS_DONE,
                    ghost_count: 0,
                });
                stage = 2;
            }
            2 if said(message::DATA_BLOCKS_DOWNLOAD_DONE) => {
                command(1, "MissionStartPhase2", &mut events);
                events.send(Event::ConnectionMessage {
                    sequence: MISSION as u32,
                    message: message::GHOST_ALWAYS_STARTING,
                    ghost_count: 3,
                });
                ghosts = vec![(0, 2, 40), (1, 0, 300), (2, 1, 17)];
                events.send(Event::ConnectionMessage {
                    sequence: MISSION as u32,
                    message: message::GHOST_ALWAYS_DONE,
                    ghost_count: 0,
                });
                stage = 3;
            }
            3 if has(b"MissionStartPhase2Ack") && said(message::READY_FOR_NORMAL_GHOSTS) => {
                command(2, "MissionStartPhase3", &mut events);
                stage = 4;
            }
            4 if has(b"MissionStartPhase3Ack") => {
                // Player ghost 1 becomes the control object. Its packet
                // data would follow; this client stops there.
                control = Control19::Object {
                    server_time: 5000,
                    index: 1,
                };
                control_sent = true;
            }
            _ => {}
        }
        let mut w = link.data(None, None).unwrap();
        tge::write_server_game(&mut w, 0, control);
        if !control_sent {
            events.write(&mut w, link.last_sent(), &classes, 3200);
            write_ghosts(&mut w, &ghosts);
        }
        server.send_to(w.bytes(), peer).unwrap();
    }
    // Let the client take the control object, then stop it.
    std::thread::sleep(Duration::from_millis(300));
    stop.store(true, Ordering::Relaxed);
    let (ended, log) = client.join().unwrap();
    assert_eq!(ended, Ok(Ended::Stopped), "{log}");
    assert!(
        moves_seen.iter().all(|&(first, n)| first == 0 && n == 0),
        "no moves: {moves_seen:?}"
    );
    (commands, messages, log)
}

#[test]
fn a_3153_client_runs_the_mission_start_and_echoes_the_phase_sequence() {
    let (commands, messages, log) = run_against_the_server(ClientVersion::V3153);
    let seq = vec![Arg::Int(MISSION)];
    assert_eq!(
        commands,
        [
            (b"MissionStartPhase1Ack".to_vec(), seq.clone()),
            (b"MissionStartPhase2Ack".to_vec(), seq.clone()),
            (b"MissionStartPhase3Ack".to_vec(), seq),
        ],
        "{log}"
    );
    assert_eq!(
        messages,
        [
            Event::ConnectionMessage {
                sequence: MISSION as u32,
                message: message::DATA_BLOCKS_DOWNLOAD_DONE,
                ghost_count: 0
            },
            Event::ConnectionMessage {
                sequence: MISSION as u32,
                message: message::READY_FOR_NORMAL_GHOSTS,
                ghost_count: 0
            },
        ],
        "{log}"
    );
    for line in [
        "3d: Connection established (9 classes)",
        "3d: MissionStartPhase1: Phase 1: Download Datablocks & Targets",
        "3d: datablock 1/2: PlayerData id 33",
        "3d: datablock 2/2: ItemData id 34",
        "3d: connection message 6 (sequence 7), answered 7",
        "3d: MissionStartPhase2: Phase 2: Download Ghost Objects",
        "3d: ghost 0 Sky",
        "3d: ghost 1 Player",
        "3d: ghost 2 Item",
        "3d: connection message 0 (sequence 7), answered 1",
        "3d: MissionStartPhase3: Phase 3: Mission Lighting",
        "3d: Mission lighting done (2 datablocks)",
        "3d: control object is ghost 1 Player",
    ] {
        assert!(
            log.lines().any(|l| l == line),
            "missing {line:?} in:\n{log}"
        );
    }
}

#[test]
fn a_404r2_client_acks_the_phases_bare() {
    let (commands, _, log) = run_against_the_server(ClientVersion::V404R2);
    let names: Vec<_> = commands.iter().map(|(n, _)| n.clone()).collect();
    assert_eq!(
        names,
        [
            b"MissionStartPhase1Ack".to_vec(),
            b"MissionStartPhase2Ack".to_vec(),
            b"MissionStartPhase3Ack".to_vec(),
        ],
        "{log}"
    );
    assert!(commands.iter().all(|(_, args)| args.is_empty()), "{log}");
    assert!(
        log.contains("3d: control object is ghost 1 Player"),
        "{log}"
    );
}
