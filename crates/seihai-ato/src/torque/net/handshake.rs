//! The out-of-band handshake that opens a Graal3D Torque UDP link.
//!
//! The client sends a challenge request, the server answers with a digest
//! of the client's address, the client echoes it in a connect request
//! and the server accepts (with Graal's class table) or rejects. Every
//! packet of the link repeats the connect sequence the client picked.
//!
//! An out-of-band packet's first bit is 0 (its first byte, the packet
//! type, is even); a data packet's first bit is 1.

use super::bits::{BitReader, BitWriter, OutOfBits};
use crate::torque::TorqueProfile;

/// Out-of-band packet types.
pub mod kind {
    pub const CHALLENGE_REQUEST: u8 = 0x1a;
    pub const CHALLENGE_REJECT: u8 = 0x1c;
    pub const CHALLENGE_RESPONSE: u8 = 0x1e;
    pub const CONNECT_REQUEST: u8 = 0x20;
    pub const CONNECT_REJECT: u8 = 0x22;
    pub const CONNECT_ACCEPT: u8 = 0x24;
    pub const DISCONNECT: u8 = 0x26;
}

/// The network class name every Graal3D client connects as.
pub const CONNECTION_CLASS: &[u8] = b"GameConnection";

/// The class CRC the 4.04r2 and 5.x clients send.
pub const ANY_CLASS_CRC: u32 = 0xffff_ffff;

/// The most connect arguments a request carries.
pub const MAX_ARGS: usize = 16;

/// Whether a UDP packet is a data packet (first bit 1) rather than out-of-band.
pub fn is_data_packet(packet: &[u8]) -> bool {
    packet.first().is_some_and(|b| b & 1 == 1)
}

/// The challenge request: type and the connect sequence.
pub fn challenge_request(sequence: u32) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(kind::CHALLENGE_REQUEST);
    w.u32(sequence);
    w.into_bytes()
}

/// What the client puts in its connect request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectRequest<'a> {
    pub sequence: u32,
    /// The digest from the challenge response, echoed.
    pub digest: [u32; 4],
    pub profile: TorqueProfile,
    /// The account password, sent in clear by 4.04r2 and 5.x.
    pub password: &'a [u8],
    /// Account name, then nickname (may be empty). A 5.x guest sends
    /// `guest` as the account.
    pub args: &'a [&'a [u8]],
}

impl ConnectRequest<'_> {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = BitWriter::new();
        w.u8(kind::CONNECT_REQUEST);
        w.u32(self.sequence);
        for d in self.digest {
            w.u32(d);
        }
        w.string(CONNECTION_CLASS);
        w.u32(0); // net class group
        w.u32(ANY_CLASS_CRC);
        w.string(self.profile.game.as_bytes());
        w.u32(self.profile.protocol); // current
        w.u32(self.profile.protocol); // minimum
        w.string(self.password);
        let args = &self.args[..self.args.len().min(MAX_ARGS)];
        w.u32(args.len() as u32);
        for a in args {
            w.string(a);
        }
        w.into_bytes()
    }
}

/// One entry of Graal's class table in the connect accept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassEntry {
    /// Object 0, then datablock 1 and event 2 (protocol 19) or event 1
    /// (protocols 21 and 24).
    pub class_type: u32,
    /// The id the server uses for this class on the wire.
    pub server_id: u32,
    pub name: Vec<u8>,
}

/// An out-of-band packet from the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerPacket {
    ChallengeResponse {
        sequence: u32,
        digest: [u32; 4],
    },
    ChallengeReject {
        sequence: u32,
    },
    ConnectAccept {
        sequence: u32,
        classes: Vec<ClassEntry>,
        protocol: u32,
    },
    ConnectReject {
        sequence: u32,
        reason: Vec<u8>,
    },
    Disconnect {
        sequence: u32,
        reason: Vec<u8>,
    },
}

impl ServerPacket {
    pub fn sequence(&self) -> u32 {
        match *self {
            ServerPacket::ChallengeResponse { sequence, .. }
            | ServerPacket::ChallengeReject { sequence }
            | ServerPacket::ConnectAccept { sequence, .. }
            | ServerPacket::ConnectReject { sequence, .. }
            | ServerPacket::Disconnect { sequence, .. } => sequence,
        }
    }
}

/// Why an out-of-band packet could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// A data packet, not out-of-band.
    DataPacket,
    /// A type a client does not take.
    UnknownType(u8),
    /// A class table longer than 12-bit ids allow.
    TooManyClasses,
    Short(OutOfBits),
}

impl From<OutOfBits> for ReadError {
    fn from(e: OutOfBits) -> Self {
        ReadError::Short(e)
    }
}

/// The most entries a class table may list (ids are 12 bits).
const MAX_CLASSES: usize = 4096;

