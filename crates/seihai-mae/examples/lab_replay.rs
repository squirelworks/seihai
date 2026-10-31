//! Replay a cleaned lab frame export (JSONL) through Seihai's codecs.
//!
//! Usage: cargo run -p seihai-mae --example lab_replay -- <frames.jsonl>...
//!
//! Gen-3 C>S: every message must decode with `Gen3`, and the iterator
//! after each frame must equal the next frame's `it_before`.
//! Gen-5: every frame must un-XOR with `Gen5` and then decompress.
//! S>C bodies go through `SessionState`; only counts and ids are
//! printed, never payload text.

use seihai::protocol::{
    bz2_decompress, split_messages, zlib_decompress, Gen3, Gen5, Gen5Comp, SessionState,
};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[derive(Default)]
struct Stats {
    frames: usize,
    dropped: usize,
    msgs: usize,
    bad: usize,
    junk_at_0: usize,
    chain_breaks: usize,
    ctypes: BTreeMap<u8, usize>,
    largest: usize,
}

fn main() {
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path).expect("read export");
        let recs: Vec<Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let mut sessions: BTreeMap<String, (u64, u8)> = BTreeMap::new();
        let mut frames: HashMap<(String, String), Vec<&Value>> = HashMap::new();
        for r in &recs {
            let s = r["session"].as_str().unwrap().to_string();
            if r["type"] == "session" {
                let gen = r["gen"].as_u64().unwrap();
                sessions.insert(s, (gen, r["key"].as_u64().unwrap() as u8));
            } else {
                let d = r["dir"].as_str().unwrap().to_string();
                frames.entry((s, d)).or_default().push(r);
            }
        }
        println!("== {path}");
        for (s, &(gen, key)) in &sessions {
            let mut state = SessionState::default();
            for dir in ["C>S", "S>C"] {
                let Some(fr) = frames.get_mut(&(s.clone(), dir.to_string())) else {
                    continue;
                };
                fr.sort_by_key(|r| r["frame"].as_u64());
                let mut st = Stats::default();
                let mut prev_end: Option<u32> = None;
                for f in fr.iter() {
                    let it_before = f["it_before"].as_u64().map(|v| v as u32);
                    if let (Some(p), Some(b)) = (prev_end, it_before) {
                        if p != b {
                            st.chain_breaks += 1;
                        }
                    }
                    prev_end = None;
                    if f["dropped"] == true {
                        st.dropped += 1;
                        continue;
                    }
                    st.frames += 1;
                    let body = hex(f["body_hex"].as_str().unwrap());
                    st.largest = st.largest.max(body.len());
                    let plain = if gen == 5 {
                        let comp = match body[0] {
                            2 => Gen5Comp::None,
                            4 => Gen5Comp::Zlib,
                            6 => Gen5Comp::Bz2,
                            _ => {
                                st.bad += 1;
                                continue;
                            }
                        };
                        *st.ctypes.entry(body[0]).or_default() += 1;
                        let mut g = Gen5::with_state(it_before.unwrap(), key);
                        let raw = g.decode(body[1..].to_vec(), comp);
                        prev_end = Some(g.state());
                        let out = match comp {
                            Gen5Comp::None => Ok(raw),
                            Gen5Comp::Zlib => zlib_decompress(&raw),
                            Gen5Comp::Bz2 => bz2_decompress(&raw),
                        };
                        match out {
                            Ok(p) => p,
                            Err(_) => {
                                st.bad += 1;
                                continue;
                            }
                        }
                    } else {
                        match zlib_decompress(&body) {
                            Ok(p) => p,
                            Err(_) => {
                                st.bad += 1;
                                continue;
                            }
                        }
                    };
                    if gen == 3 && dir == "C>S" {
                        let mut g = Gen3::with_state(it_before.unwrap(), key);
                        for line in plain.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
                            let mut enc = line.to_vec();
                            enc.push(b'\n');
                            let (_, pos) = g.clone().next_insertion(enc.len() - 1);
                            st.msgs += 1;
                            if pos == 0 {
                                st.junk_at_0 += 1;
                            }
                            if g.decode(&enc).is_none() {
                                st.bad += 1;
                            }
                        }
                        prev_end = Some(g.state());
                    } else {
                        st.msgs += split_messages(&plain).len();
                        if dir == "S>C" {
                            state.apply_body(&plain);
                        }
                    }
                }
                println!(
                    "{s} gen{gen} {dir}: frames {} (+{} dropped), msgs {}, bad {}, junk@0 {}, chain breaks {}, ctypes {:?}, largest {} B",
                    st.frames, st.dropped, st.msgs, st.bad, st.junk_at_0, st.chain_breaks, st.ctypes, st.largest
                );
            }
            let mut unhandled: Vec<_> = state.unhandled.iter().filter(|(_, &n)| n > 1).collect();
            unhandled.sort();
            println!(
                "  SessionState: level_name {} board_lines {} links {} npcs {} items {} props {} pos {:?}/{:?} dir {:?} unhandled(>1) {:?}",
                state.level_name.is_some(),
                state.board_lines.len(),
                state.links.len(),
                state.npcs.len(),
                state.items.len(),
                state.other_props.len(),
                state.player_x,
                state.player_y,
                state.player_dir,
                unhandled
            );
        }
    }
}
