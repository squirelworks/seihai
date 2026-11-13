//! The protocol 19 game layer (3.153 Mac, 4.04r2 Windows): the stock
//! Torque game sections, the datablock phase and the three-phase mission
//! start.
//!
//! Built from §10 of the clean-room note "The Torque UDP session, start
//! to first ghost" (Graal_analysis `torque/udp-session.md`) and
//! `graal3d/torque-engine-4x.md`, both static reads of the clients, and
//! from the open (MIT) Torque3D connection messages. No real 3.153 or
//! 4.04r2 client or server has been run past the handshake, so what is
//! here is the part those reads fix:
//!
//! - the server's game section up to the control object's or camera's
//!   own packet data, which is stock `Player` / `Camera` data with no
//!   length prefix and not read yet ([`Game19Error::PacketData`]);
//! - the client's game section with **no moves** (a move count of 0)
//!   until the move layout is traced;
//! - `SimDataBlockEvent` headers, each datablock's fields skipped by the
//!   event's length;
//! - the mission start answers: 4.04r2 acks each phase with the bare
//!   command, 3.153 (whose mission start runs in TorqueScript) echoes
//!   the phase sequence the server sent;
//! - the stock connection-message answers (ghosting and datablocks done).
//!
//! Ghost updates are kept by class name only ([`super::ghosts::Ghosts::opaque`]).

use super::bits::{BitReader, BitWriter, OutOfBits};
use super::events::{Arg, Command, Event};
use super::ghosts::{read_vec3, write_vec3, GhostState, Ghosts, Vec3};

/// The most moves one client packet may carry (`int5`).
pub const MAX_MOVES: usize = 30;

/// Stock Torque connection messages (`ConnectionMessageEvent`), with
/// `GameConnection`'s two datablock messages after them. The six stock
/// names are listed in every Graal build (`open-source-lineage.md`); the
/// numbering and the datablock pair follow Torque3D (MIT) and are not
/// traced on a Graal client yet.
pub mod message {
    pub const GHOST_ALWAYS_DONE: u32 = 0;
    pub const READY_FOR_NORMAL_GHOSTS: u32 = 1;
    pub const END_GHOSTING: u32 = 2;
    pub const GHOST_ALWAYS_STARTING: u32 = 3;
    pub const SEND_NEXT_DOWNLOAD_REQUEST: u32 = 4;
    pub const FILE_DOWNLOAD_SIZE: u32 = 5;
    /// Server: every datablock of this sequence has been sent.
    pub const DATA_BLOCKS_DONE: u32 = 6;
    /// Client: the datablocks of this sequence are loaded.
    pub const DATA_BLOCKS_DOWNLOAD_DONE: u32 = 7;
}

/// The client's answer to a connection message, if it needs one: after
/// the ghost-always objects it is ready for normal ghosts, and after the
/// datablocks it reports them loaded, each with the server's sequence.
pub fn connection_answer(ev: &Event) -> Option<Event> {
    let Event::ConnectionMessage {
        sequence, message, ..
    } = *ev
    else {
        return None;
    };
    let reply = match message {
        message::GHOST_ALWAYS_DONE => message::READY_FOR_NORMAL_GHOSTS,
        message::DATA_BLOCKS_DONE => message::DATA_BLOCKS_DOWNLOAD_DONE,
        _ => return None,
    };
    Some(Event::ConnectionMessage {
        sequence,
        message: reply,
        ghost_count: 0,
    })
}

/// A mission start phase of protocol 19.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    /// "Download Datablocks & Targets".
    Datablocks = 1,
    /// "Download Ghost Objects".
    Ghosts = 2,
    /// "Mission Lighting"; acknowledged once the scene is lit.
    Lighting = 3,
}

impl Phase {
    /// What the client shows for the phase.
    pub fn label(self) -> &'static str {
        match self {
            Phase::Datablocks => "Phase 1: Download Datablocks & Targets",
            Phase::Ghosts => "Phase 2: Download Ghost Objects",
            Phase::Lighting => "Phase 3: Mission Lighting",
        }
    }
}

