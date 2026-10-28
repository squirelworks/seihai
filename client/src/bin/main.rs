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

use seihai::assets::{read_png_rgb, read_png_rgba, write_png_rgb, AssetResolver, AssetRoot, Gani, GaniPlayer, Layer, Level, TileTypesFile};
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
    draw_other_player, draw_player_marker, draw_player_sprites, player_box_origin,
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

/// Load `walk.gani` or `swim.gani` (the animations whose cues are played)
/// from the first layer that has it, into the session.
fn load_motion_gani(resolver: &AssetResolver, session: &mut OfflineSession, name: &str) {
    if let Some(path) = resolver.gani(name) {
        match Gani::load(&path) {
            Ok(g) if name.eq_ignore_ascii_case("swim.gani") => session.set_swim_gani(GaniPlayer::new(g)),
            Ok(g) => session.set_walk_gani(GaniPlayer::new(g)),
            Err(e) => eprintln!("sound: cannot read {name}: {e}"),
        }
    }
}

/// An RGBA image `(width, pixels)` as a GANI sheet.
fn pose_sheet(i: &(u32, Vec<u8>)) -> seihai::render::Sheet<'_> {
    seihai::render::Sheet { px: i.1.as_slice(), w: i.0, bpp: 4 }
}

/// Load `sword.gani`, `hurt.gani` or `sit.gani` into the session, from the
/// first layer that has it. A missing file leaves the plain idle figure.
fn load_pose_gani(resolver: &AssetResolver, session: &mut OfflineSession, name: &str) {
    let (Some(kind), Some(path)) = (seihai::pose::PoseKind::from_file(name), resolver.gani(name)) else { return };
    match Gani::load(&path) {
        Ok(g) => session.set_pose_gani(kind, g),
        Err(e) => eprintln!("cannot read {name}: {e}"),
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
    // Each NPC's image and the images its script put up with showimg.
    for name in level.npcs.iter().flat_map(|n| n.image_names()) {
        if npc_images.contains_key(name) {
            continue;
        }
        let m = files.need(name);
        if !m.is_empty() {
            eprintln!("requesting {name}");
        }
        msgs.extend(m);
        if let Some(data) = files.store.read(name) {
            if let Ok((w, _, px)) = read_png_rgba(&data) {
                npc_images.insert(name.to_string(), (w, px));
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

/// How long the login 46 waits for the post-login frame before it goes alone.
const LATE_LOGIN_46: Duration = Duration::from_secs(3);

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
    let mut account_given = false;
    let mut send_tampercheck = false;
    let mut send_showimg = false;
    let mut no_scripts = false;
    // None: not given on the command line, so the settings file decides.
    let mut cli_serverwarp: Option<bool> = None;
    // None: not given on the command line, so the settings file decides.
    let mut cli_sound: Option<bool> = None;
    let mut cli_save_downloads: Option<bool> = None;
    let mut cli_cache_dir: Option<PathBuf> = None;
    while i < args.len() {
        match args[i].as_str() {
            "--assets" | "-a" => {
                i += 1;
                assets = args.get(i).map(PathBuf::from);
            }
            "--cache-dir" => {
                i += 1;
                match args.get(i) {
                    Some(d) => cli_cache_dir = Some(PathBuf::from(d)),
                    None => {
                        eprintln!("--cache-dir needs a folder");
                        std::process::exit(2);
                    }
                }
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
                    account_given = true;
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
                println!("seihai [--assets <dir>] [--level <name.nw>]");
                println!("         --assets <dir>      your own asset folder (optional). A file it lacks comes");
                println!("                             from the built-in default set; with no folder the");
                println!("                             built-in set is all there is, and offline starts in");
                println!("                             new1.nw");
                println!("         --cache-dir <dir>   where per-server download caches live (default:");
                println!("                             <settings dir>/cache; settings file: cache_dir=)");
                println!("                             search order: server cache, --assets, built-in set");
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
                println!("       seihai [--assets <dir>] --check-assets [--version 2.22]");
                println!("         --check-assets      resolve and load the files a version uses from the");
                println!("                             --assets folder, or from the built-in set when none");
                println!("                             is given, and print what is found; without");
                println!("                             --version, the five main versions. Exit 1 if a");
                println!("                             required file is missing or unreadable");
                println!("       seihai --level-info <file.graal|file.zelda> [--level-info <file> ...]");
                println!("         --level-info        print sizes, offsets and record counts of a binary");
                println!("                             .graal/.zelda level (no content). Exit 1 if a file");
                println!("                             does not parse; a Z3-V1 file must also reach its");
                println!("                             end with no anomaly");
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
            let data = std::fs::read(f).map_err(|e| e.to_string());
            // Z3-V1 (`.zelda`) has its own layout report (computers, no NPCs
            // or chests, and every deviation from the layout in words).
            if let Ok(d) = &data {
                if seihai::assets::is_zelda(d) {
                    match seihai::assets::ZeldaLevel::parse(d) {
                        Ok(z) => {
                            println!("{name}");
                            print!("{}", z.layout.summary());
                            failed |= !z.layout.is_clean();
                        }
                        Err(e) => {
                            eprintln!("{name}: {e}");
                            failed = true;
                        }
                    }
                    continue;
                }
            }
            match data.and_then(|d| seihai::assets::Level::parse_binary_with_layout(&d)) {
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
    // An empty path (an empty field in the settings window) is no folder.
    assets = assets.filter(|p| !p.as_os_str().is_empty());
    if !account_given {
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
    // `--version` wins; the version saved by the GUI applies only when the
    // flag was not given (a given 2.05 is not replaced by the saved one).
    version = settings.version_with(version_given.then_some(version));
    // `--online host:port` is used as given. A listserver host with no port
    // gets the selected version's default port.
    if let Some(a) = online.as_mut() {
        *a = version.complete_list_addr(a);
    }
    // Check the data folder for one version or the five main ones, print, exit.
    if check_assets {
        let root = match &assets {
            Some(dir) => AssetRoot::new(dir),
            None => match seihai::assets::builtin::default_dir() {
                Ok(d) => {
                    println!("no --assets folder: checking the built-in default set in {}", d.display());
                    AssetRoot::new(d)
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(2);
                }
            },
        };
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
                eprintln!("{}", seihai::net::list_failure_text(addr, &e));
                std::process::exit(1);
            }
        }
    }
    // Do not auto-fill online from settings. The GUI is the waiting area.
    // Settings only pre-fill the form.

    let mut session = OfflineSession::new();
    session.link_boxes = version.profile().levels.link_boxes;
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
    // When the login happened, for the late login 46 below.
    let mut login_since = Instant::now();
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
            if addr.contains("14921") || addr.contains("14922") || addr.contains("14923") || addr.contains("listserver") {
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
                    Err(e) => eprintln!("{}", seihai::net::list_failure_text(addr, &e)),
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
                    login_since = Instant::now();
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
    // The layers, searched in this order: the server's cache (what it sent),
    // the user's asset folder, the built-in default set.
    let settings_now = Settings::load();
    let user_root = match assets.filter(|p| !p.as_os_str().is_empty()) {
        Some(p) if p.is_dir() => {
            let r = AssetRoot::new(p);
            if !r.is_valid() {
                eprintln!("no pics1.png in {}; a file that folder lacks comes from the built-in assets", r.root.display());
            }
            Some(r)
        }
        Some(p) => {
            eprintln!("assets folder {} not found; using the built-in assets", p.display());
            None
        }
        None => None,
    };
    let builtin_dir = match seihai::assets::builtin::default_dir() {
        Ok(d) => Some(d),
        Err(e) => {
            eprintln!("{e}; running without the built-in assets");
            None
        }
    };
    if user_root.is_none() && builtin_dir.is_none() {
        eprintln!("no assets: give --assets <dir> or fix the settings folder so the built-in assets can be written");
        return;
    }
    let cache_root = cli_cache_dir
        .clone()
        .or_else(|| settings_now.cache_dir.clone())
        .unwrap_or_else(seihai::servercache::default_cache_root);
    // The server's name is known only when the settings file has it for this address.
    let server_name = |addr: &str| -> Option<String> {
        (settings_now.last_server.as_deref() == Some(addr)).then(|| settings_now.last_server_name.clone()).flatten()
    };
    let cache_for = |addr: &str, name: Option<&str>| seihai::servercache::cache_dir_for(&cache_root, addr, name);
    let cache_dir = online.as_deref().and_then(|a| cache_for(a, server_name(a).as_deref()));
    // Every lookup of a level, GANI, image or sound goes through this: it
    // knows the folders of the 2.22 data folder and what this version opens.
    // The built-in levels are for offline play only: online, a level of the
    // same name (`new1.nw`) must come from the server.
    let mut resolver = AssetResolver::empty(version.profile())
        .with_assets(user_root)
        .with_cache(cache_dir.clone())
        .with_builtin(builtin_dir)
        .with_builtin_levels(online.is_none());
    // What to ask the server for, and where downloads are saved (the cache).
    let mut files = FileSession::new(resolver.clone(), settings.save_downloads_with(cli_save_downloads));
    if let Some(a) = online.as_deref() {
        files.store.set_cache_meta(Some(seihai::servercache::meta_text(a, server_name(a).as_deref())));
        if let Some(d) = &cache_dir {
            eprintln!("download cache: {}", d.display());
        }
    }
    // A fresh install with no folder of its own starts offline in the built-in level.
    if level_name.is_none() && online.is_none() && resolver.user_root().is_none() && resolver.level("new1.nw").is_some() {
        level_name = Some("new1.nw".to_string());
    }
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
        let dirs = resolver.sound_dirs();
        if dirs.is_empty() {
            eprintln!("sound: no sounds folder; sound is silent");
        } else {
            let list: Vec<String> = dirs.iter().map(|d| d.display().to_string()).collect();
            eprintln!("sound: {why}; sounds from {}", list.join(", "));
        }
        SoundEngine::with_dirs(out, dirs, true)
    } else {
        SoundEngine::off()
    };
    // walk.gani and swim.gani are stepped once per step taken: their PLAYSOUND
    // cues are the footsteps and the water, and their frames are what is
    // drawn, so they load whether or not sound is on.
    for name in ["walk.gani", "swim.gani"] {
        load_motion_gani(&resolver, &mut session, name);
    }
    // The pose animations (idle, sword swing, hurt, sit) are drawn too.
    for name in ["idle.gani", "sword.gani", "hurt.gani", "sit.gani", "carrystill.gani", "carry.gani"] {
        load_pose_gani(&resolver, &mut session, name);
    }
    let mut sound_serial = session.level_serial;

    let Some(sheet_path) = resolver.image("pics1.png") else {
        eprintln!("no pics1.png in any asset folder");
        return;
    };
    let sheet_layer = resolver.layer_of(&sheet_path);
    eprintln!(
        "tiles: {} ({})",
        sheet_path.display(),
        match sheet_layer {
            Some(Layer::Assets) => "your assets",
            Some(Layer::Cache) => "server cache",
            Some(Layer::Builtin) => "built-in sheet (set assets to your Graal folder)",
            None => "unknown",
        }
    );
    // The open-chest art swap is for our own sheet only.
    let view_opts = seihai::render::ViewOptions { builtin_sheet: resolver.layer_of(&sheet_path) == Some(Layer::Builtin) };
    let sheet_data = match std::fs::read(&sheet_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read {}: {e}", sheet_path.display());
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
            eprintln!("tile types: {} (text)", p.display());
            let _ = session.load_tiletypes(p);
        }
        Some((p, TileTypesFile::ArraysDat)) => {
            eprintln!("tile types: {} (arrays.dat)", p.display());
            let _ = session.load_tiletypes_arrays(p);
        }
        None => eprintln!("tile types: none (walls will not match the real client)"),
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
    let body_path = resolver.image_in("bodies", "body.png");
    let head_path = resolver.image_in("heads", "head0.png");
    eprintln!(
        "player: body {}  head {}",
        body_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "(none)".into()),
        head_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "(none)".into()),
    );
    let body = load("bodies", "body.png");
    let head = load("heads", "head0.png");
    let sprites = load("", "sprites.png");
    let sword_sheet = load("swords", "sword1.png");
    // The build's own status display; without it, the plain bars.
    let state_sheet = load("", "state.png");
    // The sign panel's frame and font; without it, the plain text panel.
    let letters_sheet = load("", "letters.png");
    if body.is_none() || head.is_none() {
        eprintln!("no body.png / head0.png found; drawing a marker for the player");
    }
    let draw = |session: &OfflineSession,
                others: &[seihai::protocol::OtherPlayer],
                my_chat: &str,
                npc_imgs: &HashMap<String, (u32, Vec<u8>)>,
                item: Option<&str>|
     -> Framebuffer {
        let mut fb = seihai::render::render_view_with(session, &sheet, sw, sh, view_w, view_h, view_opts);
        let (cam_x, cam_y) = (
            ((session.player_x + 1.5) * 16.0 - (view_w as f32 * 16.0) / 2.0) as i32,
            ((session.player_y + 1.5) * 16.0 - (view_h as f32 * 16.0) / 2.0) as i32,
        );
        // NPC images and animations on top of the gray placeholder, then the
        // images their scripts put up with showimg.
        let npc_art = seihai::render::NpcArt {
            images: npc_imgs,
            sheets: seihai::render::PoseSheets {
                body: body.as_ref().map(pose_sheet),
                head: head.as_ref().map(pose_sheet),
                sprites: sprites.as_ref().map(pose_sheet),
                sword: sword_sheet.as_ref().map(pose_sheet),
            },
        };
        seihai::render::draw_npcs(&mut fb, session, &npc_art, cam_x, cam_y);
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
        // What other players hold over their heads (CARRYSPRITE), at the
        // same place as our own carried object.
        if let Some(sp) = &sprites {
            for other in others {
                if let Some(id) = other.carry {
                    let (x, y) = seihai::throwing::carry_origin(other.x, other.y);
                    let at = |v: f32| (v * 16.0).round() as i32;
                    seihai::render::draw_sprite(&mut fb, pose_sheet(sp), session.object_tables.as_ref(), id, at(x) - cam_x, at(y) - cam_y);
                }
            }
        }
        let (dx, dy) = player_box_origin(session, view_w, view_h);
        let pose = session.current_pose();
        match (&body, &head) {
            (Some(_), Some(_)) if pose.is_some() => {
                let sheets = seihai::render::PoseSheets {
                    body: body.as_ref().map(pose_sheet),
                    head: head.as_ref().map(pose_sheet),
                    sprites: sprites.as_ref().map(pose_sheet),
                    sword: sword_sheet.as_ref().map(pose_sheet),
                };
                if let Some(p) = pose {
                    seihai::render::draw_gani_frame(&mut fb, p.gani, p.frame, session.player_dir, &sheets, dx, dy);
                }
            }
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
        // What the player carries, what is in flight, and the break effects:
        // sprite-sheet cut-outs with their transparency (see throwing.md).
        if let Some(sp) = &sprites {
            seihai::render::draw_throw_layer(&mut fb, session, pose_sheet(sp), cam_x, cam_y);
        }
        ui::draw_other_labels(&mut fb, others, (cam_x, cam_y));
        ui::draw_bubble(&mut fb, my_chat, dx, dy);
        match &state_sheet {
            Some(st) => seihai::render::hud::draw_hud(
                &mut fb,
                profile.ui.hud,
                &seihai::render::hud::HudSheets {
                    state: pose_sheet(st),
                    sprites: sprites.as_ref().map(pose_sheet),
                    sword: sword_sheet.as_ref().map(pose_sheet),
                    item: item.and_then(|n| npc_imgs.get(n)).map(pose_sheet),
                },
                session.object_tables.as_ref(),
                &seihai::render::hud::HudValues {
                    half_hearts: session.hearts_half(),
                    max_half_hearts: session.max_hearts,
                    rupees: session.rupees,
                    bombs: session.bombs,
                    arrows: session.darts,
                    ap: i32::from(session.alignment),
                    mp: 0,
                    glove_power: i32::from(session.glove_power),
                    sword_power: i32::from(session.sword_power),
                    weapons: true,
                    stats: seihai::render::hud::stats::ALL,
                    keys: ['A', 'S', 'D', 'Q'],
                },
            ),
            None => fb.draw_hud(session.player_health, session.rupees),
        }
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
    // The weapon list and the selected weapon (the HUD's D slot), and whether
    // a file arrived that may be its image.
    let mut weapon_bar = seihai::weaponbar::WeaponBar::default();
    let mut weapon_asked: HashSet<String> = HashSet::new();
    let mut weapon_recheck = true;
    let mut scene = draw(&session, &[], &my_chat, &npc_images, None);
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
    // The item list (Q): open or not, and the selected weapon.
    let mut item_list = seihai::inventory::ItemList::default();
    let mut confirm_quit = false;
    // The first line of the sign page on show.
    let mut sign_page = 0usize;
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
            } else if item_list.open {
                item_list.open = false;
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
                        weapon_recheck |= !got.written.is_empty();
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
                            } else if name.to_ascii_lowercase().ends_with(".wav") {
                                // A new copy of a sound replaces the decoded one.
                                sound.forget(name);
                            } else if seihai::pose::PoseKind::from_file(name).is_some() {
                                load_pose_gani(&resolver, &mut session, name);
                            } else if name.eq_ignore_ascii_case("walk.gani") || name.eq_ignore_ascii_case("swim.gani") {
                                load_motion_gani(&resolver, &mut session, name);
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
                    let (lx, ly) = (net_state.player_x.unwrap_or(session.player_x), net_state.player_y.unwrap_or(session.player_y));
                    c.set_gmap_cell(net_state.gmap.as_ref().map(|g| g.cell));
                    let mut msgs = lb.take(
                        &net_state,
                        &account,
                        |n| files.need(n),
                        || enter_level.as_mut().map(|el| el.take_login()).unwrap_or_default(),
                        || c.first_gr_flags(lx, ly),
                    );
                    // The real 2.31 client's first id 24 rides at the end of this frame.
                    if !msgs.is_empty() {
                        if let Some(t) = showimg.as_mut() {
                            if t.advance(Duration::ZERO, !net_state.others.is_empty()) {
                                msgs.push(showimg_message());
                            }
                        }
                    }
                    if !msgs.is_empty() {
                        if let Err(e) = c.send_messages(&msgs) {
                            eprintln!("login frame send failed: {e}");
                        }
                    }
                }
            }
            // A server that never ends its burst with PLO_LISTPROCESSES would
            // leave the login 46 owed for good: send it alone after a while.
            if let (Some(el), Some(c)) = (enter_level.as_mut(), conn.as_mut()) {
                if el.login_owed() && login_since.elapsed() > LATE_LOGIN_46 {
                    let msgs = el.take_login();
                    if let Err(e) = c.send_messages(&msgs) {
                        eprintln!("late enter-level send failed: {e}");
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
                        // The board has no chest or sign records of its own:
                        // take the server's, once each has arrived.
                        if level.chests.len() != net_state.chests.iter().filter(|r| r.len() >= 3).count() {
                            level.chests.clear();
                            level.add_server_chests(&net_state.chests);
                        }
                        if level.signs.len() != net_state.signs.len() {
                            level.signs.clear();
                            level.add_server_signs(&net_state.signs);
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
                                | seihai::combat::Reject::OutOfBlast)) => {
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
                // Another player threw what they carried: the object flies
                // from where they stand, and can hit us.
                for o in std::mem::take(&mut net_state.throws) {
                    if let Some(sprite) = o.carry {
                        let own_cell = net_state.gmap.as_ref().map(|g| g.cell);
                        let (x, y) = o.position_from(own_cell);
                        session.throw_remote(sprite, x, y, o.dir, o.id);
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

        // The item list (Q). While it is open the arrows move its
        // selection and the game keys close it; none of them reach the
        // player this frame, so the player does not walk, swing or interact.
        let list_was_open = item_list.open;
        if !input.is_open() && !confirm_quit {
            let entries = item_list_entries(&session, &net_state);
            let pressed = |k| window.is_key_pressed(k, minifb::KeyRepeat::No);
            if pressed(minifb::Key::Q) {
                item_list.toggle(entries.len());
                eprintln!("item list {} ({} entries)", if item_list.open { "opened" } else { "closed" }, entries.len());
                dirty = true;
            } else if item_list.open {
                use seihai::inventory::ListKey;
                item_list.clamp(entries.len());
                let (_, _, cw, _) = seihai::inventory::content_rect(scene.width as i32, scene.height as i32);
                let cells = seihai::inventory::layout(&entries, cw);
                let before = item_list.selected;
                for (k, lk) in [
                    (minifb::Key::Up, ListKey::Up),
                    (minifb::Key::Left, ListKey::Left),
                    (minifb::Key::Down, ListKey::Down),
                    (minifb::Key::Right, ListKey::Right),
                    (minifb::Key::D, ListKey::Close),
                    (minifb::Key::S, ListKey::Close),
                    (minifb::Key::A, ListKey::Close),
                    (minifb::Key::Space, ListKey::Close),
                    (minifb::Key::Enter, ListKey::Close),
                ] {
                    if item_list.open && pressed(k) {
                        item_list.key(lk, &cells);
                        dirty = true;
                    }
                }
                if item_list.selected != before {
                    if let Some(n) = item_list.selected.and_then(|i| entries.get(i)) {
                        eprintln!("item list: selected {n}");
                    }
                }
                if !item_list.open {
                    eprintln!("item list closed");
                }
            }
        }

        // Health and death: the 50 ms death timer, the claim and the dead
        // bit on the wire, and the respawn on a movement or action key.
        {
            let dt = life_clock.elapsed().as_millis() as u64;
            life_clock = Instant::now();
            let mut events = session.tick_life(dt);
            let keys_down = !input.is_open() && !list_was_open
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
            // Thrown objects and their break effects run on the same clock.
            // A thrown object stops on the first NPC whose box holds its
            // probe point; the NPC gets `waspelt`.
            if session.tick_throw_with(dt, &|x, y| scripts.world.npc_at(f64::from(x), f64::from(y))) {
                dirty = true;
            }
            for id in session.take_pelts() {
                scripts.pelt(&mut session, id);
            }
            // Another player's throw hurt us: the hurt sprite goes out.
            sprite_changed |= session.take_throw_hurt();
            for l in session.take_landings() {
                match l.effect {
                    Some(e) => eprintln!("thrown object {} landed at {:.1},{:.1}: break effect {e}", l.kind, l.x, l.y),
                    None => eprintln!("thrown object {} landed at {:.1},{:.1}: stays on the ground", l.kind, l.x, l.y),
                }
            }
            // A hit pushed the player: the new position goes out like a step.
            sprite_changed |= session.take_knock_moved();
            if !input.is_open() && !list_was_open && window.is_key_pressed(minifb::Key::S, minifb::KeyRepeat::No) && session.start_swing() {
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
        // player faces. S while carrying throws (its swing was refused above).
        let interact = !input.is_open()
            && !list_was_open
            && !session.life.is_dead()
            && seihai::throwing::interact_pressed(
                session.carried.is_some(),
                window.is_key_pressed(minifb::Key::A, minifb::KeyRepeat::No),
                window.is_key_pressed(minifb::Key::Space, minifb::KeyRepeat::No),
                window.is_key_pressed(minifb::Key::S, minifb::KeyRepeat::No),
            );
        if interact {
            // Reading: the next three lines, or the end of the sign.
            if let Some(text) = &session.reading {
                match seihai::render::signs::next_page(text, sign_page) {
                    Some(p) => sign_page = p,
                    None => session.reading = None,
                }
            } else {
                sign_page = 0;
                match session.interact() {
                    seihai::offline::Interaction::Chest { x, y, item, got } => {
                        eprintln!("chest {x},{y}: {item}");
                        net_state.push_chat(ChatKind::Admin, "Chest".into(), format!("You got {got}."));
                        if let Some(c) = conn.as_mut() {
                            let _ = c.send_open_chest(x, y);
                        }
                    }
                    seihai::offline::Interaction::Lifted { kind } => {
                        eprintln!("lifted object {kind}");
                        // Others see the object (CARRYSPRITE) and the carry
                        // pose (the sprite value).
                        if let Some(c) = conn.as_mut() {
                            let sent = c
                                .send_carry_sprite(seihai::throwing::carry_sprite(kind))
                                .and_then(|()| c.send_walk_sprite(&session.player_update(), session.sprite_value()));
                            if let Err(e) = sent {
                                eprintln!("send failed: {e}");
                                conn = None;
                            }
                        }
                    }
                    seihai::offline::Interaction::Thrown { kind } => {
                        eprintln!("thrown object {kind} (facing {}, flies up to {} ticks)", session.player_dir, seihai::throwing::FLIGHT_TICKS);
                        if let Some(c) = conn.as_mut() {
                            match c.send_throw_carried() {
                                Ok(()) => {
                                    eprintln!("thrown object {kind}: PLI_THROWCARRIED sent");
                                    let sent = c
                                        .send_carry_sprite(None)
                                        .and_then(|()| c.send_walk_sprite(&session.player_update(), session.sprite_value()));
                                    if let Err(e) = sent {
                                        eprintln!("send failed: {e}");
                                        conn = None;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("send failed: {e}");
                                    conn = None;
                                }
                            }
                        }
                    }
                    seihai::offline::Interaction::Sign(_)
                    | seihai::offline::Interaction::Nothing => {}
                }
            }
            dirty = true;
        }
        // No arrow key held: the player stands (idle animation).
        if !window.is_key_down(minifb::Key::Left)
            && !window.is_key_down(minifb::Key::Right)
            && !window.is_key_down(minifb::Key::Up)
            && !window.is_key_down(minifb::Key::Down)
            && session.stop_walking()
        {
            dirty = true;
        }
        // Up while reading turns back a page.
        if session.reading.is_some() && !input.is_open() && window.is_key_pressed(minifb::Key::Up, minifb::KeyRepeat::No) {
            if let Some(p) = seihai::render::signs::previous_page(sign_page) {
                sign_page = p;
                dirty = true;
            }
        }
        if !input.is_open() && !item_list.open && !list_was_open && !session.life.is_dead() && session.reading.is_none() && session.warp_pending.is_none() && last_step.elapsed() >= session.step_interval() {
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
                let before = (session.player_x, session.player_y);
                session.try_move(dx, dy);
                moved = true;
                // The first steps of a connection: the files the real client
                // asks for then, if Seihai has no copy of them.
                if conn.is_some() && (session.player_x, session.player_y) != before {
                    send_file_msgs(conn.as_mut(), files.on_step());
                }
                last_step = seihai::offline::next_step_clock(last_step, Instant::now(), session.step_interval());
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
                // Pictures of this level's NPCs (online they were asked for when the
                // level arrived; a second pass asks for nothing twice).
                if let Some(level) = session.current_level.clone() {
                    npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images);
                }
                dirty = true;
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
            // A script changed an NPC picture (setimg, showimg): load it.
            if std::mem::take(&mut scripts.images_changed) {
                if let Some(level) = session.current_level.clone() {
                    npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images);
                }
                dirty = true;
            }
            // NPC animations (setcharani, setani): load the GANIs they name, then
            // step them. A missing file is replaced by an empty animation, so the
            // NPC keeps its image and the name is not looked up again.
            for name in session.wanted_npc_anis() {
                let gani = resolver.gani(&format!("{name}.gani")).and_then(|p| Gani::load(p).ok());
                if gani.is_none() {
                    eprintln!("NPC animation {name}.gani not found");
                }
                session.set_npc_gani(&name, gani.unwrap_or_else(|| Gani::parse("").expect("empty gani")));
            }
            if session.tick_npc_anis(dt) {
                dirty = true;
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
                    c.set_gmap_cell(Some(cx.to));
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

        // The gr.x/gr.y flags follow the player's position on every tick,
        // also when the server moved the player (a warp or an arrival).
        if session.warp_pending.is_none() {
            if let Some(c) = conn.as_mut() {
                c.set_gmap_cell(session.gmap.as_ref().map(|g| g.cell));
                if let Err(e) = c.send_gr_flags(session.player_x, session.player_y) {
                    eprintln!("send failed: {e}");
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
            // Not before the post-login frame: an id 24 ahead of the server's login
            // burst made the lab server warp the player to its online start level
            // first (documentation/early-enter-level.md).
            let held = login_burst.as_ref().is_some_and(|lb| lb.pending()) && login_since.elapsed() <= LATE_LOGIN_46;
            if let (Some(t), Some(c), false) = (showimg.as_mut(), conn.as_mut(), held) {
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
                eprintln!("server warp to {:?} not followed (--no-serverwarp or follow_serverwarp=0)", w.display_name());
            } else if let Some(old) = conn.as_ref() {
                let _ = old.shutdown();
                conn = None;
                frames = None;
                session.cancel_warp();
                warp_since = None;
                cached_arrival = None;
                let target = w.address();
                eprintln!("server warp to {:?}: {}", w.display_name(), target.as_deref().unwrap_or("no usable host and port"));
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
                        // The clients keep field 1 as the server's name (without
                        // the type prefix) and show it in the window title.
                        window.set_title(&profile.window_title(Some(&w.display_name())));
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
                        // The new server has its own cache folder, searched first.
                        let warp_cache = cache_for(&a, Some(&w.display_name()));
                        resolver.set_cache(warp_cache.clone());
                        files.store.set_cache(warp_cache, Some(seihai::servercache::meta_text(&a, Some(&w.display_name()))));
                        sound.set_dirs(resolver.sound_dirs());
                        login_burst = LoginBurst::for_version(version);
                        if login_burst.as_ref().is_some_and(|lb| lb.info().idle_gani) {
                            send_file_msgs(Some(&mut sess.conn), files.need("idle.gani"));
                        }
                        tamper = sess.tamper.take();
                        tamper_clock = Instant::now();
                        enter_level = Some(sess.enter_level.clone());
                        login_since = Instant::now();
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
        let enter = window.is_key_pressed(minifb::Key::Enter, minifb::KeyRepeat::No)
            || window.is_key_pressed(minifb::Key::NumPadEnter, minifb::KeyRepeat::No);
        let talk = !confirm_quit && window.is_key_pressed(minifb::Key::T, minifb::KeyRepeat::No);
        if !confirm_quit && !list_was_open && ui::chat_key_action(input.is_open(), enter, talk) != ui::ChatKeyAction::None {
            if !input.is_open() {
                eprintln!("chat: line opened ({})", if enter { "Enter" } else { "T" });
                input.open(ui::InputMode::Chat);
                dirty = true;
            } else if let Some((mode, text)) = input.submit() {
                eprintln!("chat: Enter pressed, {} characters typed", text.chars().count());
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
                            // The playerchats scripts run first and may rewrite the
                            // line (a jail NPC turns "unstick me" into something
                            // else); the real client sends only what is left.
                            let t = if no_scripts { t } else { scripts.chat(&mut session, &t) };
                            // History + bubble + held bubble clock (ui::own_say).
                            let who = me(&net_state, &account);
                            ui::own_say(&mut net_state, &who, &t, &mut my_chat, &mut my_chat_timer);
                            dirty = true;
                            let online = conn.is_some();
                            let r = conn.as_mut().map(|c| c.send_chat(&t));
                            match &r {
                                Some(Ok(())) => eprintln!("chat: sent {} characters as chat (CURCHAT), bubble and log set", t.chars().count()),
                                Some(Err(_)) => {}
                                None => eprintln!("chat: not sent, not connected (shown in the bubble and log only; online: {online})"),
                            }
                            r
                        }
                        ui::ChatSend::Toall(t) if !t.is_empty() => {
                            net_state.push_chat(ChatKind::Toall, me(&net_state, &account), t.clone());
                            match conn.as_mut().map(|c| c.send_toall(&t)) {
                                Some(r) => {
                                    if r.is_ok() {
                                        eprintln!("chat: sent {} characters as toall", t.chars().count());
                                    }
                                    Some(r)
                                }
                                None => {
                                    eprintln!("chat: toall not sent, not connected");
                                    None
                                }
                            }
                        }
                        ui::ChatSend::Toall(_) => None,
                    },
                    ui::InputMode::Pm { id, name } if !text.trim().is_empty() => {
                        net_state.push_chat(ChatKind::Private, format!("to {name}"), text.clone());
                        let r = conn.as_mut().map(|c| c.send_pm(&[id], &text));
                        match &r {
                            Some(Ok(())) => eprintln!("chat: sent {} characters as a PM to {name}", text.chars().count()),
                            Some(Err(_)) => {}
                            None => eprintln!("chat: PM not sent, not connected"),
                        }
                        r
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
            for (pressed, is_f4) in [(f4, true), (f5, false)] {
                if !pressed {
                    continue;
                }
                match ui::editor_key(profile.ui.editor, &profile.label, frames.is_some(), is_f4) {
                    ui::EditorKey::Refuse(msg) => {
                        net_state.push_chat(ChatKind::Admin, "Seihai".into(), msg);
                    }
                    ui::EditorKey::Run if is_f4 => eprintln!("F4: level editor not yet implemented"),
                    ui::EditorKey::Run => {
                        let exts = profile.level_extensions();
                        if let Some(path) =
                            seihai::gui::pick_level(resolver.root().levels_dir().as_path(), &exts)
                        {
                            if let Err(e) = session.load_level(&path) {
                                eprintln!("load failed: {e}");
                            } else {
                                eprintln!("loaded {}", path.display());
                                dirty = true;
                            }
                        }
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

        // Our own typed line is held over the head until a direction key is
        // pressed, then goes down like the others (as in the 2.05 client).
        if !input.is_open()
            && [minifb::Key::Left, minifb::Key::Right, minifb::Key::Up, minifb::Key::Down]
                .iter()
                .any(|k| window.is_key_down(*k))
        {
            my_chat_timer.release();
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
        // Weapons that arrived (at login or later) go into the list; the
        // selected one's image is asked for once and drawn when it is here.
        {
            let before = weapon_bar.current_image().map(str::to_string);
            weapon_bar.sync(
                &net_state.default_weapons,
                net_state.weapons.iter().map(|w| (w.name.as_str(), w.image.as_deref())),
                profile.files.image_ext,
            );
            let now = weapon_bar.current_image().map(str::to_string);
            if now != before {
                dirty = true;
                weapon_recheck = true;
            }
            if let Some(name) = now.filter(|n| !npc_images.contains_key(n)) {
                if weapon_asked.insert(name.clone()) {
                    let m = files.need(&name);
                    if !m.is_empty() {
                        eprintln!("requesting {name} (selected weapon)");
                    }
                    send_file_msgs(conn.as_mut(), m);
                }
                if std::mem::take(&mut weapon_recheck) {
                    if let Some((w, _, px)) = files.store.read(&name).and_then(|d| read_png_rgba(&d).ok()) {
                        npc_images.insert(name, (w, px));
                        dirty = true;
                    }
                }
            }
        }
        if dirty {
            scene = draw(&session, &net_state.others, &my_chat, &npc_images, weapon_bar.current_image());
            dirty = false;
        }
        let mut frame = scene.clone();
        if item_list.open {
            let entries = item_list_entries(&session, &net_state);
            let counts = seihai::inventory::Counts {
                rupees: session.rupees,
                bombs: session.bombs,
                arrows: session.darts,
                hearts_half: session.hearts_half(),
                max_hearts_half: session.max_hearts,
                sword: session.sword_power,
                shield: session.shield_power,
                glove: session.glove_power,
            };
            seihai::inventory::draw(&mut frame, &item_list, &entries, &counts);
        }
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
            match &letters_sheet {
                Some(l) => {
                    let row = player_box_origin(&session, view_w, view_h).1 as f32 / 16.0;
                    seihai::render::signs::draw_sign(&mut frame, &pose_sheet(l), text, sign_page, row);
                }
                None => ui::draw_panel(&mut frame, "Sign", text),
            }
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
        ui::draw_chat_overlay(&mut frame, &net_state.chat_log, net_state.chat_clock_ms, chat_scroll, &input);
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
/// The item list's entries: the server's built-in and named weapons, and
/// offline the built-in weapons got from chests.
fn item_list_entries(session: &seihai::offline::OfflineSession, state: &SessionState) -> Vec<String> {
    let mut named: Vec<String> = seihai::inventory::chest_weapons(&session.flags);
    named.extend(state.weapons.iter().map(|w| w.name.clone()));
    seihai::inventory::entries(&state.default_weapons, &named)
}

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
