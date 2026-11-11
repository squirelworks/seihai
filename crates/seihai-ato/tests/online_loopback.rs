//! Stage 2 end to end on loopback: a stand-in game server takes the login
//! of each 3.x build and sends a level the way GServer does (name, player
//! props, board, then an NPC in a later frame). Ato logs in with core's
//! login, reads it with the core readers and logs the level once.

use seihai_ato::online::{self, Login};
use seihai_core::protocol::{
    write_frame, write_zlib_frame, zlib_compress, zlib_decompress, ClientVersion, EncryptionGen,
    Gen5, Gen5Comp,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

fn gc(v: u8) -> u8 {
    v + 0x20
}

fn msg(id: u8, payload: &[u8]) -> Vec<u8> {
    let mut m = vec![gc(id)];
    m.extend_from_slice(payload);
    m.push(b'\n');
    m
}

/// `PLO_RAWDATA` (100) announcing 8194 bytes, then `PLO_BOARDPACKET` (101).
fn board(fill: u16) -> Vec<u8> {
    let mut out = msg(100, &[gc(0), gc(64), gc(2)]);
    out.push(gc(101));
    for _ in 0..64 * 64 {
        out.extend(fill.to_le_bytes());
    }
    out.push(b'\n');
    out
}

/// A script built with gs2-parser (see the fixtures' README).
const SCRIPT: &[u8] = include_bytes!("../../seihai-core/tests/fixtures/gs2/shared.gs2bc");

fn serve(version: ClientVersion, listener: TcpListener) {
    serve_with(version, listener, None);
}

/// As [`serve`]; with `warp_to`, a Login server: after the login it sends
/// only a `PLO_SERVERWARP` (178) to that port and waits for the client to
/// leave.
fn serve_with(version: ClientVersion, listener: TcpListener, warp_to: Option<u16>) {
    let (mut s, _) = listener.accept().unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    // The login frame: u16 length, plain zlib, the login line first.
    let mut len = [0u8; 2];
    s.read_exact(&mut len).unwrap();
    let mut body = vec![0u8; u16::from_be_bytes(len) as usize];
    s.read_exact(&mut body).unwrap();
    let login = zlib_decompress(&body).unwrap();
    assert_eq!(login[0], gc(version.client_type()));
    let key = login[1] - 0x20;
    assert!(login[2..].starts_with(version.token().as_bytes()));
    let mut cipher = (version.encryption() == EncryptionGen::Gen5).then(|| Gen5::new(key));
    let mut send = |body: Vec<u8>| {
        let frame = match cipher.as_mut() {
            Some(g) => write_frame(
                Gen5Comp::Zlib,
                &g.encode(zlib_compress(&body).unwrap(), Gen5Comp::Zlib),
            ),
            None => write_zlib_frame(&body).unwrap(),
        };
        s.write_all(&frame).unwrap();
    };
    if let Some(port) = warp_to {
        send(msg(
            178,
            format!("0,Lab Server,127.0.0.1,{port}").as_bytes(),
        ));
        let mut rest = Vec::new();
        let _ = s.read_to_end(&mut rest);
        return;
    }
    // PLO_PLAYERPROPS (9): X 30.5, Y 31; PLO_LEVELNAME (6); the board.
    let mut first = msg(9, &[gc(15), gc(61), gc(16), gc(62)]);
    first.extend(msg(6, b"lab3x.nw"));
    first.extend(board(0x1FF));
    send(first);
    // PLO_NPCPROPS (3) for NPC 1, in its own frame, then its GS2 script
    // (PLO_NPCBYTECODE, 131) behind PLO_RAWDATA, as GServer-v2's
    // `TLevel::getNpcsPacket` sends it.
    let mut npc = msg(3, &[gc(0), gc(0), gc(1)]);
    let mut code = vec![gc(131), gc(0), gc(0), gc(1)];
    code.extend_from_slice(SCRIPT);
    let n = code.len();
    npc.extend(msg(
        100,
        &[
            gc((n >> 14) as u8 & 0x7F),
            gc((n >> 7) as u8 & 0x7F),
            gc(n as u8 & 0x7F),
        ],
    ));
    npc.extend(code);
    send(npc);
    // Keep the socket open until the client hangs up.
    let mut rest = Vec::new();
    let _ = s.read_to_end(&mut rest);
}

fn run(version: ClientVersion) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let server = std::thread::spawn(move || serve(version, listener));
    let login = Login {
        account: "lab".into(),
        password: "pw".into(),
        nick: "lab".into(),
        language: "English".into(),
    };
    let mut out = Vec::new();
    let mut levels = Vec::new();
    online::run(&addr, version, &login, None, true, &mut out, &mut |l, a| {
        levels.push((l.clone(), a.clone()))
    })
    .unwrap();
    // The level handed to the view is the one logged, built from the board.
    assert_eq!(levels.len(), 1);
    assert_eq!(levels[0].0.tile_id_at(10, 10), Some(0x1FF));
    server.join().unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn each_3x_build_logs_in_and_logs_the_level() {
    // Both 3.x rows carry a recorded post-login frame.
    for v in [ClientVersion::V304, ClientVersion::V3153] {
        assert!(
            seihai_core::protocol::login::LoginBurst::for_version(v).is_some(),
            "{}",
            v.label()
        );
    }
    for v in [ClientVersion::V304, ClientVersion::V3153] {
        let out = run(v);
        assert!(out.contains(&format!("as lab ({})", v.label())), "{out}");
        assert!(
            out.contains("level lab3x.nw: from the server's board, start (30.5, 31), 1 NPCs (1 from the server, 0 in the file)"),
            "{}: {out}",
            v.label()
        );
        assert_eq!(out.matches("level lab3x.nw").count(), 1, "{out}");
        // The frame the real 3.x client answers the board with goes out
        // (core's LoginBurst), before the level is logged.
        if seihai_core::protocol::login::LoginBurst::for_version(v).is_some() {
            assert!(
                out.contains("sent the post-login frame"),
                "{}: {out}",
                v.label()
            );
        }
        // Only 3.153 handles 131: it reads the script and logs its opcodes.
        let read = out.contains("npc 1: GS2 script, ") && out.contains("OP_CALL");
        assert_eq!(read, v == ClientVersion::V3153, "{}: {out}", v.label());
    }
}

/// A 4.x or 5.x build logs in to its Login server, follows the warp to the
/// game server, logs in again there and logs the level.
#[test]
fn lobby_builds_follow_the_login_servers_warp() {
    for v in [
        ClientVersion::V420R8,
        ClientVersion::V500R7,
        ClientVersion::V512,
    ] {
        let game = TcpListener::bind("127.0.0.1:0").unwrap();
        let game_port = game.local_addr().unwrap().port();
        let lobby = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = lobby.local_addr().unwrap().to_string();
        let a = std::thread::spawn(move || serve_with(v, lobby, Some(game_port)));
        let b = std::thread::spawn(move || serve(v, game));
        let login = Login {
            account: "lab".into(),
            password: "pw".into(),
            nick: "lab".into(),
            language: "English".into(),
        };
        let mut out = Vec::new();
        let mut levels = 0;
        online::run(&addr, v, &login, None, true, &mut out, &mut |_, _| {
            levels += 1
        })
        .unwrap();
        a.join().unwrap();
        b.join().unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(
            out.contains(&format!(
                "server warp to \"Lab Server\": 127.0.0.1:{game_port}"
            )),
            "{}: {out}",
            v.label()
        );
        assert_eq!(out.matches("logged in at").count(), 2, "{out}");
        assert!(out.contains("level lab3x.nw"), "{}: {out}", v.label());
        assert_eq!(levels, 1);
        // On the game server the build's post-login frames go out.
        if seihai_core::protocol::lobby::LobbyBurst::for_version(v).is_some() {
            assert!(
                out.contains("sent the post-login frames"),
                "{}: {out}",
                v.label()
            );
        }
    }
}

/// 4.04r2, the 4.x build with recorded after-login messages, sends them
/// once its level has been in for 240 ms without a 182.
#[test]
fn the_404_lobby_burst_goes_out() {
    let v = ClientVersion::V404R2;
    assert!(seihai_core::protocol::lobby::LobbyBurst::for_version(v).is_some());
    let out = run(v);
    assert!(out.contains("sent the post-login frames"), "{out}");
}