/// How a build acknowledges the mission phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseAcks {
    /// 4.04r2: native code, the bare `MissionStartPhaseNAck`.
    Native,
    /// 3.153: TorqueScript, the ack carries the phase sequence (the
    /// command's first argument) back.
    Scripted,
}

/// The phase a `commandToClient` starts and the ack to send for it, or
/// `None` for any other command.
pub fn mission_start(cmd: &Command, acks: PhaseAcks) -> Option<(Phase, Vec<u8>, Vec<Arg>)> {
    let phase = match cmd.name.as_slice() {
        b"MissionStartPhase1" => Phase::Datablocks,
        b"MissionStartPhase2" => Phase::Ghosts,
        b"MissionStartPhase3" => Phase::Lighting,
        _ => return None,
    };
    let mut name = cmd.name.clone();
    name.extend_from_slice(b"Ack");
    let args = match acks {
        PhaseAcks::Native => Vec::new(),
        PhaseAcks::Scripted => cmd.args.first().cloned().into_iter().collect(),
    };
    Some((phase, name, args))
}

/// The control part of the server's game section.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Control19 {
    /// Flag clear: no control object in this packet.
    #[default]
    Unchanged,
    /// The control object, with the server's clock (ms). Its packet data
    /// follows on the wire.
    Object { server_time: u32, index: u32 },
    /// The control object is not dirty: only the compression point.
    CompressionPoint(Vec3),
}

/// The server's game section of a protocol 19 data packet.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServerGame19 {
    /// The last of our moves the server has applied.
    pub last_move_ack: u32,
    /// 0 to 1 (`float7`).
    pub damage_flash: Option<f32>,
    pub white_out: Option<f32>,
    pub control: Control19,
    pub camera: Option<u32>,
    pub field_of_view: Option<u8>,
}

/// Why a protocol 19 game section stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum Game19Error {
    Short(OutOfBits),
    /// The control object or camera names an index the client lacks.
    UnknownGhost(u32),
    /// The control object's or camera's own packet data follows. It has no
    /// length prefix and its layout is not read yet, so the rest of the
    /// packet (camera, field of view, events, ghosts) cannot be found.
    /// `game` holds what was read before it.
    PacketData {
        index: u32,
        class: Option<String>,
        game: ServerGame19,
    },
}

impl From<OutOfBits> for Game19Error {
    fn from(e: OutOfBits) -> Self {
        Game19Error::Short(e)
    }
}

fn float7(r: &mut BitReader<'_>) -> Result<f32, OutOfBits> {
    Ok(r.int(7)? as f32 / 127.0)
}

fn class_of(ghosts: &Ghosts, index: u32) -> Result<Option<String>, Game19Error> {
    match ghosts.ghosts.get(&index) {
        Some(GhostState::Other(name)) => Ok(name.clone()),
        Some(_) => Ok(None),
        None => Err(Game19Error::UnknownGhost(index)),
    }
}

