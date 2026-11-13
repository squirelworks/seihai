//! The protocol 19 game layer (3.153 Mac, 4.04r2 Windows): the stock
//! Torque game sections, the datablock phase and the three-phase mission
//! start.
//!
//! Built from §10 of the clean-room note "The Torque UDP session, start
//! to first ghost" (Graal_analysis `torque/udp-session.md`),
//! `graal3d/torque-engine-4x.md` and the later static read of the 4.04r2
//! `Graal3DEngine.dll` behind the protocol 19 lab server (connection
//! message numbers, moves, `Camera` packet data). No real 3.153 or
//! 4.04r2 client has been run past the handshake, so what is here is the
//! part those reads fix:
//!
//! - the server's game section, with a `Camera` control object's or
//!   camera's packet data read ([`CameraPacket`]); any other class's
//!   packet data (stock `Player`) has no length prefix and is not read
//!   yet ([`Game19Error::PacketData`]);
//! - the client's game section with moves ([`Move`]): only the walk
//!   axes are filled, the angles and triggers stay clear;
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

/// Connection messages (`ConnectionMessageEvent`) as the 4.04r2 client
/// numbers them: TGE 1.x's five `NetConnection` messages, then
/// `GameConnection`'s datablock pair at 5 and 6. (Torque3D has a sixth
/// `NetConnection` message and numbers the pair 6 and 7.) Static read of
/// the 4.04r2 client, not traced yet.
pub mod message {
    pub const GHOST_ALWAYS_DONE: u32 = 0;
    pub const READY_FOR_NORMAL_GHOSTS: u32 = 1;
    pub const END_GHOSTING: u32 = 2;
    pub const GHOST_ALWAYS_STARTING: u32 = 3;
    pub const SEND_NEXT_DOWNLOAD_REQUEST: u32 = 4;
    /// Server: every datablock of this sequence has been sent.
    pub const DATA_BLOCKS_DONE: u32 = 5;
    /// Client: the datablocks of this sequence are loaded.
    pub const DATA_BLOCKS_DOWNLOAD_DONE: u32 = 6;
}

/// A `Camera`'s packet data, as the control object or camera in the
/// server's game section:
///
/// ```text
/// f32 energy, f32 recharge rate   (ShapeBase)
/// 3 × f32 position, f32 rot.x (pitch), f32 rot.z (yaw)
/// int3 mode: 0 stationary, 1 free rotate, 2 fly, 3 orbit object, 4 orbit point
///   3 and 4: 3 × f32 orbit distances (min, max, current)
///   3: flag, int12 the orbited ghost; 4: a compressed point (not read)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CameraPacket {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub mode: u32,
}

impl CameraPacket {
    /// `None` for a mode this reader cannot get past (orbit point, or a
    /// value above 4).
    pub fn read(r: &mut BitReader<'_>) -> Result<Option<Self>, OutOfBits> {
        r.f32()?;
        r.f32()?;
        let position = read_vec3(r)?;
        let pitch = r.f32()?;
        let yaw = r.f32()?;
        let mode = r.int(3)?;
        match mode {
            0..=2 => {}
            3 => {
                read_vec3(r)?;
                r.flag()?;
                r.int(12)?;
            }
            _ => return Ok(None),
        }
        Ok(Some(CameraPacket {
            position,
            pitch,
            yaw,
            mode,
        }))
    }

    /// Writes it with zero energy and recharge (orbit modes not written).
    pub fn write(&self, w: &mut BitWriter) {
        w.f32(0.0);
        w.f32(0.0);
        write_vec3(w, self.position);
        w.f32(self.pitch);
        w.f32(self.yaw);
        w.int(self.mode.min(2), 3);
    }
}

/// One move, as the 4.04r2 client packs it:
///
/// ```text
/// flag → int16 yaw, flag → int16 pitch, flag → int16 roll
/// int6 x, int6 y, int6 z     (16 is no movement)
/// flag free look, 10 trigger flags
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub yaw: Option<u16>,
    pub pitch: Option<u16>,
    pub roll: Option<u16>,
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub free_look: bool,
    pub triggers: [bool; 10],
}

