//! Seihai G3D — a client for Graal3D (February 2000).
//!
//!   seihai-g3d --online host[:port] --account A --password-env VAR
//!              [--nick N] [--assets DIR] [--levels DIR] [--no-udp]
//!   seihai-g3d ... --no-window [--first-level] [--say TEXT] [--walk TICKS] [--seconds N]
//!   seihai-g3d ... --screenshot FILE.bmp [--walk TICKS] [--turn TICKS]
//!   seihai-g3d --level-info FILE.x
//!
//! With a window: numpad 8 / 2 (or Up / Down) walk forward and back,
//! numpad 4 / 6 (or Left / Right) step sideways, numpad 5 (or Space)
//! jumps, numpad 0 turns the follow camera off and on, Enter opens and
//! sends the chat line, Esc closes it or quits.
//!
//! `--assets` is the Graal3D folder (textures and player meshes; the
//! server sends neither). `--no-window` only logs. `--screenshot` renders
//! one frame after the level is in (and after `--walk` steps forward and
//! `--turn` steps right), writes it as a BMP and exits.

use seihai_g3d::config::Config;
use seihai_g3d::props::Player;
use seihai_g3d::render::Frame;
use seihai_g3d::session::{self, Event, Login, Session, TICK};
use seihai_g3d::view::{self, Assets, ChatLog, Scene, HEIGHT, WIDTH};
use seihai_g3d::world::{FollowCam, Ground, Input, Walker};
use seihai_g3d::xfile::XFile;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Default)]
struct Args {
    online: Option<String>,
    account: Option<String>,
    password_env: Option<String>,
    nick: Option<String>,
    no_udp: bool,
    levels: Option<String>,
    assets: Option<String>,
    no_window: bool,
    first_level: bool,
    say: Option<String>,
    walk: u32,
    turn: u32,
    seconds: Option<f64>,
    level_info: Option<String>,
    screenshot: Option<String>,
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
            "--assets" => a.assets = Some(val(&f)?),
            "--say" => a.say = Some(val(&f)?),
            "--walk" => a.walk = val(&f)?.parse().map_err(|_| "--walk needs a number")?,
            "--turn" => a.turn = val(&f)?.parse().map_err(|_| "--turn needs a number")?,
            "--seconds" => a.seconds = Some(val(&f)?.parse().map_err(|_| "--seconds needs a number")?),
            "--level-info" => a.level_info = Some(val(&f)?),
            "--screenshot" => a.screenshot = Some(val(&f)?),
            "--no-udp" => a.no_udp = true,
            "--no-window" => a.no_window = true,
            "--first-level" => a.first_level = true,
            // Passed through by the launcher.
            "--version" => {
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
        return match std::fs::read(f).map_err(|e| e.to_string()).and_then(|b| XFile::parse(&b).map_err(|e| e.to_string())) {
            Ok(x) => {
                println!("{f}: {} meshes, {} vertices, {} faces", x.meshes.len(), x.vertex_count(), x.face_count());
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
    let password = a.password_env.as_deref().and_then(|v| std::env::var(v).ok()).unwrap_or_default();
    let account = a.account.clone().unwrap_or_else(|| "guest".into());
    let nick = a.nick.clone().unwrap_or_else(|| account.clone());
    let mut me = Player { nick, ..Player::default() };
    let assets = a.assets.as_ref().map(PathBuf::from);
    // The look and UDP options from the install's graal3dconfig.txt. Its
    // server address is never used: the server is always --online.
    // TODO(G3D-4): fixedport / fixedudpport (bind UDP to a set port).
    let cfg = assets.as_ref().and_then(|d| std::fs::read_to_string(d.join("graal3dconfig.txt")).ok()).map(|t| Config::parse(&t));
    if let Some(c) = &cfg {
        c.apply_look(&mut me);
    }
    let no_udp = a.no_udp || cfg.as_ref().is_some_and(|c| c.no_udp);
    let dir = a.levels.clone().map(Into::into).unwrap_or_else(|| session::default_level_dir(assets.as_deref()));
    let s = match Session::connect(&addr, &Login { account, password }, me, !no_udp, Some(dir.clone())) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seihai-g3d: {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("connected to {addr}, udp port {}", s.me.udp_port);
    let mut dirs = vec![dir];
    dirs.extend(assets);
    let mut g = Game::new(s, Assets::new(dirs));
    if a.no_window {
        headless(&a, &mut g)
    } else if let Some(out) = &a.screenshot {
        screenshot(&a, &mut g, out)
    } else {
        windowed(&mut g)
    }
}

/// Everything one tick needs.
struct Game {
    s: Session,
    assets: Assets,
    level: Option<XFile>,
    ground: Ground,
    walker: Walker,
    cam: Option<FollowCam>,
    follow: bool,
    chat: ChatLog,
    walking: bool,
    failed: bool,
}

impl Game {
    fn new(s: Session, assets: Assets) -> Game {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.subsec_nanos());
        Game {
            s,
            assets,
            level: None,
            ground: Ground::default(),
            walker: Walker::new(seed),
            cam: None,
            follow: true,
            chat: ChatLog::default(),
            walking: false,
            failed: false,
        }
    }

    /// One tick: network, then movement. Returns the events for logging.
    fn tick(&mut self, inp: Input) -> Result<Vec<Event>, String> {
        let ev = self.s.tick().map_err(|e| e.to_string())?;
        for e in &ev {
            match e {
                Event::Level { file, .. } => {
                    if file.is_some() || self.level.is_none() {
                        let text = match &self.s.level_text {
                            Some(t) if file.is_some() => Some(t.clone()),
                            _ => self.s.level_path().and_then(|p| std::fs::read(p).ok()),
                        };
                        self.level = text.and_then(|t| XFile::parse(&t).ok());
                        if let Some(l) = &self.level {
                            self.ground = Ground::of(l);
                            self.assets.level_textures(l);
                        }
                    }
                }
                Event::OwnProps { .. } if self.cam.is_none() => self.cam = Some(FollowCam::behind(&self.s.me)),
                Event::Chat { nick, text, .. } => self.chat.push(nick, text),
                Event::DiscMessage(_) => self.failed = true,
                _ => {}
            }
        }
        if self.level.is_some() {
            if let Some(cam) = self.cam {
                self.walking = inp.forward || inp.back || inp.left || inp.right;
                self.walker.tick(&mut self.s.me, &cam, inp, &self.ground);
            }
        }
        if let (Some(cam), true) = (self.cam.as_mut(), self.follow) {
            cam.tick(&self.s.me);
        }
        self.chat.tick();
        Ok(ev)
    }

    fn say(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.s.say(text).ok();
        let nick = self.s.me.nick.clone();
        self.chat.push(&nick, text);
    }

    fn draw(&mut self, f: &mut Frame, typing: Option<&str>) {
        let cam = self.cam.unwrap_or_else(|| FollowCam::behind(&self.s.me));
        let mut others = self.s.others.values();
        let scene = Scene {
            level: self.level.as_ref(),
            me: &self.s.me,
            others: &mut others,
            cam: &cam,
            walking: self.walking,
            chat: &self.chat,
            typing,
        };
        view::draw(f, &mut self.assets, scene);
    }
}

fn headless(a: &Args, g: &mut Game) -> ExitCode {
    let start = Instant::now();
    let (mut said, mut walked) = (false, 0);
    loop {
        let fwd = g.level.is_some() && walked < a.walk;
        let ev = match g.tick(Input { forward: fwd, ..Input::default() }) {
            Ok(ev) => ev,
            Err(e) => {
                eprintln!("seihai-g3d: {e}");
                return ExitCode::FAILURE;
            }
        };
        if fwd {
            walked += 1;
        }
        ev.iter().for_each(|e| log(&g.s, e));
        if g.s.is_closed() {
            break;
        }
        if g.level.is_some() {
            if let (Some(t), false) = (&a.say, said) {
                g.say(t);
                said = true;
            }
            if a.first_level && a.seconds.is_none() && walked >= a.walk {
                g.s.tick().ok();
                break;
            }
        }
        if a.seconds.is_some_and(|n| start.elapsed().as_secs_f64() >= n) {
            break;
        }
        std::thread::sleep(TICK);
    }
    if g.failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn screenshot(a: &Args, g: &mut Game, out: &str) -> ExitCode {
    let start = Instant::now();
    let mut after = 0u32;
    while start.elapsed().as_secs() < 20 {
        let ready = g.level.is_some() && g.cam.is_some();
        let inp = Input { forward: ready && after < a.walk, right: ready && after >= a.walk && after < a.walk + a.turn, ..Input::default() };
        match g.tick(inp) {
            Ok(ev) => ev.iter().for_each(|e| log(&g.s, e)),
            Err(e) => {
                eprintln!("seihai-g3d: {e}");
                return ExitCode::FAILURE;
            }
        }
        if ready {
            if after == 0 {
                if let Some(t) = &a.say {
                    g.say(t);
                }
            }
            after += 1;
            // Let the camera settle for a second after the moves.
            if after >= a.walk + a.turn + 20 {
                let mut f = Frame::new(WIDTH, HEIGHT);
                g.draw(&mut f, None);
                return match std::fs::write(out, seihai_g3d::bmp::encode(f.w, f.h, &f.px)) {
                    Ok(()) => {
                        let m = &g.s.me;
                        println!("screenshot {out}: player at ({}, {}, {}) facing {}", m.x, m.y, m.z, m.angle);
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("seihai-g3d: {out}: {e}");
                        ExitCode::FAILURE
                    }
                };
            }
        }
        if g.s.is_closed() {
            break;
        }
        std::thread::sleep(TICK);
    }
    eprintln!("seihai-g3d: no level arrived for the screenshot");
    ExitCode::FAILURE
}

struct TypedChars(Arc<Mutex<Vec<char>>>);

impl minifb::InputCallback for TypedChars {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            self.0.lock().unwrap().push(c);
        }
    }
}

fn windowed(g: &mut Game) -> ExitCode {
    use minifb::{Key, KeyRepeat, Window, WindowOptions};
    let mut w = match Window::new("Seihai G3D", WIDTH, HEIGHT, WindowOptions { resize: true, ..WindowOptions::default() }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("seihai-g3d: could not open the window: {e} (use --no-window to only log)");
            return ExitCode::FAILURE;
        }
    };
    let typed: Arc<Mutex<Vec<char>>> = Arc::default();
    w.set_input_callback(Box::new(TypedChars(typed.clone())));
    let mut f = Frame::new(WIDTH, HEIGHT);
    let mut typing: Option<String> = None;
    let mut next = Instant::now();
    while w.is_open() {
        let chars: Vec<char> = std::mem::take(&mut *typed.lock().unwrap());
        if w.is_key_pressed(Key::Enter, KeyRepeat::No) || w.is_key_pressed(Key::NumPadEnter, KeyRepeat::No) {
            match typing.take() {
                Some(t) => g.say(&t),
                None => typing = Some(String::new()),
            }
        } else if w.is_key_pressed(Key::Escape, KeyRepeat::No) {
            if typing.take().is_none() {
                break;
            }
        } else if let Some(t) = typing.as_mut() {
            if w.is_key_pressed(Key::Backspace, KeyRepeat::Yes) {
                t.pop();
            }
            for c in chars {
                if !c.is_control() && t.len() < 200 {
                    t.push(c);
                }
            }
        }
        if typing.is_none() && w.is_key_pressed(Key::NumPad0, KeyRepeat::No) {
            g.follow = !g.follow;
        }
        let held = |k: &[Key]| typing.is_none() && k.iter().any(|&k| w.is_key_down(k));
        let inp = Input {
            forward: held(&[Key::NumPad8, Key::Up]),
            back: held(&[Key::NumPad2, Key::Down]),
            left: held(&[Key::NumPad4, Key::Left]),
            right: held(&[Key::NumPad6, Key::Right]),
            jump: held(&[Key::NumPad5, Key::Space]),
        };
        if Instant::now() >= next {
            next += TICK;
            match g.tick(inp) {
                Ok(ev) => ev.iter().for_each(|e| log(&g.s, e)),
                Err(e) => {
                    eprintln!("seihai-g3d: {e}");
                    return ExitCode::FAILURE;
                }
            }
            if g.s.is_closed() {
                break;
            }
            g.draw(&mut f, typing.as_deref());
        }
        if w.update_with_buffer(&f.px, f.w, f.h).is_err() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    if g.failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn log(s: &Session, e: &Event) {
    match e {
        Event::Accepted => println!("accepted"),
        Event::OwnProps { id, props } => println!("own props: player {id}, {} props", props.len()),
        Event::Level { name, file: None } => println!("level {name}: already have it"),
        Event::Level { name, file: Some(f) } => {
            let saved = f.path.as_ref().map(|p| format!(", saved to {}", p.display())).unwrap_or_default();
            println!("level {name}: {} bytes, {} vertices, {} faces{saved}", f.bytes, f.vertices, f.faces);
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
            println!("player {id} ({}): props {}", p.map(|p| p.nick.as_str()).unwrap_or(""), ids.join(","));
        }
        Event::PlayerLeft { id } => println!("player {id} left"),
        Event::Chat { id, nick, text, udp } => {
            println!("chat {nick}: {text} (player {id}, {})", if *udp { "udp" } else { "tcp" })
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