/// Reads an out-of-band packet a server sends.
pub fn read_server_packet(packet: &[u8]) -> Result<ServerPacket, ReadError> {
    if is_data_packet(packet) {
        return Err(ReadError::DataPacket);
    }
    let mut r = BitReader::new(packet);
    let ty = r.u8()?;
    let sequence = r.u32()?;
    Ok(match ty {
        kind::CHALLENGE_RESPONSE => {
            let mut digest = [0; 4];
            for d in &mut digest {
                *d = r.u32()?;
            }
            ServerPacket::ChallengeResponse { sequence, digest }
        }
        kind::CHALLENGE_REJECT => ServerPacket::ChallengeReject { sequence },
        kind::CONNECT_ACCEPT => {
            let mut classes = Vec::new();
            while r.flag()? {
                if classes.len() == MAX_CLASSES {
                    return Err(ReadError::TooManyClasses);
                }
                classes.push(ClassEntry {
                    class_type: r.int(12)?,
                    server_id: r.int(12)?,
                    name: r.string()?,
                });
            }
            ServerPacket::ConnectAccept {
                sequence,
                classes,
                protocol: r.u32()?,
            }
        }
        kind::CONNECT_REJECT => ServerPacket::ConnectReject {
            sequence,
            reason: r.string()?,
        },
        kind::DISCONNECT => ServerPacket::Disconnect {
            sequence,
            reason: r.string()?,
        },
        other => return Err(ReadError::UnknownType(other)),
    })
}

