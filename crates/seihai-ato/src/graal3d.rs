//! The Graal3D side of an online session: the Torque UDP link started
//! beside the TCP login (`--graal3d`).
//!
//! Both start at the same moment, to the same host: TCP on the listed
//! port, UDP on the listed port + 1. The UDP link stops when the TCP side
//! ends. Only 4.20r6 and 5.x (protocols 21 and 24) run here; 3.153 and
//! 4.04r2 use the Torque game layer, which Ato does not run yet.
//!
//! [`Side`] follows the TCP session through core's session hooks: the
//! first server opens the link when it was picked as a `3 ` server
//! (`--graal3d`, which the single launcher passes), and each server warp
//! opens or closes it by the warp's name. While the link is up the flat
//! view leaves out the TCP level boards, as the real clients ignore them.
//! The shapes and Sky textures the ghosts name are fetched over TCP
//! through the same session (login step 6).

use crate::online::{FileAsks, Hooks, Login, ServerJoin};
#[cfg(not(feature = "era6"))]
use crate::torque::net::session::{self, Identity};
use crate::torque::{self, TorqueProfile};
use crate::view3d::Shared3d;
use seihai_core::protocol::ClientVersion;
#[cfg(not(feature = "era6"))]
use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// A running UDP link.
pub struct Udp3d {
    stop: Arc<AtomicBool>,
    /// How the link ended, for the log.
    thread: JoinHandle<Result<String, String>>,
}

impl Udp3d {
    /// Stops the link (sending a disconnect) and waits for it.
    pub fn stop(self) -> Result<String, String> {
        self.stop.store(true, Ordering::Relaxed);
        self.thread
            .join()
            .unwrap_or_else(|_| Err("the 3D link thread panicked".into()))
    }
}

/// The Torque profile of a build whose 3D side Ato runs.
pub fn profile(version: ClientVersion) -> Result<TorqueProfile, String> {
    match torque::of(version) {
        Some(p) if !p.tge_game_layer => Ok(p),
        Some(p) => Err(format!(
            "--graal3d: the 3D side of {} (protocol {}) is not run by Ato yet; use 4.20r6 or 5.x",
            p.builds, p.protocol
        )),
        None => Err(format!(
            "--graal3d: {} has no Graal3D side",
            version.label()
        )),
    }
}

/// The UDP address beside the TCP address `addr` (`host:port`).
pub fn udp_addr(addr: &str) -> Result<SocketAddr, String> {
    let tcp = addr
        .to_socket_addrs()
        .map_err(|e| format!("{addr}: {e}"))?
        .next()
        .ok_or_else(|| format!("{addr}: no address"))?;
    Ok(SocketAddr::new(tcp.ip(), torque::udp_port(tcp.port())))
}

/// The `era6` build leaves the Torque reimplementation out.
#[cfg(feature = "era6")]
pub fn start(
    _addr: &str,
    _version: ClientVersion,
    _login: &Login,
    _files: Option<FileAsks>,
    _world: Option<Shared3d>,
) -> Result<Udp3d, String> {
    Err("--graal3d: this build leaves out the Torque engine (era6)".into())
}

/// Starts the UDP link for `login` on its own thread; progress goes to
/// stdout. A 5.x guest's account on the link is `guest`. The files its
/// ghosts name are asked for through `files` (core's TCP session).
#[cfg(not(feature = "era6"))]
pub fn start(
    addr: &str,
    version: ClientVersion,
    login: &Login,
    files: Option<FileAsks>,
    world: Option<Shared3d>,
) -> Result<Udp3d, String> {
    let profile = profile(version)?;
    let server = udp_addr(addr)?;
    let bind = if server.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind).map_err(|e| format!("UDP: {e}"))?;
    let who = Identity {
        account: if login.account.is_empty() {
            "guest".into()
        } else {
            login.account.clone()
        },
        password: login.password.clone(),
        nickname: login.nick.clone(),
    };
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let thread = std::thread::spawn(move || {
        let ended = session::run(
            &socket,
            server,
            profile,
            &who,
            &flag,
            files.as_ref(),
            world.as_ref(),
            &mut std::io::stdout(),
        );
        match &ended {
            Ok(e) => println!("3d: link ended: {e:?}"),
            Err(e) => println!("3d: link failed: {e}"),
        }
        ended.map(|e| format!("{e:?}"))
    });
    Ok(Udp3d { stop, thread })
}

