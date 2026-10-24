//! Seihai offline viewer.
//!
//! Cross-platform (Windows / Linux / macOS). Uses a software framebuffer
//! and minifb, so no system SDL2 is required.
//!
//! Usage:
//!   seihai --assets <dir> [--level <name.nw>]
//!   seihai --online host:port --list-only [--account A --password-env VAR --version 2.22]
//!
//! The asset dir should contain pics1.png and optionally tiledefs.txt
//! and a levels/ subdirectory.

use seihai::assets::{read_png_rgb, read_png_rgba, write_png_rgb, AssetRoot, Level};
use seihai::gmap::{sync_world, GmapRules, WorldLoader, WorldRequests};
use seihai::net::{enter_level_extras, login_gserver, Connection, EnterLevelSender};
use seihai::offline::OfflineSession;
use seihai::protocol::{showimg_message, ChatKind, ClientVersion, SessionState, ShowImgSender, TamperSender};
use seihai::scripting::engine::host::{SessionScripts, WeaponScript};
use seihai::settings::Settings;
use seihai::ui;
use seihai::render::{
    draw_other_player, draw_player_marker, draw_player_sprites, player_box_origin, render_view,
    Framebuffer,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Gmap files and levels from the asset folder.
struct RootLoader<'a>(&'a AssetRoot);

impl WorldLoader for RootLoader<'_> {
    fn gmap_text(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.0.level(name)).ok()
    }

    fn level(&self, name: &str) -> Option<Level> {
        Level::load(self.0.level(name)).ok()
    }
}

