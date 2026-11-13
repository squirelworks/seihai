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

/// The double-click mode: prompts, one 3.04 list login per port, a short
/// verdict per port.
fn interactive() {
    use std::io::Write;
    println!("Seihai Ato listserver check (3.04 login, listserver.graal.in)");
    print!("Account: ");
    let _ = std::io::stdout().flush();
    let mut account = String::new();
    let _ = std::io::stdin().read_line(&mut account);
    let account = account.trim().to_string();
    print!("Password (not shown): ");
    let _ = std::io::stdout().flush();
    let mut password = String::new();
    echo(false);
    let _ = std::io::stdin().read_line(&mut password);
    echo(true);
    println!();
    let pw = password.trim_end_matches(['\r', '\n']).to_string();
    password.clear();
    for (i, port) in [14922, 14923].into_iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
        let addr = format!("listserver.graal.in:{port}");
        let verdict = match Connection::fetch_server_list(&addr, ClientVersion::V304, &account, &pw)
        {
            Ok(r) => match (&r.error, r.servers.len()) {
                (Some(e), _) => format!("rejected: {e}"),
                (None, 0) => "no error and no servers".to_string(),
                (None, n) => format!("accepted, {n} servers listed"),
            },
            Err(e) => list_failure_text(&addr, &e),
        };
        println!("port {port}: {verdict}");
    }
    drop(pw);
    print!("Done. Press Enter to close.");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
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
