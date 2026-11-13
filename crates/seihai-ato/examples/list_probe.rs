//! One server-list fetch, no window: the request Ato's Start sends.
//!
//! ```text
//! cargo run -p seihai-ato --example list_probe -- 3.04 listserver.graal.in:14922 <made-up account>
//! ```
//!
//! Logs in once at the listserver with the build's own listserver login
//! (core's `Connection::fetch_server_list`, as `seihai-play::servers` calls
//! it), prints what came back, and exits. Meant for one polite check of a
//! public listserver or a lab one; it does not retry.
//!
//! The account is the third argument, or `GRAAL_ACCOUNT` when it is left
//! out. The password is read from `GRAAL_PW` only (never an argument, so it
//! stays out of shell history); without it a fixed dummy is sent, which is
//! what a made-up account wants. Neither is printed.
//!
//! Run with no arguments (a double-click on Windows), it asks for the
//! account and the password (typed without echo), logs in once at
//! `listserver.graal.in` on 14922 and once on 14923 with the 3.04 login,
//! prints only accepted or rejected and the number of servers per port, and
//! waits for Enter before it closes. The password lives only in memory.

use seihai_core::net::{list_failure_text, Connection};
use seihai_core::protocol::ClientVersion;
use std::time::Instant;

fn main() {
    if std::env::args().len() == 1 {
        return interactive();
    }
    let mut args = std::env::args().skip(1);
    let (Some(ver), Some(addr)) = (args.next(), args.next()) else {
        eprintln!("usage: list_probe <version> <host:port> [account]  (or GRAAL_ACCOUNT; password from GRAAL_PW)");
        std::process::exit(2);
    };
    let Some(account) = args.next().or_else(|| std::env::var("GRAAL_ACCOUNT").ok()) else {
        eprintln!("no account: pass it as the third argument or set GRAAL_ACCOUNT");
        std::process::exit(2);
    };
    let password = std::env::var("GRAAL_PW").ok().filter(|p| !p.is_empty());
    let Some(version) = ClientVersion::from_label(&ver) else {
        eprintln!("unknown version {ver}");
        std::process::exit(2);
    };
    let list = version.list_version();
    println!(
        "build {} (list login of {}), token {:?}, shape {:?}, asking {addr}, password {}",
        version.label(),
        list.label(),
        list.listserver().token,
        list.listserver().shape,
        if password.is_some() {
            "from GRAAL_PW"
        } else {
            "dummy"
        }
    );
    let t = Instant::now();
    match Connection::fetch_server_list(
        &addr,
        version,
        &account,
        password.as_deref().unwrap_or("x"),
    ) {
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

/// The double-click mode: prompts, then either the 3.04 login on both
/// ports or every version profile's own list login on both ports, one
/// connection each, a short verdict per line.
fn interactive() {
    use std::io::Write;
    let ask = |q: &str| {
        print!("{q}");
        let _ = std::io::stdout().flush();
        let mut a = String::new();
        let _ = std::io::stdin().read_line(&mut a);
        a.trim().to_string()
    };
    println!("Seihai Ato listserver check (the public listserver)");
    let account = ask("Account: ");
    print!("Password (not shown): ");
    let _ = std::io::stdout().flush();
    let mut password = String::new();
    echo(false);
    let _ = std::io::stdin().read_line(&mut password);
    echo(true);
    println!();
    let pw = password.trim_end_matches(['\r', '\n']).to_string();
    password.clear();
    println!("What to check:");
    println!("  Enter = the 3.04 list login on both listserver ports");
    println!("  y     = every version profile's list login on both listserver ports");
    println!("  C     = every 4.x to 6.x build's login at the community Login server");
    println!("          (the Login entry of the community server list)");
    let choice = ask("Choice: ");
    let login_at = if choice.eq_ignore_ascii_case("c") {
        Some(community_login(&account, &pw).unwrap_or_default())
    } else {
        None
    };
    if let Some(addr) = login_at {
        if !addr.is_empty() {
            login_servers(&addr, &account, &pw);
        }
        drop(pw);
        print!("Done. Press Enter to close.");
        let _ = std::io::stdout().flush();
        let _ = std::io::stdin().read_line(&mut String::new());
        return;
    }
    let all = choice.eq_ignore_ascii_case("y");
    // One row per distinct list login: builds with no listserver of their
    // own borrow another's, so they are asked once, under that build.
    let mut builds: Vec<ClientVersion> = Vec::new();
    if all {
        for v in ClientVersion::all_versions() {
            let l = v.list_version();
            if !builds.contains(&l) {
                builds.push(l);
            }
        }
    } else {
        builds.push(ClientVersion::V304);
    }
    println!(
        "{} list login(s), 2 ports each, 10 s apart (about 7 minutes); stops at the first sign of throttling.",
        builds.len()
    );
    let mut first = true;
    'run: for v in builds {
        for port in [14922u16, 14923] {
            if !first {
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
            first = false;
            let addr = format!("{}:{port}", v.listserver_host());
            let verdict = match Connection::fetch_server_list(&addr, v, &account, &pw) {
                Ok(r) => match (&r.error, r.servers.len()) {
                    (Some(e), _) => format!("rejected: {e}"),
                    (None, 0) => "no error and no servers".to_string(),
                    (None, n) => format!("accepted, {n} servers listed"),
                },
                Err(e) => list_failure_text(&addr, &e),
            };
            println!("{:<8} port {port}: {verdict}", v.label());
            let _ = std::io::stdout().flush();
            if throttled(&verdict) {
                println!("Stopping: that reply looks like throttling or a ban.");
                break 'run;
            }
        }
    }
    drop(pw);
    print!("Done. Press Enter to close.");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
}

/// The community's Login server: the entry whose name starts with "login"
/// in the list the 3.04 login gets (one list login on 14923). Prints the
/// list's names when there is none. The address is not printed.
fn community_login(account: &str, pw: &str) -> Option<String> {
    let v = ClientVersion::V304;
    let addr = format!("{}:14923", v.listserver_host());
    let reply = match Connection::fetch_server_list(&addr, v, account, pw) {
        Ok(r) => r,
        Err(e) => {
            println!("server list: {}", list_failure_text(&addr, &e));
            return None;
        }
    };
    if let Some(e) = reply.error {
        println!("server list: rejected: {e}");
        return None;
    }
    let found = reply
        .servers
        .iter()
        .find(|s| s.name.to_ascii_lowercase().starts_with("login"));
    match found {
        Some(s) => {
            println!("Community Login server found in the list: \"{}\".", s.name);
            std::thread::sleep(std::time::Duration::from_secs(10));
            Some(format!("{}:{}", s.host, s.port))
        }
        None => {
            println!("No server named Login in the list. The list has:");
            for s in &reply.servers {
                println!("  {}", s.name);
            }
            None
        }
    }
}

/// One game login per lobby build (4.04r2 to 6.03r7) at the Login server,
/// 10 s apart: read what comes back for a few seconds, then disconnect.
/// Prints only the verdict per build; never the host, account or password.
fn login_servers(addr: &str, account: &str, pw: &str) {
    use seihai_core::net::{login_gserver, random_key};
    use seihai_core::protocol::split_messages;
    use std::io::Write;
    let builds: Vec<ClientVersion> = ClientVersion::all_versions()
        .iter()
        .copied()
        .filter(|v| v.profile().protocol.lobby.is_some())
        .collect();
    println!(
        "{} builds, one login each at the Login server, 10 s apart (about {} minutes); stops at the first sign of throttling.",
        builds.len(),
        (builds.len() * 15).div_ceil(60)
    );
    for (i, v) in builds.into_iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
        let verdict = match login_gserver(addr, v, random_key(), account, pw, false) {
            Err(e) => format!("failed: {e}"),
            Ok(mut s) => {
                let mut seen = Vec::new();
                let mut verdict = None;
                let start = Instant::now();
                while verdict.is_none() && start.elapsed().as_secs() < 8 {
                    let body = match s.conn.read_body() {
                        Ok(b) => b,
                        Err(e) => {
                            verdict = Some(if seen.is_empty() {
                                format!("closed with no reply ({})", e.kind())
                            } else {
                                format!("closed after {} messages ({})", seen.len(), e.kind())
                            });
                            break;
                        }
                    };
                    for m in split_messages(&body) {
                        match m.id {
                            16 => {
                                verdict = Some(format!(
                                    "rejected: {}",
                                    String::from_utf8_lossy(&m.payload).trim()
                                ))
                            }
                            178 => verdict = Some("accepted, sent a server warp (178)".to_string()),
                            _ => {}
                        }
                        seen.push(m.id);
                    }
                }
                verdict.unwrap_or_else(|| {
                    let lvl = seen.contains(&6);
                    format!(
                        "accepted: {} messages in 8 s{}",
                        seen.len(),
                        if lvl { ", a level sent" } else { "" }
                    )
                })
            }
        };
        println!("{:<13} {verdict}", v.label());
        let _ = std::io::stdout().flush();
        if throttled(&verdict) {
            println!("Stopping: that reply looks like throttling or a ban.");
            break;
        }
    }
}

/// A reply that looks like the listserver pushing back: stop asking.
fn throttled(verdict: &str) -> bool {
    let v = verdict.to_ascii_lowercase();
    [
        "ban",
        "too many",
        "too fast",
        "flood",
        "limit",
        "try again",
        "wait",
        "reset",
        "refused",
    ]
    .iter()
    .any(|w| v.contains(w))
}

/// Turn the console's echo of typed keys off or back on.
#[cfg(windows)]
fn echo(on: bool) {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(n: u32) -> *mut c_void;
        fn GetConsoleMode(h: *mut c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(h: *mut c_void, mode: u32) -> i32;
    }
    const STD_INPUT_HANDLE: u32 = -10i32 as u32;
    const ENABLE_ECHO_INPUT: u32 = 0x4;
    // SAFETY: plain Win32 console calls on this process's own stdin handle.
    unsafe {
        let h = GetStdHandle(STD_INPUT_HANDLE);
        let mut mode = 0u32;
        if GetConsoleMode(h, &mut mode) != 0 {
            let mode = if on {
                mode | ENABLE_ECHO_INPUT
            } else {
                mode & !ENABLE_ECHO_INPUT
            };
            SetConsoleMode(h, mode);
        }
    }
}

/// Turn the terminal's echo of typed keys off or back on.
#[cfg(not(windows))]
fn echo(on: bool) {
    let _ = std::process::Command::new("stty")
        .arg(if on { "echo" } else { "-echo" })
        .stdin(std::process::Stdio::inherit())
        .status();
}