/// Reads the server's game section (§10):
///
/// ```text
/// int32 last move ack
/// flag → { flag → float7 damage flash; flag → float7 white-out }
/// flag control → flag dirty → u32 server time, int12 ghost, its packet data
///                else 3 × f32 compression point
/// flag camera → int12 ghost, flag → its packet data
/// flag → int8 field of view
/// ```
pub fn read_server_game(
    r: &mut BitReader<'_>,
    ghosts: &mut Ghosts,
) -> Result<ServerGame19, Game19Error> {
    let mut g = ServerGame19 {
        last_move_ack: r.u32()?,
        ..Default::default()
    };
    if r.flag()? {
        if r.flag()? {
            g.damage_flash = Some(float7(r)?);
        }
        if r.flag()? {
            g.white_out = Some(float7(r)?);
        }
    }
    if r.flag()? {
        if r.flag()? {
            let server_time = r.u32()?;
            let index = r.int(12)?;
            let class = class_of(ghosts, index)?;
            ghosts.control = Some(index);
            g.control = Control19::Object { server_time, index };
            return Err(Game19Error::PacketData {
                index,
                class,
                game: g,
            });
        }
        g.control = Control19::CompressionPoint(read_vec3(r)?);
    }
    if r.flag()? {
        let index = r.int(12)?;
        let class = class_of(ghosts, index)?;
        ghosts.camera = Some(index);
        g.camera = Some(index);
        if r.flag()? {
            return Err(Game19Error::PacketData {
                index,
                class,
                game: g,
            });
        }
    }
    if r.flag()? {
        g.field_of_view = Some(r.u8()?);
    }
    Ok(g)
}

/// The client's game section of a protocol 19 data packet:
///
/// ```text
/// flag first person
/// 3 × f32 control object position, flag → int12 its ghost
/// flag "client sends its own state" → its packet data
///   else int32 first move index, int5 move count, the moves
/// flag → int8 field of view
/// ```
///
/// This client never sends its own state and sends no moves (the layouts
/// of both are not read yet): the first move index is the server's last
/// move ack and the count is 0.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientGame19 {
    pub first_person: bool,
    pub position: Vec3,
    pub control: Option<u32>,
    pub first_move: u32,
    pub field_of_view: Option<u8>,
}

impl ClientGame19 {
    /// Before the server names a control object, as the 5.12 client
    /// starts (first person, no object).
    pub fn before_control() -> Self {
        ClientGame19 {
            first_person: true,
            ..Default::default()
        }
    }

    pub fn write(&self, w: &mut BitWriter) {
        w.flag(self.first_person);
        write_vec3(w, self.position);
        if let Some(i) = self.control {
            w.flag(true);
            w.int(i, 12);
        } else {
            w.flag(false);
        }
        w.flag(false); // not its own state: moves
        w.u32(self.first_move);
        w.int(0, 5); // no moves
        if let Some(f) = self.field_of_view {
            w.flag(true);
            w.u8(f);
        } else {
            w.flag(false);
        }
    }

    /// Reads what [`write`](Self::write) wrote (for a server or a test).
    /// Moves, when a packet carries any, are not read.
    pub fn read(r: &mut BitReader<'_>) -> Result<(Self, usize), OutOfBits> {
        let first_person = r.flag()?;
        let position = read_vec3(r)?;
        let control = if r.flag()? { Some(r.int(12)?) } else { None };
        let own_state = r.flag()?;
        let (first_move, moves) = if own_state {
            (0, 0)
        } else {
            (r.u32()?, r.int(5)? as usize)
        };
        let field_of_view = if moves == 0 && !own_state && r.flag()? {
            Some(r.u8()?)
        } else {
            None
        };
        Ok((
            ClientGame19 {
                first_person,
                position,
                control,
                first_move,
                field_of_view,
            },
            moves,
        ))
    }
}