impl Default for Move {
    fn default() -> Self {
        Move {
            yaw: None,
            pitch: None,
            roll: None,
            x: Move::STILL,
            y: Move::STILL,
            z: Move::STILL,
            free_look: false,
            triggers: [false; 10],
        }
    }
}

impl Move {
    /// An axis value meaning no movement.
    pub const STILL: u8 = 16;

    /// A walk: `forward` and `strafe` from -1 to 1 (keys held), no turn.
    pub fn walk(forward: i8, strafe: i8) -> Self {
        let axis = |v: i8| (i32::from(Move::STILL) + 16 * i32::from(v.signum())) as u8;
        Move {
            x: axis(strafe),
            y: axis(forward),
            ..Move::default()
        }
    }

    pub fn write(&self, w: &mut BitWriter) {
        for a in [self.yaw, self.pitch, self.roll] {
            if let Some(v) = a {
                w.flag(true);
                w.int(v.into(), 16);
            } else {
                w.flag(false);
            }
        }
        for v in [self.x, self.y, self.z] {
            w.int(u32::from(v) & 0x3f, 6);
        }
        w.flag(self.free_look);
        for t in self.triggers {
            w.flag(t);
        }
    }

    pub fn read(r: &mut BitReader<'_>) -> Result<Self, OutOfBits> {
        let mut angle = || -> Result<Option<u16>, OutOfBits> {
            Ok(if r.flag()? {
                Some(r.int(16)? as u16)
            } else {
                None
            })
        };
        let (yaw, pitch, roll) = (angle()?, angle()?, angle()?);
        let (x, y, z) = (r.int(6)? as u8, r.int(6)? as u8, r.int(6)? as u8);
        let free_look = r.flag()?;
        let mut triggers = [false; 10];
        for t in &mut triggers {
            *t = r.flag()?;
        }
        Ok(Move {
            yaw,
            pitch,
            roll,
            x,
            y,
            z,
            free_look,
            triggers,
        })
    }
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
    /// The control object's packet data, when it is a `Camera`.
    pub control_camera: Option<CameraPacket>,
    pub camera: Option<u32>,
    /// The camera's packet data, when sent and the camera is a `Camera`.
    pub camera_data: Option<CameraPacket>,
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
        game: Box<ServerGame19>,
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

/// Reads a ghost's packet data: a `Camera`'s, or the error that stops
/// the packet for any other class.
fn camera_packet(
    r: &mut BitReader<'_>,
    index: u32,
    class: Option<String>,
    g: &ServerGame19,
) -> Result<Option<CameraPacket>, Game19Error> {
    if class.as_deref() == Some("Camera") {
        if let Some(c) = CameraPacket::read(r)? {
            return Ok(Some(c));
        }
    }
    Err(Game19Error::PacketData {
        index,
        class,
        game: Box::new(g.clone()),
    })
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
            g.control_camera = camera_packet(r, index, class, &g)?;
        } else {
            g.control = Control19::CompressionPoint(read_vec3(r)?);
        }
    }
    if r.flag()? {
        let index = r.int(12)?;
        let class = class_of(ghosts, index)?;
        ghosts.camera = Some(index);
        g.camera = Some(index);
        if r.flag()? {
            g.camera_data = camera_packet(r, index, class, &g)?;
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
/// This client never sends its own state (only vehicles do): it sends
/// moves, numbered from `first_move`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientGame19 {
    pub first_person: bool,
    pub position: Vec3,
    pub control: Option<u32>,
    pub first_move: u32,
    /// At most [`MAX_MOVES`] are written.
    pub moves: Vec<Move>,
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
        let moves = &self.moves[..self.moves.len().min(MAX_MOVES)];
        w.int(moves.len() as u32, 5);
        for m in moves {
            m.write(w);
        }
        if let Some(f) = self.field_of_view {
            w.flag(true);
            w.u8(f);
        } else {
            w.flag(false);
        }
    }

    /// Reads a client game section (for a server or a test). `None` when
    /// the client sent its own state, which has no length to skip by.
    pub fn read(r: &mut BitReader<'_>) -> Result<Option<Self>, OutOfBits> {
        let first_person = r.flag()?;
        let position = read_vec3(r)?;
        let control = if r.flag()? { Some(r.int(12)?) } else { None };
        if r.flag()? {
            return Ok(None);
        }
        let first_move = r.u32()?;
        let count = r.int(5)?;
        let moves = (0..count)
            .map(|_| Move::read(r))
            .collect::<Result<Vec<_>, _>>()?;
        let field_of_view = if r.flag()? { Some(r.u8()?) } else { None };
        Ok(Some(ClientGame19 {
            first_person,
            position,
            control,
            first_move,
            moves,
            field_of_view,
        }))
    }
}

