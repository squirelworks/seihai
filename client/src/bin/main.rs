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

use seihai::assets::{read_png_rgb, read_png_rgba, write_png_rgb, AssetResolver, AssetRoot, Gani, GaniPlayer, Level, TileTypesFile};
use seihai::sound::SoundEngine;
use seihai::gmap::{sync_world, GmapRules, WorldLoader, WorldRequests};
use seihai::protocol::login::LoginBurst;
use seihai::net::{enter_level_extras, login_gserver, Connection, EnterLevelSender};
use seihai::filecache::FileSession;
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

/// Start an online level change (a walked-into link or a script warp), log
/// the request and report a send failure.
#[allow(clippy::too_many_arguments)]
fn online_link(
    conn: &mut Connection,
    version: ClientVersion,
    session: &mut OfflineSession,
    net_state: &mut SessionState,
    enter_level: Option<&mut EnterLevelSender>,
    tamper: Option<&mut TamperSender>,
    dest: &str,
    x: f32,
    y: f32,
) -> std::io::Result<Option<seihai::net::LinkStart>> {
    let r = seihai::net::start_online_link(
        |m| conn.send_messages(m),
        version,
        session,
        net_state,
        enter_level,
        tamper,
        dest,
        x,
        y,
    );
    match &r {
        Ok(Some(s)) => {
            if let Some(t) = s.cached_mod_time {
                eprintln!("cached link to {dest} (mod time {t})");
            }
        }
        Ok(None) => eprintln!("level request for {dest} dropped: a warp is already pending"),
        Err(e) => eprintln!("send failed: {e}"),
    }
    r
}

/// Gmap files and levels from the asset folder.
struct RootLoader<'a>(&'a AssetResolver);

impl WorldLoader for RootLoader<'_> {
    fn gmap_text(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.0.level_path(name)).ok()
    }

    fn level(&self, name: &str) -> Option<Level> {
        Level::load(self.0.level_path(name)).ok()
    }
}

/// Send the file requests the rules ask for.
fn send_file_msgs(conn: Option<&mut Connection>, msgs: Vec<Vec<u8>>) {
    if msgs.is_empty() {
        return;
    }
    if let Some(c) = conn {
        if let Err(e) = c.send_messages(&msgs) {
            eprintln!("file request send failed: {e}");
        }
    }
}

/// For each NPC image of a level: ask the server for a missing file or, if
/// the file is there, check it (the file-cache rules decide which), and
/// load the image we have.
fn npc_image_pass(
    level: &Level,
    files: &mut FileSession,
    conn: Option<&mut Connection>,
    npc_images: &mut HashMap<String, (u32, Vec<u8>)>,
) {
    let mut msgs = Vec::new();
    for npc in &level.npcs {
        if npc.image.is_empty() || npc_images.contains_key(&npc.image) {
            continue;
        }
        let m = files.need(&npc.image);
        if !m.is_empty() {
            eprintln!("requesting {}", npc.image);
        }
        msgs.extend(m);
        if let Some(data) = files.store.read(&npc.image) {
            if let Ok((w, _, px)) = read_png_rgba(&data) {
                npc_images.insert(npc.image.clone(), (w, px));
            }
        }
    }
    send_file_msgs(conn, msgs);
}

