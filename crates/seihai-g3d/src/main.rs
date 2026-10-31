//! Seihai G3D — a client for Graal3D (February 2000).
//!
//!   seihai-g3d --online host[:port] --account A --password-env VAR
//!              [--nick N] [--no-udp] [--levels DIR] [--first-level]
//!              [--say TEXT] [--walk TICKS] [--seconds N]
//!   seihai-g3d --level-info FILE.x
//!
//! Headless for now: it logs in, logs what the server sends (own props,
//! the level and its mesh counts, other players, chat) and saves each level
//! file. `--say` sends one chat line once the level is in; `--walk` steps
//! forward 8 units a tick for that many ticks, as the real client walks.
//! The port defaults to the shipped server's.

use seihai_g3d::props::Player;
use seihai_g3d::session::{self, Event, Login, Session, TICK};
use seihai_g3d::xfile::XFile;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Default)]
struct Args {
    online: Option<String>,
    account: Option<String>,
    password_env: Option<String>,
    nick: Option<String>,
    no_udp: bool,
    levels: Option<String>,
    first_level: bool,
    say: Option<String>,
    walk: u32,
    seconds: Option<f64>,
    level_info: Option<String>,
}

fn parse() -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(f) = it.next() {
        let mut val = |f: &str| it.next().ok_or_else(|| format!("{f} needs a value"));
        match f.as_str() {
            "--online" | "-o" | "--server" => a.online = Some(val(&f)?),
            "--account" => a.account = Some(val(&f)?),
            "--password-env" => a.password_env = Some(val(&f)?),
            "--nick" => a.nick = Some(val(&f)?),
            "--levels" => a.levels = Some(val(&f)?),
            "--say" => a.say = Some(val(&f)?),
            "--walk" => a.walk = val(&f)?.parse().map_err(|_| "--walk needs a number")?,
            "--seconds" => {
                a.seconds = Some(val(&f)?.parse().map_err(|_| "--seconds needs a number")?)
            }
            "--level-info" => a.level_info = Some(val(&f)?),
            "--no-udp" => a.no_udp = true,
            "--first-level" => a.first_level = true,
            // Flags meant for the other Seihai binaries, passed through by
            // the launcher.
            "--version" | "--assets" => {
                val(&f)?;
            }
            _ => {}
        }
    }
    Ok(a)
}

fn main() -> ExitCode {
    let a = match parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("seihai-g3d: {e}");
            return ExitCode::from(2);
        }
    };
    if let Some(f) = &a.level_info {
        return match std::fs::read(f)
            .map_err(|e| e.to_string())
            .and_then(|b| XFile::parse(&b).map_err(|e| e.to_string()))
        {
            Ok(x) => {
                println!(
                    "{f}: {} meshes, {} vertices, {} faces",
                    x.meshes.len(),
                    x.vertex_count(),
                    x.face_count()
                );
                println!("textures: {}", x.textures().join(", "));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("seihai-g3d: {f}: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let Some(addr) = a.online.as_deref().map(session::with_default_port) else {
        eprintln!("seihai-g3d: needs --online host[:port] (or --level-info FILE)");
        return ExitCode::from(2);
    };
    let password = a
        .password_env
        .as_deref()
        .and_then(|v| std::env::var(v).ok())
        .unwrap_or_default();
    let account = a.account.clone().unwrap_or_else(|| "guest".into());
    let nick = a.nick.clone().unwrap_or_else(|| account.clone());
    let me = Player {
        nick,
        ..Player::default()
    };
    let dir = a
        .levels
        .clone()
        .map(Into::into)
        .unwrap_or_else(|| session::default_level_dir(None));
    let mut s = match Session::connect(
        &addr,
        &Login { account, password },
        me,
        !a.no_udp,
        Some(dir),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seihai-g3d: {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("connected to {addr}, udp port {}", s.me.udp_port);
    let start = Instant::now();
    let mut have_level = false;
    let mut said = false;
    let mut walked = 0;
    let mut ok = true;
    loop {
        let ev = match s.tick() {
            Ok(ev) => ev,
            Err(e) => {
                eprintln!("seihai-g3d: {e}");
                return ExitCode::FAILURE;
            }
        };
        for e in &ev {
            log(&s, e);
            match e {
                Event::Level { .. } => have_level = true,
                Event::DiscMessage(_) => ok = false,
                _ => {}
            }
        }
        if s.is_closed() {
            break;
        }
        if have_level {
            if let (Some(t), false) = (&a.say, said) {
                s.say(t).ok();
                said = true;
            }
            if walked < a.walk {
                // Forward along x, as with the camera behind the player.
                s.me.x = (s.me.x + 8.0).min(4000.0);
                s.me.anim = s.me.anim.wrapping_add(1);
                walked += 1;
            }
            if a.first_level && a.seconds.is_none() && walked >= a.walk {
                // One more tick to flush the last props.
                s.tick().ok();
                break;
            }
        }
        if a.seconds
            .is_some_and(|n| start.elapsed().as_secs_f64() >= n)
        {
            break;
        }
        std::thread::sleep(TICK);
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn log(s: &Session, e: &Event) {
    match e {
        Event::Accepted => println!("accepted"),
        Event::OwnProps { id, props } => println!("own props: player {id}, {} props", props.len()),
        Event::Level { name, file: None } => println!("level {name}: already have it"),
        Event::Level {
            name,
            file: Some(f),
        } => {
            let saved = f
                .path
                .as_ref()
                .map(|p| format!(", saved to {}", p.display()))
                .unwrap_or_default();
            println!(
                "level {name}: {} bytes, {} vertices, {} faces{saved}",
                f.bytes, f.vertices, f.faces
            );
            if !f.textures.is_empty() {
                println!("level {name}: textures {}", f.textures.join(", "));
            }
            if let Some(err) = &f.error {
                println!("level {name}: {err}");
            }
        }
        Event::Leader => println!("first in the level"),
        Event::PlayerJoined { id } => println!("player {id} seen"),
        Event::PlayerProps { id, props } => {
            let p = &s.others.get(id);
            let ids: Vec<String> = props.iter().map(|p| p.id().to_string()).collect();
            println!(
                "player {id} ({}): props {}",
                p.map(|p| p.nick.as_str()).unwrap_or(""),
                ids.join(",")
            );
        }
        Event::PlayerLeft { id } => println!("player {id} left"),
        Event::Chat {
            id,
            nick,
            text,
            udp,
        } => {
            println!(
                "chat {nick}: {text} (player {id}, {})",
                if *udp { "udp" } else { "tcp" }
            )
        }
        Event::UdpPosition { id } => {
            if let Some(p) = s.others.get(id) {
                println!("player {id} at ({}, {}, {}) via udp", p.x, p.y, p.z);
            }
        }
        Event::AdminMessage(t) => println!("admin message: {t}"),
        Event::DiscMessage(t) => println!("disconnect message: {t}"),
        Event::Closed => println!("connection closed"),
    }
}
