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
use seihai::keys::{Action, KeyMap};
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
/// load the image we have. A name that could not be loaded goes into
/// `not_received` and is not looked for again until a file arrives: each
/// look searches the folders of every layer, and a script that sets an image
/// every frame runs this pass every frame.
fn npc_image_pass(
    level: &Level,
    files: &mut FileSession,
    conn: Option<&mut Connection>,
    npc_images: &mut HashMap<String, (u32, Vec<u8>)>,
    not_received: &mut HashSet<String>,
) {
    let mut msgs = Vec::new();
    // Each NPC's image and the images its script put up with showimg.
    for name in level.npcs.iter().flat_map(|n| n.image_names()) {
        if npc_images.contains_key(name) || not_received.contains(name) {
            continue;
        }
        let m = files.need(name);
        if !m.is_empty() {
            eprintln!("requesting {name}");
        }
        msgs.extend(m);
        match files.store.read(name).and_then(|data| read_png_rgba(&data).ok()) {
            Some((w, _, px)) => {
                npc_images.insert(name.to_string(), (w, px));
            }
            None => {
                not_received.insert(name.to_string());
            }
        }
    }
    send_file_msgs(conn, msgs);
}

/// The default picture of a shield power (`shield1.png`), or an empty
/// name for power 0.
fn local_shield_image(power: u8) -> String {
    if power == 0 { String::new() } else { format!("shield{power}.png") }
}

/// Load the pictures other players are drawn with (their sword, shield,
/// head and body props) and the local player's shield into `images`: from
/// the asset folders first, else from the server. `tried` keeps the names
/// already looked for in the folders, and `not_received` the names the
/// server has not sent yet, so a missing one is not searched for every frame
/// (each search reads folders on every layer). `not_received` is emptied when
/// a file arrives.
fn look_image_pass(
    wanted: &[(&str, String)],
    resolver: &AssetResolver,
    files: &mut FileSession,
    conn: Option<&mut Connection>,
    images: &mut HashMap<String, (u32, Vec<u8>)>,
    tried: &mut std::collections::HashSet<String>,
    not_received: &mut HashSet<String>,
) -> bool {
    let mut msgs = Vec::new();
    let mut added = false;
    for (dir, name) in wanted {
        if images.contains_key(name) {
            continue;
        }
        let read = |data: Vec<u8>| read_png_rgba(&data).ok().map(|(w, _, px)| (w, px));
        let mut got = None;
        if tried.insert(name.clone()) {
            got = resolver.image_in(dir, name).and_then(|p| std::fs::read(p).ok()).and_then(read);
        }
        // Asked for already and nothing arrived since: asking again would
        // search the folders again for nothing.
        if got.is_none() && not_received.contains(name) {
            continue;
        }
        if got.is_none() {
            got = files.store.read(name).and_then(read);
        }
        match got {
            Some(img) => {
                images.insert(name.clone(), img);
                added = true;
            }
            None => {
                not_received.insert(name.clone());
                msgs.extend(files.need(name));
            }
        }
    }
    send_file_msgs(conn, msgs);
    added
}

/// Draw body pictures in the colours they are worn in (see `looks.rs`) and
/// keep each under [`seihai::looks::colored_body_key`] in `images`. The
/// picture's bytes come from the asset folders (looked for once a name, kept
/// in `raw`) or from the files the server sent. A body not here yet is
/// asked for by [`look_image_pass`] under its plain name. `not_received`
/// is as there.
fn colored_body_pass(
    wanted: &[(String, [u8; 5])],
    resolver: &AssetResolver,
    files: &mut FileSession,
    raw: &mut HashMap<String, Vec<u8>>,
    tried: &mut HashSet<String>,
    images: &mut HashMap<String, (u32, Vec<u8>)>,
    not_received: &mut HashSet<String>,
) -> bool {
    let mut added = false;
    for (name, colors) in wanted {
        let key = seihai::looks::colored_body_key(name, colors);
        if images.contains_key(&key) {
            continue;
        }
        if !raw.contains_key(name) {
            let mut got = None;
            if tried.insert(format!("colored body {name}")) {
                got = resolver.image_in("bodies", name).and_then(|p| std::fs::read(p).ok());
            }
            let miss = format!("colored body {name}");
            if got.is_none() && !not_received.contains(&miss) {
                got = files.store.read(name);
            }
            match got {
                Some(data) => {
                    raw.insert(name.clone(), data);
                }
                None => {
                    not_received.insert(miss);
                }
            }
        }
        let Some(data) = raw.get(name) else { continue };
        match seihai::assets::read_png_rgba_recolored(data, &seihai::looks::body_recolor(colors)) {
            Ok((w, _, px)) => {
                images.insert(key, (w, px));
                added = true;
            }
            // Not decoded again every frame: only when a new copy arrives.
            Err(_) => {
                raw.remove(name);
                not_received.insert(format!("colored body {name}"));
            }
        }
    }
    added
}

/// Heart containers `--dev-repair-hearts` asks for: GServer-v2's default
/// `heartlimit`.
const DEV_REPAIR_HEARTS: u8 = 3;

