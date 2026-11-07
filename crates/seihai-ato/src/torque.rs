//! The Torque side of each Graal build: what its `GameConnection` connect
//! request carries, and the UDP socket Seihai opens beside the Graal TCP login.
//!
//! Profiles were read statically from the client binaries (Graal_analysis
//! `torque/wire-by-build.md` and `torque/network.md`); only 5.12 was also
//! seen live. Ato runs 3.04 and 3.153; the 4.x and 5.12 rows are here so a
//! later Torque client picks its profile by protocol number in one place.
//! [`open_udp`] does not write a connect request. That handshake is not
//! reproduced here.
//!
//! **Graal 3.04 Linux (Gemini pack, string scan):** The real client is an
//! older Torque Game Engine, not the vendored Torque3D v4.0.3 tree. It
//! exposes TorqueScript preferences `$pref::Graal::fixedport`,
//! `$pref::Graal::fixedudpport`, and `$pref::Graal::noudp`. [`open_udp`]
//! honors them: the socket connects to the listed host on [`udp_port`]
//! unless `noudp` is set, and binds `fixedudpport` when that port is set.
//! v4.0.3 stays the open submodule pin (`documentation/torque-lineage.md`).
//! It is not this socket and it is not the 3.04 binary.

use seihai_core::protocol::ClientVersion;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};

/// TorqueScript preference names from the Graal 3.04 Linux string scan.
/// [`open_udp`] honors them. Torque3D v4.0.3 does not.
pub const GRAAL_PREF_FIXED_PORT: &str = "$pref::Graal::fixedport";
/// See [`GRAAL_PREF_FIXED_PORT`].
pub const GRAAL_PREF_FIXED_UDP_PORT: &str = "$pref::Graal::fixedudpport";
/// See [`GRAAL_PREF_FIXED_PORT`].
pub const GRAAL_PREF_NO_UDP: &str = "$pref::Graal::noudp";

/// One Torque connect profile, keyed by the protocol number the client
/// sends twice (current and minimum) and wants back exactly in the accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorqueProfile {
    /// Which builds use it, for messages.
    pub builds: &'static str,
    /// The game string in the connect request.
    pub game: &'static str,
    pub protocol: u32,
    /// Class types listed in Graal's class table at the front of the
    /// accept: 0 for no table (stock Torque), 3 (object, datablock, event)
    /// or 2 (object, event; no datablocks).
    pub class_table_types: u8,
    /// Class ids on the wire are a fixed 12 bits in the server's numbering
    /// (Graal) rather than stock Torque's count-based width.
    pub class_ids_12bit: bool,
    /// Every event and ghost update carries a 14-bit bit length.
    pub length_prefixes: bool,
    /// The Torque game layer (datablocks, `Player`, moves) is present;
    /// `false` for Graal's own `GameObject` model.
    pub tge_game_layer: bool,
}

/// 3.04 Linux: stock TGE 1.x networking.
pub const TGE_9: TorqueProfile = TorqueProfile {
    builds: "3.04",
    game: "Torque Game Engine Demo",
    protocol: 9,
    class_table_types: 0,
    class_ids_12bit: false,
    length_prefixes: false,
    tge_game_layer: true,
};

/// 3.153 Mac and 4.04r2 Windows.
pub const GRAAL3D_19: TorqueProfile = TorqueProfile {
    builds: "3.153, 4.04r2",
    game: "Graal3D",
    protocol: 19,
    class_table_types: 3,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: true,
};

/// 4.20r6 Mac.
pub const GRAAL3D_21: TorqueProfile = TorqueProfile {
    builds: "4.20r6",
    game: "Graal3D",
    protocol: 21,
    class_table_types: 2,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: false,
};

/// 5.12 Windows (also 5.00r7's protocol number).
pub const GRAAL3D_24: TorqueProfile = TorqueProfile {
    builds: "5.12",
    game: "Graal3D",
    protocol: 24,
    class_table_types: 2,
    class_ids_12bit: true,
    length_prefixes: true,
    tge_game_layer: false,
};