/// Writes a server game section with no packet data (for a server or a
/// test): the move ack, no flash, the control part as given (an
/// [`Control19::Object`] is written without packet data, so only a
/// reader that stops there can take it), no camera and no field of view.
pub fn write_server_game(w: &mut BitWriter, last_move_ack: u32, control: Control19) {
    w.u32(last_move_ack);
    w.flag(false);
    match control {
        Control19::Unchanged => {
            w.flag(false);
        }
        Control19::Object { server_time, index } => {
            w.flag(true);
            w.flag(true);
            w.u32(server_time);
            w.int(index, 12);
            return;
        }
        Control19::CompressionPoint(p) => {
            w.flag(true);
            w.flag(false);
            write_vec3(w, p);
        }
    }
    w.flag(false);
    w.flag(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torque::net::events::{ClassTable, Events};
    use crate::torque::net::handshake::ClassEntry;

    #[test]
    fn the_client_game_section_carries_no_moves() {
        let g = ClientGame19 {
            first_person: true,
            position: [1.0, 2.0, 3.0],
            control: Some(7),
            first_move: 42,
            field_of_view: Some(90),
        };
        let mut w = BitWriter::new();
        g.write(&mut w);
        // 1 + 96 + 13 + 1 + 32 + 5 + 9 bits.
        assert_eq!(w.bit_len(), 157);
        let bytes = w.into_bytes();
        let (back, moves) = ClientGame19::read(&mut BitReader::new(&bytes)).unwrap();
        assert_eq!((back, moves), (g, 0));

        let mut w = BitWriter::new();
        ClientGame19::before_control().write(&mut w);
        assert_eq!(w.bit_len(), 1 + 96 + 1 + 1 + 32 + 5 + 1);
    }

    #[test]
    fn the_server_game_section_reads_flash_compression_point_and_view() {
        let mut w = BitWriter::new();
        w.u32(9);
        w.flag(true);
        w.flag(true);
        w.int(127, 7);
        w.flag(false);
        w.flag(true);
        w.flag(false);
        write_vec3(&mut w, [4.0, 5.0, 6.0]);
        w.flag(false);
        w.flag(true);
        w.u8(75);
        let bytes = w.into_bytes();
        let mut ghosts = Ghosts::opaque();
        let g = read_server_game(&mut BitReader::new(&bytes), &mut ghosts).unwrap();
        assert_eq!(
            g,
            ServerGame19 {
                last_move_ack: 9,
                damage_flash: Some(1.0),
                white_out: None,
                control: Control19::CompressionPoint([4.0, 5.0, 6.0]),
                camera: None,
                field_of_view: Some(75),
            }
        );

        let mut w = BitWriter::new();
        write_server_game(&mut w, 3, Control19::Unchanged);
        let bytes = w.into_bytes();
        let g = read_server_game(&mut BitReader::new(&bytes), &mut ghosts).unwrap();
        assert_eq!(g.last_move_ack, 3);
        assert_eq!(g.control, Control19::Unchanged);
    }

    #[test]
    fn a_control_object_stops_at_its_packet_data() {
        let mut ghosts = Ghosts::opaque();
        let mut w = BitWriter::new();
        write_server_game(
            &mut w,
            0,
            Control19::Object {
                server_time: 1000,
                index: 2,
            },
        );
        let bytes = w.into_bytes();
        assert_eq!(
            read_server_game(&mut BitReader::new(&bytes), &mut ghosts),
            Err(Game19Error::UnknownGhost(2))
        );
        ghosts
            .ghosts
            .insert(2, GhostState::Other(Some("Player".into())));
        match read_server_game(&mut BitReader::new(&bytes), &mut ghosts) {
            Err(Game19Error::PacketData { index, class, game }) => {
                assert_eq!((index, class.as_deref()), (2, Some("Player")));
                assert_eq!(
                    game.control,
                    Control19::Object {
                        server_time: 1000,
                        index: 2
                    }
                );
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(ghosts.control, Some(2));
    }

    fn cmd(name: &[u8], args: Vec<Arg>) -> Command {
        Command {
            name: name.to_vec(),
            args,
        }
    }

    #[test]
    fn mission_phases_ack_by_build() {
        let c = cmd(
            b"MissionStartPhase1",
            vec![Arg::Int(4), Arg::Text(b"levels/x.mis".to_vec())],
        );
        assert_eq!(
            mission_start(&c, PhaseAcks::Native),
            Some((Phase::Datablocks, b"MissionStartPhase1Ack".to_vec(), vec![]))
        );
        assert_eq!(
            mission_start(&c, PhaseAcks::Scripted),
            Some((
                Phase::Datablocks,
                b"MissionStartPhase1Ack".to_vec(),
                vec![Arg::Int(4)]
            ))
        );
        let c3 = cmd(b"MissionStartPhase3", vec![Arg::Text(b"4".to_vec())]);
        assert_eq!(
            mission_start(&c3, PhaseAcks::Scripted),
            Some((
                Phase::Lighting,
                b"MissionStartPhase3Ack".to_vec(),
                vec![Arg::Text(b"4".to_vec())]
            ))
        );
        assert_eq!(
            mission_start(&cmd(b"ChatMessage", vec![]), PhaseAcks::Native),
            None
        );
        assert!(Phase::Datablocks < Phase::Lighting);
    }

    #[test]
    fn connection_messages_answered_with_the_same_sequence() {
        let done = Event::ConnectionMessage {
            sequence: 5,
            message: message::DATA_BLOCKS_DONE,
            ghost_count: 0,
        };
        assert_eq!(
            connection_answer(&done),
            Some(Event::ConnectionMessage {
                sequence: 5,
                message: message::DATA_BLOCKS_DOWNLOAD_DONE,
                ghost_count: 0
            })
        );
        let ghosts_done = Event::ConnectionMessage {
            sequence: 6,
            message: message::GHOST_ALWAYS_DONE,
            ghost_count: 12,
        };
        assert_eq!(
            connection_answer(&ghosts_done),
            Some(Event::ConnectionMessage {
                sequence: 6,
                message: message::READY_FOR_NORMAL_GHOSTS,
                ghost_count: 0
            })
        );
        let starting = Event::ConnectionMessage {
            sequence: 6,
            message: message::GHOST_ALWAYS_STARTING,
            ghost_count: 12,
        };
        assert_eq!(connection_answer(&starting), None);
    }

    fn table() -> ClassTable {
        let e = |class_type, server_id, name: &str| ClassEntry {
            class_type,
            server_id,
            name: name.as_bytes().to_vec(),
        };
        ClassTable::new(
            &[
                e(0, 0, "Player"),
                e(1, 0, "PlayerData"),
                e(1, 1, "ItemData"),
                e(2, 0, "SimDataBlockEvent"),
                e(2, 1, "RemoteCommandEvent"),
                e(2, 2, "ConnectionMessageEvent"),
            ],
            2,
        )
    }

    #[test]
    fn datablock_events_read_their_header_and_skip_the_fields() {
        let classes = table();
        assert_eq!(classes.datablock_name(1), Some("ItemData"));
        assert_eq!(classes.object_name(0), Some("Player"));
        assert_eq!(classes.event_name(0), Some("SimDataBlockEvent"));
        // A server's Events writes the datablock; the client's reads it.
        let mut server = Events::new();
        server.send(Event::SimDataBlock {
            id: 33,
            class_id: 1,
            index: 0,
            total: 2,
        });
        let mut w = BitWriter::new();
        server.write(&mut w, 1, &classes, 4000);
        let bytes = w.into_bytes();
        let mut client = Events::new();
        let got = client.read(&mut BitReader::new(&bytes), &classes).unwrap();
        assert_eq!(
            got,
            [Event::SimDataBlock {
                id: 33,
                class_id: 1,
                index: 0,
                total: 2
            }]
        );
    }

    #[test]
    fn an_opaque_table_keeps_class_names_and_skips_payloads() {
        let classes = table();
        let mut w = BitWriter::new();
        w.flag(true); // ghost data
        w.int(1, 4); // index width
        w.flag(true);
        w.int(1, 1);
        w.flag(false); // not deleted
        w.int(0, 12); // Player
        w.int(20, 14);
        w.int(0xfffff, 20); // a payload this reader does not know
        w.flag(false);
        let bytes = w.into_bytes();
        let mut ghosts = Ghosts::opaque();
        let changes = ghosts
            .read_ghosts(&mut BitReader::new(&bytes), &classes)
            .unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(
            ghosts.ghosts.get(&1),
            Some(&GhostState::Other(Some("Player".into())))
        );
    }
}
