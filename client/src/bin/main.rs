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
use std::collections::HashMap;
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
                    version = ClientVersion::from_label(v).unwrap_or_else(|| {
                        eprintln!("unknown version {v}; using 2.05");
                        ClientVersion::V205
                    });
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
            version = ClientVersion::from_label(v).unwrap_or(ClientVersion::V205);
        }
    }
    // Do not auto-fill online from settings. The GUI is the waiting area.
    // Settings only pre-fill the form.

    let mut session = OfflineSession::new();
    let mut net_state = SessionState::default();
    let mut conn: Option<Connection> = None;
    let mut frames = None;
    let mut loaded_level: Option<String> = None;
    // True while the level on screen was built from the server's board.
    let mut level_from_board = false;

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
                // Offline chosen, or the window was closed.
                let s = Settings::load();
                if s.offline == Some(true) {
                    online = None;
                    if let Some(v) = s.version.as_deref().and_then(ClientVersion::from_label) {
                        version = v;
                    }
                    if let Some(a) = s.assets {
                        assets = Some(a);
                    }
                    break;
                }
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
            if let Some(v) = s.version.as_deref().and_then(ClientVersion::from_label) {
                version = v;
            }
        }
        // Read server traffic the way the chosen version does.
        net_state = SessionState::for_version(version);

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
                        eprintln!("logged in to {addr} as {account} ({})", version.profile().summary());
                        if let Err(e) = c.send_post_login(version, "English") {
                            eprintln!("post-login send failed: {e}");
                        }
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

    // A version can have its own asset folder (its own install's images).
    if let Some(p) = Settings::load().assets_for(version.label()) {
        assets = Some(p.to_path_buf());
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

    let profile = version.profile();
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
    let draw = |session: &OfflineSession, others: &[seihai::protocol::OtherPlayer], my_chat: &str, npc_imgs: &HashMap<String, (u32, Vec<u8>)>| -> Framebuffer {
        let mut fb = render_view(session, &sheet, sw, sh, view_w, view_h);
        let (cam_x, cam_y) = (
            ((session.player_x + 1.5) * 16.0 - (view_w as f32 * 16.0) / 2.0) as i32,
            ((session.player_y + 1.5) * 16.0 - (view_h as f32 * 16.0) / 2.0) as i32,
        );
        // NPC images on top of the gray placeholder.
        if let Some(level) = &session.current_level {
            for npc in &level.npcs {
                if !npc.visible {
                    continue;
                }
                if let Some((w, px)) = npc_imgs.get(&npc.image) {
                    let dx = (npc.x * 16.0) as i32 - cam_x;
                    let dy = (npc.y * 16.0) as i32 - cam_y;
                    fb.blit_sprite(px, *w, 4, 0, 0, *w as i32, (px.len() / (*w as usize * 4)) as i32, dx, dy);
                }
            }
        }
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
        fb.draw_hud(session.player_health, session.rupees);
        // The minimap is set by the game server from 2.1 on.
        if profile.ui.minimap {
            if let Some(level) = &session.current_level {
                fb.draw_minimap(level, session.player_x, session.player_y);
            }
        }
        fb
    };
    let mut npc_images: HashMap<String, (u32, Vec<u8>)> = HashMap::new();
    let mut scene = draw(&session, &[], &my_chat, &npc_images);
    let fb = scene.clone();
    let mut buffer: Vec<u32> = Vec::new();

    let server_name = Settings::load().last_server_name;
    let title = profile.window_title(server_name.as_deref().filter(|_| online.is_some()));
    let mut window = match minifb::Window::new(
        &title,
        fb.width as usize,
        fb.height as usize,
        minifb::WindowOptions {
            resize: true,
            scale: minifb::Scale::X2,
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
            // 1.39 added Up/Down recall of earlier chat lines.
            if profile.ui.chat_recall {
                if window.is_key_pressed(minifb::Key::Up, minifb::KeyRepeat::No) {
                    input.recall_prev();
                }
                if window.is_key_pressed(minifb::Key::Down, minifb::KeyRepeat::No) {
                    input.recall_next();
                }
            }
            if window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No) {
                input.cancel();
            }
        } else if window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No) {
            if session.reading.take().is_none() {
                break;
            }
            dirty = true;
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
                                if session.load_level(root.level(name)).is_ok() {
                                    level_from_board = false;
                                    // Scan for missing NPC images after the file arrives.
                                    if let Some(level) = &session.current_level {
                                        for npc in &level.npcs {
                                            if npc.image.is_empty() || npc_images.contains_key(&npc.image) {
                                                continue;
                                            }
                                            let local = root.find_image("", &npc.image)
                                                .or_else(|| root.find_image("levels", &npc.image))
                                                .or_else(|| root.find_image("images", &npc.image));
                                            if local.is_none() {
                                                if let Some(c) = conn.as_mut() {
                                                    let _ = c.send_wantfile(&npc.image);
                                                    eprintln!("requesting {}", npc.image);
                                                }
                                            } else if let Some(path) = local {
                                                if let Ok(data) = std::fs::read(&path) {
                                                    if let Ok((w, _, px)) = read_png_rgba(&data) {
                                                        npc_images.insert(npc.image.clone(), (w, px));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                eprintln!("loaded {name}");
                            } else if name.ends_with(".png") || name.ends_with(".gif") {
                                if let Ok(data) = std::fs::read(root.levels_dir().join(name)) {
                                    if let Ok((w, _, px)) = read_png_rgba(&data) {
                                        npc_images.insert(name.clone(), (w, px));
                                        eprintln!("image {name}");
                                    }
                                }
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
                        level_from_board = false;
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
                        // Request missing NPC images.
                        if let Some(level) = &session.current_level {
                            for npc in &level.npcs {
                                if npc.image.is_empty() || npc_images.contains_key(&npc.image) {
                                    continue;
                                }
                                let local = root.find_image("", &npc.image)
                                    .or_else(|| root.find_image("levels", &npc.image))
                                    .or_else(|| root.find_image("images", &npc.image));
                                if local.is_none() {
                                    if let Some(c) = conn.as_mut() {
                                        let _ = c.send_wantfile(&npc.image);
                                        eprintln!("requesting {}", npc.image);
                                    }
                                } else if let Some(path) = local {
                                    if let Ok(data) = std::fs::read(&path) {
                                        if let Ok((w, _, px)) = read_png_rgba(&data) {
                                            npc_images.insert(npc.image.clone(), (w, px));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // No local copy: draw the tiles the server sent, as the
                // real client does, until the file (if any) arrives.
                if session.current_level.is_none() {
                    if let Some(tiles) = net_state.board.take() {
                        session.current_level = Some(seihai::assets::Level::from_board(&tiles));
                        level_from_board = true;
                        eprintln!("level from the server's board");
                    }
                }
                // Links can arrive a frame after the board.
                if level_from_board {
                    if let Some(level) = session.current_level.as_mut() {
                        if level.links.len() != net_state.links.len() {
                            level.links.clear();
                            level.add_server_links(&net_state.links);
                        }
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
                session.opened_chests.extend(net_state.opened_chests.drain(..));
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
        // A or Space: close the reading panel, or interact with what the
        // player faces.
        let interact = !input.is_open()
            && (window.is_key_pressed(minifb::Key::A, minifb::KeyRepeat::No)
                || window.is_key_pressed(minifb::Key::Space, minifb::KeyRepeat::No));
        if interact {
            if session.reading.take().is_none() {
                match session.interact() {
                    seihai::offline::Interaction::Chest { x, y, item, got } => {
                        eprintln!("chest {x},{y}: {item}");
                        net_state.push_chat(ChatKind::Admin, "Chest".into(), format!("You got {got}."));
                        if let Some(c) = conn.as_mut() {
                            let _ = c.send_open_chest(x, y);
                        }
                    }
                    seihai::offline::Interaction::Sign(_) | seihai::offline::Interaction::Nothing => {}
                }
            }
            dirty = true;
        }
        if !input.is_open() && session.reading.is_none() && last_step.elapsed() >= STEP_EVERY {
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
                } else if session.check_links() {
                    // The server owns the warp: ask for the level and let
                    // its reply load it and set the position.
                    if let Some((dest, x, y)) = session.pending_link.take() {
                        eprintln!("link to {dest} at {x} {y}");
                        if let Err(e) = c.send_levelwarp(x, y, &dest) {
                            eprintln!("send failed: {e}");
                            conn = None;
                        }
                    }
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
                                net_state.push_chat(ChatKind::Say, me(&net_state, &account), t.clone());
                            }
                            conn.as_mut().map(|c| c.send_chat(&t))
                        }
                        ui::ChatSend::Toall(t) if !t.is_empty() => {
                            net_state.push_chat(ChatKind::Toall, me(&net_state, &account), t.clone());
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
            let f4 = window.is_key_pressed(minifb::Key::F4, minifb::KeyRepeat::No);
            let f5 = window.is_key_pressed(minifb::Key::F5, minifb::KeyRepeat::No);
            if (f4 || f5) && !profile.ui.editor {
                // 2.3 moved the editor and level loading to GraalEditor.
                net_state.push_chat(
                    ChatKind::Admin,
                    "Seihai".into(),
                    format!("{} has no built-in editor", profile.label),
                );
            } else if f4 {
                eprintln!("F4: level editor not yet implemented");
            }
            if f5 && profile.ui.editor {
                let exts = profile.level_extensions();
                if let Some(path) = seihai::gui::pick_level(root.levels_dir().as_path(), &exts) {
                    if let Err(e) = session.load_level(&path) {
                        eprintln!("load failed: {e}");
                    } else {
                        eprintln!("loaded {}", path.display());
                        dirty = true;
                    }
                }
            }
        }
        if window.is_key_pressed(minifb::Key::PageUp, minifb::KeyRepeat::Yes) {
            chat_scroll = (chat_scroll + 3).min(net_state.chat_log.len().saturating_sub(1));
        }
        if window.is_key_pressed(minifb::Key::PageDown, minifb::KeyRepeat::Yes) {
            chat_scroll = chat_scroll.saturating_sub(3);
        }

        if dirty {
            scene = draw(&session, &net_state.others, &my_chat, &npc_images);
            dirty = false;
        }
        let mut frame = scene.clone();
        if show_players {
            ui::draw_player_list(&mut frame, &me(&net_state, &account), &net_state.others);
        }
        if show_debug {
            ui::draw_debug(
                &mut frame,
                &ui::DebugInfo {
                    fps,
                    rupees: session.rupees,
                    bombs: session.bombs,
                    darts: session.darts,
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
        if let Some(text) = &session.reading {
            ui::draw_panel(&mut frame, "Sign", text);
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

/// The name others see for us: the server's nickname, else the account.
fn me(state: &SessionState, account: &str) -> String {
    state.nickname.clone().unwrap_or_else(|| account.to_string())
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
