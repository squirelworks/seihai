//! Stage 2 from the lab: what the patched lab GServer-v2 sent a 3.153 login
//! for gs2test.nw (`fixtures/lab-3153-gs2test.txt`), read with the core
//! readers, kept by the level store and logged with its fields.

use seihai_ato::{fields, level::LevelStore};
use seihai_core::assets::Level;
use seihai_core::protocol::{ClientVersion, SessionState};

const FIXTURE: &str = include_str!("fixtures/lab-3153-gs2test.txt");

fn gc(v: u8) -> u8 {
    v + 0x20
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

/// `PLO_RAWDATA` (100) announcing `n` bytes.
fn rawdata(n: usize) -> Vec<u8> {
    vec![gc(100), gc((n >> 14) as u8 & 0x7F), gc((n >> 7) as u8 & 0x7F), gc(n as u8 & 0x7F), b'\n']
}

/// The fixture's frames as bodies, in order.
fn frames() -> Vec<Vec<u8>> {
    let mut out: Vec<(u32, Vec<u8>)> = Vec::new();
    for line in FIXTURE.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let frame: u32 = f[0].parse().unwrap();
        let id: u8 = f[1].parse().unwrap();
        let mut m = Vec::new();
        match f.get(2..).unwrap_or(&[]) {
            ["board", tile] => {
                let tile = unhex(tile);
                let mut body: Vec<u8> = tile.iter().copied().cycle().take(8192).collect();
                body.push(b'\n');
                m.extend(rawdata(body.len() + 1));
                m.push(gc(id));
                m.extend(body);
            }
            ["raw", hex] => {
                let body = unhex(hex);
                m.extend(rawdata(body.len() + 1));
                m.push(gc(id));
                m.extend(body);
            }
            [hex] => {
                let body = unhex(hex);
                assert!(!body.contains(&b'\n'), "line {line}");
                m.push(gc(id));
                m.extend(body);
                m.push(b'\n');
            }
            [] => m.extend([gc(id), b'\n']),
            _ => panic!("line {line}"),
        }
        match out.last_mut() {
            Some((n, b)) if *n == frame => b.extend(m),
            _ => out.push((frame, m)),
        }
    }
    out.into_iter().map(|(_, b)| b).collect()
}

fn none(_: &str, _: Option<u32>) -> Option<Level> {
    None
}

#[test]
fn the_lab_level_is_kept_and_its_fields_logged() {
    let v = ClientVersion::V3153;
    let mut st = SessionState::for_version(v);
    let mut store = LevelStore::default();
    for body in frames() {
        st.apply_body(&body);
        store.update(&mut st, none);
    }
    let a = store.arrival(&st).unwrap();
    // The line the live run printed.
    assert_eq!(a.line(), "level gs2test.nw: from the server's board, start (30, 32), 1 NPCs (1 from the server, 0 in the file)");
    let s = a.script_lines();
    assert_eq!(s[0], "npc 300: GS2 script, 19 ops, 3 strings, functions [onCreated]");
    let level = store.level.as_ref().unwrap();
    let out = fields::lines(v, level, &a.name, &st);
    let text = out.join("\n");
    assert_eq!(out[0], "board: 4096 tiles, 1 distinct [2047], 0 links, 0 chests, 0 signs", "{text}");
    assert!(out.contains(&"  1 MAXPOWER: 23 = 3".to_string()), "{text}");
    assert!(out.contains(&"  10 GANI: 2469646c65 \"idle\"".to_string()), "{text}");
    // The stream ends with 82 (`%guest`), which 3.153's prop switch does
    // not take: reading stops after Z2 (80), as the client's does.
    let last = out.iter().rposition(|l| l.starts_with("  ")).unwrap();
    assert_eq!(out[last], "  80 Z2: 2020", "{text}");
    assert_eq!(
        out[last + 1],
        "npc 300 props: at (30, 28), image \"\", sprite 2, vis 1, message \"\", GS1 script 0 bytes",
        "{text}"
    );
}

#[test]
fn a_build_without_a_torque_profile_logs_no_fields() {
    let st = SessionState::for_version(ClientVersion::V231);
    assert!(fields::lines(ClientVersion::V231, &Level::from_board(&[0; 4096]), "x.nw", &st).is_empty());
}
