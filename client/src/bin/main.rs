//! Seihai offline viewer.
//!
//! Cross-platform (Windows / Linux / macOS). Uses a software framebuffer
//! and minifb, so no system SDL2 is required.
//!
//! Usage:
//!   seihai --assets <dir> [--level <name.nw>]
//!
//! The asset dir should contain pics1.png and optionally tiledefs.txt
//! and a levels/ subdirectory.

use seihai::assets::{read_png_rgb, read_png_rgba, AssetRoot};
use seihai::net::Connection;
use seihai::offline::OfflineSession;
use seihai::protocol::{parse_list_reply, ClientVersion, EncryptionGen, SessionState};
use seihai::settings::Settings;
use seihai::render::{
    draw_other_player, draw_player_marker, draw_player_sprites, player_box_origin, render_view,
    Framebuffer,
};
use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut assets: Option<PathBuf> = None;
    let mut level_name: Option<String> = None;
    let mut online: Option<String> = None;
    let mut account = String::from("player");
    let mut password = String::new();
    let mut version = ClientVersion::V205;
    let mut i = 0;
    let mut cli_mode = false;
    while i < args.len() {
        match args[i].as_str() {
            "--assets" | "-a" => {
                i += 1;
                assets = args.get(i).map(PathBuf::from);
            }
            "--level" | "-l" => {
                i += 1;
                level_name = args.get(i).cloned();
                cli_mode = true;
            }
            "--online" | "-o" => {
                i += 1;
                online = args.get(i).cloned();
                cli_mode = true;
            }
            "--account" => {
                i += 1;
                if let Some(a) = args.get(i) {
                    account = a.clone();
                }
            }
            "--password" => {
                i += 1;
                if let Some(p) = args.get(i) {
                    password = p.clone();
                }
            }
            "--version" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    version = match v.as_str() {
                        "1.39" => ClientVersion::V139,
                        "2.17" => ClientVersion::V217,
                        "2.22" => ClientVersion::V222,
                        "2.31" => ClientVersion::V231,
                        _ => ClientVersion::V205,
                    };
                }
            }
            "--help" | "-h" => {
                println!("seihai --assets <dir> [--level <name.nw>]");
                println!("       seihai --assets <dir> --online host:port --account A --password P [--version 2.05]");
                println!("       seihai                  (menu if no mode given)");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Load settings and fill defaults.
    let mut settings = Settings::load();
    if assets.is_none() {
        assets = settings.assets.clone();
    }
    if account == "player" {
        if let Some(a) = &settings.account {
            account = a.clone();
        }
    }
    if password.is_empty() {
        if let Some(p) = &settings.password {
            password = p.clone();
        }
    }
    // Use the version saved by the GUI if no CLI --version was given.
    if version == ClientVersion::V205 {
        if let Some(v) = &settings.version {
            version = match v.as_str() {
                "2.22" => ClientVersion::V222,
                "2.31" => ClientVersion::V231,
                "2.17" => ClientVersion::V217,
                "1.39" => ClientVersion::V139,
                _ => ClientVersion::V205,
            };
        }
    }
    // Do not auto-fill online from settings. The GUI is the waiting area.
    // Settings only pre-fill the form.

    // GUI when no CLI mode is given.
    if !cli_mode {
        if let Some(addr) = seihai::gui::run() {
            online = Some(addr);
            // Fall through to the game connection.
        } else {
            return;
        }
    }

    let root = match assets {
        Some(p) => AssetRoot::new(p),
        None => {
            eprintln!("usage: seihai --assets <dir> [--level <name.nw>]");
            return;
        }
    };
    if !root.is_valid() {
        eprintln!("no pics1.png in {}", root.pics1().display());
        return;
    }

    let sheet_data = match std::fs::read(root.pics1()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read pics1.png: {e}");
            return;
        }
    };
    let (sw, sh, sheet) = match read_png_rgb(&sheet_data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse pics1.png: {e}");
            return;
        }
    };

    let mut session = OfflineSession::new();
    let mut net_state = SessionState::default();
    let mut conn: Option<Connection> = None;
    let mut frames = None;
    let mut loaded_level: Option<String> = None;

    if let Some(addr) = &online {
        // If this looks like a listserver, fetch the list and pick a gserver.
        if addr.contains("14921") || addr.contains("14922") || addr.contains("listserver") {
            match Connection::connect(addr) {
                Ok(mut c) => {
                    let key = 0x40u8;
                    if c.login_versioned(version, key, &account, &password).is_ok() {
                        let mut state = SessionState::default();
                        if let Ok(body) = c.read_frame_into(&mut state) {
                            let reply = parse_list_reply(&body);
                            if !reply.servers.is_empty() {
                                println!("Servers:");
                                for (i, e) in reply.servers.iter().enumerate() {
                                    println!("  {i}) {} ({}:{})", e.name, e.host, e.port);
                                }
                                println!("pick a number:");
                                let mut pick = String::new();
                                let _ = std::io::stdin().read_line(&mut pick);
                                if let Ok(n) = pick.trim().parse::<usize>() {
                                    if let Some(e) = reply.servers.get(n) {
                                        online = Some(format!("{}:{}", e.host, e.port));
                                        settings.last_server = online.clone();
                                        settings.save();
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("listserver failed: {e}"),
            }
        }
    }

    if let Some(addr) = &online {
        if version.encryption() != EncryptionGen::Gen3 {
            eprintln!("online mode reads gen-3 (2.05/2.17) sessions only; trying anyway");
        }
        match Connection::connect(addr) {
            Ok(mut c) => {
                // The real client sends `rand & 0x7F`; a fixed even key
                // would always put the gen-3 byte at index 0 of short lines.
                let key = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(1)
                    & 0x7F) as u8;
                if let Err(e) = c.login_versioned(version, key, &account, &password) {
                    eprintln!("login failed: {e}");
                    if !cli_mode {
                        let _ = seihai::gui::run_with_error(format!("login failed: {e}"));
                        return;
                    }
                } else {
                    eprintln!("logged in to {addr} as {account}");
                    match c.spawn_reader(version.encryption() == EncryptionGen::Gen5, key) {
                        Ok(rx) => frames = Some(rx),
                        Err(e) => eprintln!("reader failed: {e}"),
                    }
                    conn = Some(c);
                }
            }
            Err(e) => {
                eprintln!("connect failed: {e}");
                if !cli_mode {
                    let _ = seihai::gui::run_with_error(format!("connect failed: {e}"));
                    return;
                }
            }
        }
    }

    if let Some(name) = level_name {
        let path = root.level(&name);
        if let Err(e) = session.load_level(&path) {
            eprintln!("failed to load level: {e}");
        }
    }
    if root.tiledefs().is_file() {
        let _ = session.load_tiletypes(root.tiledefs());
    } else if root.arrays_dat().is_file() {
        let _ = session.load_tiletypes_arrays(root.arrays_dat());
    }

    let view_w = 40u32;
    let view_h = 30u32;
    let load = |dir: &str, name: &str| {
        let data = std::fs::read(root.find_image(dir, name)?).ok()?;
        read_png_rgba(&data).ok().map(|(w, _, px)| (w, px))
    };
    let body = load("bodies", "body.png");
    let head = load("heads", "head0.png");
    let sprites = load("", "sprites.png");
    if body.is_none() || head.is_none() {
        eprintln!("no body.png / head0.png found; drawing a marker for the player");
    }
    let draw = |session: &OfflineSession, others: &[seihai::protocol::OtherPlayer]| -> (Framebuffer, Vec<u32>) {
        let mut fb = render_view(session, &sheet, sw, sh, view_w, view_h);
        let (cam_x, cam_y) = (
            ((session.player_x + 1.5) * 16.0 - (view_w as f32 * 16.0) / 2.0) as i32,
            ((session.player_y + 1.5) * 16.0 - (view_h as f32 * 16.0) / 2.0) as i32,
        );
        // Other players first, so the local player draws on top.
        if let (Some((bw, b)), Some((hw, h))) = (&body, &head) {
            for other in others {
                draw_other_player(
                    &mut fb,
                    b,
                    *bw,
                    4,
                    h,
                    *hw,
                    4,
                    sprites.as_ref().map(|(w, p)| (p.as_slice(), *w, 4)),
                    other.dir,
                    other.x,
                    other.y,
                    cam_x,
                    cam_y,
                );
            }
        }
        let (dx, dy) = player_box_origin(session, view_w, view_h);
        match (&body, &head) {
            (Some((bw, b)), Some((hw, h))) => draw_player_sprites(
                &mut fb,
                b,
                *bw,
                4,
                h,
                *hw,
                4,
                sprites.as_ref().map(|(w, p)| (p.as_slice(), *w, 4)),
                session.player_dir,
                dx,
                dy,
            ),
            _ => draw_player_marker(&mut fb, session.in_water, dx, dy),
        }
        fb.draw_hud(session.player_health, 0);
        // Convert RGB to minifb's 0RGB u32.
        let buffer = fb
            .pixels
            .chunks(3)
            .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
            .collect();
        (fb, buffer)
    };
    let (fb, mut buffer) = draw(&session, &[]);

    let mut window = match minifb::Window::new(
        "Seihai",
        fb.width as usize,
        fb.height as usize,
        minifb::WindowOptions {
            resize: false,
            ..minifb::WindowOptions::default()
        },
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("failed to open window: {e}");
            return;
        }
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    const STEP_EVERY: Duration = Duration::from_millis(50);
    let mut last_step = Instant::now();
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        // Online: apply every frame that has arrived, without blocking.
        if let Some(rx) = &frames {
            let mut changed = false;
            loop {
                match rx.try_recv() {
                    Ok(Ok(body)) => {
                        net_state.apply_body(&body);
                        // Write any completed files and reload levels.
                        let written = net_state.flush_files(&root.levels_dir());
                        for name in &written {
                            if name.ends_with(".nw") {
                                let _ = session.load_level(root.level(name));
                                eprintln!("loaded {name}");
                            }
                        }
                        changed = true;
                    }
                    Ok(Err(e)) => {
                        eprintln!("connection closed: {e}");
                        if !cli_mode {
                            let _ = seihai::gui::run_with_error(format!("connection closed: {e}"));
                            return;
                        }
                        conn = None;
                        break;
                    }
                    Err(_) => break,
                }
            }
            if changed {
                if let Some(name) = &net_state.level_name {
                    if loaded_level.as_ref() != Some(name) {
                        // The server sends the file name, e.g. "onlinestartlocal.nw".
                        let file = if name.ends_with(".nw") {
                            name.clone()
                        } else {
                            format!("{name}.nw")
                        };
                        match session.load_level(root.level(&file)) {
                            Ok(()) => eprintln!("level {file}"),
                            Err(e) => {
                                eprintln!("level {file} not loaded ({e}); empty board");
                                session.current_level = None;
                            }
                        }
                        loaded_level = Some(name.clone());
                    }
                }
                // Take server positions once, so local walking is not
                // snapped back by later frames.
                let warped = net_state.player_x.is_some() || net_state.player_y.is_some();
                if let Some(x) = net_state.player_x.take() {
                    session.player_x = x;
                }
                if let Some(y) = net_state.player_y.take() {
                    session.player_y = y;
                }
                if let Some(d) = net_state.player_dir.take() {
                    session.player_dir = d;
                }
                if warped {
                    eprintln!(
                        "position {} {} dir {}",
                        session.player_x, session.player_y, session.player_dir
                    );
                }
                buffer = draw(&session, &net_state.others).1;
            }
        }

        // Arrow keys move the player half a tile per step, at most one
        // step every 50 ms.
        let mut moved = false;
        if last_step.elapsed() >= STEP_EVERY {
            let (mut dx, mut dy) = (0.0, 0.0);
            if window.is_key_down(minifb::Key::Left) {
                dx -= 0.5;
            }
            if window.is_key_down(minifb::Key::Right) {
                dx += 0.5;
            }
            if window.is_key_down(minifb::Key::Up) {
                dy -= 0.5;
            }
            if window.is_key_down(minifb::Key::Down) {
                dy += 0.5;
            }
            if dx != 0.0 || dy != 0.0 {
                session.try_move(dx, dy);
                moved = true;
                last_step = Instant::now();
            }
        }
        if moved {
            buffer = draw(&session, &net_state.others).1;
            // Online: send the new position as PLI_PLAYERPROPS.
            if let Some(c) = conn.as_mut() {
                if let Err(e) = c.send_walk(&session.player_update()) {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
        }
        let _ = window.update_with_buffer(&buffer, fb.width as usize, fb.height as usize);
    }
}
