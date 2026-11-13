//! One server-list fetch, no window: the request Ato's Start sends.
//!
//! ```text
//! cargo run -p seihai-ato --example list_probe -- 3.04 listserver.graal.in:14922 <made-up account>
//! ```
//!
//! Logs in once at the listserver with the build's own listserver login
//! (core's `Connection::fetch_server_list`, as `seihai-play::servers` calls
//! it), prints what came back, and exits. The password is a fixed dummy:
//! use a made-up account, never a real one. Meant for one polite check of
//! a public listserver or a lab one; it does not retry.

use seihai_core::net::{list_failure_text, Connection};
use seihai_core::protocol::ClientVersion;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(ver), Some(addr), Some(account)) = (args.next(), args.next(), args.next()) else {
        eprintln!("usage: list_probe <version> <host:port> <made-up account>");
        std::process::exit(2);
    };
    let Some(version) = ClientVersion::from_label(&ver) else {
        eprintln!("unknown version {ver}");
        std::process::exit(2);
    };
    let list = version.list_version();
    println!(
        "build {} (list login of {}), token {:?}, shape {:?}, asking {addr} as {account:?}",
        version.label(),
        list.label(),
        list.listserver().token,
        list.listserver().shape
    );
    let t = Instant::now();
    match Connection::fetch_server_list(&addr, version, &account, "x") {
        Ok(r) => {
            println!("reply after {:?}", t.elapsed());
            println!("status: {:?}", r.status);
            println!("error: {:?}", r.error);
            println!("site_url: {:?}", r.site_url);
            println!("upgrade_url: {:?}", r.upgrade_url);
            println!("servers: {}", r.servers.len());
            for (i, s) in r.servers.iter().enumerate() {
                println!("{}", s.display_line(i + 1));
            }
        }
        Err(e) => {
            println!("after {:?}: {}", t.elapsed(), list_failure_text(&addr, &e));
            std::process::exit(1);
        }
    }
}
