//! Ato's Torque UDP link against a replaying loopback server.
//!
//! The server answers the handshake and then sends, one per client data
//! packet, the five packets of the Seihai gserver session from the
//! research note (mission phases, the Sun and player creates, the control
//! object, two keep-alives). The client must answer both phases, reach
//! the control object and send a disconnect when stopped.

#![cfg(not(feature = "era6"))]

use seihai_ato::torque::net::bits::BitReader;
use seihai_ato::torque::net::events::{ClassTable, Events};
use seihai_ato::torque::net::ghosts::{GameObjectUpdate, Transform};
use seihai_ato::torque::net::handshake::{self, ServerPacket};
use seihai_ato::torque::net::session::{self, Ended, Identity};
use seihai_ato::torque::GRAAL3D_24;
use seihai_core::client::{FileAnswer, FileAsks};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn hex(parts: &[&str]) -> Vec<u8> {
    let s: String = parts.concat();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// A protocol-24 accept naming all 19 classes (sequence 0x400).
const ACCEPT: &[&str] = &[
    "24000400000100001aac96e82d0340006470ae5022c6b196ad05000168e15ca1448c632d5bebfa7a304e500b0003d0c2",
    "b94289184fe7acbfebebc138412d00104085d844bfe26ef1180240011c984e2a01c0000e4c3d058003c8b07eb3648a18",
    "8a270100012a3838e12d62289e048004a8b039997a92e77e860100002322b8774ec8d5664f93b8f404032000464470ef",
    "9c90534883b8f4040340005aaca6448f81955cdf5af61f979e60000c408bd8591b748e222b6dc5dde231c4a527180004",
    "d0c1ed610ac6d4c7a52718000550a2eb91c2afa6082585b8f40403c0004a747d64f3dfa0eb8766884b4f30000e60c4fa",
    "cd9269b5d9d3242e3dc10040801aeb374ba6ae8f6cfe1bc8b10e09dfe2d213c000000000",
];

/// The Seihai gserver's first five data packets to a client on 0x400.
const SESSION: [&[&str]; 5] = [
    &[
        "05084002080108000d004ab8b835569a421aeccdd95ebc001880203062007b4107a69e6c9a277e6c9a277e6c9a277e9b",
        "99997e9a99997e9a99997e0000007f3433337d3433337d6666667d0000007f4600d7411b621d9df74656de642377ba30",
        "7287fdee4bc7d00400000000000000000000fc0100000000000000000000fc01000000003a3072f7ffffff1f0900",
    ],
    &[
        "09104006080908400d084ab8b835569a421aeccdd97eff0260008201",
    ],
    &[
        "0d18400e989300002000000000",
    ],
    &[
        "1120401e0800",
    ],
    &[
        "1528403e0800",
    ],
];

#[test]
fn reaches_the_control_object_against_a_replaying_server() {
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    server
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let addr = server.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    // The TCP side's file session, played here: each file the 3D world
    // asks for is sent, as core's session would answer it.
    let asks = FileAsks::default();
    let tcp = asks.clone();
    let tcp_stop = stop.clone();
    let tcp_side = std::thread::spawn(move || {
        let mut asked = Vec::new();
        while !tcp_stop.load(Ordering::Relaxed) {
            let names = std::mem::take(&mut *tcp.wanted.lock().unwrap());
            for name in names {
                asked.push(name.clone());
                tcp.answers.lock().unwrap().push(FileAnswer {
                    name,
                    data: Some(b"3DS!".to_vec()),
                });
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        asked
    });
    // The window holds the up arrow and says one line from the start; both
    // wait for the control object.
    let keys = seihai_ato::view3d::SharedInput3d::default();
    {
        let mut k = keys.lock().unwrap();
        k.forward = 1;
        k.say.push("hi e:smile".into());
    }
    let world = seihai_ato::view3d::Shared3d::default();
    let view = world.clone();
    let client = std::thread::spawn(move || {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let who = Identity {
            account: "lab".into(),
            password: "pw".into(),
            nickname: "Lab".into(),
        };
        let mut out = Vec::new();
        let ended = session::run_with_sequence(
            &socket,
            addr,
            GRAAL3D_24,
            &who,
            &flag,
            session::Shared {
                files: Some(&asks),
                world: Some(&world),
                input: Some(&keys),
            },
            &mut out,
            0x400,
        );
        (ended, String::from_utf8(out).unwrap())
    });

    let mut buf = [0u8; 2048];
    // Handshake: answer the challenge, then the connect request.
    let (n, peer) = server.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], handshake::challenge_request(0x400));
    let mut resp = vec![0x1e, 0x00, 0x04, 0x00, 0x00];
    resp.extend(0u8..16);
    server.send_to(&resp, peer).unwrap();
    let request = loop {
        let (n, _) = server.recv_from(&mut buf).unwrap();
        if buf[0] == handshake::kind::CONNECT_REQUEST {
            break buf[..n].to_vec();
        }
    };
    let mut r = BitReader::new(&request[21..]);
    assert_eq!(r.string().unwrap(), b"GameConnection");
    r.u32().unwrap();
    r.u32().unwrap();
    assert_eq!(r.string().unwrap(), b"Graal3D");
    assert_eq!((r.u32().unwrap(), r.u32().unwrap()), (24, 24));
    assert_eq!(r.string().unwrap(), b"pw");
    assert_eq!(r.u32().unwrap(), 2);
    assert_eq!(
        (r.string().unwrap(), r.string().unwrap()),
        (b"lab".to_vec(), b"Lab".to_vec())
    );
    let accept = hex(ACCEPT);
    let ServerPacket::ConnectAccept { classes, .. } =
        handshake::read_server_packet(&accept).unwrap()
    else {
        panic!("the accept vector")
    };
    let classes = ClassTable::new(&classes, 1);
    server.send_to(&accept, peer).unwrap();

    // One server packet per client data packet, reading the client's
    // events as the server would.
    let mut last_seq = 0;
    let mut events = Events::new();
    let mut acks = Vec::new();
    let mut own_updates = Vec::new();
    let mut ack_packets = 0;
    let mut packet = 0;
    while packet < SESSION.len() {
        let (n, _) = server.recv_from(&mut buf).unwrap();
        if !handshake::is_data_packet(&buf[..n]) {
            continue;
        }
        // The header (the server's own numbering is not kept here: the
        // replayed packets carry theirs), then the two rate fields.
        let body = &mut BitReader::new(&buf[..n]);
        body.flag().unwrap();
        assert_eq!(body.int(1).unwrap(), 0, "connect sequence bit of 0x400");
        let seq = body.int(9).unwrap();
        body.int(9).unwrap();
        let ty = body.int(2).unwrap();
        let ack_bytes = body.int(3).unwrap();
        body.int(ack_bytes * 8).unwrap();
        if ty == 2 {
            ack_packets += 1;
        }
        if ty != 0 || seq == last_seq {
            continue;
        }
        assert_eq!(seq, last_seq + 1, "client data packets in order");
        last_seq = seq;
        for _ in 0..2 {
            if body.flag().unwrap() {
                body.int(20).unwrap();
            }
        }
        // The client's game section: first person, its position, its
        // object's index flag, the "controls it" flag (each always on the
        // wire), then the field-of-view flag.
        assert!(body.flag().unwrap());
        for _ in 0..3 {
            body.f32().unwrap();
        }
        let object = if body.flag().unwrap() {
            Some(body.int(12).unwrap())
        } else {
            None
        };
        if body.flag().unwrap() {
            assert_eq!(object, Some(1), "an update of its control object");
            own_updates.push(GameObjectUpdate::read(body, None).unwrap());
        }
        assert!(!body.flag().unwrap());
        for ev in events.read(body, &classes).unwrap() {
            acks.push(events.command(&ev).unwrap().name);
        }
        server.send_to(&hex(SESSION[packet]), peer).unwrap();
        packet += 1;
    }
    // Let the client read and ack the last packet, then stop it.
    std::thread::sleep(Duration::from_millis(300));
    while let Ok((n, _)) = {
        server
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        server.recv_from(&mut buf)
    } {
        if buf[0] & 1 == 1
            && (u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) >> 20) & 3 == 2
            && n > 0
        {
            ack_packets += 1;
        }
    }
    server
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stop.store(true, Ordering::Relaxed);
    let disconnect = loop {
        let (n, _) = server.recv_from(&mut buf).unwrap();
        if !handshake::is_data_packet(&buf[..n]) {
            break buf[..n].to_vec();
        }
    };
    assert_eq!(disconnect[0], handshake::kind::DISCONNECT);

    let (ended, log) = client.join().unwrap();
    assert_eq!(ended, Ok(Ended::Stopped), "{log}");
    assert_eq!(
        acks,
        [
            b"MissionStartPhase1Ack".to_vec(),
            b"MissionStartPhase2Ack".to_vec()
        ],
        "{log}"
    );
    assert_eq!(
        ack_packets,
        SESSION.len(),
        "one ack packet per server data packet"
    );
    // After taking ghost 1, the client echoes its shape, transform (zero
    // rotation as axis 1, 0, 0) and flags once, then sends empty updates.
    let echo = GameObjectUpdate {
        shape: Some(b"labcube.3ds".to_vec()),
        transform: Some(Transform {
            position: [0.0, 0.0, 1.0],
            axis: [1.0, 0.0, 0.0],
            angle: 0.0,
        }),
        flags: Some(0x48),
        ..Default::default()
    };
    assert_eq!(own_updates.first(), Some(&echo), "{log}");
    // Then the walk: the transform moved along +y, and the line said.
    let walked = &own_updates[1];
    let t = walked.transform.expect("the walk's transform");
    assert!(t.position[1] > 0.0 && t.position[2] == 1.0, "{t:?}");
    assert_eq!(walked.chat.as_deref(), Some(&b"hi e:smile"[..]));
    assert!(log.lines().any(|l| l == "3d: say hi e:smile"), "{log}");
    for line in [
        "3d: Connection established (19 classes)",
        "3d: MissionStartPhase1",
        "3d: MissionStartPhase2",
        "3d: Mission lighting done",
        "3d: ghost 0 Sun",
        "3d: ghost 1 GameObject G3DPlayer_lab",
        "3d: control object is ghost 1 GameObject G3DPlayer_lab",
        "3d: requesting labcube.3ds over TCP",
        "3d: got labcube.3ds (4 bytes)",
        "3d: login step 6: every file request answered (files: 1, not on the server: 0)",
    ] {
        assert!(
            log.lines().any(|l| l == line),
            "missing {line:?} in:\n{log}"
        );
    }
    // The view's world has the player as the control object, where it
    // walked, and the Sun; the shape's bytes are not a .3ds here.
    {
        let w = view.lock().unwrap();
        assert_eq!(w.control, Some(1));
        let me = &w.objects[&1];
        assert_eq!(me.shape.as_deref(), Some("labcube.3ds"));
        // Walked from (0, 0, 1) along +y while the test ran.
        let at = me.transform.unwrap().position;
        assert!(at[0] == 0.0 && at[1] > 0.0 && at[2] == 1.0, "{at:?}");
        assert!(w.sun.is_some());
        assert_eq!(w.models.get("labcube.3ds"), Some(&None));
    }
    // The player's shape is asked for once.
    assert_eq!(tcp_side.join().unwrap(), ["labcube.3ds"]);
}