/// The Graal3D side of one TCP session: opens and closes the UDP link as
/// the session moves between servers.
pub struct Side {
    version: ClientVersion,
    login: Login,
    /// The first server was picked as a `3 ` server (`--graal3d`).
    first_is_3d: bool,
    link: Mutex<Option<Udp3d>>,
    /// Set while the link is up; the flat view leaves the boards out.
    pub boards_hidden: Arc<AtomicBool>,
    /// The files the 3D world asks the TCP session for.
    pub files: FileAsks,
    /// The 3D world, for the window's view.
    pub world: Shared3d,
}

impl Side {
    pub fn new(version: ClientVersion, login: Login, first_is_3d: bool) -> Arc<Side> {
        Arc::new(Side {
            version,
            login,
            first_is_3d,
            link: Mutex::new(None),
            boards_hidden: Arc::default(),
            files: FileAsks::default(),
            world: Shared3d::default(),
        })
    }

    /// The session hooks: [`Side::join`] before each login, and the file
    /// requests.
    pub fn hooks(self: &Arc<Self>) -> Hooks {
        let me = self.clone();
        Hooks {
            on_server: Some(Arc::new(move |j: &ServerJoin| me.join(j))),
            files: Some(self.files.clone()),
        }
    }

    /// Whether the link is opened for `join`: the first server by
    /// `--graal3d`, a warp target by its `3 ` name.
    pub fn is_3d(&self, join: &ServerJoin) -> bool {
        match join.name {
            None => self.first_is_3d,
            Some(_) => join.is_graal3d(),
        }
    }

    /// The TCP session is about to log in at `join`: close the link to the
    /// server it leaves, and open one to this server if it is a 3D one.
    pub fn join(&self, join: &ServerJoin) {
        self.stop();
        if !self.is_3d(join) {
            return;
        }
        *self.world.lock().unwrap_or_else(|e| e.into_inner()) = Default::default();
        match start(
            &join.addr,
            self.version,
            &self.login,
            Some(self.files.clone()),
            Some(self.world.clone()),
        ) {
            Ok(u) => {
                println!("3d: opening the Graal3D link beside {}", join.addr);
                *self.link.lock().unwrap_or_else(|e| e.into_inner()) = Some(u);
                self.boards_hidden.store(true, Ordering::Relaxed);
            }
            Err(e) => println!("3d: {e}"),
        }
    }

    /// The TCP side ended (or moves on): stop the link, if one is up.
    pub fn stop(&self) {
        let link = self.link.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(u) = link {
            let _ = u.stop();
        }
        self.boards_hidden.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_graal_object_model_builds_run() {
        assert_eq!(profile(ClientVersion::V512).unwrap().protocol, 24);
        assert_eq!(profile(ClientVersion::V420R6).unwrap().protocol, 21);
        assert!(profile(ClientVersion::V3153).is_err());
        assert!(profile(ClientVersion::V404R2).is_err());
    }

    #[test]
    fn the_first_server_follows_graal3d_and_a_warp_its_name() {
        let side = |first| Side::new(ClientVersion::V512, Login::default(), first);
        let join = |name: Option<&str>| ServerJoin {
            addr: "127.0.0.1:14900".into(),
            name: name.map(String::from),
        };
        assert!(side(true).is_3d(&join(None)));
        assert!(!side(false).is_3d(&join(None)));
        assert!(side(false).is_3d(&join(Some("3 Lab3D"))));
        assert!(!side(true).is_3d(&join(Some("Lab Server"))));
    }

    #[test]
    fn the_udp_port_is_the_listed_port_plus_one() {
        assert_eq!(
            udp_addr("127.0.0.1:14900").unwrap(),
            "127.0.0.1:14901".parse().unwrap()
        );
    }
}