/// Writes a server game section (for a server or a test): the move ack,
/// no flash, the control part as given, no camera and no field of view.
/// An [`Control19::Object`] is followed by `data` (a `Camera`'s packet
/// data); with no data the section stops there, as for a class whose
/// packet data this module cannot write.
pub fn write_server_game(
    w: &mut BitWriter,
    last_move_ack: u32,
    control: Control19,
    data: Option<&CameraPacket>,
) {
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
            let Some(d) = data else { return };
            d.write(w);
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
    fn the_client_game_section_carries_moves() {
        let mut g = ClientGame19 {
            first_person: true,
            position: [1.0, 2.0, 3.0],
            control: Some(7),
            first_move: 42,
            moves: Vec::new(),
            field_of_view: Some(90),
        };
        let mut w = BitWriter::new();
        g.write(&mut w);
        // 1 + 96 + 13 + 1 + 32 + 5 + 9 bits.
        assert_eq!(w.bit_len(), 157);
        let bytes = w.into_bytes();
        assert_eq!(
            ClientGame19::read(&mut BitReader::new(&bytes)).unwrap(),
            Some(g.clone())
        );

        // A still move is 3 + 18 + 11 bits; a turn adds 16.
        g.moves = vec![
            Move::walk(1, 0),
            Move {
                yaw: Some(0x1234),
                triggers: [true; 10],
                ..Move::walk(-1, 1)
            },
        ];
        let mut w = BitWriter::new();
        g.write(&mut w);
        assert_eq!(w.bit_len(), 157 + 32 + 48);
        let bytes = w.into_bytes();
        let back = ClientGame19::read(&mut BitReader::new(&bytes))
            .unwrap()
            .unwrap();
        assert_eq!(back, g);
        assert_eq!((back.moves[0].x, back.moves[0].y), (16, 32));
        assert_eq!((back.moves[1].x, back.moves[1].y), (32, 0));

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
                control_camera: None,
                camera: None,
                camera_data: None,
                field_of_view: Some(75),
            }
        );

        let mut w = BitWriter::new();
        write_server_game(&mut w, 3, Control19::Unchanged, None);
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
            None,
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

    #[test]
    fn a_camera_control_object_is_read_and_the_packet_goes_on() {
        let mut ghosts = Ghosts::opaque();
        ghosts
            .ghosts
            .insert(0, GhostState::Other(Some("Camera".into())));
        let cam = CameraPacket {
            position: [1.0, 2.0, 3.0],
            pitch: 0.5,
            yaw: -1.0,
            mode: 2,
        };
        let mut w = BitWriter::new();
        write_server_game(
            &mut w,
            4,
            Control19::Object {
                server_time: 7,
                index: 0,
            },
            Some(&cam),
        );
        w.flag(true); // a marker after the game section
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let g = read_server_game(&mut r, &mut ghosts).unwrap();
        assert_eq!(g.control_camera, Some(cam));
        assert_eq!(g.last_move_ack, 4);
        assert!(r.flag().unwrap());

        // Orbiting an object carries three distances and the object.
        let mut w = BitWriter::new();
        for v in [0.0, 0.0, 1.0, 2.0, 3.0, 0.5, -1.0] {
            w.f32(v);
        }
        w.int(3, 3);
        for v in [1.0, 9.0, 5.0] {
            w.f32(v);
        }
        w.flag(true);
        w.int(12, 12);
        let bytes = w.into_bytes();
        let c = CameraPacket::read(&mut BitReader::new(&bytes)).unwrap();
        assert_eq!(c.map(|c| c.mode), Some(3));
        // Orbiting a point is not read.
        let mut w = BitWriter::new();
        for _ in 0..7 {
            w.f32(0.0);
        }
        w.int(4, 3);
        let bytes = w.into_bytes();
        assert_eq!(CameraPacket::read(&mut BitReader::new(&bytes)), Ok(None));
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
