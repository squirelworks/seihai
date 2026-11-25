//! Consumer mode's join loop with no window, for a lab check.
//!
//! ```text
//! cargo run -p seihai-play --example consumer_probe -- HOST:PORT [--3d] [--account NAME]
//! ```
//!
//! It runs the same steps as the start window in consumer mode
//! (`seihai_play::consumer`). It logs in as `first_version`, reads a version
//! refusal's `Allowed:` list, and logs in again as `after_refusal` says,
//! until the server sends a level or the loop stops. When the start window
//! would ask the player, the probe prints the options and takes the first
//! (recommended) one. Each login is core's session (`client::run`, first
//! level only).
//!
//! The password is read from `GRAAL_PW` only; without it a dummy is sent,
//! which a lab server with `noverifylogin` accepts. The last line is
//! `RESULT joined <version>` or `RESULT stopped: <why>`.

use seihai_core::client::{self, Login};
use seihai_play::consumer::{self, Next, Pending, MAX_TRIES};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(addr) = args.next() else {
        eprintln!("usage: consumer_probe HOST:PORT [--3d] [--account NAME]");
        std::process::exit(2);
    };
    let mut three_d = false;
    let mut account = "seihaiprobe".to_string();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--3d" => three_d = true,
            "--account" => account = args.next().unwrap_or(account),
            _ => {
                eprintln!("unknown argument {a}");
                std::process::exit(2);
            }
        }
    }
    let login = Login {
        account,
        password: std::env::var("GRAAL_PW").unwrap_or_else(|_| "probe".into()),
        ..Login::default()
    };
    let mut pending = Pending {
        server: addr.clone(),
        three_d,
        ..Pending::default()
    };
    let mut v = consumer::first_version(None, three_d);
    for _ in 0..=MAX_TRIES {
        pending.tried = v.label().to_string();
        println!("TRY {} ({})", v.label(), v.token());
        let mut log = Vec::new();
        let mut joined = false;
        let r = client::run(&addr, v, &login, None, true, &mut log, &mut |_, _| {
            joined = true;
        });
        let text = String::from_utf8_lossy(&log).to_string();
        let reason = match &r {
            Err(e) => Some(e.clone()),
            Ok(()) => text.rfind("connection closed: ").map(|at| {
                text[at + "connection closed: ".len()..]
                    .trim_end()
                    .to_string()
            }),
        };
        if joined {
            println!("RESULT joined {}", v.label());
            return;
        }
        let msg = reason.unwrap_or_else(|| "closed with no message".into());
        println!("CLOSED {}", msg.replace(['\r', '\n'], " | "));
        match consumer::after_refusal(&mut pending, &format!("disconnected: {msg}")) {
            Next::Join(next) => {
                println!("PICK {}", next.label());
                v = next;
            }
            Next::Ask(options) => {
                let shown: Vec<String> =
                    options.iter().map(|&o| consumer::option_text(o)).collect();
                println!("ASK {}", shown.join(" / "));
                v = options[0];
            }
            Next::Stop(why) => {
                println!("RESULT stopped: {why}");
                return;
            }
            Next::Done => {
                println!("RESULT stopped: not a version refusal");
                return;
            }
        }
    }
    println!("RESULT stopped: too many tries");
}
