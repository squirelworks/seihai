//! In-process fuzzing of Seihai's server-facing parsers.
//! fuzzcore TARGET ITERS SEED [SEEDFILE...]
use seihai_core::protocol::{ClientVersion, SessionState};
use std::panic;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next() % n } }
    fn byte(&mut self) -> u8 {
        match self.below(6) { 0 => 32 + self.below(96) as u8, 1 => 255, 2 => 32, 3 => b'\n', _ => self.next() as u8 }
    }
}

fn mutate(r: &mut Rng, seeds: &[Vec<u8>]) -> Vec<u8> {
    let mut v = if seeds.is_empty() || r.below(4) == 0 {
        let k = r.below(4) as usize; let n = r.below([16, 256, 4096, 70000][k]) as usize;
        (0..n).map(|_| r.byte()).collect()
    } else {
        seeds[r.below(seeds.len() as u64) as usize].clone()
    };
    for _ in 0..r.below(8) {
        if v.is_empty() { v.push(r.byte()); continue; }
        let i = r.below(v.len() as u64) as usize;
        match r.below(6) {
            0 => v[i] = r.byte(),
            1 => v.truncate(i),
            2 => { let b = r.byte(); let n = r.below(3000) as usize; v.splice(i..i, std::iter::repeat(b).take(n)); }
            3 => { let j = r.below(v.len() as u64) as usize; let (a, b) = (i.min(j), i.max(j)); let s = v[a..b].to_vec(); v.splice(i..i, s); }
            4 => v.insert(i, r.byte()),
            _ => { v.remove(i); }
        }
    }
    v
}

const VERSIONS: &[&str] = &["1.39", "1.41r1", "2.05", "2.22", "5.00r7", "5.12", "5.24r4", "5.30r3", "6.00r2", "6.015", "6.034", "6.03r7", "6.03r7 Linux"];

fn run(target: &str, data: &[u8], vi: usize) {
    use seihai_core::protocol as p;
    let s = String::from_utf8_lossy(data);
    match target {
        "session" => {
            let v = ClientVersion::from_label(VERSIONS[vi % VERSIONS.len()]).unwrap();
            let mut st = SessionState::for_version(v);
            st.apply_body(data);
            st.apply_body(data);
        }
        "msg" => {
            // every message id with this payload, on a session per version
            let v = ClientVersion::from_label(VERSIONS[vi % VERSIONS.len()]).unwrap();
            let mut st = SessionState::for_version(v);
            for id in 0..=255u8 {
                st.apply(&p::Message { id, payload: data.to_vec() });
            }
        }
        "parsers" => {
            let _ = p::npcprops::parse_npc_props(data, false);
            let _ = p::npcprops::parse_npc_props(data, true);
            let _ = p::parse_player_props_with(data, false);
            let _ = p::parse_player_props_with(data, true);
            let _ = p::showimg::parse_showimg_update(data);
            let _ = p::layouts::parse_player_warp2(data);
            let _ = p::layouts::parse_profile(data);
            let _ = p::layouts::parse_npc_del2(data);
            let _ = p::layouts::parse_server_warp(data);
            let _ = p::layouts::parse_server_warp_lobby(data);
            let _ = p::layouts::parse_board_heights(data);
            let _ = p::layouts::parse_board_modify2(data);
            let _ = p::layouts::parse_board_modify(data);
            let _ = p::layouts::parse_showimg_relay(data);
            let _ = p::extras::parse_map_spec(data);
            let _ = p::extras::parse_text_list(data);
            let _ = p::parse_svrlist(data);
            let _ = p::parse_list_reply(data);
            let _ = seihai_core::assets::decode_sign_text(data);
            let _ = seihai_core::assets::decode_sign_record(data);
        }
        "level" => {
            let _ = seihai_core::assets::Level::parse(&s);
            let _ = seihai_core::assets::Level::parse_binary_with_layout(data);
            let _ = seihai_core::assets::ZeldaLevel::parse(data);
        }
        "gani" => { let _ = seihai_core::assets::Gani::parse(&s); }
        "image" => {
            let _ = seihai_core::assets::read_png_rgba(data);
            let _ = seihai_core::assets::gif::read_gif_rgba(data);
        }
        "gs1" => {
            for era in [seihai_core::scripting::engine::Era::V205, seihai_core::scripting::engine::Era::V21] {
                let _ = seihai_core::scripting::engine::parser::parse(&s, era);
            }
        }
        "gs2" => {
            let _ = seihai_core::scripting::gs2::Script::parse(data);
            let _ = seihai_core::scripting::Bytecode::parse(data);
        }
        _ => panic!("unknown target"),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let target = a[1].clone();
    let iters: u64 = a[2].parse().unwrap();
    let mut r = Rng(a[3].parse::<u64>().unwrap() | 1);
    let mut seeds: Vec<Vec<u8>> = Vec::new();
    for f in &a[4..] {
        if let Ok(d) = std::fs::read(f) {
            // A seed file is either one input or a list of hex lines.
            if f.ends_with(".hex") {
                for l in String::from_utf8_lossy(&d).lines() {
                    let b: Vec<u8> = (0..l.len() / 2).filter_map(|i| u8::from_str_radix(&l[2 * i..2 * i + 2], 16).ok()).collect();
                    if !b.is_empty() { seeds.push(b); }
                }
            } else {
                seeds.push(d);
            }
        }
    }
    if iters == 0 {
        // Run each seed as it is, once per version slot.
        for (k, d) in seeds.iter().enumerate() {
            let t0 = std::time::Instant::now();
            let res = panic::catch_unwind(|| run(&target, d, k));
            println!("seed {k} ({} bytes): {} in {:.2} s", d.len(), if res.is_ok() { "ok" } else { "PANIC" }, t0.elapsed().as_secs_f64());
        }
        return;
    }
    panic::set_hook(Box::new(|_| {}));
    let mut found = std::collections::BTreeMap::new();
    for it in 0..iters {
        let data = mutate(&mut r, &seeds);
        let vi = r.below(64) as usize;
        let res = panic::catch_unwind(|| run(&target, &data, vi));
        if let Err(e) = res {
            let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
            if !found.contains_key(&msg) {
                let path = format!("crash-{target}-{}.bin", found.len());
                std::fs::write(&path, &data).unwrap();
                println!("PANIC it={it} v={} {msg} -> {path} ({} bytes)", VERSIONS[vi % VERSIONS.len()], data.len());
                found.insert(msg, path);
            }
        }
        if it % 20000 == 0 { eprintln!("{target} {it}"); }
    }
    println!("{target}: {iters} iterations, {} distinct panics", found.len());
}
