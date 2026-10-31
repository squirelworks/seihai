//! Decode a lab UDP pcap with Seihai's peer-channel codec.
//!
//! Usage: cargo run -p seihai-mae --example udp_replay -- <file.pcap> <port=id>...
//!        [--from <seconds> port=id...]
//!
//! Each `port=id` maps a destination port to the player id that owns
//! it. `--from <s>` starts a new map for datagrams at or after `s`
//! seconds from the first packet (players get new ids on reconnect).
//! Checks that `seq` counts from 0 per sender→recipient pair.

use seihai::net::{decode_udp, udp_key};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0]).expect("pcap");
    // (start seconds, port -> id)
    let mut maps: Vec<(f64, BTreeMap<u16, u16>)> = vec![(0.0, BTreeMap::new())];
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--from" {
            maps.push((args[i + 1].parse().unwrap(), BTreeMap::new()));
            i += 2;
            continue;
        }
        let (p, id) = args[i].split_once('=').unwrap();
        maps.last_mut()
            .unwrap()
            .1
            .insert(p.parse().unwrap(), id.parse().unwrap());
        i += 1;
    }

    let le = data[..4] == [0xd4, 0xc3, 0xb2, 0xa1];
    let rd = |b: &[u8]| {
        let a = [b[0], b[1], b[2], b[3]];
        if le {
            u32::from_le_bytes(a)
        } else {
            u32::from_be_bytes(a)
        }
    };
    assert_eq!(rd(&data[20..24]), 1, "expects Ethernet link type");
    let mut off = 24;
    let mut t0 = None;
    let mut next_seq: BTreeMap<(usize, u16, u16), u8> = BTreeMap::new();
    let mut ids: BTreeMap<(usize, u8, usize), usize> = BTreeMap::new();
    let (mut total, mut seq_bad) = (0, 0);
    while off + 16 <= data.len() {
        let t = rd(&data[off..]) as f64 + rd(&data[off + 4..]) as f64 / 1e6;
        let cl = rd(&data[off + 8..]) as usize;
        let pkt = &data[off + 16..off + 16 + cl];
        off += 16 + cl;
        let ip = &pkt[14..];
        if ip[9] != 17 {
            continue;
        }
        let ihl = (ip[0] & 15) as usize * 4;
        let udp = &ip[ihl..];
        let sp = u16::from_be_bytes([udp[0], udp[1]]);
        let dp = u16::from_be_bytes([udp[2], udp[3]]);
        let payload = &udp[8..];
        let rel = t - *t0.get_or_insert(t);
        let run = maps.iter().rposition(|(s, _)| rel >= *s).unwrap();
        let Some(&recipient) = maps[run].1.get(&dp) else {
            continue;
        };
        let (seq, id, _) = decode_udp(payload, udp_key(recipient)).unwrap();
        total += 1;
        let want = next_seq.entry((run, sp, dp)).or_insert(0);
        if seq != *want {
            seq_bad += 1;
        }
        *want = seq.wrapping_add(1);
        *ids.entry((run, id, payload.len())).or_default() += 1;
    }
    println!("datagrams {total}, seq mismatches {seq_bad}");
    for ((run, id, len), n) in ids {
        println!("  map {run}: id {id} len {len} x{n}");
    }
}
