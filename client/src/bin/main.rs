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

use seihai::assets::{read_png_rgb, read_png_rgba, write_png_rgb, AssetRoot};
use seihai::net::Connection;
use seihai::offline::OfflineSession;
use seihai::protocol::{ChatKind, ClientVersion, SessionState};
use seihai::settings::Settings;
use seihai::ui;
use seihai::render::{
    draw_other_player, draw_player_marker, draw_player_sprites, player_box_origin, render_view,
    Framebuffer,
};
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
            version = version_from_label(v).unwrap_or(ClientVersion::V205);
        }
    }
    // Do not auto-fill online from settings. The GUI is the waiting area.
    // Settings only pre-fill the form.

    let mut session = OfflineSession::new();
    let mut net_state = SessionState::default();
    let mut conn: Option<Connection> = None;
    let mut frames = None;
    let mut loaded_level: Option<String> = None;

    // GUI when no CLI mode is given. A failed connect or login goes back
    // to the GUI with the error, and a later Join tries again.
    let mut gui_error: Option<String> = None;
    loop {
        if !cli_mode {
            let pick = match gui_error.take() {
                Some(e) => seihai::gui::run_with_error(e),
                None => seihai::gui::run(),
            };
            let Some(addr) = pick else {
                return;
            };
            online = Some(addr);
            // The GUI saved its form; use it for this connection.
            let s = Settings::load();
            if s.assets.is_some() {
                assets = s.assets;
            }
            if let Some(a) = s.account {
                account = a;
            }
            if let Some(p) = s.password {
                password = p;
            }
            if let Some(v) = s.version.as_deref().and_then(version_from_label) {
                version = v;
            }
        }
        net_state.file_has_modtime = version.file_has_modtime();

        if let Some(addr) = &online {
            // If this looks like a listserver, fetch the list and pick a gserver.
            if addr.contains("14921") || addr.contains("14922") || addr.contains("listserver") {
                match Connection::fetch_server_list(addr, version, &account, &password) {
                    Ok(reply) => {
                        if let Some(err) = &reply.error {
                            eprintln!("listserver error: {err}");
                        }
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
                    Err(e) => eprintln!("listserver failed: {e}"),
                }
            }
        }

        if let Some(addr) = &online {
            match Connection::connect(addr) {
                Ok(mut c) => {
                    let key = seihai::net::random_key();
                    if let Err(e) = c.login_versioned(version, key, &account, &password) {
                        eprintln!("login failed: {e}");
                        if !cli_mode {
                            gui_error = Some(format!("login failed: {e}"));
                            continue;
                        }
                    } else {
                        eprintln!("logged in to {addr} as {account}");
                        match c.spawn_reader() {
                            Ok(rx) => frames = Some(rx),
                            Err(e) => eprintln!("reader failed: {e}"),
                        }
                        conn = Some(c);
                    }
                }
                Err(e) => {
                    eprintln!("connect failed: {e}");
                    if !cli_mode {
                        gui_error = Some(format!("connect failed: {e}"));
                        continue;
                    }
                }
            }
        }
        break;
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

    let offline_level = level_name.clone();
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
    let draw = |session: &OfflineSession, others: &[seihai::protocol::OtherPlayer], my_chat: &str| -> Framebuffer {
        let mut fb = render_view(session, &sheet, sw, sh, view_w, view_h);
        let (cam_x, cam_y) = (
            ((session.player_x + 1.5) * 16.0 - (view_w as f32 * 16.0) / 2.0) as i32,
            ((session.player_y + 1.5) * 16.0 - (view_h as f32 * 16.0) / 2.0) as i32,
        );
        // Other players first, so the local player draws on top.
        if body.is_none() || head.is_none() {
            for other in others {
                let ox = (other.x * 16.0) as i32 - cam_x;
                let oy = (other.y * 16.0) as i32 - cam_y;
                draw_player_marker(&mut fb, false, ox, oy);
            }
        }
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
        ui::draw_other_labels(&mut fb, others, (cam_x, cam_y));
        ui::draw_bubble(&mut fb, my_chat, dx, dy);
        fb.draw_hud(session.player_health, 0);
        if let Some(level) = &session.current_level {
            fb.draw_minimap(level, session.player_x, session.player_y);
        }
        fb
    };
    let mut my_chat = String::new();
    let mut scene = draw(&session, &[], &my_chat);
    let fb = scene.clone();
    let mut buffer: Vec<u32> = Vec::new();

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

    // Typed characters arrive through minifb's input callback.
    let typed: Arc<Mutex<Vec<char>>> = Arc::default();
    window.set_input_callback(Box::new(TypedChars(typed.clone())));

    let mut input = ui::Input::default();
    let mut chat_scroll = 0usize;
    let mut show_debug = false;
    let mut show_players = false;
    let mut frames_seen = 0usize;
    let mut fps = 0.0f32;
    let mut fps_count = 0u32;
    let mut fps_since = Instant::now();
    let mut dirty = true;

    const STEP_EVERY: Duration = Duration::from_millis(50);
    let mut last_step = Instant::now();
    while window.is_open() {
        let typed_now: Vec<char> = std::mem::take(&mut *typed.lock().unwrap());
        if input.is_open() {
            for c in typed_now {
                input.type_char(c);
            }
            if window.is_key_pressed(minifb::Key::Backspace, minifb::KeyRepeat::Yes) {
                input.backspace();
            }
            if window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No) {
                input.cancel();
            }
        } else if window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No) {
            break;
        }
        // Online: apply every frame that has arrived, without blocking.
        if let Some(rx) = &frames {
            let mut changed = false;
            loop {
                match rx.try_recv() {
                    Ok(Ok(body)) => {
                        frames_seen += 1;
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
                        // The server's own reason, when it sent one.
                        let why = match &net_state.disconnect_message {
                            Some(m) => format!("disconnected: {m}"),
                            None => format!("connection closed: {e}"),
                        };
                        eprintln!("{why}");
                        if !cli_mode {
                            let _ = seihai::gui::run_with_error(why);
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
                                eprintln!("level {file} not loaded ({e}); requesting");
                                session.current_level = None;
                                if let Some(c) = conn.as_mut() {
                                    let _ = c.send_wantfile(&file);
                                }
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
                dirty = true;
            }
        }

        // Arrow keys move the player half a tile per step, at most one
        // step every 50 ms.
        let mut moved = false;
        if !input.is_open() && last_step.elapsed() >= STEP_EVERY {
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
            dirty = true;
            // Online: send the new position as PLI_PLAYERPROPS.
            if let Some(c) = conn.as_mut() {
                if let Err(e) = c.send_walk(&session.player_update()) {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
        }

        // Enter opens the chat bar, and sends it when it is open. F2 opens
        // a PM to the nearest player.
        if window.is_key_pressed(minifb::Key::Enter, minifb::KeyRepeat::No) {
            if !input.is_open() {
                input.open(ui::InputMode::Chat);
            } else if let Some((mode, text)) = input.submit() {
                let sent = match mode {
                    ui::InputMode::Chat => match ui::classify_chat(&text) {
                        ui::ChatSend::Say(t) => {
                            my_chat = t.clone();
                            dirty = true;
                            if !t.is_empty() {
                                net_state.push_chat(ChatKind::Say, account.clone(), t.clone());
                            }
                            conn.as_mut().map(|c| c.send_chat(&t))
                        }
                        ui::ChatSend::Toall(t) if !t.is_empty() => {
                            net_state.push_chat(ChatKind::Toall, account.clone(), t.clone());
                            conn.as_mut().map(|c| c.send_toall(&t))
                        }
                        ui::ChatSend::Toall(_) => None,
                    },
                    ui::InputMode::Pm { id, name } if !text.trim().is_empty() => {
                        net_state.push_chat(ChatKind::Private, format!("to {name}"), text.clone());
                        conn.as_mut().map(|c| c.send_pm(&[id], &text))
                    }
                    ui::InputMode::Pm { .. } => None,
                };
                if let Some(Err(e)) = sent {
                    eprintln!("send failed: {e}");
                }
                chat_scroll = 0;
            }
        }
        if !input.is_open() {
            if window.is_key_pressed(minifb::Key::F2, minifb::KeyRepeat::No) {
                match ui::nearest(&net_state.others, session.player_x, session.player_y) {
                    Some(o) => {
                        let name = o.nick.clone().unwrap_or_else(|| format!("#{}", o.id));
                        input.open(ui::InputMode::Pm { id: o.id, name });
                    }
                    None => net_state.push_chat(
                        ChatKind::Admin,
                        "Seihai".into(),
                        "nobody to PM".into(),
                    ),
                }
            }
            if window.is_key_pressed(minifb::Key::F3, minifb::KeyRepeat::No) {
                show_debug = !show_debug;
            }
            if window.is_key_pressed(minifb::Key::F4, minifb::KeyRepeat::No) {
                show_players = !show_players;
            }
        }
        if window.is_key_pressed(minifb::Key::PageUp, minifb::KeyRepeat::Yes) {
            chat_scroll = (chat_scroll + 3).min(net_state.chat_log.len().saturating_sub(1));
        }
        if window.is_key_pressed(minifb::Key::PageDown, minifb::KeyRepeat::Yes) {
            chat_scroll = chat_scroll.saturating_sub(3);
        }

        if dirty {
            scene = draw(&session, &net_state.others, &my_chat);
            dirty = false;
        }
        let mut frame = scene.clone();
        if show_players {
            ui::draw_player_list(&mut frame, &account, &net_state.others);
        }
        if show_debug {
            ui::draw_debug(
                &mut frame,
                &ui::DebugInfo {
                    fps,
                    x: session.player_x,
                    y: session.player_y,
                    dir: session.player_dir,
                    level: loaded_level.clone().or_else(|| offline_level.clone()).unwrap_or_default(),
                    frames: frames_seen,
                    messages: net_state.messages_seen,
                    others: net_state.others.len(),
                    online: frames.is_some(),
                },
            );
        }
        ui::draw_chat(&mut frame, &net_state.chat_log, chat_scroll, &input);
        if !input.is_open() && window.is_key_pressed(minifb::Key::F12, minifb::KeyRepeat::No) {
            let line = match save_screenshot(&frame) {
                Ok(path) => format!("saved {}", path.display()),
                Err(e) => format!("screenshot failed: {e}"),
            };
            eprintln!("{line}");
            net_state.push_chat(ChatKind::Admin, "Seihai".into(), line);
        }
        buffer.clear();
        buffer.extend(
            frame
                .pixels
                .chunks(3)
                .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)),
        );
        let _ = window.update_with_buffer(&buffer, fb.width as usize, fb.height as usize);

        fps_count += 1;
        if fps_since.elapsed() >= Duration::from_secs(1) {
            fps = fps_count as f32 / fps_since.elapsed().as_secs_f32();
            fps_count = 0;
            fps_since = Instant::now();
        }
    }
}

/// Collects typed characters for the chat bar.
struct TypedChars(Arc<Mutex<Vec<char>>>);

impl minifb::InputCallback for TypedChars {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            self.0.lock().unwrap().push(c);
        }
    }
}

/// Write the frame to `screenshots/seihai-<unix time>.png` in the
/// working directory.
fn save_screenshot(fb: &Framebuffer) -> std::io::Result<PathBuf> {
    let dir = PathBuf::from("screenshots");
    std::fs::create_dir_all(&dir)?;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("seihai-{secs}.png"));
    let mut out = std::fs::File::create(&path)?;
    write_png_rgb(&mut out, fb.width, fb.height, &fb.pixels)?;
    Ok(path)
}

/// Version for a GUI label such as "2.22".
fn version_from_label(label: &str) -> Option<ClientVersion> {
    ClientVersion::all_versions()
        .iter()
        .copied()
        .find(|v| v.label() == label)
}