pub const ALL: [TorqueProfile; 4] = [TGE_9, GRAAL3D_19, GRAAL3D_21, GRAAL3D_24];

/// The profile with this protocol number.
pub fn by_protocol(protocol: u32) -> Option<TorqueProfile> {
    ALL.into_iter().find(|p| p.protocol == protocol)
}

/// The profile of a build Ato runs.
pub fn of(v: ClientVersion) -> Option<TorqueProfile> {
    match v {
        ClientVersion::V304 => Some(TGE_9),
        ClientVersion::V3153 => Some(GRAAL3D_19),
        _ => None,
    }
}

/// The Torque UDP port for a 3D server listed on `listed_port`: the
/// listed port + 1, on the same host, beside the Graal TCP connection.
/// (Torque's own default, 28000, only applies to an address with no port.)
pub fn udp_port(listed_port: u16) -> u16 {
    listed_port.wrapping_add(1)
}

/// Shipped `$pref::Graal::fixedudpport` when the fixed port is on and the
/// caller did not pass a number. Seihai binds this itself; the Torque3D
/// v4.0.3 tree does not.
pub const DEFAULT_FIXED_UDP_PORT: u16 = 14899;

/// `$pref::Graal::noudp`, `$pref::Graal::fixedport`, and
/// `$pref::Graal::fixedudpport` for [`open_udp`].
///
/// `no_udp` leaves the socket closed. Otherwise Seihai binds its own UDP
/// socket and connects it to the listed host on [`udp_port`]. `fixed_port`,
/// or a port passed to [`Self::from_flags`], binds `fixed_udp_port` locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraalUdpPrefs {
    pub no_udp: bool,
    pub fixed_port: bool,
    pub fixed_udp_port: u16,
}

impl Default for GraalUdpPrefs {
    fn default() -> Self {
        Self {
            no_udp: false,
            fixed_port: false,
            fixed_udp_port: DEFAULT_FIXED_UDP_PORT,
        }
    }
}

impl GraalUdpPrefs {
    /// `noudp` wins over a fixed port. A `Some` port counts as fixed.
    pub fn from_flags(no_udp: bool, fixed_port: bool, fixed_udp_port: Option<u16>) -> Self {
        Self {
            no_udp,
            fixed_port: fixed_port || fixed_udp_port.is_some(),
            fixed_udp_port: fixed_udp_port.unwrap_or(DEFAULT_FIXED_UDP_PORT),
        }
    }
}

/// Seihai's Torque UDP socket: bound locally, connected to listed port + 1.
/// No handshake bytes are written.
#[derive(Debug)]
pub struct TorqueUdp {
    socket: UdpSocket,
    local: SocketAddr,
    remote: SocketAddr,
    fixed: bool,
}

impl TorqueUdp {
    pub fn local_addr(&self) -> SocketAddr {
        self.local
    }

    pub fn remote(&self) -> SocketAddr {
        self.remote
    }

    /// One line for the login log.
    pub fn describe(&self) -> String {
        let how = if self.fixed {
            GRAAL_PREF_FIXED_UDP_PORT
        } else {
            "ephemeral local port"
        };
        format!("{} -> {} (listed port + 1; {how})", self.local, self.remote)
    }

    /// Deliver `bytes` to the connected Torque port. Not a handshake.
    pub fn send(&self, bytes: &[u8]) -> io::Result<usize> {
        self.socket.send(bytes)
    }
}

/// Loopback listed hosts bind that same loopback address, which is what the
/// lab tests use. Any other listed host binds the unspecified address of
/// the same family, so a reply can come back. Callers pass the host; this
/// crate does not name a public server.
fn bind_ip_for(remote: IpAddr) -> IpAddr {
    match remote {
        IpAddr::V4(ip) if ip.is_loopback() => IpAddr::V4(ip),
        IpAddr::V6(ip) if ip.is_loopback() => IpAddr::V6(ip),
        IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    }
}

