//! The Graal3D side of an online session: the Torque UDP link started
//! beside the TCP login (`--graal3d`).
//!
//! Both start at the same moment, to the same host: TCP on the listed
//! port, UDP on the listed port + 1. The UDP link stops when the TCP side
//! ends. Only 4.20r6 and 5.x (protocols 21 and 24) run here; 3.153 and
//! 4.04r2 use the Torque game layer, which Ato does not run yet.

use crate::online::Login;
#[cfg(not(feature = "era6"))]
use crate::torque::net::session::{self, Identity};
use crate::torque::{self, TorqueProfile};
use seihai_core::protocol::ClientVersion;
#[cfg(not(feature = "era6"))]
use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
pub fn start(_addr: &str, _version: ClientVersion, _login: &Login) -> Result<Udp3d, String> {
    Err("--graal3d: this build leaves out the Torque engine (era6)".into())
}

/// Starts the UDP link for `login` on its own thread; progress goes to
/// stdout. A 5.x guest's account on the link is `guest`.
#[cfg(not(feature = "era6"))]
pub fn start(addr: &str, version: ClientVersion, login: &Login) -> Result<Udp3d, String> {
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
    fn the_udp_port_is_the_listed_port_plus_one() {
        assert_eq!(
            udp_addr("127.0.0.1:14900").unwrap(),
            "127.0.0.1:14901".parse().unwrap()
        );
    }
}