/// Send what [`sync_world`] asked for: the `.gmap` file, and the
/// `PLI_ADJACENTLEVEL` messages in one frame.
fn send_world_requests(conn: Option<&mut Connection>, req: WorldRequests) {
    let Some(c) = conn else { return };
    if let Some(f) = &req.gmap_file {
        eprintln!("requesting {f}");
        let _ = c.send_wantfile(f);
    }
    if !req.messages.is_empty() {
        if let Err(e) = c.send_messages(&req.messages) {
            eprintln!("adjacent-level send failed: {e}");
        }
    }
}

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
    let mut list_only = false;
    let mut send_tampercheck = false;
    let mut send_showimg = false;
    let mut no_scripts = false;
    // None: not given on the command line, so the settings file decides.
    let mut cli_serverwarp: Option<bool> = None;
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
            "--list-only" => list_only = true,
            "--send-tampercheck" => send_tampercheck = true,
            "--send-showimg" => send_showimg = true,
            "--no-serverwarp" => cli_serverwarp = Some(false),
            "--serverwarp" => cli_serverwarp = Some(true),
            "--no-scripts" => no_scripts = true,
            "--password-env" => {
                i += 1;
                if let Some(var) = args.get(i) {
                    match env::var(var) {
                        Ok(p) if !p.is_empty() => password = p,
                        _ => eprintln!("warning: environment variable {var} is unset or empty"),
                    }
                } else {
                    eprintln!("warning: --password-env needs a variable name");
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
                println!("       seihai --assets <dir> --online host:port --account A --password-env VAR [--version 2.05]");
                println!("         --password-env VAR  read the password from environment variable VAR");
                println!("                             (keeps it off the command line)");
                println!("       seihai --online host:port --list-only [--account A --password-env VAR] [--version 2.22]");
                println!("         --list-only         fetch and print the server list from a listserver,");
                println!("                             then exit; no login to a gserver, no prompt");
                println!("         --send-tampercheck  as 2.17, 2.22 and 2.31: send PLI_TAMPERCHECK (39) after");
                println!("                             login, and again as that build does (default: off;");
                println!("                             GServer-v2 ignores it)");
                println!("         --send-showimg      as 2.22 and 2.31: send PLI_SHOWIMGPLAYER (24) = 9 on");
                println!("                             the build's period (default: off; meaning unknown)");
                println!("         --no-serverwarp     do not follow a PLO_SERVERWARP (178); by default");
                println!("                             Seihai reconnects to the host and port it names");
                println!("                             and logs in again, as the real client does");
                println!("                             (settings file: follow_serverwarp=0)");
                println!("         --serverwarp        follow it even if the settings file says off");
                println!("         --no-scripts        do not run the client-side GS1 scripts of level NPCs");
                println!("                             and weapons (default: they run)");
                println!("       seihai                  (menu if no mode given)");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Load settings and fill defaults.
    let mut settings = Settings::load();
    // Command line, then settings file, then the default (follow).
    let follow_serverwarp = settings.follow_serverwarp_with(cli_serverwarp);
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
    // List-only: ask the listserver, print, exit. Never prompts and never
    // logs in to a gserver. Exit 0 on a list, 1 on a failure, 2 on bad use.
    if list_only {
        let Some(addr) = &online else {
            eprintln!("--list-only needs --online host:port (a listserver)");
            std::process::exit(2);
        };
        match Connection::fetch_server_list(addr, version, &account, &password) {
            Ok(reply) => {
                if let Some(err) = &reply.error {
                    eprintln!("listserver error: {err}");
                    std::process::exit(1);
                }
                if let Some(status) = &reply.status {
                    println!("{status}");
                }
                if let Some(url) = &reply.site_url {
                    println!("Site: {url}");
                }
                if let Some(url) = &reply.upgrade_url {
                    println!("Upgrade: {url}");
                }
                if reply.servers.is_empty() {
                    println!("no servers in reply");
                } else {
                    println!("Servers:");
                    for (i, e) in reply.servers.iter().enumerate() {
                        println!("{}", e.display_line(i));
                    }
                }
                return;
            }
            Err(e) => {
                eprintln!("listserver failed: {e}");
                std::process::exit(1);
            }
        }
    }
    // Do not auto-fill online from settings. The GUI is the waiting area.
    // Settings only pre-fill the form.

    let mut session = OfflineSession::new();
    // Client-side GS1 scripts of the level's NPCs and the player's weapons.
    let mut scripts = SessionScripts::new(version);
    let mut scripts_serial = u64::MAX;
    let mut scripts_clock = Instant::now();
    let mut net_state = SessionState::default();
    let mut conn: Option<Connection> = None;
    // Opt-in tamper check (`--send-tampercheck`): the build's constant.
    let mut tamper: Option<TamperSender> = None;
    let mut tamper_clock = Instant::now();
    // PLI_ENTERLEVEL (46): login and every level change, for profiles with it.
    let mut enter_level: Option<EnterLevelSender> = None;
    // Opt-in PLI_SHOWIMGPLAYER (24) = 9 (`--send-showimg`), on the build's period.
    let mut showimg: Option<ShowImgSender> = None;
    let mut showimg_clock = Instant::now();
    let mut frames = None;
    let mut loaded_level: Option<String> = None;
    // Gmaps: what the profile says, and what was already asked of the server.
    let gmap_rules = GmapRules::for_version(version);
    let mut gmap_asked: HashSet<String> = HashSet::new();
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
        scripts = SessionScripts::new(version);
        scripts_serial = u64::MAX;

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
            match login_gserver(addr, version, seihai::net::random_key(), &account, &password, send_tampercheck) {
                Ok(mut sess) => {
                    eprintln!("logged in to {addr} as {account} ({})", version.profile().summary());
                    if sess.tamper_unavailable {
                        eprintln!(
                            "--send-tampercheck: {} has no tamper-check value; not sent",
                            version.label()
                        );
                    }
                    for w in &sess.warnings {
                        eprintln!("{w}");
                    }
                    tamper = sess.tamper.take();
                    tamper_clock = Instant::now();
                    enter_level = Some(sess.enter_level.clone());
                    showimg = if send_showimg { ShowImgSender::for_version(version) } else { None };
                    if send_showimg && showimg.is_none() {
                        eprintln!("--send-showimg: {} has no recorded id 24 behaviour; not sent", version.label());
                    }
                    showimg_clock = Instant::now();
                    match sess.conn.spawn_reader() {
                        Ok(rx) => frames = Some(rx),
                        Err(e) => eprintln!("reader failed: {e}"),
                    }
                    conn = Some(sess.conn);
                }
                Err(e) => {
                    eprintln!("{e}");
                    if !cli_mode {
                        gui_error = Some(e.to_string());
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
        } else {
            // The level the session starts in is the first entry point.
            session.note_entry(&name);
        }
    }
    if root.tiledefs().is_file() {
        let _ = session.load_tiletypes(root.tiledefs());
    } else if root.arrays_dat().is_file() {
        let _ = session.load_tiletypes_arrays(root.arrays_dat());
    }

    let profile = version.profile();
    // The death sequence of this build (none for builds before 2.05).
    session.life.set_rules(profile.life.death);
    session.combat.set_rules(profile.life.combat);
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
        // On a gmap another player's position is local to its own cell.
        let own_cell = session.gmap.as_ref().map(|w| w.cell);
        let others: Vec<seihai::protocol::OtherPlayer> = others
            .iter()
            .map(|o| {
                let (x, y) = o.position_from(own_cell);
                seihai::protocol::OtherPlayer { x, y, ..o.clone() }
            })
            .collect();
        let others = others.as_slice();
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
                session.in_water,
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
    let mut my_chat = String::new();
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
    let mut confirm_quit = false;
    let mut show_help = false;
    let started = Instant::now();
    let mut warp_since: Option<Instant> = None;
    // Where a cached level link (PLI 30) lands the player; see the link send.
    let mut cached_arrival: Option<(f32, f32)> = None;
    let mut was_online = false;
    let mut frames_seen = 0usize;
    let mut fps = 0.0f32;
    let mut fps_count = 0u32;
    let mut fps_since = Instant::now();
    let mut dirty = true;

    let mut last_step = Instant::now();
    // STATUS (prop 18) as the server last gave it; the dead bit is toggled on it.
    let mut status_base = 0u8;
    let mut life_clock = Instant::now();
    let mut combat_clock = Instant::now();
    while window.is_open() {
        let typed_now: Vec<char> = std::mem::take(&mut *typed.lock().unwrap());
        if !input.is_open()
            && (window.is_key_pressed(minifb::Key::F1, minifb::KeyRepeat::No) || typed_now.contains(&'?'))
        {
            show_help = !show_help;
        }
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
            if show_help {
                show_help = false;
            } else if session.reading.take().is_none() {
                confirm_quit = true;
                dirty = true;
            }
            dirty = true;
        }
        if confirm_quit {
            if window.is_key_pressed(minifb::Key::Y, minifb::KeyRepeat::No)
                || window.is_key_pressed(minifb::Key::Enter, minifb::KeyRepeat::No)
            {
                break;
            }
            if window.is_key_pressed(minifb::Key::N, minifb::KeyRepeat::No)
                || window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No)
            {
                confirm_quit = false;
                dirty = true;
            }
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
                            // A neighbour's file (gmap) is not the current level.
                            let is_current = net_state
                                .current_level_name()
                                .map_or(true, |c| c.eq_ignore_ascii_case(name));
                            if name.ends_with(".nw") && !is_current {
                                eprintln!("loaded {name}");
                            } else if name.ends_with(".nw") {
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
                // True when this batch put the player in a new level.
                let mut entered = false;
                if !no_scripts {
                    let ws: Vec<WeaponScript> = net_state
                        .weapons
                        .iter()
                        .filter_map(|w| {
                            w.script.as_deref().map(|script| WeaponScript { name: &w.name, image: w.image.as_deref().unwrap_or(""), script })
                        })
                        .collect();
                    scripts.sync_weapons(&mut session, &ws);
                }
                if let Some(name) = &net_state.current_level_name() {
                    if loaded_level.as_deref().map_or(true, |l| !l.eq_ignore_ascii_case(name)) {
                        entered = true;
                        // The first level after login is not a "level change":
                        // the login send already covers it.
                        // A new level (not the first after login) owes a
                        // PLI 46. It goes out once the player's position is
                        // known, below, as the first message of the frame.
                        if let Some(el) = enter_level.as_mut() {
                            el.note_level(name);
                        }
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
                    // On a gmap the board of the player's level is kept by
                    // name; otherwise it is the one just sent.
                    let board = net_state
                        .current_level_name()
                        .and_then(|n| net_state.board_of(&n).cloned())
                        .or_else(|| net_state.board.take());
                    if let Some(tiles) = board {
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
                // A cached link got no PLO_PLAYERWARP: the position is the
                // one the link request carried (the server's, if it did send
                // one, wins).
                if let Some((x, y)) = cached_arrival.take_if(|_| entered) {
                    net_state.player_x.get_or_insert(x);
                    net_state.player_y.get_or_insert(y);
                }
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
                // Hearts, AP and STATUS the server gave.
                if let Some(st) = net_state.own_status.take() {
                    status_base = st;
                }
                session.apply_server_power(
                    net_state.own_max_power.take(),
                    net_state.own_cur_power.take(),
                    net_state.own_alignment.take(),
                );
                // Hits from other players, judged here, on the victim's side:
                // sword swings seen in their props, and ids 40 and 46.
                {
                    let swings = std::mem::take(&mut net_state.swings);
                    let hits = std::mem::take(&mut net_state.hits);
                    if !swings.is_empty() || !hits.is_empty() {
                        let own_cell = net_state.gmap.as_ref().map(|g| g.cell);
                        let mut took = false;
                        for r in session.receive_all(&net_state.others, own_cell, &swings, &hits) {
                            match r.outcome {
                                Ok(half) => {
                                    took = true;
                                    eprintln!("hit by {:?} ({:?}): {half} half hearts", r.attacker, r.kind);
                                }
                                Err(why @ (seihai::combat::Reject::Immune
                                | seihai::combat::Reject::Dead
                                | seihai::combat::Reject::AlreadyHurt
                                | seihai::combat::Reject::Swimming
                                | seihai::combat::Reject::NpcSource
                                | seihai::combat::Reject::UnknownPower
                                | seihai::combat::Reject::CentreUnknown)) => {
                                    eprintln!("hit by {:?} ({:?}) refused: {why:?}", r.attacker, r.kind);
                                }
                                Err(_) => {}
                            }
                        }
                        if took {
                            // Hurt mode shows in the sprite others see.
                            if let Some(c) = conn.as_mut() {
                                if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value()) {
                                    eprintln!("send failed: {e}");
                                    conn = None;
                                }
                            }
                        }
                    }
                }
                if entered || (warped && session.warp_pending.is_some()) {
                    // Arrival: the server's position is final. Links stay
                    // quiet until the player moves off it.
                    //
                    // PLI 46 first: after our own link request, 46 and the
                    // props (position) in one frame; after a server-side
                    // warp (or login), 46 alone.
                    let own_link = session.warp_pending.is_some();
                    if let (Some(el), Some(c)) = (enter_level.as_mut(), conn.as_mut()) {
                        let extras = enter_level_extras(own_link, &session.player_update());
                        if let Err(e) = el.send_pending(c, &extras) {
                            eprintln!("enter-level send failed: {e}");
                        }
                    }
                    warp_since = None;
                    if session.arrive(session.player_x, session.player_y) {
                        net_state.push_chat(ChatKind::Admin, "Seihai".into(), "developer mode off".into());
                    }
                    // Every level entry (and login) is where a respawn returns to.
                    if let Some(n) = net_state.current_level_name() {
                        session.note_entry(&n);
                    }
                } else if warped {
                    session.refresh_water();
                }
                if warped {
                    eprintln!(
                        "position {} {} dir {}",
                        session.player_x, session.player_y, session.player_dir
                    );
                }
                // Gmap: the grid, the neighbouring levels, 186 blocks, and
                // a request for whatever the server still has to send.
                let req = sync_world(&mut session, &mut net_state, &gmap_rules, &RootLoader(&root), &mut gmap_asked);
                send_world_requests(conn.as_mut(), req);
                dirty = true;
            }
        }

        // Health and death: the 50 ms death timer, the claim and the dead
        // bit on the wire, and the respawn on a movement or action key.
        {
            let dt = life_clock.elapsed().as_millis() as u64;
            life_clock = Instant::now();
            let mut events = session.tick_life(dt);
            let keys_down = !input.is_open()
                && [minifb::Key::Left, minifb::Key::Right, minifb::Key::Up, minifb::Key::Down]
                    .iter()
                    .any(|k| window.is_key_down(*k))
                || !input.is_open()
                    && [minifb::Key::A, minifb::Key::Space]
                        .iter()
                        .any(|k| window.is_key_pressed(*k, minifb::KeyRepeat::No));
            if session.life.can_respawn() && keys_down {
                let online = conn.is_some();
                if let Some(to) = session.respawn(online) {
                    eprintln!("respawn: {to:?}");
                    // Online the level is sent again at the respawn point,
                    // as an ordinary level warp (no PLI 30 form).
                    let dest = match &to {
                        seihai::offline::RespawnTo::Saved(e) => Some(e.level.clone()),
                        seihai::offline::RespawnTo::Default { .. } => net_state.current_level_name(),
                        seihai::offline::RespawnTo::InPlace => None,
                    };
                    if let (Some(dest), Some(c)) = (dest.as_deref(), conn.as_mut()) {
                        session.begin_warp(dest);
                        warp_since = Some(Instant::now());
                        let (x, y, dir) = (session.player_x, session.player_y, session.player_dir);
                        if let Err(e) = c.send_link(version, None, x, y, dir, dest, tamper.as_mut()) {
                            eprintln!("send failed: {e}");
                            session.cancel_warp();
                            conn = None;
                        }
                    } else if let (seihai::offline::RespawnTo::Saved(e), None) = (&to, &conn) {
                        if !session.is_loaded(&e.level) && session.load_level(root.level(&e.level)).is_err() {
                            eprintln!("respawn: no local {}", e.level);
                        }
                        session.arrive(e.x, e.y);
                    }
                    events.extend(session.life.take_events());
                    dirty = true;
                }
            }
            for ev in events {
                match ev {
                    seihai::life::LifeEvent::Died { attacker } => {
                        eprintln!("died (attacker {attacker:?})");
                        if !no_scripts {
                            scripts.player_dies(&mut session);
                        }
                        dirty = true;
                    }
                    seihai::life::LifeEvent::StatusDead => eprintln!("dead bit sent"),
                    seihai::life::LifeEvent::Respawned { .. } => {}
                }
                let msgs = seihai::life::messages_for(ev, version, status_base);
                if let (false, Some(c)) = (msgs.is_empty(), conn.as_mut()) {
                    if let Err(e) = c.send_messages(&msgs) {
                        eprintln!("send failed: {e}");
                    }
                }
            }
        }
        // Combat: S swings the sword (others judge the hit from the sprite
        // we send), and the swing and hurt mode run on the 50 ms clock.
        {
            let dt = combat_clock.elapsed().as_millis() as u64;
            combat_clock = Instant::now();
            let mut sprite_changed = session.tick_combat(dt);
            if !input.is_open() && window.is_key_pressed(minifb::Key::S, minifb::KeyRepeat::No) && session.start_swing() {
                sprite_changed = true;
            }
            if sprite_changed {
                dirty = true;
                if let Some(c) = conn.as_mut() {
                    if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value()) {
                        eprintln!("send failed: {e}");
                        conn = None;
                    }
                }
            }
        }
        // Arrow keys move the player half a tile per step, at most one
        // step every 50 ms.
        let mut moved = false;
        // A or Space: close the reading panel, or interact with what the
        // player faces.
        let interact = !input.is_open()
            && !session.life.is_dead()
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
        if !input.is_open() && !session.life.is_dead() && session.reading.is_none() && session.warp_pending.is_none() && last_step.elapsed() >= session.step_interval() {
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
        // Client-side scripts: load the level's NPCs when a level arrives,
        // touch events after a step, timeouts on the clock.
        if !no_scripts {
            if session.level_serial != scripts_serial {
                scripts_serial = session.level_serial;
                let label = loaded_level.clone().or_else(|| offline_level.clone()).unwrap_or_default();
                scripts.load_level(&mut session, &label);
            }
            if moved {
                scripts.touch(&mut session);
            }
            let dt = scripts_clock.elapsed().as_millis() as u64;
            scripts_clock = Instant::now();
            scripts.tick(&mut session, dt);
            // A script asked for a level change (setlevel / setlevel2).
            if let Some((dest, x, y)) = session.pending_link.take() {
                if conn.is_some() {
                    // The wire form of a script-initiated warp has not been
                    // captured, so it is not sent.
                    eprintln!("script warp to {dest} ignored while online");
                } else if session.load_level(root.level(&dest)).is_ok() {
                    if session.arrive(x, y) {
                        net_state.push_chat(ChatKind::Admin, "Seihai".into(), "developer mode off".into());
                    }
                    eprintln!("script warp to {dest}");
                } else {
                    eprintln!("script warp: no local {dest}");
                }
                dirty = true;
            }
            for e in scripts.unhandled.drain(..) {
                eprintln!("script effect not applied: {e:?}");
            }
            for l in scripts.world.take_log() {
                eprintln!("script: {l}");
            }
        }
        if moved {
            dirty = true;
            if let Some(cx) = session.take_gmap_crossing() {
                // The step crossed a gmap edge. The player is in the next
                // cell already; the frame carries the cell prop and the
                // wrapped coordinate (not the plain walk update), and the
                // server follows with the new level.
                net_state.set_gmap_cell(cx.to);
                let now = net_state.current_level_name();
                eprintln!("gmap crossing {:?} -> {:?} at {} {}", cx.from, cx.to, cx.local_x, cx.local_y);
                if let Some(c) = conn.as_mut() {
                    match c.send_gmap_crossing(version, &cx, tamper.as_mut()) {
                        Ok(sent46) => {
                            if let (Some(el), Some(n)) = (enter_level.as_mut(), now.as_deref()) {
                                el.crossing(n, sent46);
                            }
                        }
                        Err(e) => {
                            eprintln!("send failed: {e}");
                            conn = None;
                        }
                    }
                }
                if let Some(n) = &now {
                    level_from_board = !root.level(n).is_file();
                }
                loaded_level = now;
                if let Some(n) = &loaded_level {
                    session.note_entry(n);
                }
                let req = sync_world(&mut session, &mut net_state, &gmap_rules, &RootLoader(&root), &mut gmap_asked);
                send_world_requests(conn.as_mut(), req);
            } else if let Some(c) = conn.as_mut() {
                // Online: send the new position as PLI_PLAYERPROPS.
                if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value()) {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
            // Check links after every move, online or offline.
            if session.check_links() {
                if let Some((dest, x, y)) = session.pending_link.take() {
                    eprintln!("link to {dest} at {x} {y}");
                    if let Some(c) = conn.as_mut() {
                        // Online: the server owns the warp. Ask once and
                        // wait; its reply moves the player.
                        session.begin_warp(&dest);
                        warp_since = Some(Instant::now());
                        // A level the player has been in is asked for with
                        // PLI 30 and its mod time (profiles with cached
                        // links), and a 2.31 client then sends 46 and the
                        // props in the same frame; 2.22 sends the tamper
                        // check in the same frame instead.
                        let cached = if version.sends_cached_links() { net_state.cached_mod_time(&dest) } else { None };
                        let dir = session.player_update().dir;
                        match c.send_link(version, cached, x, y, dir, &dest, tamper.as_mut()) {
                            Ok(frame) => {
                                if let Some(t) = cached {
                                    eprintln!("cached link to {dest} (mod time {t})");
                                    net_state.expect_cached_level(&dest);
                                    // The server sends no PLO_PLAYERWARP for
                                    // this form: the player lands where the
                                    // link said.
                                    cached_arrival = Some((x, y));
                                    if let Some(el) = enter_level.as_mut() {
                                        el.cached_link(&dest, frame.with_enter_level);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("send failed: {e}");
                                session.cancel_warp();
                                conn = None;
                            }
                        }
                    } else {
                        // Offline: load the destination if it is local.
                        let path = root.level(&dest);
                        if session.load_level(&path).is_ok() {
                            if session.arrive(x, y) {
                                net_state.push_chat(ChatKind::Admin, "Seihai".into(), "developer mode off".into());
                            }
                            session.note_entry(&dest);
                            eprintln!("warped to {dest}");
                        } else {
                            eprintln!("no local {dest}");
                        }
                    }
                }
            }
        }

        // Tell the server when swimming starts or stops (2.1+ clients; the
        // gani prop is the bow slot before that).
        if let Some(swimming) = session.take_swim_change() {
            dirty = true;
            if let (Some(c), false) = (conn.as_mut(), profile.props.bow_prop10) {
                if let Err(e) = c.send_messages(&[seihai::offline::swim_message(swimming)]) {
                    eprintln!("send failed: {e}");
                }
            }
        }
        // A warp the server never answers is given up after a few seconds.
        if warp_since.is_some_and(|t| t.elapsed() > Duration::from_secs(4)) {
            warp_since = None;
            session.cancel_warp();
            cached_arrival = None;
            net_state.cancel_cached_request();
            eprintln!("warp not answered");
        }
        // 2.31 sends the tamper check again every 3.2 s.
        {
            let dt = tamper_clock.elapsed();
            tamper_clock = Instant::now();
            if let (Some(t), Some(c)) = (tamper.as_mut(), conn.as_mut()) {
                if t.advance(dt) {
                    if let Err(e) = c.send_tampercheck(t) {
                        eprintln!("tamper-check send failed: {e}");
                    }
                }
            }
        }
        // 2.22 and 2.31 send PLI 24 = 9 on a fixed period; Seihai only when
        // asked. 2.22 sends it only while another player is in the level.
        {
            let dt = showimg_clock.elapsed();
            showimg_clock = Instant::now();
            if let (Some(t), Some(c)) = (showimg.as_mut(), conn.as_mut()) {
                if t.advance(dt, !net_state.others.is_empty()) {
                    if let Err(e) = c.send_messages(&[showimg_message()]) {
                        eprintln!("showimg send failed: {e}");
                    }
                }
            }
        }
        // PLO_SERVERWARP (178): leave this server for the host and port it
        // names and log in again there (new key, same account and password,
        // no listserver step), as the real client does. A text whose host
        // and port cannot be read gives a failed connection, as it did in
        // the real client.
        if let Some(w) = net_state.server_warp.take() {
            if !follow_serverwarp {
                eprintln!("server warp to {:?} not followed (--no-serverwarp or follow_serverwarp=0)", w.name);
            } else if let Some(old) = conn.as_ref() {
                let _ = old.shutdown();
                conn = None;
                frames = None;
                session.cancel_warp();
                warp_since = None;
                cached_arrival = None;
                let target = w.address();
                eprintln!("server warp to {:?}: {}", w.name, target.as_deref().unwrap_or("no usable host and port"));
                let result = match target {
                    Some(a) => seihai::net::login_gserver(
                        &a,
                        version,
                        seihai::net::random_key(),
                        &account,
                        &password,
                        send_tampercheck,
                    )
                    .map(|s| (a, s))
                    .map_err(|e| e.to_string()),
                    None => Err(format!("server warp to {:?} has no usable host and port", w.name)),
                };
                match result {
                    Ok((a, mut sess)) => {
                        eprintln!("logged in to {a} as {account} ({})", version.profile().summary());
                        for m in &sess.warnings {
                            eprintln!("{m}");
                        }
                        // A fresh session on the new server.
                        net_state = SessionState::for_version(version);
                        loaded_level = None;
                        session.leave_gmap();
                        gmap_asked.clear();
                        tamper = sess.tamper.take();
                        tamper_clock = Instant::now();
                        enter_level = Some(sess.enter_level.clone());
                        showimg = if send_showimg { ShowImgSender::for_version(version) } else { None };
                        showimg_clock = Instant::now();
                        match sess.conn.spawn_reader() {
                            Ok(rx) => frames = Some(rx),
                            Err(e) => eprintln!("reader failed: {e}"),
                        }
                        conn = Some(sess.conn);
                    }
                    Err(why) => {
                        eprintln!("{why}");
                        if !cli_mode {
                            let _ = seihai::gui::run_with_error(why);
                            return;
                        }
                    }
                }
            }
        }
        // Developer mode does not survive a lost connection.
        was_online |= conn.is_some();
        if was_online && conn.is_none() {
            was_online = false;
            session.cancel_warp();
            if session.dev.reset() {
                net_state.push_chat(ChatKind::Admin, "Seihai".into(), "developer mode off".into());
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
                            if !no_scripts {
                                scripts.chat(&mut session, &t);
                            }
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
            if window.is_key_pressed(minifb::Key::Tab, minifb::KeyRepeat::No) {
                show_players = !show_players;
            }
            if window.is_key_pressed(minifb::Key::F9, minifb::KeyRepeat::No) {
                let msg = if !seihai::dev::DevMode::AVAILABLE {
                    "developer mode is not in this build"
                } else if session.dev.toggle() {
                    "developer mode: wallhack on"
                } else {
                    "developer mode: wallhack off"
                };
                eprintln!("{msg}");
                net_state.push_chat(ChatKind::Admin, "Seihai".into(), msg.into());
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
        if confirm_quit {
            ui::draw_panel(&mut frame, "Quit", "Quit Seihai?  Y / Enter = yes,  N / Esc = no");
        }
        if show_help {
            seihai::help::draw_help(&mut frame);
        } else if started.elapsed() < Duration::from_secs(10) {
            seihai::help::draw_hint(&mut frame);
        }
        seihai::help::draw_dev_badge(&mut frame, &session.dev);
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