/// Open the Torque UDP socket for a server listed at `remote_host:listed_port`.
///
/// `Ok(None)` when [`GraalUdpPrefs::no_udp`] is set. Otherwise the socket
/// is connected to [`udp_port`]`(listed_port)` and left nonblocking. Nothing
/// is sent.
pub fn open_udp(
    remote_host: IpAddr,
    listed_port: u16,
    prefs: GraalUdpPrefs,
) -> io::Result<Option<TorqueUdp>> {
    if prefs.no_udp {
        return Ok(None);
    }
    let remote_port = udp_port(listed_port);
    if remote_port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "listed port 65535 has no Torque UDP port",
        ));
    }
    if prefs.fixed_port && prefs.fixed_udp_port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixedudpport is 0",
        ));
    }
    let local_port = if prefs.fixed_port {
        prefs.fixed_udp_port
    } else {
        0
    };
    let bind = SocketAddr::new(bind_ip_for(remote_host), local_port);
    let socket = UdpSocket::bind(bind).map_err(|e| {
        if prefs.fixed_port {
            io::Error::new(e.kind(), format!("fixed UDP port {local_port}: {e}"))
        } else {
            e
        }
    })?;
    let remote = SocketAddr::new(remote_host, remote_port);
    socket
        .connect(remote)
        .map_err(|e| io::Error::new(e.kind(), format!("Torque UDP {remote}: {e}")))?;
    socket.set_nonblocking(true)?;
    let local = socket.local_addr()?;
    Ok(Some(TorqueUdp {
        socket,
        local,
        remote,
        fixed: prefs.fixed_port,
    }))
}

