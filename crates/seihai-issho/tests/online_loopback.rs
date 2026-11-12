//! Issho on loopback: a stand-in Login server takes the 6.x login and
//! warps to a stand-in game server, which sends a level the way GServer
//! does (name, player props, board, then an NPC with a GS2 script). Issho
//! logs in with core's client session at both, as `seihai-ato` does, and
//! logs the level once.

use seihai_core::protocol::{
    write_frame, write_zlib_frame, zlib_compress, zlib_decompress, ClientVersion, EncryptionGen,
    Gen5, Gen5Comp,
};
use seihai_issho::cli::{online, run_online};
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

/// A script built with gs2-parser (copied from seihai-core's
/// `tests/fixtures/gs2`; see the README there).
const SCRIPT: &[u8] = include_bytes!("fixtures/gs2/shared.gs2bc");

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

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().copied().map(str::to_string).collect()
}

/// Each 6.x build logs in to its Login server, follows the warp, logs in
/// again at the game server and logs the level from its board.
#[test]
fn v6_builds_log_in_follow_the_warp_and_log_the_level() {
    for v in [
        ClientVersion::V600R2,
        ClientVersion::V6015,
        ClientVersion::V6034,
        ClientVersion::V603R7,
        ClientVersion::V603R7Linux,
    ] {
        let game = TcpListener::bind("127.0.0.1:0").unwrap();
        let game_port = game.local_addr().unwrap().port();
        let lobby = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = lobby.local_addr().unwrap().to_string();
        let a = std::thread::spawn(move || serve_with(v, lobby, Some(game_port)));
        let b = std::thread::spawn(move || serve(v, game));
        let argv = args(&[
            "--version",
            v.label(),
            "--online",
            &addr,
            "--account",
            "lab",
            "--password-env",
            "LAB_PW",
            "--first-level",
        ]);
        let o = online(&argv, |n| (n == "LAB_PW").then(|| "pw".to_string()))
            .unwrap()
            .unwrap();
        let mut out = Vec::new();
        run_online(&o, &mut out).unwrap();
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
        assert!(out.contains(&format!("as lab ({})", v.label())), "{out}");
        assert!(
            out.contains("level lab3x.nw: from the server's board, start (30.5, 31), 1 NPCs (1 from the server, 0 in the file)"),
            "{}: {out}",
            v.label()
        );
        assert_eq!(out.matches("level lab3x.nw").count(), 1, "{out}");
    }
}

/// The game loop (`seihai-play`, the one Ato runs) on a 6.x login: the
/// level from the server's board arrives in the game with the player at
/// the server's start, and the arrows walk it and hand the pose to the
/// network thread.
#[cfg(feature = "window")]
#[test]
fn the_v6_game_loop_takes_the_level_and_walks_it() {
    use seihai_play::game::{self, Game};
    use std::time::Instant;

    let v = ClientVersion::V603R7;
    let game_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let game_port = game_listener.local_addr().unwrap().port();
    let lobby = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = lobby.local_addr().unwrap().to_string();
    let a = std::thread::spawn(move || serve_with(v, lobby, Some(game_port)));
    let b = std::thread::spawn(move || serve(v, game_listener));
    let argv = args(&[
        "--version",
        v.label(),
        "--online",
        &addr,
        "--account",
        "lab",
        "--password-env",
        "LAB_PW",
    ]);
    let o = online(&argv, |n| (n == "LAB_PW").then(|| "pw".to_string()))
        .unwrap()
        .unwrap();
    let (rx, live) = game::spawn_online(o.addr, o.version, o.login, None, game::no_lines, || {});
    let pose = live.pose.clone();
    let mut g = Game::new(rx, None, false).unwrap();
    g.attach(live);
    let deadline = Instant::now() + Duration::from_secs(10);
    while g.session.current_level.is_none() && Instant::now() < deadline {
        g.tick(0, 0, Instant::now());
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(g.status.starts_with("level lab3x.nw"), "{}", g.status);
    assert_eq!((g.session.player_x, g.session.player_y), (30.5, 31.0));
    let frame = g.draw((40, 30));
    assert_eq!((frame.width, frame.height), (640, 480));
    // Walk right until a step lands (the session paces the steps).
    let start = Instant::now() + Duration::from_secs(1);
    for i in 0..10 {
        g.tick(1, 0, start + g.session.step_interval() * i);
    }
    assert!(g.session.player_x > 30.5, "{}", g.session.player_x);
    let sent = (*pose.lock().unwrap()).expect("the pose is shared");
    assert_eq!(sent.x, g.session.player_x);
    assert!(sent.walking);
    drop(g);
    a.join().unwrap();
    b.join().unwrap();
}