/// Send what [`sync_world`] asked for: the `.gmap` file (or a check of the
/// one we have), and the `PLI_ADJACENTLEVEL` messages in one frame.
fn send_world_requests(conn: Option<&mut Connection>, req: WorldRequests, files: &mut FileSession) {
    let Some(c) = conn else { return };
    if let Some(f) = req.gmap_file.as_ref().or(req.gmap_local.as_ref()) {
        let msgs = files.need(f);
        if !msgs.is_empty() {
            eprintln!("requesting {f}");
            if let Err(e) = c.send_messages(&msgs) {
                eprintln!("file request send failed: {e}");
            }
        }
    }
    if !req.messages.is_empty() {
        if let Err(e) = c.send_messages(&req.messages) {
            eprintln!("adjacent-level send failed: {e}");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    // `seihai capture ...`: the capture viewer and layout explorer. Reads
    // files only; no window, no network.
    if args.first().map(String::as_str) == Some("capture") {
        std::process::exit(seihai::viewer::cli::run(&args[1..]));
    }
    let mut assets: Option<PathBuf> = None;
    let mut level_name: Option<String> = None;
    let mut online: Option<String> = None;
    let mut account = String::from("player");
    let mut password = String::new();
    let mut version = ClientVersion::V205;
    let mut i = 0;
    let mut cli_mode = false;
    let mut list_only = false;
    let mut check_assets = false;
    let mut level_info: Vec<String> = Vec::new();
    let mut version_chain = false;
    let mut chain_format = seihai::protocol::chain::Format::Text;
    let mut version_given = false;
    let mut send_tampercheck = false;
    let mut send_showimg = false;
    let mut no_scripts = false;
    // None: not given on the command line, so the settings file decides.
    let mut cli_serverwarp: Option<bool> = None;
    // None: not given on the command line, so the settings file decides.
    let mut cli_sound: Option<bool> = None;
    let mut cli_save_downloads: Option<bool> = None;
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
            "--check-assets" => check_assets = true,
            "--level-info" => {
                i += 1;
                match args.get(i) {
                    Some(f) => level_info.push(f.clone()),
                    None => {
                        eprintln!("--level-info needs a level file");
                        std::process::exit(2);
                    }
                }
            }
            "--version-chain" => version_chain = true,
            "--chain-format" => {
                i += 1;
                chain_format = match args.get(i).map(String::as_str) {
                    Some("text") => seihai::protocol::chain::Format::Text,
                    Some("md") => seihai::protocol::chain::Format::Markdown,
                    Some("tsv") => seihai::protocol::chain::Format::Tsv,
                    other => {
                        eprintln!("--chain-format needs text, md or tsv (got {})", other.unwrap_or("nothing"));
                        std::process::exit(2);
                    }
                };
            }
            "--send-tampercheck" => send_tampercheck = true,
            "--send-showimg" => send_showimg = true,
            "--no-serverwarp" => cli_serverwarp = Some(false),
            "--serverwarp" => cli_serverwarp = Some(true),
            "--no-scripts" => no_scripts = true,
            "--no-sound" => cli_sound = Some(false),
            "--sound" => cli_sound = Some(true),
            "--no-save-downloads" => cli_save_downloads = Some(false),
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
                version_given = true;
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
                println!("         chat bar: /ignore NAME, /unignore NAME, /buddy NAME, /unbuddy NAME,");
                println!("                   /mute NAME, /unmute NAME, /lists (lists are files in the");
                println!("                   settings directory; /mute sends PLI_MUTEPLAYER where the");
                println!("                   version has it)");
                println!("         --no-scripts        do not run the client-side GS1 scripts of level NPCs");
                println!("                             and weapons (default: they run)");
                println!("         --no-sound          do not play sound effects (GANI PLAYSOUND cues);");
                println!("                             by default they play from <assets>/sounds when");
                println!("                             the build has an audio device backend");
                println!("                             (settings file: sound=0)");
                println!("         --sound             play them even if the settings file says off");
                println!("         --no-save-downloads  keep downloaded files in memory only, do not");
                println!("                             write them to disk (settings file: save_downloads=0)");
                println!("       seihai --assets <dir> --check-assets [--version 2.22]");
                println!("         --check-assets      resolve and load the files a version uses from the");
                println!("                             data folder and print what is found; without");
                println!("                             --version, the five main versions. Exit 1 if a");
                println!("                             required file is missing or unreadable");
                println!("       seihai --level-info <file.graal> [--level-info <file> ...]");
                println!("         --level-info        print sizes, offsets and record counts of a binary");
                println!("                             .graal/.zelda level (no content). Exit 1 if a file");
                println!("                             does not parse");
                println!("       seihai --version-chain [--chain-format text|md|tsv] [--version V]");
                println!("         --version-chain     print what each client build changed against the");
                println!("                             build before it (all builds, or just --version)");
                println!("       seihai capture show FILE [--version V] [--dir c2s|s2c] [--id N,NAME] [--time A..B]");
                println!("                               [--frame A..B] [--format text|tsv|json] [--raw] [--layouts]");
                println!("       seihai capture layout ID|NAME [--dir c2s|s2c] [--version V]");
                println!("       seihai capture tables [--tables DIR]");
                println!("         capture             read a capture (pcap or difftest trace) and print the");
                println!("                             decoded messages, or show a packet's field layout across");
                println!("                             versions; login frames are redacted (seihai capture help)");
                println!("       seihai                  (menu if no mode given)");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Derived facts about binary level files, print, exit.
    if !level_info.is_empty() {
        let mut failed = false;
        for f in &level_info {
            let name = std::path::Path::new(f).file_name().map_or_else(|| f.clone(), |n| n.to_string_lossy().into_owned());
            match std::fs::read(f).map_err(|e| e.to_string()).and_then(|d| seihai::assets::Level::parse_binary_with_layout(&d)) {
                Ok((level, layout)) => {
                    println!("{name}");
                    print!("{}", layout.summary(&level));
                }
                Err(e) => {
                    eprintln!("{name}: {e}");
                    failed = true;
                }
            }
        }
        std::process::exit(i32::from(failed));
    }
    // Load settings and fill defaults.
    let mut settings = Settings::load();
    // Command line, then settings file, then the default (follow).
    let follow_serverwarp = settings.follow_serverwarp_with(cli_serverwarp);
    // Command line, then settings file, then the default (on).
    let sound_on = settings.sound_with(cli_sound);
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
    // Print what each build changed against the one before, then exit.
    if version_chain {
        let only = version_given.then_some(version);
        print!("{}", seihai::protocol::chain::render(chain_format, only));
        return;
    }
    // Use the version saved by the GUI if no CLI --version was given.
    if version == ClientVersion::V205 {
        if let Some(v) = &settings.version {
            version = ClientVersion::from_label(v).unwrap_or(ClientVersion::V205);
        }
    }
    // Check the data folder for one version or the five main ones, print, exit.
    if check_assets {
        let Some(dir) = &assets else {
            eprintln!("--check-assets needs --assets <dir>");
            std::process::exit(2);
        };
        let root = AssetRoot::new(dir);
        let versions: Vec<ClientVersion> = if version_given { vec![version] } else { ClientVersion::main_versions().to_vec() };
        let mut failed = false;
        for v in versions {
            let rep = seihai::assets::check_folder(&root, v.profile());
            print!("{}", seihai::assets::format_report(v.label(), &rep));
            failed |= rep.findings.iter().any(|f| {
                (f.need.required && !f.is_ok()) || matches!(f.status, seihai::assets::Status::BadFile(_))
            });
        }
        std::process::exit(i32::from(failed));
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
    // Buddy, ignore and mute lists from the settings directory.
    let mut lists = seihai::chatlists::ChatLists::load_default();
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
        lists = seihai::chatlists::ChatLists::load_default();
        net_state.set_filter(lists.filter(&version.profile().protocol.pli));
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
    // Every lookup of a level, GANI, image or sound goes through this: it
    // knows the folders of the 2.22 data folder and what this version opens.
    let resolver = AssetResolver::new(root.clone(), version.profile());
    // What to ask the server for, and where downloads are saved.
    let mut files = FileSession::new(resolver.clone(), settings.save_downloads_with(cli_save_downloads));
    // The frame a real client sends once the server's login burst is in
    // (image checks, process list, props), and the GANI it asks for at once.
    let mut login_burst = conn.as_ref().and_then(|_| LoginBurst::for_version(version));
    if login_burst.as_ref().is_some_and(|lb| lb.info().idle_gani) {
        send_file_msgs(conn.as_mut(), files.need("idle.gani"));
    }

    // Sound effects: optional. With no sounds folder, no audio device, or
    // `--no-sound`, the client runs the same and plays nothing.
    let sound_clock = Instant::now();
    let mut sound = if sound_on {
        let (out, why) = seihai::sound::default_backend();
        let dir = root.sounds_dir();
        match &dir {
            Some(d) => eprintln!("sound: {why}; sounds from {}", d.display()),
            None => eprintln!("sound: no sounds folder in {}; sound is silent", root.root.display()),
        }
        SoundEngine::new(out, dir, true)
    } else {
        SoundEngine::off()
    };
    if sound.is_enabled() {
        // The animations whose PLAYSOUND cues are played: footsteps and
        // water, one frame per step taken.
        for (name, swim) in [("walk.gani", false), ("swim.gani", true)] {
            if let Some(path) = resolver.gani(name) {
                match Gani::load(&path) {
                    Ok(g) if swim => session.set_swim_gani(GaniPlayer::new(g)),
                    Ok(g) => session.set_walk_gani(GaniPlayer::new(g)),
                    Err(e) => eprintln!("sound: cannot read {name}: {e}"),
                }
            }
        }
    }
    let mut sound_serial = session.level_serial;

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
        let path = resolver.level_path(&name);
        if resolver.level(&name).is_none() {
            eprintln!(
                "level {name} not found for {} (it opens {}); trying {}",
                version.label(),
                version.profile().level_extensions().join(" "),
                path.display()
            );
        }
        if let Err(e) = session.load_level(&path) {
            eprintln!("failed to load level: {e}");
        } else {
            // The level the session starts in is the first entry point.
            session.note_entry(&name);
        }
    }
    match resolver.tile_types() {
        Some((p, TileTypesFile::Text)) => {
            let _ = session.load_tiletypes(p);
        }
        Some((p, TileTypesFile::ArraysDat)) => {
            let _ = session.load_tiletypes_arrays(p);
        }
        None => {}
    }

    let profile = version.profile();
    // The death sequence of this build (none for builds before 2.05).
    session.life.set_rules(profile.life.death);
    session.combat.set_rules(profile.life.combat);
    let view_w = 40u32;
    let view_h = 30u32;
    let load = |dir: &str, name: &str| {
        let data = std::fs::read(resolver.image_in(dir, name)?).ok()?;
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
    // How long our own bubble has left, and the clock that runs it.
    let mut my_chat_timer = seihai::chatlists::BubbleTimer::default();
    let mut bubble_clock = Instant::now();
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
                        // Save completed files by the clients' folder rules,
                        // settle the replies, and reload levels.
                        let got = files.process(&mut net_state);
                        send_file_msgs(conn.as_mut(), got.send);
                        for name in &got.written {
                            // A neighbour's file (gmap) is not the current level.
                            let is_current = net_state
                                .current_level_name()
                                .map_or(true, |c| c.eq_ignore_ascii_case(name));
                            if name.ends_with(".nw") && !is_current {
                                eprintln!("loaded {name}");
                            } else if name.ends_with(".nw") {
                                if session.load_level(resolver.level_path(name)).is_ok() {
                                    level_from_board = false;
                                    // Scan for missing NPC images after the file arrives.
                                    if let Some(level) = session.current_level.clone() {
                                        npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images);
                                    }
                                }
                                eprintln!("loaded {name}");
                            } else if name.ends_with(".png") || name.ends_with(".gif") {
                                // A new copy of an image replaces the one in memory.
                                if let Some(data) = files.store.read(name) {
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
            // The burst has ended: the file checks, process list and props
            // a real client answers with.
            if changed {
                if let (Some(lb), Some(c)) = (login_burst.as_mut(), conn.as_mut()) {
                    let msgs = lb.take(&net_state, &account, |n| files.need(n));
                    if !msgs.is_empty() {
                        if let Err(e) = c.send_messages(&msgs) {
                            eprintln!("login frame send failed: {e}");
                        }
                    }
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
                        match session.load_level(resolver.level_path(&file)) {
                            Ok(()) => eprintln!("level {file}"),
                            Err(e) => {
                                eprintln!("level {file} not loaded ({e}); requesting");
                                session.current_level = None;
                                send_file_msgs(conn.as_mut(), files.need(&file));
                            }
                        }
                        loaded_level = Some(name.clone());
                        // Request missing NPC images.
                        if let Some(level) = session.current_level.clone() {
                            npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images);
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
                        // The client's second-of-day clock (UTC) for its once-a-second limit per attacker.
                        let second = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0, |d| (d.as_secs() % 86_400) as u32);
                        let own_nick = net_state.nickname.clone();
                        for r in session.receive_all(&net_state.others, own_cell, own_nick.as_deref(), second, &swings, &hits) {
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
                                | seihai::combat::Reject::NoPkZone
                                | seihai::combat::Reject::Friendly
                                | seihai::combat::Reject::AttackerBlocked
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
                let req = sync_world(&mut session, &mut net_state, &gmap_rules, &RootLoader(&resolver), &mut gmap_asked);
                send_world_requests(conn.as_mut(), req, &mut files);
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
                        if !session.is_loaded(&e.level) && session.load_level(resolver.level_path(&e.level)).is_err() {
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
        // A script warp request went out this pass: it carries the position,
        // so the plain walk update is left out.
        let mut script_warp_sent = false;
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
        // Sound: GANI cues from the steps just taken. Leaving a level stops
        // looped effects.
        if sound.is_enabled() {
            if session.level_serial != sound_serial {
                sound_serial = session.level_serial;
                sound.stop_all_loops();
            }
            let now = sound_clock.elapsed().as_millis() as u64;
            let listener = (
                session.player_x + seihai::assets::LISTENER_DX,
                session.player_y + seihai::assets::LISTENER_DY,
            );
            for ev in session.take_sound_events() {
                sound.play(&ev, listener, now);
            }
        } else {
            session.sound_events.clear();
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
                if let Some(c) = conn.as_mut() {
                    // Online the server owns the warp: ask as for a link the
                    // player walked into (PLI 0, or 30 with a cached copy)
                    // and load the level when the server names it.
                    eprintln!("script warp to {dest} at {x} {y}");
                    match online_link(
                        c,
                        version,
                        &mut session,
                        &mut net_state,
                        enter_level.as_mut(),
                        tamper.as_mut(),
                        &dest,
                        x,
                        y,
                    ) {
                        Ok(Some(started)) => {
                            warp_since = Some(Instant::now());
                            cached_arrival = started.cached_arrival;
                            script_warp_sent = true;
                        }
                        // Another warp is pending: nothing more to do.
                        Ok(None) => {}
                        // A profile that cannot send the request: the
                        // connection is fine, the warp just does not go.
                        Err(e) if e.kind() == std::io::ErrorKind::Unsupported => {
                            eprintln!("script warp to {dest} not sent: {e}");
                        }
                        Err(_) => conn = None,
                    }
                } else if session.load_level(resolver.level_path(&dest)).is_ok() {
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
                    level_from_board = resolver.level(n).is_none();
                }
                loaded_level = now;
                if let Some(n) = &loaded_level {
                    session.note_entry(n);
                }
                let req = sync_world(&mut session, &mut net_state, &gmap_rules, &RootLoader(&resolver), &mut gmap_asked);
                send_world_requests(conn.as_mut(), req, &mut files);
            } else if let (Some(c), false) = (conn.as_mut(), script_warp_sent) {
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
                        match online_link(
                            c,
                            version,
                            &mut session,
                            &mut net_state,
                            enter_level.as_mut(),
                            tamper.as_mut(),
                            &dest,
                            x,
                            y,
                        ) {
                            Ok(Some(started)) => {
                                warp_since = Some(Instant::now());
                                cached_arrival = started.cached_arrival;
                            }
                            Ok(None) => {}
                            Err(_) => conn = None,
                        }
                    } else {
                        // Offline: load the destination if it is local.
                        let path = resolver.level_path(&dest);
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
                        // The new server has seen no mute messages yet.
                        lists.mute_sent = 0;
                        net_state.set_filter(lists.filter(&version.profile().protocol.pli));
                        loaded_level = None;
                        session.leave_gmap();
                        gmap_asked.clear();
                        // The request lists belong to one connection.
                        files.new_connection();
                        login_burst = LoginBurst::for_version(version);
                        if login_burst.as_ref().is_some_and(|lb| lb.info().idle_gani) {
                            send_file_msgs(Some(&mut sess.conn), files.need("idle.gani"));
                        }
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
                    // /ignore, /buddy, /mute and friends change the lists
                    // and are not sent as chat.
                    ui::InputMode::Chat if seihai::chatlists::parse_command(&text).is_some() => {
                        let cmd = seihai::chatlists::parse_command(&text).unwrap();
                        let out = seihai::chatlists::apply_command(&cmd, &mut lists, &mut net_state, &profile.protocol.pli);
                        for n in out.notes {
                            net_state.push_chat(ChatKind::Admin, "Seihai".into(), n);
                        }
                        for (id, on) in out.mute_sends {
                            if let Some(Err(e)) = conn.as_mut().map(|c| c.send_mute(id, on)) {
                                eprintln!("mute send failed: {e}");
                            }
                        }
                        dirty = true;
                        None
                    }
                    ui::InputMode::Chat => match ui::classify_chat(&text) {
                        ui::ChatSend::Say(t) => {
                            my_chat = t.clone();
                            my_chat_timer = if t.is_empty() {
                                seihai::chatlists::BubbleTimer::default()
                            } else {
                                seihai::chatlists::BubbleTimer::start()
                            };
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

        // Chat bubbles go down after a few seconds.
        {
            let dt = bubble_clock.elapsed().as_millis().min(u32::MAX as u128) as u32;
            bubble_clock = Instant::now();
            if net_state.tick_bubbles(dt) {
                dirty = true;
            }
            if my_chat_timer.tick(dt) {
                my_chat.clear();
                dirty = true;
            }
        }
        if dirty {
            scene = draw(&session, &net_state.others, &my_chat, &npc_images);
            dirty = false;
        }
        let mut frame = scene.clone();
        if show_players {
            ui::draw_player_list(&mut frame, &me(&net_state, &account), &net_state.others, &lists.buddy, &net_state.filter);
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