/// A server is a 3D server when its listed name starts with `"3 "`.
pub fn is_3d_server(listed_name: &str) -> bool {
    listed_name.starts_with("3 ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

    #[test]
    fn protocols_are_distinct() {
        for p in ALL {
            assert_eq!(by_protocol(p.protocol), Some(p));
        }
        assert_eq!(by_protocol(12), None);
    }

    #[test]
    fn ato_builds() {
        assert_eq!(of(ClientVersion::V304).unwrap().protocol, 9);
        assert_eq!(of(ClientVersion::V3153).unwrap().protocol, 19);
        assert_eq!(of(ClientVersion::V231), None);
    }

    #[test]
    fn three_d_servers() {
        assert!(is_3d_server("3 Seihai Test"));
        assert!(!is_3d_server("Seihai 3"));
        assert_eq!(udp_port(14800), 14801);
    }

    #[test]
    fn gemini_304_graal_torque_pref_names() {
        assert_eq!(GRAAL_PREF_FIXED_PORT, "$pref::Graal::fixedport");
        assert_eq!(GRAAL_PREF_FIXED_UDP_PORT, "$pref::Graal::fixedudpport");
        assert_eq!(GRAAL_PREF_NO_UDP, "$pref::Graal::noudp");
    }

    fn loopback_listener() -> UdpSocket {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        socket
    }

    #[test]
    fn noudp_does_not_bind() {
        let held = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        let prefs = GraalUdpPrefs::from_flags(true, true, Some(port));
        let opened = open_udp(IpAddr::V4(Ipv4Addr::LOCALHOST), 14900, prefs).unwrap();
        assert!(opened.is_none());
        let err = UdpSocket::bind(("127.0.0.1", port)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AddrInUse);
    }

    #[test]
    fn opens_toward_listed_port_plus_one_on_loopback() {
        let server = loopback_listener();
        let remote_port = server.local_addr().unwrap().port();
        let listed = remote_port.wrapping_sub(1);
        assert_eq!(udp_port(listed), remote_port);
        let sock = open_udp(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            listed,
            GraalUdpPrefs::default(),
        )
        .unwrap()
        .expect("socket");
        assert_eq!(sock.local_addr().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_ne!(sock.local_addr().port(), 0);
        assert_eq!(
            sock.remote(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), remote_port)
        );
        assert!(!sock.describe().contains(GRAAL_PREF_FIXED_UDP_PORT));
        sock.send(b"seihai-ato").unwrap();
        let mut buf = [0u8; 32];
        let (n, from) = server.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"seihai-ato");
        assert_eq!(from, sock.local_addr());
    }

    #[test]
    fn fixedudpport_is_bound_on_loopback() {
        let hold = UdpSocket::bind("127.0.0.1:0").unwrap();
        let fixed = hold.local_addr().unwrap().port();
        drop(hold);
        let server = loopback_listener();
        let remote_port = server.local_addr().unwrap().port();
        assert_ne!(fixed, remote_port);
        let listed = remote_port.wrapping_sub(1);
        let prefs = GraalUdpPrefs::from_flags(false, false, Some(fixed));
        assert!(prefs.fixed_port);
        assert_eq!(prefs.fixed_udp_port, fixed);
        let sock = open_udp(IpAddr::V4(Ipv4Addr::LOCALHOST), listed, prefs)
            .unwrap()
            .expect("socket");
        assert_eq!(
            sock.local_addr(),
            SocketAddr::from((Ipv4Addr::LOCALHOST, fixed))
        );
        assert!(sock.describe().contains(GRAAL_PREF_FIXED_UDP_PORT));
        let err = open_udp(IpAddr::V4(Ipv4Addr::LOCALHOST), listed, prefs).unwrap_err();
        assert!(
            err.to_string().contains(&format!("fixed UDP port {fixed}")),
            "{err}"
        );
        sock.send(b"fixed").unwrap();
        let mut buf = [0u8; 8];
        let (n, from) = server.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"fixed");
        assert_eq!(from, SocketAddr::from((Ipv4Addr::LOCALHOST, fixed)));
    }

    #[test]
    fn listed_loopback_uses_that_address() {
        let second = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        assert_eq!(bind_ip_for(second), second);
        assert_eq!(
            bind_ip_for(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert_eq!(
            bind_ip_for(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
    }

    #[test]
    fn second_loopback_address_binds_there() {
        let host: IpAddr = "127.0.0.2".parse().unwrap();
        let server = match UdpSocket::bind(SocketAddr::new(host, 0)) {
            Ok(socket) => socket,
            // macOS configures only 127.0.0.1 on lo0 unless an alias is
            // added. CI adds 127.0.0.2 before the tests. Without that
            // alias the bind cannot succeed; the address choice itself
            // is `listed_loopback_uses_that_address`. Linux and Windows
            // have the whole 127/8, so a miss there is a real failure.
            Err(e) if e.kind() == io::ErrorKind::AddrNotAvailable && cfg!(target_os = "macos") => {
                eprintln!("skip second_loopback_address_binds_there: {host} is not assigned ({e})");
                return;
            }
            Err(e) => panic!("bind {host}: {e}"),
        };
        server
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let remote_port = server.local_addr().unwrap().port();
        let listed = remote_port.wrapping_sub(1);
        let sock = open_udp(host, listed, GraalUdpPrefs::default())
            .unwrap()
            .expect("socket");
        assert_eq!(sock.local_addr().ip(), host);
        assert_eq!(sock.remote(), SocketAddr::new(host, remote_port));
        sock.send(b"lo").unwrap();
        let mut buf = [0u8; 4];
        let (n, from) = server.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"lo");
        assert_eq!(from.ip(), host);
    }

    #[test]
    fn listed_port_at_the_end_of_the_range_is_refused() {
        let err = open_udp(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            u16::MAX,
            GraalUdpPrefs::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("65535"), "{err}");
    }

    #[test]
    fn from_flags_noudp_wins_and_fixedport_uses_the_default() {
        let off = GraalUdpPrefs::from_flags(true, true, Some(14923));
        assert!(off.no_udp && off.fixed_port && off.fixed_udp_port == 14923);
        let fixed = GraalUdpPrefs::from_flags(false, true, None);
        assert!(!fixed.no_udp && fixed.fixed_port);
        assert_eq!(fixed.fixed_udp_port, DEFAULT_FIXED_UDP_PORT);
    }
}