/// A disconnect the client sends when its TCP side closes.
pub fn disconnect(sequence: u32, reason: &[u8]) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.u8(kind::DISCONNECT);
    w.u32(sequence);
    w.string(reason);
    w.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torque::GRAAL3D_24;

    #[test]
    fn the_challenge_request_matches_the_512_capture() {
        assert_eq!(challenge_request(0x400), [0x1a, 0x00, 0x04, 0x00, 0x00]);
    }

    #[test]
    fn reads_the_captured_challenge_response() {
        let mut p = vec![0x1e, 0x00, 0x04, 0x00, 0x00];
        p.extend(0u8..16);
        assert_eq!(
            read_server_packet(&p),
            Ok(ServerPacket::ChallengeResponse {
                sequence: 0x400,
                digest: [0x0302_0100, 0x0706_0504, 0x0b0a_0908, 0x0f0e_0d0c],
            })
        );
    }

    /// The real 5.12 client's connect request (guest, empty nickname).
    const REQUEST_512: &str = "2000040000000102030405060708090a0b0c0d0e0f1d6213fd6a4af4185809000000e0ffffffff41ac650e47670c0000000c00008008d1e976dc8548169b000000c082de7f0300";
    /// A protocol-24 accept naming all 19 classes, from the research tools.
    const FULL_ACCEPT_24: &str = "24000400000100001aac96e82d0340006470ae5022c6b196ad05000168e15ca1448c632d5bebfa7a304e500b0003d0c2b94289184fe7acbfebebc138412d00104085d844bfe26ef1180240011c984e2a01c0000e4c3d058003c8b07eb3648a188a270100012a3838e12d62289e048004a8b039997a92e77e860100002322b8774ec8d5664f93b8f404032000464470ef9c90534883b8f4040340005aaca6448f81955cdf5af61f979e60000c408bd8591b748e222b6dc5dde231c4a527180004d0c1ed610ac6d4c7a52718000550a2eb91c2afa6082585b8f40403c0004a747d64f3dfa0eb8766884b4f30000e60c4facd9269b5d9d3242e3dc10040801aeb374ba6ae8f6cfe1bc8b10e09dfe2d213c000000000";

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn writes_the_512_connect_request_byte_for_byte() {
        let real = hex(REQUEST_512);
        assert_eq!(real.len(), 71);
        // Take the password from the capture, then write the whole request.
        let mut r = BitReader::new(&real[21..]);
        r.string().unwrap();
        for _ in 0..2 {
            r.u32().unwrap();
        }
        r.string().unwrap();
        r.u32().unwrap();
        r.u32().unwrap();
        let password = r.string().unwrap();
        let req = ConnectRequest {
            sequence: 0x400,
            digest: [0x0302_0100, 0x0706_0504, 0x0b0a_0908, 0x0f0e_0d0c],
            profile: GRAAL3D_24,
            password: &password,
            args: &[b"guest", b""],
        };
        assert_eq!(req.encode(), real);
    }

    #[test]
    fn reads_the_captured_empty_accept() {
        assert_eq!(
            read_server_packet(&hex("24000400003000000000")),
            Ok(ServerPacket::ConnectAccept {
                sequence: 0x400,
                classes: vec![],
                protocol: 24
            })
        );
    }

    #[test]
    fn reads_a_full_protocol_24_accept() {
        let ServerPacket::ConnectAccept {
            classes, protocol, ..
        } = read_server_packet(&hex(FULL_ACCEPT_24)).unwrap()
        else {
            panic!("an accept");
        };
        assert_eq!(protocol, 24);
        assert_eq!(classes.len(), 19);
        assert_eq!(
            (
                classes[0].class_type,
                classes[0].server_id,
                &classes[0].name[..]
            ),
            (0, 0, &b"Camera"[..])
        );
        assert_eq!(classes[4].name, b"GameObject");
        assert_eq!((classes[18].class_type, classes[18].server_id), (1, 8));
        assert_eq!(classes[18].name, b"TerrainRequestUpdatesEvent");
        // Written back with this module's strings, it is the same bytes.
        let table: Vec<(u32, u32, &[u8])> = classes
            .iter()
            .map(|c| (c.class_type, c.server_id, &c.name[..]))
            .collect();
        assert_eq!(accept(&table, 24), hex(FULL_ACCEPT_24));
    }

    #[test]
    fn the_connect_request_carries_the_512_fields_in_order() {
        let req = ConnectRequest {
            sequence: 0x400,
            digest: [0x0302_0100, 0x0706_0504, 0x0b0a_0908, 0x0f0e_0d0c],
            profile: GRAAL3D_24,
            password: b"secret",
            args: &[b"alice", b""],
        };
        let bytes = req.encode();
        assert_eq!(&bytes[..5], [0x20, 0x00, 0x04, 0x00, 0x00]);
        assert_eq!(&bytes[5..21], (0u8..16).collect::<Vec<_>>());
        let mut r = BitReader::new(&bytes[21..]);
        assert_eq!(r.string().unwrap(), b"GameConnection");
        assert_eq!(r.u32().unwrap(), 0, "class group");
        assert_eq!(r.u32().unwrap(), ANY_CLASS_CRC);
        assert_eq!(r.string().unwrap(), b"Graal3D");
        assert_eq!((r.u32().unwrap(), r.u32().unwrap()), (24, 24));
        assert_eq!(r.string().unwrap(), b"secret");
        assert_eq!(r.u32().unwrap(), 2);
        assert_eq!(r.string().unwrap(), b"alice");
        assert_eq!(r.string().unwrap(), b"");
        assert!(r.remaining() < 8, "nothing after the arguments");
    }

    fn accept(classes: &[(u32, u32, &[u8])], protocol: u32) -> Vec<u8> {
        let mut w = BitWriter::new();
        w.u8(kind::CONNECT_ACCEPT);
        w.u32(0x400);
        for (t, id, name) in classes {
            w.flag(true);
            w.int(*t, 12);
            w.int(*id, 12);
            w.string(name);
        }
        w.flag(false);
        w.u32(protocol);
        w.into_bytes()
    }

    #[test]
    fn reads_the_class_table_of_an_accept() {
        let p = accept(&[(0, 4, b"GameObject"), (1, 5, b"RemoteCommandEvent")], 24);
        let ServerPacket::ConnectAccept {
            sequence,
            classes,
            protocol,
        } = read_server_packet(&p).unwrap()
        else {
            panic!("an accept");
        };
        assert_eq!((sequence, protocol), (0x400, 24));
        assert_eq!(
            classes[0],
            ClassEntry {
                class_type: 0,
                server_id: 4,
                name: b"GameObject".to_vec()
            }
        );
        assert_eq!(classes[1].name, b"RemoteCommandEvent");
        let empty = accept(&[], 24);
        assert!(matches!(
            read_server_packet(&empty),
            Ok(ServerPacket::ConnectAccept { protocol: 24, .. })
        ));
    }

    #[test]
    fn rejects_and_disconnects_carry_a_reason() {
        let mut w = BitWriter::new();
        w.u8(kind::CONNECT_REJECT);
        w.u32(7);
        w.string(b"Only official Graal clients are allowed to connect.");
        assert_eq!(
            read_server_packet(&w.into_bytes()),
            Ok(ServerPacket::ConnectReject {
                sequence: 7,
                reason: b"Only official Graal clients are allowed to connect.".to_vec()
            })
        );
        assert_eq!(
            read_server_packet(&disconnect(0x400, b"bye")),
            Ok(ServerPacket::Disconnect {
                sequence: 0x400,
                reason: b"bye".to_vec()
            })
        );
    }

    #[test]
    fn data_packets_and_strays_are_not_handshake_packets() {
        assert!(is_data_packet(&[0x05, 0x00]));
        assert_eq!(
            read_server_packet(&[0x05, 0x00]),
            Err(ReadError::DataPacket)
        );
        assert_eq!(
            read_server_packet(&[0x30, 0, 0, 0, 0]),
            Err(ReadError::UnknownType(0x30))
        );
        assert!(matches!(
            read_server_packet(&[0x1e, 0x00]),
            Err(ReadError::Short(_))
        ));
    }
}
