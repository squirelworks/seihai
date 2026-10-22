//! UDP peer channel scaffold.
//!
//! Ids 21/23/24 are noted in the cleaned protocol notes. No test traffic
//! is available, so this is a framing scaffold only.
//!
//! Fixed UDP port 14899. IP and port are exchanged via player props 30/31.

use std::net::UdpSocket;

/// UDP peer message ids (cleaned notes).
pub const UDP_ID_POS: u8 = 21;
pub const UDP_ID_PROPS: u8 = 23;
pub const UDP_ID_BOARD: u8 = 24;

/// Fixed peer port.
pub const UDP_PORT: u16 = 14899;

/// A UDP peer socket bound to the local peer port.
pub struct PeerChannel {
    sock: UdpSocket,
}

impl PeerChannel {
    /// Bind to `0.0.0.0:14899`.
    pub fn bind() -> std::io::Result<Self> {
        let sock = UdpSocket::bind(("0.0.0.0", UDP_PORT))?;
        sock.set_nonblocking(true)?;
        Ok(Self { sock })
    }

    /// Send a raw datagram to `addr`.
    pub fn send_to(&self, data: &[u8], addr: &str) -> std::io::Result<usize> {
        self.sock.send_to(data, addr)
    }

    /// Receive one datagram, if available.
    pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<(usize, std::net::SocketAddr)> {
        self.sock.recv_from(buf)
    }
}

/// Build a minimal UDP position frame (id 21).
/// Layout is provisional: id byte + classic x + classic y + dir.
pub fn encode_udp_pos(x: u8, y: u8, dir: u8) -> Vec<u8> {
    vec![UDP_ID_POS, x, y, dir]
}