/// Tell the server what changed in the player's hearts and pose
/// ([`OfflineSession::take_power_report`]): the real client reports its own
/// CURPOWER, and GServer keeps whatever it last heard. Only for builds with
/// the death rules, the ones whose health handling was traced. Returns true
/// when a message went out. A failed send drops the connection.
fn send_power_report(conn: &mut Option<Connection>, session: &mut OfflineSession, version: ClientVersion) -> bool {
    let Some(report) = session.take_power_report() else { return false };
    if version.death_rules().is_none() {
        return false;
    }
    let Some(msg) = report.message(version.profile().ui.ganis) else { return false };
    let Some(c) = conn.as_mut() else { return false };
    eprintln!("hearts report: {report:?}");
    if let Err(e) = c.send_messages(&[msg]) {
        eprintln!("send failed: {e}");
        *conn = None;
        return false;
    }
    true
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
    // Developer option: answer a server MAXPOWER of 0 with our own maximum.
    let mut dev_repair_hearts = false;
    let mut bigmap_file: Option<PathBuf> = None;
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
            "--dev-repair-hearts" => {
                if seihai::dev::DevMode::AVAILABLE {
                    dev_repair_hearts = true;
                } else {
                    eprintln!("--dev-repair-hearts needs a build with the dev feature; ignored");
                }
            }
            "--bigmap" => {
                i += 1;
                bigmap_file = args.get(i).map(PathBuf::from);
            }
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
                println!("         --cache-dir <dir>   also keep a copy of each download here (downloads go to");
                println!("                             _cache/ next to the program; without this the old");
                println!("                             <settings dir>/cache is only read; settings file: cache_dir=)");
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
                println!("         --bigmap FILE       the big map (a bigmap.txt grid of level names): edge links");
                println!("                             to the level next door on it keep the offset across the");
                println!("                             seam (online, the server's PLO_BIGMAP file is used)");
                println!("         --no-scripts        do not run the client-side GS1 scripts of level NPCs");
                println!("         --dev-repair-hearts (developer) when the server says max hearts 0, send max 3 once;");
                println!("                             not what a real client does, and a server with an NPC server ignores it");
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
    // Server NPCs (`PLO_NPCPROPS`): the level they were linked into, and
    // the values the server has for each (see `offline/servernpcs.rs`).
    let mut npc_link_serial = u64::MAX;
    let mut npc_sync = seihai::offline::ServerNpcSync::default();
    let mut npc_sync_clock = Instant::now();

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
        npc_link_serial = u64::MAX;

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
    // The cache in the Seihai folder (`_cache/<host>_<port>`), searched and
    // written before the settings' one.
    let seihai_dir = seihai::servercache::seihai_dir();
    let game_cache_for = |addr: &str| seihai_dir.as_deref().and_then(|d| seihai::servercache::game_cache_dir_for(d, addr));
    let game_cache_dir = online.as_deref().and_then(game_cache_for);
    // Every lookup of a level, GANI, image or sound goes through this: it
    // knows the folders of the 2.22 data folder and what this version opens.
    // The built-in levels are for offline play only: online, a level of the
    // same name (`new1.nw`) must come from the server.
    let mut resolver = AssetResolver::empty(version.profile())
        .with_assets(user_root)
        .with_game_cache(game_cache_dir.clone())
        .with_cache(cache_dir.clone())
        .with_builtin(builtin_dir)
        .with_builtin_levels(online.is_none());
    // What to ask the server for, and where downloads are saved (the cache).
    let mut files = FileSession::new(resolver.clone(), settings.save_downloads_with(cli_save_downloads));
    // Downloads go to the Seihai folder's `_cache`. The settings' cache is
    // still searched (what older runs saved there); it gets a copy only when
    // the user named it, and takes the download when `_cache` is unavailable.
    let settings_copy = cli_cache_dir.is_some() || settings_now.cache_dir.is_some();
    files.store.set_settings_copy(settings_copy);
    if let Some(a) = online.as_deref() {
        files.store.set_cache_meta(Some(seihai::servercache::meta_text(a, server_name(a).as_deref())));
        match (&game_cache_dir, &cache_dir) {
            (Some(g), Some(c)) if settings_copy => eprintln!("download cache: {} (copy in {})", g.display(), c.display()),
            (Some(g), _) => eprintln!("download cache: {}", g.display()),
            (None, Some(c)) => eprintln!("download cache: {}", c.display()),
            (None, None) => {}
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
    // A gen-2 build (1.33 to 1.39) sends its own props once the first level
    // has loaded instead (see `classic_login_props_message`).
    let mut classic_props_owed = conn.is_some() && seihai::protocol::login::sends_classic_login_props(version);
    // 2.22 sends its own walk and sword props on a 100 ms cadence, once a
    // tick, with the tile changes of the tick (see `protocol::walkcadence`).
    let mut walk_cadence = conn.as_ref().and_then(|_| seihai::protocol::walkcadence::WalkCadence::for_version(version));
    let mut cadence_clock = Instant::now();
    session.gani_swing = version == ClientVersion::V222;
    let mut held_boards: Vec<Vec<u8>> = Vec::new();

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
    // walk.gani is stepped once per step taken and swim.gani once per tick: their PLAYSOUND
    // cues are the footsteps and the water, and their frames are what is
    // drawn, so they load whether or not sound is on.
    for name in ["walk.gani", "swim.gani"] {
        load_motion_gani(&resolver, &mut session, name);
    }
    // The pose animations (idle, sword swing, hurt, sit) are drawn too.
    for name in [
        "idle.gani", "sword.gani", "hurt.gani", "sit.gani", "carrystill.gani", "carry.gani", "dead.gani", "push.gani", "grab.gani", "pull.gani",
        "lift.gani", "spin.gani", "hiddenstill.gani", "hidden.gani",
    ] {
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
    if let Some(p) = &bigmap_file {
        match std::fs::read(p) {
            Ok(d) => {
                session.big_map = seihai::gmap::Gmap::parse_grid(&String::from_utf8_lossy(&d));
                eprintln!("big map: {} ({})", p.display(), if session.big_map.is_some() { "read" } else { "no levels" });
            }
            Err(e) => eprintln!("big map: cannot read {}: {e}", p.display()),
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
    // GANI attributes (`ATTR1` … `ATTR5`, the hat) exist from 2.02, the
    // build that added props 37 to 41; before it they draw nothing.
    let gani_attrs = profile.props.own.contains(37);
    // A build that reads player art as .gif only (up to 2.05; see
    // `AssetResolver::image_in`). With a part of the figure missing it draws
    // the rest, never a marker; with no body, no shadow either (lab, real
    // 2.05 on a PNG-only folder).
    let gif_art = profile.files.image_ext.eq_ignore_ascii_case(".gif");
    // The readme keys; from 1.38 `keyset_<action>` lines in the settings
    // file replace them.
    let keymap = KeyMap::for_version(profile.ui.key_config, &Settings::load().keysets);
    let button_keys = [keymap.label(Action::Act), keymap.label(Action::Attack), keymap.label(Action::Item), keymap.label(Action::ItemList)];
    // The death sequence of this build (none for builds before 2.05).
    session.life.set_rules(profile.life.death);
    session.combat.set_rules(profile.life.combat);
    session.set_mannerisms(profile.mannerisms);
    // Who drops bush and pot items: the 1.x roll, or the server.
    session.set_drops(profile.drops);
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
    // The chat font of this version (see render::chattext): the game
    // folders first (2.x ship tempsitc.ttf), then the system's fonts.
    let chat_font = {
        let dirs: Vec<std::path::PathBuf> = resolver.user_root().into_iter().chain(resolver.builtin_root()).map(|r| r.root.clone()).collect();
        let f = seihai::render::chattext::ChatFont::find(profile.ui.chat_text, &dirs);
        match &f {
            Some(f) => eprintln!("chat font: {}", f.path.display()),
            None => eprintln!("chat font: none found for {:?}; using the built-in letters", profile.ui.chat_text.face),
        }
        f
    };
    if (body.is_none() || head.is_none()) && !gif_art {
        eprintln!("no body.png / head0.png found; drawing a marker for the player");
    }
    let draw = |session: &OfflineSession,
                others: &[seihai::protocol::OtherPlayer],
                my_chat: &str,
                my_nick: Option<&str>,
                npc_imgs: &HashMap<String, (u32, Vec<u8>)>,
                item: Option<&str>,
                own_failed: (bool, bool)|
     -> Framebuffer {
        let mut fb = seihai::render::render_view_with(session, &sheet, sw, sh, view_w, view_h, view_opts);
        let (cam_x, cam_y) = (
            ((session.player_x + 1.5) * 16.0 - (view_w as f32 * 16.0) / 2.0) as i32,
            ((session.player_y + 1.5) * 16.0 - (view_h as f32 * 16.0) / 2.0) as i32,
        );
        // NPCs under and with the players now; `drawoverplayer` NPCs, the
        // showimg images and `drawaslight` NPCs after the player.
        let npc_art = seihai::render::NpcArt {
            images: npc_imgs,
            sheets: seihai::render::PoseSheets {
                body: body.as_ref().map(pose_sheet),
                head: head.as_ref().map(pose_sheet),
                sprites: sprites.as_ref().map(pose_sheet),
                sword: sword_sheet.as_ref().map(pose_sheet),
                shield: None,
                attrs: Default::default(),
            },
            gani_attrs,
        };
        seihai::render::draw_npcs(&mut fb, session, &npc_art, cam_x, cam_y, seihai::render::NpcPass::BelowPlayers);
        // The items on the floor, under the players.
        if let Some(sp) = &sprites {
            seihai::render::draw_floor_items(&mut fb, session, pose_sheet(sp), cam_x, cam_y);
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
        if (body.is_none() || head.is_none()) && !gif_art {
            for other in others {
                let ox = (other.x * 16.0) as i32 - cam_x;
                let oy = (other.y * 16.0) as i32 - cam_y;
                draw_player_marker(&mut fb, false, ox, oy);
            }
        }
        if gif_art || (body.is_some() && head.is_some()) {
            let clock_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
            for other in others {
                // Drawn through its GANI with its own pictures when the pose is
                // loaded; a picture not here yet falls back to the default one.
                if let Some((gani, frame)) = seihai::render::other_pose(session, other, clock_ms) {
                    let own = |n: &Option<String>| n.as_deref().and_then(|n| npc_imgs.get(n)).map(pose_sheet);
                    let l = &other.looks;
                    let colored = npc_imgs.get(&other_body_key(other)).map(pose_sheet);
                    let other_body = colored.or(own(&l.body)).or(body.as_ref().map(pose_sheet));
                    let sheets = seihai::render::PoseSheets {
                        body: other_body,
                        head: own(&l.head).or(head.as_ref().map(pose_sheet)),
                        // No body in a GIF build: no shadow either.
                        sprites: sprites.as_ref().filter(|_| other_body.is_some() || !gif_art).map(pose_sheet),
                        // No sword prop yet: the default sword; power 0: none.
                        sword: match l.sword.as_deref() {
                            None => sword_sheet.as_ref().map(pose_sheet),
                            Some("") => None,
                            Some(_) => own(&l.sword).or(sword_sheet.as_ref().map(pose_sheet)),
                        },
                        shield: own(&l.shield),
                        attrs: if gani_attrs { attr_sheets(&l.attrs, gani, npc_imgs) } else { Default::default() },
                    };
                    let (ox, oy) = ((other.x * 16.0) as i32 - cam_x, (other.y * 16.0) as i32 - cam_y);
                    seihai::render::draw_gani_frame(&mut fb, gani, frame, other.dir, &sheets, ox, oy);
                    continue;
                }
                let (Some((bw, b)), Some((hw, h))) = (&body, &head) else { continue };
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
            (b, h) if pose.is_some() && (gif_art || (b.is_some() && h.is_some())) => {
                // Our own head and body, the body in our colours; until
                // they are here, the default pictures. A GIF build whose
                // server could not send one draws none for it.
                let looks = &session.looks;
                let own_body = npc_imgs.get(&seihai::looks::colored_body_key(looks.body_name(), &looks.colors));
                let own_head = npc_imgs.get(looks.head_name());
                let body_sheet = own_body.or(b.as_ref().filter(|_| !(gif_art && own_failed.0))).map(pose_sheet);
                let sheets = seihai::render::PoseSheets {
                    body: body_sheet,
                    head: own_head.or(h.as_ref().filter(|_| !(gif_art && own_failed.1))).map(pose_sheet),
                    // No body in a GIF build: no shadow either.
                    sprites: sprites.as_ref().filter(|_| body_sheet.is_some() || !gif_art).map(pose_sheet),
                    sword: sword_sheet.as_ref().map(pose_sheet),
                    // The default shield of the player's shield power.
                    shield: npc_imgs.get(&local_shield_image(session.shield_power)).map(pose_sheet),
                    attrs: pose.filter(|_| gani_attrs).map(|p| attr_sheets(&looks.attrs, p.gani, npc_imgs)).unwrap_or_default(),
                };
                // The hurt blink: on its off ticks only the shadow (the
                // SPRITES sheet's placements) is drawn.
                let shadow_only = seihai::render::PoseSheets { sprites: sheets.sprites, ..Default::default() };
                let sheets = if session.blink_off() { &shadow_only } else { &sheets };
                if let Some(p) = pose {
                    seihai::render::draw_gani_frame(&mut fb, p.gani, p.frame, session.player_dir, sheets, dx, dy);
                }
            }
            // The hurt blink without GANIs: nothing on the off ticks. Hidden
            // under the carried object without its GANIs: only the object.
            (Some(_), Some(_)) if session.blink_off() || session.figure_hidden() => {}
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
            _ if gif_art => {}
            _ => draw_player_marker(&mut fb, session.in_water, dx, dy),
        }
        // What the player carries, what is in flight, and the break effects:
        // sprite-sheet cut-outs with their transparency (see throwing.md).
        if let Some(sp) = &sprites {
            seihai::render::draw_throw_layer(&mut fb, session, pose_sheet(sp), cam_x, cam_y);
        }
        seihai::render::draw_npcs(&mut fb, session, &npc_art, cam_x, cam_y, seihai::render::NpcPass::AbovePlayers);
        seihai::render::draw_npc_messages(&mut fb, session, &npc_art, chat_font.as_ref(), cam_x, cam_y);
        ui::draw_other_labels(&mut fb, chat_font.as_ref(), others, (cam_x, cam_y));
        ui::draw_bubble(&mut fb, chat_font.as_ref(), my_chat, dx, dy, session.carried.is_some());
        if let Some(nick) = my_nick {
            seihai::render::chattext::draw_nick_text(&mut fb, chat_font.as_ref(), nick, session.alignment, dx, dy);
        }
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
                    keys: button_keys,
                },
            ),
            None => fb.draw_hud(session.player_health, session.rupees),
        }
        // Images on the screen layer (`changeimgvis i,4`) go over the status bar.
        seihai::render::draw_npcs(&mut fb, session, &npc_art, cam_x, cam_y, seihai::render::NpcPass::Screen);
        // The minimap is set by the game server from 2.1 on.
        if profile.ui.minimap {
            if let Some(level) = &session.current_level {
                fb.draw_minimap(level, session.player_x, session.player_y);
            }
        }
        fb
    };
    let mut npc_images: HashMap<String, (u32, Vec<u8>)> = HashMap::new();
    let mut look_tried = std::collections::HashSet::new();
    let mut look_not_received: HashSet<String> = HashSet::new();
    // Body pictures as read, for drawing them in other colours.
    let mut body_raw: HashMap<String, Vec<u8>> = HashMap::new();
    let mut my_chat = String::new();
    // How long our own bubble has left, and the clock that runs it.
    let mut my_chat_timer = seihai::chatlists::BubbleTimer::default();
    let mut bubble_clock = Instant::now();
    // The weapon list and the selected weapon (the HUD's D slot), and whether
    // a file arrived that may be its image.
    let mut weapon_bar = seihai::weaponbar::WeaponBar::default();
    // The looks last sent to the server (none yet on this connection).
    let mut looks_sent: Option<seihai::looks::Looks> = None;
    let mut weapon_asked: HashSet<String> = HashSet::new();
    let mut weapon_recheck = true;
    // Our own nickname shows for 60 frames after the start, a change of our
    // AP and a typed `showname` (2.05); other players' always shows.
    let mut my_name_timer = seihai::chatlists::BubbleTimer::start();
    let mut my_name_ap = session.alignment;
    let mut scene = draw(&session, &[], &my_chat, Some(&me(&net_state, &account)), &npc_images, None, (false, false));
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
    let debug_on = Settings::load().debug.unwrap_or(false);
    let mut show_debug = debug_on;
    // Debug diagnostics go to one folder per run, named for the time
    // this process started. The file is opened once and appended to;
    // nothing creates a second folder while the process is alive.
    // The run's debug folder, also used for the weapon scripts.
    let mut debug_dir: Option<PathBuf> = None;
    let mut debug_file: Option<std::fs::File> = if debug_on {
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("_debug");
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                let s = d.as_secs();
                // YYYYMMDD-HHMMSS in UTC.
                let days = s / 86400;
                let tod = s % 86400;
                let (h, m, sec) = (tod / 3600, (tod % 3600) / 60, tod % 60);
                // Civil from days since 1970-01-01 (Howard Hinnant).
                let z = days as i64 + 719468;
                let era = if z >= 0 { z } else { z - 146096 } / 146097;
                let doe = (z - era * 146097) as u64;
                let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
                let y = yoe as i64 + era * 400;
                let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
                let mp = (5 * doy + 2) / 153;
                let d = doy - (153 * mp + 2) / 5 + 1;
                let mth = if mp < 10 { mp + 3 } else { mp - 9 };
                let y = if mth <= 2 { y + 1 } else { y };
                format!("{y:04}{mth:02}{d:02}-{h:02}{m:02}{sec:02}")
            })
            .unwrap_or_else(|_| "unknown".into());
        let dir = root.join(format!("debug-{stamp}"));
        std::fs::create_dir_all(&dir).ok();
        debug_dir = Some(dir.clone());
        let path = dir.join("script.log");
        let pkt_path = dir.join("packets.log");
        match std::fs::File::create(&path) {
            Ok(f) => {
                eprintln!("debug: writing to {}", path.display());
                if let Ok(pf) = std::fs::File::create(&pkt_path) {
                    eprintln!("debug: packets to {}", pkt_path.display());
                    // Store both in a tuple for now; we restructure below.
                    let _ = pf;
                }
                Some(f)
            }
            Err(e) => {
                eprintln!("debug: could not open {}: {e}", path.display());
                None
            }
        }
    } else {
        None
    };
    // Packet log: every frame sent and received, with id and length.
    let mut packet_log: Option<std::fs::File> = if debug_on {
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("_debug");
        // Find the most recent debug folder.
        let dir = std::fs::read_dir(&root).ok().and_then(|rd| {
            rd.filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().starts_with("debug-")).max_by_key(|e| e.file_name().to_string_lossy().to_string())
        });
        dir.and_then(|d| std::fs::File::create(d.path().join("packets.log")).ok())
    } else {
        None
    };
    // Each unique script diagnostic is written once. The same unhandled
    // effect otherwise fills the file every tick.
    let mut seen_debug: std::collections::HashSet<String> = std::collections::HashSet::new();
    // The level the debug log last named.
    let mut debug_level: Option<String> = None;
    let mut show_players = false;
    // The item list (Q): open or not, and the selected weapon.
    let mut item_list = seihai::inventory::ItemList::default();
    let mut confirm_quit = false;
    // The first line of the sign page on show.
    let mut sign_page = 0usize;
    // Some game key was down last frame (the 2.05 sign debounce).
    let mut game_keys_held = false;
    let mut show_help = false;
    let started = Instant::now();
    let mut warp_since: Option<Instant> = None;
    let mut hearts_repair_sent = false;
    // Where our own link request lands the player, held by the destination's
    // name until the server names that level (see `LinkArrival`).
    let mut link_arrival: Option<seihai::net::LinkArrival> = None;
    let mut was_online = false;
    let mut frames_seen = 0usize;
    let mut fps = 0.0f32;
    let mut fps_count = 0u32;
    let mut fps_since = Instant::now();
    let mut dirty = true;

    let mut last_step = Instant::now();
    // STATUS (prop 18) as the server last gave it; the dead bit is toggled on it.
    let mut status_base = 0u8;
    // STATUS bit 0x02 last sent (hiding under a carried object).
    let mut last_hide_status = 0u8;
    let mut life_clock = Instant::now();
    let mut combat_clock = Instant::now();
    // The push / grab / pull sprite number last sent (see `pushpull_sprite_s`).
    let mut last_pp_sprite: Option<u8> = None;
    let mut char_repeat: ui::CharRepeat<minifb::Key> = ui::CharRepeat::new();
    let run_clock = Instant::now();
    let mut slow_log = SlowFrameLog::default();
    while window.is_open() {
        let mut ft = FrameTimes::start();
        let typed_now: Vec<char> = std::mem::take(&mut *typed.lock().unwrap());
        if !input.is_open() {
            char_repeat.clear();
        }
        if !input.is_open()
            && (window.is_key_pressed(minifb::Key::F1, minifb::KeyRepeat::No) || typed_now.contains(&'?'))
        {
            show_help = !show_help;
        }
        if input.is_open() {
            // The key that went down with the typed text (not a modifier),
            // so a held key repeats where the window system does not.
            let pressed = window.get_keys_pressed(minifb::KeyRepeat::No).into_iter().find(|k| !is_modifier(*k));
            for c in char_repeat.frame(run_clock.elapsed(), &typed_now, pressed, |k| window.is_key_down(k)) {
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
        ft.mark("input");
        // Online: apply every frame that has arrived, without blocking.
        if let Some(rx) = &frames {
            let mut changed = false;
            loop {
                match rx.try_recv() {
                    Ok(Ok(body)) => {
                        frames_seen += 1;
                        if let Some(f) = packet_log.as_mut() {
                            use std::io::Write;
                            let id = body.first().copied().unwrap_or(0);
                            let _ = writeln!(f, "recv id {id} len {}", body.len());
                            let _ = f.flush();
                        }
                        net_state.apply_body(&body);
                        // Save completed files by the clients' folder rules,
                        // settle the replies, and reload levels.
                        let got = files.process(&mut net_state);
                        send_file_msgs(conn.as_mut(), got.send);
                        weapon_recheck |= !got.written.is_empty();
                        if !got.written.is_empty() {
                            look_not_received.clear();
                        }
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
                                        npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
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
                            } else if seihai::filecache::is_image_name(name) {
                                // A new copy of an image replaces the one in memory.
                                if let Some(data) = files.store.read(name) {
                                    match read_png_rgba(&data) {
                                        Ok((w, _, px)) => {
                                            npc_images.insert(name.clone(), (w, px));
                                            eprintln!("image {name}");
                                        }
                                        Err(e) => eprintln!("image {name} not read: {e}"),
                                    }
                                }
                                // The level's NPCs may name it in another case:
                                // look their names up again (the store's lookup
                                // ignores case).
                                if let Some(level) = session.current_level.clone() {
                                    npc_images.retain(|k, _| k == name || !k.eq_ignore_ascii_case(name));
                                    npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
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
                    let mut msgs = lb.take(
                        &net_state,
                        &account,
                        |n| files.need(n),
                        || enter_level.as_mut().map(|el| el.take_login()).unwrap_or_default(),
                        || c.take_flags(),
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
                    // Debug: keep each weapon's script text, so a parse error
                    // in the script log can be read against its source.
                    if let Some(dir) = &debug_dir {
                        let dir = dir.join("weapons");
                        for w in &ws {
                            let safe: String = w.name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
                            let path = dir.join(format!("{safe}.txt"));
                            if !path.exists() && std::fs::create_dir_all(&dir).is_ok() {
                                let _ = std::fs::write(&path, w.script);
                            }
                        }
                    }
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
                            npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
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
                        // A new level for the scripts too: the server's NPCs
                        // (`PLO_NPCPROPS`) are put on it.
                        session.level_serial += 1;
                        level_from_board = true;
                        eprintln!("level from the server's board");
                    }
                }
                if classic_props_owed && session.current_level.is_some() {
                    classic_props_owed = false;
                    let m = seihai::protocol::login::classic_login_props_message(version, &net_state.player_props, &account, net_state.own_prop_ids.as_ref());
                    if let Some(c) = conn.as_mut() {
                        if let Err(e) = c.send_messages(&[m]) {
                            eprintln!("login props send failed: {e}");
                        }
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
                // Our own link: the position is the one the request carried.
                // A cached link gets no PLO_PLAYERWARP, and GServer-v2 sends
                // none for a plain level either (only the level). The
                // server's, if it did send one, wins.
                // The level may arrive after the warp was given up (a level
                // the server sends in full can be slow): the arrival is held
                // by name, not by the pending warp.
                let arrival = match (entered, net_state.current_level_name()) {
                    (true, Some(n)) => seihai::net::LinkArrival::take_for(&mut link_arrival, &n),
                    _ => None,
                };
                if let Some((x, y)) = arrival {
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
                // Our head, body and colours as the server has them (after a
                // `sethead` chat command, for one).
                if let Some(h) = net_state.own_head.take() {
                    session.looks.head = Some(h);
                    dirty = true;
                }
                if let Some(b) = net_state.own_body.take() {
                    session.looks.body = Some(b);
                    dirty = true;
                }
                if let Some(c) = net_state.own_colors.take() {
                    session.looks.colors = c;
                    dirty = true;
                }
                for (own, new) in session.looks.attrs.iter_mut().zip(net_state.own_attrs.iter_mut()) {
                    if let Some(a) = new.take() {
                        *own = Some(a);
                        dirty = true;
                    }
                }
                // Hearts, AP and STATUS the server gave.
                if let Some(st) = net_state.own_status.take() {
                    status_base = st;
                    session.combat.set_has_spin(st & seihai::combat::HAS_SPIN_STATUS != 0);
                }
                if let Some(g) = net_state.own_glove_power.take() {
                    session.glove_power = g;
                }
                // Rupees, bombs and arrows: the server's count (a pickup
                // online from 1.38 is given this way).
                if let Some(v) = net_state.own_rupees.take() {
                    session.rupees = v as i32;
                }
                if let Some(v) = net_state.own_bombs.take() {
                    session.bombs = i32::from(v);
                }
                if let Some(v) = net_state.own_darts.take() {
                    session.darts = i32::from(v);
                }
                // Developer option: the account's maximum is 0, so the server
                // holds 0 hearts and every respawn gets 0. Send our own
                // maximum once per connection; the server answers with CURPOWER.
                if dev_repair_hearts && !hearts_repair_sent && net_state.own_max_power == Some(0) {
                    if let Some(c) = conn.as_mut() {
                        eprintln!("dev: server max power 0; sending max power {DEV_REPAIR_HEARTS}");
                        hearts_repair_sent = true;
                        if let Err(e) = c.send_messages(&[seihai::net::max_power_message(DEV_REPAIR_HEARTS)]) {
                            eprintln!("send failed: {e}");
                        }
                    }
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
                                | seihai::combat::Reject::Hidden
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
                            // The hearts report carries the new CURPOWER,
                            // GANI `hurt` and the hurt sprite, as the real
                            // client sends it in the tick of the hit. Without
                            // one, hurt mode still shows in the sprite.
                            if !send_power_report(&mut conn, &mut session, version) {
                                if let Some(c) = conn.as_mut() {
                                    if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)) {
                                        eprintln!("send failed: {e}");
                                        conn = None;
                                    }
                                }
                            }
                        }
                    }
                }
                // Items the server put down or took away (after a level
                // change above cleared the old level's).
                for it in std::mem::take(&mut net_state.items) {
                    match it {
                        seihai::protocol::ServerItem::Add(it) => session.add_server_item(it),
                        seihai::protocol::ServerItem::Del(x, y) => session.remove_server_item(x, y),
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
                // The big map the server named (PLO_BIGMAP): its grid
                // decides which levels are next door for edge links.
                if let (None, Some(spec)) = (session.big_map.as_ref(), net_state.bigmap.as_ref()) {
                    if let Some(data) = files.store.read(&spec.txt) {
                        session.big_map = seihai::gmap::Gmap::parse_grid(&String::from_utf8_lossy(&data));
                        eprintln!("big map: {} ({})", spec.txt, if session.big_map.is_some() { "read" } else { "no levels" });
                    } else if gmap_asked.insert(format!("file:{}", spec.txt.to_ascii_lowercase())) {
                        let msgs = files.need(&spec.txt);
                        if let (false, Some(c)) = (msgs.is_empty(), conn.as_mut()) {
                            eprintln!("requesting {}", spec.txt);
                            if let Err(e) = c.send_messages(&msgs) {
                                eprintln!("file request send failed: {e}");
                            }
                        }
                    }
                }
                dirty = true;
            }
        }

        // A sign being read takes the game keys this frame (2.05): Up turns
        // back a page, the map key closes it, any other game key turns to
        // the next page or closes it on the last. The player does not walk,
        // swing, lift or open the item list meanwhile.
        let sign_was_open = session.reading.is_some();
        if sign_was_open && !input.is_open() && !confirm_quit {
            use seihai::render::signs::{next_page, previous_page, sign_key, SignKey};
            match sign_key(|a| keymap.pressed(&window, a), game_keys_held) {
                Some(SignKey::Close) => session.reading = None,
                Some(SignKey::Back) => sign_page = previous_page(sign_page).unwrap_or(sign_page),
                Some(SignKey::Next) => match session.reading.as_deref().and_then(|t| next_page(t, sign_page)) {
                    Some(p) => sign_page = p,
                    None => session.reading = None,
                },
                None => {}
            }
            dirty = true;
        }
        game_keys_held = Action::ALL.iter().any(|&a| a != Action::SwitchToChat && keymap.down(&window, a));

        // The item list (Q). While it is open the arrows move its
        // selection and the game keys close it; none of them reach the
        // player this frame, so the player does not walk, swing or interact.
        let list_was_open = item_list.open;
        if !input.is_open() && !confirm_quit && !sign_was_open {
            let entries = item_list_entries(&session, &net_state);
            let pressed = |k| window.is_key_pressed(k, minifb::KeyRepeat::No);
            if keymap.pressed(&window, Action::ItemList) {
                item_list.toggle(entries.len());
                eprintln!("item list {} ({} entries)", if item_list.open { "opened" } else { "closed" }, entries.len());
                dirty = true;
            } else if item_list.open {
                use seihai::inventory::ListKey;
                item_list.clamp(entries.len());
                let (_, _, cw, _) = seihai::inventory::content_rect(scene.width as i32, scene.height as i32);
                let cells = seihai::inventory::layout(&entries, cw);
                let before = item_list.selected;
                for (a, lk) in [
                    (Action::Up, ListKey::Up),
                    (Action::Left, ListKey::Left),
                    (Action::Down, ListKey::Down),
                    (Action::Right, ListKey::Right),
                    (Action::Item, ListKey::Close),
                    (Action::Attack, ListKey::Close),
                    (Action::Act, ListKey::Close),
                ] {
                    if item_list.open && keymap.pressed(&window, a) {
                        item_list.key(lk, &cells);
                        dirty = true;
                    }
                }
                if item_list.open && pressed(minifb::Key::Enter) {
                    item_list.key(ListKey::Close, &cells);
                    dirty = true;
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
            let keys_down = !input.is_open() && !list_was_open && keymap.any_direction_down(&window)
                || !input.is_open() && (keymap.pressed(&window, Action::Act) || keymap.pressed(&window, Action::Attack));
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
                        // On the same level GServer only moves the player
                        // (X and Y props, no level), and the real client does
                        // not wait: it walks at once (2.05 live, 2026-10-29).
                        // A level change waits for the level as any warp does.
                        let same_level = net_state.current_level_name().is_some_and(|n| n.eq_ignore_ascii_case(dest));
                        let (x, y, dir) = (session.player_x, session.player_y, session.player_dir);
                        if same_level {
                            session.arrive(x, y);
                        } else {
                            session.begin_warp(dest);
                            warp_since = Some(Instant::now());
                        }
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
            // A death (CURPOWER 0, GANI `dead`), a respawn (the hearts by AP,
            // after the warp and the status clear), a lava burn, a heart.
            send_power_report(&mut conn, &mut session, version);
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
            // The items on the floor: bounce, blink, pickup and timeout.
            if !session.floor_items.is_empty() {
                dirty = true;
            }
            session.tick_items(dt, conn.is_some());
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
            // A ledge jump in the air, or a pull that brought a block along:
            // the position goes out the same way.
            sprite_changed |= session.tick_pushpull(dt);
            if session.pushpull.pull_pose() || session.push_pose() {
                dirty = true;
            }
            // The swim stroke and the lift run on the tick, moving or not.
            dirty |= session.tick_poses(dt);
            // The spin attack's pose changes every tick.
            dirty |= session.combat.spin_elapsed_ms().is_some();
            // The hurt blink changes the picture every tick of hurt mode.
            dirty |= session.combat.is_hurt();
            // Push, grab and pull have their own sprite numbers.
            let pp_sprite = session.pushpull_sprite_s();
            if pp_sprite != last_pp_sprite {
                last_pp_sprite = pp_sprite;
                sprite_changed = true;
                dirty = true;
            }
            if !input.is_open() && !list_was_open && !sign_was_open && keymap.pressed(&window, Action::Attack) && session.start_swing() {
                sprite_changed = true;
            }
            // A thrown object's hit, or the end of hurt mode (GANI `idle`).
            send_power_report(&mut conn, &mut session, version);
            if sprite_changed {
                dirty = true;
                if let (Some(c), None) = (conn.as_mut(), &walk_cadence) {
                    if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)) {
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
        // Action (A): close the reading panel, or interact with what the
        // player faces. Attack (S, Space) while carrying throws (its swing
        // was refused above).
        let interact = !input.is_open()
            && !list_was_open
            && !sign_was_open
            && !session.life.is_dead()
            && seihai::throwing::interact_pressed(
                session.carried.is_some(),
                keymap.pressed(&window, Action::Act),
                false,
                keymap.pressed(&window, Action::Attack),
            );
        // D while carrying hides under the object (or comes out); A comes out
        // too. Others see STATUS bit 0x02 and the hidden sprites.
        let hide_key = !input.is_open() && !list_was_open && !sign_was_open && keymap.pressed(&window, Action::Item);
        let unhide_key = session.hidden && keymap.pressed(&window, Action::Act) && !keymap.pressed(&window, Action::Attack);
        let interact = interact && !unhide_key;
        let hide_changed = if unhide_key {
            session.leave_hiding();
            true
        } else {
            hide_key && session.toggle_hide()
        };
        dirty |= hide_changed;
        // The hidden bit goes out whenever it changes (D, A, or a throw
        // from hiding, which is seen one frame later).
        if session.hide_status() != last_hide_status {
            last_hide_status = session.hide_status();
            if let Some(c) = conn.as_mut() {
                let sent = c
                    .send_messages(&[seihai::life::status_message(status_base | last_hide_status)])
                    .and_then(|()| c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)));
                if let Err(e) = sent {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
        }
        if interact {
            sign_page = 0;
            match session.interact() {
                seihai::offline::Interaction::Chest { x, y, item, got } => {
                    eprintln!("chest {x},{y}: {item}");
                    net_state.push_chat(ChatKind::Admin, "Chest".into(), format!("You got {got}."));
                    if let Some(c) = conn.as_mut() {
                        let _ = c.send_open_chest(x, y);
                    }
                }
                seihai::offline::Interaction::Grabbed => {
                    eprintln!("grabbed the wall in front (held while A is down)");
                }
                seihai::offline::Interaction::Lifted { kind } => {
                    eprintln!("lifted object {kind}");
                    // Others see the object (CARRYSPRITE) and the carry
                    // pose (the sprite value).
                    if let Some(c) = conn.as_mut() {
                        let sent = c
                            .send_carry_sprite(seihai::throwing::carry_sprite(kind))
                            .and_then(|()| c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)));
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
                                    .and_then(|()| c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)));
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
                seihai::offline::Interaction::Nothing => {}
            }
            dirty = true;
        }
        // Letting go of A (or Space) ends a grab; letting go of an arrow
        // ends a push or a pull.
        if session.pushpull.grab.is_some() && !window.is_key_down(minifb::Key::A) && !window.is_key_down(minifb::Key::Space) && session.release_grab() {
            dirty = true;
        }
        if [minifb::Key::Left, minifb::Key::Right, minifb::Key::Up, minifb::Key::Down].into_iter().any(|k| window.is_key_released(k)) && session.arrow_released() {
            dirty = true;
        }
        // No arrow key held: the player stands (idle animation).
        if !keymap.any_direction_down(&window) && session.stop_walking()
        {
            dirty = true;
        }
        if !input.is_open() && !item_list.open && !list_was_open && !session.life.is_dead() && session.reading.is_none() && !sign_was_open && session.warp_pending.is_none() && last_step.elapsed() >= session.step_interval() {
            let (mut dx, mut dy) = (0.0, 0.0);
            if keymap.down(&window, Action::Left) {
                dx -= 0.5;
            }
            if keymap.down(&window, Action::Right) {
                dx += 0.5;
            }
            if keymap.down(&window, Action::Up) {
                dy -= 0.5;
            }
            if keymap.down(&window, Action::Down) {
                dy += 0.5;
            }
            if dx != 0.0 || dy != 0.0 {
                let before = (session.player_x, session.player_y);
                session.try_move(dx, dy);
                // Walking or pushing up into a sign opens it on its first page.
                if session.reading.is_some() {
                    sign_page = 0;
                }
                moved = true;
                // The first steps of a connection: the files the real client
                // asks for then, if Seihai has no copy of them.
                if conn.is_some() && (session.player_x, session.player_y) != before {
                    send_file_msgs(conn.as_mut(), files.on_step());
                }
                last_step = seihai::offline::next_step_clock(last_step, Instant::now(), session.step_interval());
            }
        }
        // The `pushed` / `pulled` tries of this frame's steps and pulls, and
        // the NPCs they moved. Online the trigger goes out before the board
        // modify or NPC props of the move, as the 2.05 client sends them.
        for t in session.take_triggers() {
            if !no_scripts && scripts.trigger(&mut session, &t) {
                eprintln!("{} at {:.1},{:.1} (dir {})", t.action, t.x, t.y, t.dir);
            }
            // Online the 2.05 client sends every push and pull try to
            // the server too.
            if let Some(c) = conn.as_mut() {
                if let Err(e) = c.send_trigger_action(t.x, t.y, t.action, &t.dir.to_string()) {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
        }
        for m in session.take_npc_moves() {
            eprintln!("npc {} moved to {:.1},{:.1} ({})", m.index, m.x, m.y, m.event);
            if !no_scripts {
                scripts.npc_moved(&mut session, &m);
            }
            dirty = true;
        }
        // Sound: GANI cues from the steps just taken and the scripts'
        // `play` / `stopsound`. Leaving a level stops looped effects. Online,
        // a sound that is not on disk is asked for (WANTFILE) and plays on a
        // later trigger once it has arrived, as in 2.05.
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
            for name in session.take_sound_stops() {
                sound.stop(&name);
            }
            for ev in session.take_sound_events() {
                if sound.play(&ev, listener, now) == seihai::sound::Outcome::Missing && conn.is_some() {
                    let name = seihai::sound::wanted_name(&ev.file);
                    let msgs = files.need(&name);
                    if !msgs.is_empty() {
                        eprintln!("requesting {name}");
                    }
                    send_file_msgs(conn.as_mut(), msgs);
                }
            }
        } else {
            session.sound_events.clear();
            session.sound_stops.clear();
        }
        // The debug log names each level the player enters, online (the
        // server's name for it) or offline (the file or gmap cell).
        if let Some(f) = debug_file.as_mut() {
            let now = if conn.is_some() { net_state.current_level_name() } else { None }.or_else(|| session.level_name());
            if let Some(line) = seihai::offline::level_change_line(&mut debug_level, now.as_deref()) {
                use std::io::Write;
                let _ = writeln!(f, "{line}");
                let _ = f.flush();
            }
        }
        // Server NPCs (`PLO_NPCPROPS`): on a new level, every record the
        // server sent for it; after that, each update as it comes. An NPC
        // the level file has takes the server's id, any other is added.
        {
            let changes = std::mem::take(&mut net_state.npc_changes);
            if conn.is_some() && session.current_level.is_some() {
                let mut todo: Vec<(u32, seihai::protocol::npcprops::NpcProps)> = Vec::new();
                if session.level_serial != npc_link_serial {
                    npc_link_serial = session.level_serial;
                    npc_sync.clear();
                    let here = net_state.current_level_name();
                    todo.extend(
                        net_state
                            .server_npcs
                            .iter()
                            .filter(|(_, (l, _))| l.as_deref().zip(here.as_deref()).is_some_and(|(a, b)| a.eq_ignore_ascii_case(b)))
                            .map(|(&id, (_, p))| (id, p.clone())),
                    );
                } else {
                    todo = changes;
                }
                for (id, p) in todo {
                    let linked = session.take_server_npc(id, &p);
                    let index = match linked {
                        seihai::offline::Linked::Known(i) => i,
                        seihai::offline::Linked::Added(i) => {
                            // Scripts already running on this level start it now;
                            // otherwise it starts with the level's below.
                            if !no_scripts && scripts_serial == session.level_serial {
                                scripts.add_npc(&mut session, i);
                            }
                            npc_image_pass(&session.current_level.clone().unwrap(), &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
                            i
                        }
                        seihai::offline::Linked::None => continue,
                    };
                    if let Some(n) = session.current_level.as_ref().and_then(|l| l.npcs.get(index)) {
                        npc_sync.heard(id, &p, n);
                        if p.x.is_some() || p.y.is_some() {
                            scripts.set_npc_pos(index, n.x, n.y);
                        }
                    }
                    dirty = true;
                }
            }
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
                    npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
                }
                dirty = true;
            }
            if moved {
                scripts.touch(&mut session);
            }
            ft.mark("network and game");
            let dt = scripts_clock.elapsed().as_millis() as u64;
            scripts_clock = Instant::now();
            scripts.tick(&mut session, dt);
            ft.mark("scripts");
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
                            link_arrival = started.arrival.map(|a| seihai::net::LinkArrival::new(&dest, a));
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
            // `sendrpgmessage`: the player's message window is the chat log here.
            for text in std::mem::take(&mut scripts.rpg_messages) {
                net_state.push_chat(ChatKind::Admin, String::new(), text);
                dirty = true;
            }
            if let Some(f) = debug_file.as_mut() {
                use std::io::Write;
                // Script lines are kept once per level, so a message seen
                // on one level shows again on the next.
                let level = debug_level.clone().unwrap_or_default();
                for e in scripts.unhandled.drain(..) {
                    let line = format!("[{level}] effect not applied: {e:?}");
                    if seen_debug.insert(line.clone()) {
                        let _ = writeln!(f, "{line}");
                    }
                }
                for l in scripts.world.take_log() {
                    let line = format!("[{level}] script: {l}");
                    if seen_debug.insert(line.clone()) {
                        let _ = writeln!(f, "{line}");
                    }
                }
                let _ = f.flush();
            } else {
                // Drain so the buffers do not grow when debug is off.
                scripts.unhandled.clear();
                let _ = scripts.world.take_log();
            }
            // A script changed an NPC picture (setimg, showimg): load it.
            if std::mem::take(&mut scripts.images_changed) {
                if let Some(level) = session.current_level.clone() {
                    npc_image_pass(&level, &mut files, conn.as_mut(), &mut npc_images, &mut look_not_received);
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
            } else if let (Some(c), false, None) = (conn.as_mut(), script_warp_sent, &walk_cadence) {
                // Online: send the new position as PLI_PLAYERPROPS.
                if let Err(e) = c.send_walk_sprite(&session.player_update(), session.sprite_value_for(profile.ui.ganis)) {
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
                                link_arrival = started.arrival.map(|a| seihai::net::LinkArrival::new(&dest, a));
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

        // Player flags scripts changed go to the server (PLI 18 / 19). They
        // ride ahead of the next walk update; ones no walk took within a
        // tick go out on their own, but not before the post-login frame,
        // which carries the first ones.
        for (name, value) in std::mem::take(&mut session.flags_out) {
            if let Some(c) = conn.as_mut() {
                c.queue_flag(&name, value.as_deref());
            }
        }
        if !login_burst.as_ref().is_some_and(|lb| lb.pending()) {
            if let Some(c) = conn.as_mut() {
                if let Err(e) = c.send_stale_flags(Duration::from_millis(60)) {
                    eprintln!("send failed: {e}");
                }
            }
        }

        // Big map: ask for the level across a near edge before the player
        // walks the link, so it is here when they cross. Once per name per
        // connection.
        if let (Some(map), Some(here), true, true) =
            (session.big_map.as_ref(), net_state.current_level_name(), conn.is_some(), session.warp_pending.is_none())
        {
            let ahead: Vec<String> = map
                .neighbours_near_edge(&here, session.player_x, session.player_y)
                .into_iter()
                .filter(|n| gmap_asked.insert(format!("near:{}", n.to_ascii_lowercase())))
                .map(str::to_string)
                .collect();
            for n in ahead {
                let file = if n.contains('.') { n } else { format!("{n}.nw") };
                // Logged even when the request waits for a free slot.
                eprintln!("asking for {file} ahead (big map neighbour)");
                send_file_msgs(conn.as_mut(), files.need(&file));
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
        // A warp the server never answers is given up after a few seconds:
        // the player may walk again. The link's arrival stays held, so a
        // level that still comes lands the player where the link said.
        if warp_since.is_some_and(|t| t.elapsed() > Duration::from_secs(4)) {
            warp_since = None;
            session.cancel_warp();
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
                link_arrival = None;
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
                        looks_sent = None;
                        // The new server has its own cache folder, searched first.
                        let warp_cache = cache_for(&a, Some(&w.display_name()));
                        let warp_game_cache = game_cache_for(&a);
                        resolver.set_game_cache(warp_game_cache.clone());
                        resolver.set_cache(warp_cache.clone());
                        files.store.set_caches(warp_game_cache, warp_cache, Some(seihai::servercache::meta_text(&a, Some(&w.display_name()))));
                        sound.set_dirs(resolver.sound_dirs());
                        login_burst = LoginBurst::for_version(version);
                        if login_burst.as_ref().is_some_and(|lb| lb.info().idle_gani) {
                            send_file_msgs(Some(&mut sess.conn), files.need("idle.gani"));
                        }
                        classic_props_owed = seihai::protocol::login::sends_classic_login_props(version);
                        walk_cadence = seihai::protocol::walkcadence::WalkCadence::for_version(version);
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

        // Enter, T or the switch-to-chat key (Tab) opens the chat bar, and
        // Enter sends it when it is open. F2 opens a PM to the nearest player.
        let enter = window.is_key_pressed(minifb::Key::Enter, minifb::KeyRepeat::No)
            || window.is_key_pressed(minifb::Key::NumPadEnter, minifb::KeyRepeat::No);
        let talk = !confirm_quit
            && (window.is_key_pressed(minifb::Key::T, minifb::KeyRepeat::No) || keymap.pressed(&window, Action::SwitchToChat));
        if !confirm_quit && !list_was_open && ui::chat_key_action(input.is_open(), enter, talk) != ui::ChatKeyAction::None {
            if !input.is_open() {
                eprintln!("chat: line opened ({})", if enter { "Enter" } else { "T or switch-to-chat" });
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
                            // Looks commands (`sethead head927`, `set body body13`,
                            // `setcoat red`). A server runs them itself and sends
                            // the props back, so online the line goes out in the
                            // server's form; offline Seihai applies it.
                            let t = match seihai::looks::parse_chat_look(&t) {
                                Some(cmd) if conn.is_some() => cmd.server_line(),
                                Some(cmd) => {
                                    apply_chat_look(&mut session, &cmd);
                                    t
                                }
                                None => t,
                            };
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
                        ui::ChatSend::ShowName => {
                            my_name_timer = seihai::chatlists::BubbleTimer::start();
                            dirty = true;
                            None
                        }
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
            if window.is_key_pressed(minifb::Key::F7, minifb::KeyRepeat::No) {
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
        if !input.is_open() && keymap.any_direction_down(&window) {
            my_chat_timer.release();
        }
        // Chat bubbles go down after a few seconds.
        {
            let dt = bubble_clock.elapsed().as_millis().min(u32::MAX as u128) as u32;
            bubble_clock = Instant::now();
            if net_state.tick_bubbles(dt) {
                dirty = true;
            }
            if session.alignment != my_name_ap {
                my_name_ap = session.alignment;
                my_name_timer = seihai::chatlists::BubbleTimer::start();
                dirty = true;
            }
            if my_name_timer.tick(dt) {
                dirty = true;
            }
            if my_chat_timer.tick(dt) {
                // The 2.05 client sends an empty CURCHAT when its own bubble
                // ends, which is what takes the line down on other screens.
                if !my_chat.is_empty() {
                    if let Some(Err(e)) = conn.as_mut().map(|c| c.send_chat("")) {
                        eprintln!("chat clear send failed: {e}");
                    }
                }
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
            // `#w` in scripts: the selected weapon's name.
            scripts.world.core.player.weapon = weapon_bar.current().map(|e| e.name.clone()).unwrap_or_default();
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
        // A script changed our looks (`sethead`, `setcoatcolor` …): the
        // server and the other players see it from our props.
        // Only a real change is sent: a script that sets the same head every
        // frame must not send PLAYERPROPS every frame.
        if std::mem::take(&mut session.looks_changed) && looks_sent.as_ref() != Some(&session.looks) {
            if let Some(Err(e)) = conn.as_mut().map(|c| c.send_looks(&session.looks, &profile.props.own)) {
                eprintln!("looks send failed: {e}");
            }
            looks_sent = Some(session.looks.clone());
            dirty = true;
        }
        // Server NPCs a push, a pull or a script changed: the 2.05 client
        // sends the new X, Y and BLOCKFLAGS, at most every 100 ms, and the
        // server passes them on.
        let npc_dt = npc_sync_clock.elapsed();
        npc_sync_clock += npc_dt;
        if let (Some(c), Some(level)) = (conn.as_mut(), session.current_level.as_ref()) {
            let msgs = npc_sync.tick(level, npc_dt);
            if !msgs.is_empty() {
                if let Err(e) = c.send_messages(&msgs) {
                    eprintln!("send failed: {e}");
                    conn = None;
                }
            }
        }
        // Tiles the player changed this frame (a lift, a cut, a moved stone
        // block): the server writes them into the level for the others.
        let mut boards = Vec::new();
        for b in session.take_board_changes() {
            dirty = true;
            boards.push(seihai::net::board_modify_message(b.x as u8, b.y as u8, b.width as u8, b.height as u8, &b.tiles));
        }
        // What the floor items tell the server, by the build: a 1.38 to 1.41
        // roll (PLI_ITEMADD), a pickup (PLI_ITEMTAKE, or ITEMDEL before
        // 1.38) and a timeout (ITEMDEL from 1.38). After the cut's
        // BOARDMODIFY, as 1.39 sends them.
        boards.extend(session.take_item_messages(conn.is_some()));
        // A build with a walk cadence sends on its 50 ms tick only: the tile
        // changes, the flags and that tick's walk props in one frame, as the
        // real 2.22 sends a bush cut with its sword GANI.
        let cadence_tick = walk_cadence.is_some() && cadence_clock.elapsed() >= Duration::from_millis(50);
        if cadence_tick {
            cadence_clock += Duration::from_millis(50);
            if cadence_clock.elapsed() >= Duration::from_millis(50) {
                cadence_clock = Instant::now();
            }
        }
        if walk_cadence.is_some() && !cadence_tick {
            held_boards.extend(boards);
        } else if let (Some(c), Some(wc)) = (conn.as_mut(), walk_cadence.as_mut()) {
            use seihai::pose::PoseKind;
            let gani = session.current_pose().and_then(|p| match p.kind {
                PoseKind::Walk => Some("walk"),
                PoseKind::Idle => Some("idle"),
                PoseKind::Sword => Some("sword"),
                _ => None,
            });
            let sprite = if gani == Some("sword") { session.player_dir % 4 } else { session.sprite_value_for(true) };
            let upd = session.player_update();
            let props = if script_warp_sent { None } else { wc.tick(seihai::protocol::walkcadence::WalkState::new(upd.x, upd.y, sprite, gani)) };
            let mut all = std::mem::take(&mut held_boards);
            all.extend(boards);
            if let Err(e) = c.send_tick_frame(all, props) {
                eprintln!("send failed: {e}");
                conn = None;
            }
        } else if let Some(c) = conn.as_mut() {
            for m in boards {
                if let Err(e) = c.send_messages(&[m]) {
                    eprintln!("send failed: {e}");
                    conn = None;
                    break;
                }
            }
        }
        ft.mark("after scripts");
        // The pictures other players and our own shield are drawn with.
        {
            let mut wanted: Vec<(&str, String)> =
                net_state.others.iter().flat_map(|o| seihai::render::look_images(o).map(|(d, n)| (d, n.to_string())).collect::<Vec<_>>()).collect();
            let own_shield = local_shield_image(session.shield_power);
            if !own_shield.is_empty() {
                wanted.push(("shields", own_shield));
            }
            wanted.push(("heads", session.looks.head_name().to_string()));
            // The character NPCs' own pictures (`setcharprop`).
            let npc_looks: Vec<(seihai::looks::Looks, Option<String>, Option<String>)> = session
                .current_level
                .iter()
                .flat_map(|l| &l.npcs)
                .filter(|n| n.ani.is_some())
                .map(|n| (seihai::looks::npc_looks(n), n.shield.clone(), n.props.get("#1").cloned()))
                .collect();
            for (l, shield, sword) in &npc_looks {
                for (dir, n) in [("heads", &l.head), ("bodies", &l.body), ("shields", shield), ("swords", sword)] {
                    if let Some(n) = n.as_deref().filter(|n| !n.is_empty()) {
                        wanted.push((dir, n.to_string()));
                    }
                }
            }
            // GANI attribute pictures (the hat): everyone's own, and the
            // defaults of the player GANIs, for a build that has them.
            if gani_attrs {
                for attrs in net_state.others.iter().map(|o| &o.looks.attrs).chain([&session.looks.attrs]).chain(npc_looks.iter().map(|l| &l.0.attrs)) {
                    for (i, a) in attrs.iter().enumerate() {
                        if let Some(a) = a.as_deref().filter(|a| !a.is_empty()) {
                            wanted.push((seihai::looks::attr_dir(i), a.to_string()));
                        }
                    }
                }
                for g in session.poses.all().chain(session.walk_gani.as_ref().map(|w| &w.gani)).chain(session.npc_ganis.values()) {
                    for (i, a) in g.default_attrs.iter().enumerate() {
                        if let Some(a) = a.as_deref().filter(|a| !a.is_empty()) {
                            wanted.push((seihai::looks::attr_dir(i), a.to_string()));
                        }
                    }
                }
            }
            wanted.push(("bodies", session.looks.body_name().to_string()));
            if look_image_pass(&wanted, &resolver, &mut files, conn.as_mut(), &mut npc_images, &mut look_tried, &mut look_not_received) {
                dirty = true;
            }
            let mut bodies: Vec<(String, [u8; 5])> = net_state
                .others
                .iter()
                .map(|o| (o.looks.body.clone().unwrap_or_else(|| seihai::looks::DEFAULT_BODY.into()), o.looks.colors.unwrap_or(seihai::looks::DEFAULT_COLORS)))
                .collect();
            bodies.push((session.looks.body_name().to_string(), session.looks.colors));
            bodies.extend(npc_looks.iter().map(|(l, _, _)| (l.body_name().to_string(), l.colors)));
            if colored_body_pass(&bodies, &resolver, &mut files, &mut body_raw, &mut look_tried, &mut npc_images, &mut look_not_received) {
                dirty = true;
            }
        }
        ft.mark("look images");
        if dirty {
            let my_nick = my_name_timer.active().then(|| me(&net_state, &account));
            let own_failed = (files.requests.has_failed(session.looks.body_name()), files.requests.has_failed(session.looks.head_name()));
            scene = draw(&session, &net_state.others, &my_chat, my_nick.as_deref(), &npc_images, weapon_bar.current_image(), own_failed);
            dirty = false;
        }
        ft.mark("draw");
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
        ft.mark("overlays");
        let _ = window.update_with_buffer(&buffer, fb.width as usize, fb.height as usize);
        ft.mark("present");
        slow_log.report(&ft);

        fps_count += 1;
        if fps_since.elapsed() >= Duration::from_secs(1) {
            fps = fps_count as f32 / fps_since.elapsed().as_secs_f32();
            fps_count = 0;
            fps_since = Instant::now();
        }
    }
}

/// How long each part of one pass of the main loop took.
struct FrameTimes {
    last: Instant,
    began: Instant,
    parts: Vec<(&'static str, Duration)>,
}

impl FrameTimes {
    fn start() -> Self {
        let now = Instant::now();
        Self { last: now, began: now, parts: Vec::new() }
    }

    /// The time since the previous mark goes to `part`.
    fn mark(&mut self, part: &'static str) {
        let now = Instant::now();
        self.parts.push((part, now - self.last));
        self.last = now;
    }
}

/// Logs a pass of the main loop that took over [`SlowFrameLog::SLOW`], with
/// the time of each part, at most once every two seconds: the way to find
/// what keeps the frame rate low on a given server.
#[derive(Default)]
struct SlowFrameLog {
    last: Option<Instant>,
    /// With `SEIHAI_FRAME_LOG` set, every pass's parts are summed and their
    /// averages logged every five seconds.
    sums: Vec<(&'static str, Duration)>,
    passes: u32,
    since: Option<Instant>,
}

impl SlowFrameLog {
    const SLOW: Duration = Duration::from_millis(100);

    fn report(&mut self, ft: &FrameTimes) {
        let total = ft.began.elapsed();
        if std::env::var_os("SEIHAI_FRAME_LOG").is_some() {
            self.passes += 1;
            for (i, (n, d)) in ft.parts.iter().enumerate() {
                match self.sums.get_mut(i) {
                    Some(s) => s.1 += *d,
                    None => self.sums.push((n, *d)),
                }
            }
            let since = *self.since.get_or_insert_with(Instant::now);
            if since.elapsed() >= Duration::from_secs(5) {
                let secs = since.elapsed().as_secs_f64();
                let parts: Vec<String> =
                    self.sums.iter().map(|(n, d)| format!("{n} {:.1}", d.as_secs_f64() * 1000.0 / self.passes as f64)).collect();
                eprintln!("frames: {:.1} fps, average ms ({})", self.passes as f64 / secs, parts.join(", "));
                self.sums.clear();
                self.passes = 0;
                self.since = Some(Instant::now());
            }
        }
        if total < Self::SLOW || self.last.is_some_and(|t| t.elapsed() < Duration::from_secs(2)) {
            return;
        }
        self.last = Some(Instant::now());
        let parts: Vec<String> = ft.parts.iter().map(|(n, d)| format!("{n} {}", d.as_millis())).collect();
        eprintln!("slow frame: {} ms ({})", total.as_millis(), parts.join(", "));
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

/// Shift, Control, Alt and the like: keys that type nothing themselves.
fn is_modifier(k: minifb::Key) -> bool {
    use minifb::Key as K;
    matches!(k, K::LeftShift | K::RightShift | K::LeftCtrl | K::RightCtrl | K::LeftAlt | K::RightAlt | K::LeftSuper | K::RightSuper | K::CapsLock | K::Menu)
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

/// The key another player's body is drawn under: their body (or the
/// default) in their colours (or the defaults).
fn other_body_key(o: &seihai::protocol::OtherPlayer) -> String {
    let body = o.looks.body.as_deref().filter(|b| !b.is_empty()).unwrap_or(seihai::looks::DEFAULT_BODY);
    seihai::looks::colored_body_key(body, &o.looks.colors.unwrap_or(seihai::looks::DEFAULT_COLORS))
}

/// Apply a looks command typed in chat, offline.
fn apply_chat_look(session: &mut OfflineSession, cmd: &seihai::looks::ChatLook) {
    use seihai::looks::{with_image_ext, ChatLook};
    match cmd {
        ChatLook::Head(h) => session.looks.head = Some(with_image_ext(h)),
        ChatLook::Body(b) => session.looks.body = Some(with_image_ext(b)),
        ChatLook::Color { slot, color } => session.looks.colors[*slot] = *color,
    }
}

/// The `ATTR1` … `ATTR5` sheets of `gani` for a player with `attrs`: their
/// pictures, else the GANI's defaults, when loaded.
fn attr_sheets<'a>(
    attrs: &[Option<String>; 5],
    gani: &Gani,
    images: &'a HashMap<String, (u32, Vec<u8>)>,
) -> [Option<seihai::render::Sheet<'a>>; 5] {
    [0, 1, 2, 3, 4].map(|i| seihai::looks::attr_image(attrs, gani, i).and_then(|n| images.get(n)).map(pose_sheet))
}
