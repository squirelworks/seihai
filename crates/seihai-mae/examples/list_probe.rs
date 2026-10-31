//! Ask a lab listserver for its list the 2.05 way and print the reply.
//!
//! Usage: cargo run -p seihai-mae --example list_probe -- <host:port> [--raw]
//!
//! Sends id 0 `newmain` in a zlib frame (2.05 needs no account for the
//! list), reads the first frame and runs `parse_list_reply` on it.
//! Hosts are printed as `<host>`. `--raw` also prints the frame bytes,
//! which can hold host names: keep that output in the lab.

use seihai::protocol::{parse_list_reply, read_zlib_frame, split_messages, write_zlib_frame};
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let addr = args.first().expect("host:port");
    let raw = args.iter().any(|a| a == "--raw");

    let mut s = TcpStream::connect(addr).expect("connect");
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(&write_zlib_frame(b" newmain\n").unwrap())
        .unwrap();

    let body = read_zlib_frame(&mut s).expect("first frame");
    println!("body {} bytes after inflate", body.len());
    let ids: Vec<u8> = split_messages(&body).iter().map(|m| m.id).collect();
    println!("message ids {ids:?}");
    if raw {
        println!(
            "body hex {}",
            body.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
    }
    let reply = parse_list_reply(&body);
    for e in &reply.servers {
        println!(
            "server kind {:?} name {:?} lang {:?} version {:?} players {:?} host <host> port {}",
            e.kind, e.name, e.language, e.version, e.players, e.port
        );
    }
    println!(
        "status {} site_url {} error {:?} upgrade_url {}",
        reply.status.is_some(),
        reply.site_url.is_some(),
        reply.error,
        reply.upgrade_url.is_some()
    );
}
