//! Ghosts (the 3D world's objects) and the game section of each data
//! packet, for protocols 21 and 24.
//!
//! The server sends objects as ghosts: an index, a class id and an update
//! of the sections that changed. The first update of an index creates
//! it. The client names its control object (its own player) in the game
//! section and reports that object's position in every packet it sends;
//! these protocols have no moves. Layouts follow §6 and §8 of the
//! clean-room note "The Torque UDP session, start to first ghost".

use super::bits::{BitReader, BitWriter, OutOfBits};
use super::events::ClassTable;
use std::collections::HashMap;

/// Width of a ghost update's bit-length prefix.
const LENGTH_BITS: u32 = 14;

/// The most ghosts the client's table holds.
pub const MAX_GHOSTS: u32 = 4096;

/// A position, axis or colour.
pub type Vec3 = [f32; 3];

/// Position, then rotation as an axis and an angle in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub axis: Vec3,
    pub angle: f32,
}

/// A movement object's data (section 7).
#[derive(Debug, Clone, PartialEq)]
pub enum Movement {
    /// `list`: points of eight 32-bit fields each.
    List(Vec<[u32; 8]>),
    /// `interpolate` / `rigid`: kept as raw bits (3 × 32 + 4 × 8 + 6 × 9 + 1).
    Rigid(Vec<u8>),
}

/// The movement types that create a movement object.
fn moves(movetype: &[u8]) -> bool {
    matches!(movetype, b"interpolate" | b"list" | b"rigid")
}

/// One GameObject update: each section is present only when it changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GameObjectUpdate {
    pub name: Option<Vec<u8>>,
    pub velocity: Option<Vec3>,
    /// `.dts`, `.dif`, `.3ds` or `.ft`; fetched over TCP when missing.
    pub shape: Option<Vec<u8>>,
    pub skin: Option<Vec<u8>>,
    pub transform: Option<Transform>,
    pub movetype: Option<Vec<u8>>,
    pub movement: Option<Movement>,
    /// The shape's own data (animation threads), kept raw with its bit length.
    pub shape_data: Option<(u32, Vec<u8>)>,
    pub chat: Option<Vec<u8>>,
    pub script_classes: Option<Vec<u8>>,
    /// `attr[1]` to `attr[8]`, each when it changed.
    pub attributes: [Option<Vec<u8>>; 8],
    /// Nickname and its colour (R, G, B, A).
    pub nickname: Option<(Vec<u8>, [u8; 4])>,
    pub flags: Option<u16>,
}

fn read_vec3(r: &mut BitReader<'_>) -> Result<Vec3, OutOfBits> {
    Ok([r.f32()?, r.f32()?, r.f32()?])
}

fn write_vec3(w: &mut BitWriter, v: Vec3) {
    for x in v {
        w.f32(x);
    }
}

/// Reads `n` bits into bytes, low bit first.
fn read_bits(r: &mut BitReader<'_>, n: u32) -> Result<Vec<u8>, OutOfBits> {
    let mut out = vec![0u8; n.div_ceil(8) as usize];
    for i in 0..n as usize {
        if r.flag()? {
            out[i / 8] |= 1 << (i % 8);
        }
    }
    Ok(out)
}

fn write_bits(w: &mut BitWriter, bytes: &[u8], n: u32) {
    for i in 0..n as usize {
        w.flag(bytes[i / 8] >> (i % 8) & 1 == 1);
    }
}

const RIGID_BITS: u32 = 3 * 32 + 4 * 8 + 6 * 9 + 1;

impl GameObjectUpdate {
    /// Reads the thirteen sections. `has_movement` says whether the
    /// object already has a movement object, for section 7; a movetype in
    /// this update overrides it.
    pub fn read(
        r: &mut BitReader<'_>,
        mut has_movement: Option<Vec<u8>>,
    ) -> Result<Self, OutOfBits> {
        let mut u = GameObjectUpdate::default();
        if r.flag()? {
            u.name = Some(r.pstring()?);
        }
        if r.flag()? {
            u.velocity = Some(read_vec3(r)?);
        }
        if r.flag()? {
            u.shape = Some(r.pstring()?);
        }
        if r.flag()? {
            u.skin = Some(r.pstring()?);
        }
        if r.flag()? {
            u.transform = Some(Transform {
                position: read_vec3(r)?,
                axis: read_vec3(r)?,
                angle: r.f32()?,
            });
        }
        if r.flag()? {
            let m = r.pstring()?;
            has_movement = Some(m.clone());
            u.movetype = Some(m);
        }
        if r.flag()? {
            match has_movement.as_deref() {
                Some(b"list") => {
                    let mut points = Vec::new();
                    while r.flag()? {
                        let mut p = [0u32; 8];
                        for x in &mut p {
                            *x = r.u32()?;
                        }
                        points.push(p);
                    }
                    u.movement = Some(Movement::List(points));
                }
                Some(m) if moves(m) => {
                    u.movement = Some(Movement::Rigid(read_bits(r, RIGID_BITS)?))
                }
                _ => {}
            }
        }
        if r.flag()? {
            let n = r.int(LENGTH_BITS)?;
            u.shape_data = Some((n, read_bits(r, n)?));
        }
        if r.flag()? {
            u.chat = Some(r.pstring()?);
        }
        if r.flag()? {
            u.script_classes = Some(r.pstring()?);
        }
        if r.flag()? {
            for a in &mut u.attributes {
                if r.flag()? {
                    *a = Some(r.pstring()?);
                }
            }
        }
        if r.flag()? {
            let nick = r.pstring()?;
            u.nickname = Some((nick, [r.u8()?, r.u8()?, r.u8()?, r.u8()?]));
        }
        if r.flag()? {
            u.flags = Some(r.int(16)? as u16);
        }
        Ok(u)
    }

    /// Writes the thirteen sections (strings never share a prefix).
    pub fn write(&self, w: &mut BitWriter) {
        if w.flag(self.name.is_some()) {
            w.pstring_alone(self.name.as_deref().unwrap_or_default());
        }
        if w.flag(self.velocity.is_some()) {
            write_vec3(w, self.velocity.unwrap_or_default());
        }
        for s in [&self.shape, &self.skin] {
            if w.flag(s.is_some()) {
                w.pstring_alone(s.as_deref().unwrap_or_default());
            }
        }
        if let Some(t) = self.transform {
            w.flag(true);
            write_vec3(w, t.position);
            write_vec3(w, t.axis);
            w.f32(t.angle);
        } else {
            w.flag(false);
        }
        if w.flag(self.movetype.is_some()) {
            w.pstring_alone(self.movetype.as_deref().unwrap_or_default());
        }
        match &self.movement {
            Some(Movement::List(points)) => {
                w.flag(true);
                for p in points {
                    w.flag(true);
                    for x in p {
                        w.u32(*x);
                    }
                }
                w.flag(false);
            }
            Some(Movement::Rigid(bits)) => {
                w.flag(true);
                write_bits(w, bits, RIGID_BITS);
            }
            None => {
                w.flag(false);
            }
        }
        if let Some((n, bits)) = &self.shape_data {
            w.flag(true);
            w.int(*n, LENGTH_BITS);
            write_bits(w, bits, *n);
        } else {
            w.flag(false);
        }
        for s in [&self.chat, &self.script_classes] {
            if w.flag(s.is_some()) {
                w.pstring_alone(s.as_deref().unwrap_or_default());
            }
        }
        if w.flag(self.attributes.iter().any(Option::is_some)) {
            for a in &self.attributes {
                if w.flag(a.is_some()) {
                    w.pstring_alone(a.as_deref().unwrap_or_default());
                }
            }
        }
        if let Some((nick, c)) = &self.nickname {
            w.flag(true);
            w.pstring_alone(nick);
            for x in c {
                w.u8(*x);
            }
        } else {
            w.flag(false);
        }
        if let Some(f) = self.flags {
            w.flag(true);
            w.int(f.into(), 16);
        } else {
            w.flag(false);
        }
    }
}

/// A ghost's state on the client: its class and every section so far.
#[derive(Debug, Clone, PartialEq)]
pub enum GhostState {
    GameObject(GameObjectUpdate),
    Camera(GameObjectUpdate),
    Sun {
        name: Vec<u8>,
        direction: Vec3,
        colour: [f32; 4],
        ambient: [f32; 4],
    },
    Sky(Sky),
    /// A class this client does not read yet, by name.
    Other(Option<String>),
}

impl GameObjectUpdate {
    /// Folds a later update into the state so far.
    pub fn merge(&mut self, u: GameObjectUpdate) {
        macro_rules! take {
            ($($f:ident),*) => { $( if u.$f.is_some() { self.$f = u.$f; } )* };
        }
        take!(
            name,
            velocity,
            shape,
            skin,
            transform,
            movetype,
            movement,
            shape_data,
            chat,
            script_classes,
            nickname,
            flags
        );
        for (a, b) in self.attributes.iter_mut().zip(u.attributes) {
            if b.is_some() {
                *a = b;
            }
        }
    }
}

/// What changed in one ghost section.
#[derive(Debug, Clone, PartialEq)]
pub enum GhostChange {
    Created(u32),
    Updated(u32),
    Deleted(u32),
}

/// Why a ghost or game section could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GhostError {
    Short(OutOfBits),
    /// An update read past its own bit length.
    Overrun {
        index: u32,
    },
    /// The control object or camera names an index the client lacks.
    UnknownGhost(u32),
}

impl From<OutOfBits> for GhostError {
    fn from(e: OutOfBits) -> Self {
        GhostError::Short(e)
    }
}

/// The control object and camera part of a server packet's game section.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerGame {
    /// The server's clock (ms) when the control object was sent.
    pub server_time: Option<u32>,
    pub control: Option<u32>,
    pub camera: Option<u32>,
    pub field_of_view: Option<u8>,
}

/// The client's ghost table and its control object.
#[derive(Debug, Default)]
pub struct Ghosts {
    pub ghosts: HashMap<u32, GhostState>,
    pub control: Option<u32>,
    pub camera: Option<u32>,
}

impl Ghosts {
    pub fn new() -> Self {
        Self::default()
    }

    fn movetype_of(&self, index: u32) -> Option<Vec<u8>> {
        match self.ghosts.get(&index) {
            Some(GhostState::GameObject(g) | GhostState::Camera(g)) => g.movetype.clone(),
            _ => None,
        }
    }

    fn apply_object(&mut self, index: u32, u: GameObjectUpdate) {
        match self.ghosts.get_mut(&index) {
            Some(GhostState::GameObject(g) | GhostState::Camera(g)) => g.merge(u),
            _ => {
                self.ghosts.insert(index, GhostState::GameObject(u));
            }
        }
    }

    /// Reads the server's game section (§6).
    pub fn read_game(&mut self, r: &mut BitReader<'_>) -> Result<ServerGame, GhostError> {
        let mut g = ServerGame::default();
        if r.flag()? && r.flag()? {
            g.server_time = Some(r.u32()?);
            let index = r.int(12)?;
            if !matches!(self.ghosts.get(&index), Some(GhostState::GameObject(_))) {
                return Err(GhostError::UnknownGhost(index));
            }
            let u = GameObjectUpdate::read(r, self.movetype_of(index))?;
            self.apply_object(index, u);
            self.control = Some(index);
            g.control = Some(index);
        }
        if r.flag()? {
            let index = r.int(12)?;
            if !matches!(self.ghosts.get(&index), Some(GhostState::Camera(_))) {
                return Err(GhostError::UnknownGhost(index));
            }
            if r.flag()? {
                let u = GameObjectUpdate::read(r, self.movetype_of(index))?;
                if let Some(GhostState::Camera(c)) = self.ghosts.get_mut(&index) {
                    c.merge(u);
                }
                read_camera_tail(r)?;
            }
            self.camera = Some(index);
            g.camera = Some(index);
        }
        if r.flag()? {
            g.field_of_view = Some(r.u8()?);
        }
        Ok(g)
    }

    /// Reads the ghost section after the events (§8).
    pub fn read_ghosts(
        &mut self,
        r: &mut BitReader<'_>,
        classes: &ClassTable,
    ) -> Result<Vec<GhostChange>, GhostError> {
        let mut changes = Vec::new();
        if !r.flag()? {
            return Ok(changes);
        }
        let width = r.int(4)?;
        while r.flag()? {
            let index = r.int(width)?;
            if r.flag()? {
                self.ghosts.remove(&index);
                if self.control == Some(index) {
                    self.control = None;
                }
                if self.camera == Some(index) {
                    self.camera = None;
                }
                changes.push(GhostChange::Deleted(index));
                continue;
            }
            let class_id = r.int(12)?;
            let len = r.int(LENGTH_BITS)? as usize;
            let end = r.position() + len;
            let created = !self.ghosts.contains_key(&index);
            let name = classes.object_name(class_id);
            match name {
                Some("GameObject") | Some("Camera") => {
                    let u = GameObjectUpdate::read(r, self.movetype_of(index))?;
                    if name == Some("Camera") {
                        read_camera_tail(r)?;
                        match self.ghosts.get_mut(&index) {
                            Some(GhostState::Camera(c)) => c.merge(u),
                            _ => {
                                self.ghosts.insert(index, GhostState::Camera(u));
                            }
                        }
                    } else {
                        self.apply_object(index, u);
                    }
                }
                Some("Sun") => {
                    if r.flag()? {
                        let name = r.pstring()?;
                        let direction = read_vec3(r)?;
                        let colour = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
                        let ambient = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
                        self.ghosts.insert(
                            index,
                            GhostState::Sun {
                                name,
                                direction,
                                colour,
                                ambient,
                            },
                        );
                    }
                }
                Some("Sky") => {
                    let u = read_sky(r)?;
                    match self.ghosts.get_mut(&index) {
                        Some(GhostState::Sky(sky)) => sky.merge(u),
                        _ => {
                            self.ghosts.insert(index, GhostState::Sky(u));
                        }
                    }
                }
                other => {
                    self.ghosts
                        .entry(index)
                        .or_insert_with(|| GhostState::Other(other.map(str::to_owned)));
                }
            }
            if r.position() > end {
                return Err(GhostError::Overrun { index });
            }
            r.seek(end)?;
            changes.push(if created {
                GhostChange::Created(index)
            } else {
                GhostChange::Updated(index)
            });
        }
        Ok(changes)
    }
}

/// What the client keeps of a Sky.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sky {
    pub name: Option<Vec<u8>>,
    pub visible_distance: Option<f32>,
    /// Comma-separated: front, right, back, left, top, bottom. The client
    /// asks for each over TCP without an extension.
    pub textures: Option<Vec<u8>>,
    pub fog_colour: Option<Vec3>,
    pub solid_colour: Option<Vec3>,
    /// Cloud layers: texture, height, speed.
    pub clouds: Option<[(Vec<u8>, f32, f32); 3]>,
    pub wind: Option<Vec3>,
}

impl Sky {
    fn merge(&mut self, u: Sky) {
        macro_rules! take {
            ($($f:ident),*) => { $( if u.$f.is_some() { self.$f = u.$f; } )* };
        }
        take!(
            name,
            visible_distance,
            textures,
            fog_colour,
            solid_colour,
            clouds,
            wind
        );
    }
}

/// Reads a Sky update: every part after the name is guarded, and the
/// init flag guards the whole fog, colour, cloud and wind block.
fn read_sky(r: &mut BitReader<'_>) -> Result<Sky, OutOfBits> {
    let mut sky = Sky::default();
    let init = r.flag()?;
    if init {
        sky.name = Some(r.pstring()?);
    }
    if r.flag()? {
        sky.visible_distance = Some(r.f32()?);
        sky.textures = Some(r.pstring()?);
    }
    if init {
        sky.fog_colour = Some(read_vec3(r)?);
        let volumes = r.u32()?;
        r.u8()?;
        r.u8()?;
        sky.solid_colour = Some(read_vec3(r)?);
        r.u8()?;
        r.flag()?;
        for _ in 0..volumes {
            for _ in 0..6 {
                r.f32()?;
            }
            r.u8()?;
        }
        let mut layer =
            || -> Result<(Vec<u8>, f32, f32), OutOfBits> { Ok((r.pstring()?, r.f32()?, r.f32()?)) };
        sky.clouds = Some([layer()?, layer()?, layer()?]);
        sky.wind = Some(read_vec3(r)?);
        r.u32()?;
        if r.flag()? {
            for _ in 0..5 {
                r.u32()?;
            }
        }
    }
    for n in [8, 8] {
        if r.flag()? {
            r.int(n)?;
        }
    }
    r.flag()?;
    for words in [2, 3, 4, 3] {
        if r.flag()? {
            for _ in 0..words {
                r.u32()?;
            }
        }
    }
    Ok(sky)
}

/// After a Camera's GameObject sections: mode, then a followed object.
fn read_camera_tail(r: &mut BitReader<'_>) -> Result<(), OutOfBits> {
    if r.flag()? {
        r.int(1)?;
    }
    if r.flag()? && r.flag()? {
        r.int(12)?;
    }
    Ok(())
}

/// The client's game section (§6): its view, its position and, once it
/// has a control object, that object's changed sections.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientGame {
    pub first_person: bool,
    pub position: Vec3,
    pub control: Option<u32>,
    pub update: Option<GameObjectUpdate>,
    pub field_of_view: Option<u8>,
}

impl ClientGame {
    /// What the real 5.12 client sends before it has a control object.
    pub fn before_control() -> Self {
        ClientGame {
            first_person: true,
            ..Default::default()
        }
    }

    pub fn write(&self, w: &mut BitWriter) {
        w.flag(self.first_person);
        write_vec3(w, self.position);
        // The ghost index and the "client controls it" flag are two
        // separate flags: both are on the wire even with no object (the
        // real 5.12 writes 100 bits here before it has one).
        if let Some(i) = self.control {
            w.flag(true);
            w.int(i, 12);
        } else {
            w.flag(false);
        }
        match (&self.update, self.control) {
            (Some(u), Some(_)) => {
                w.flag(true);
                u.write(w);
            }
            _ => {
                w.flag(false);
            }
        }
        if let Some(f) = self.field_of_view {
            w.flag(true);
            w.u8(f);
        } else {
            w.flag(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torque::net::handshake::ClassEntry;
    use crate::torque::net::link::{Link, Rate};

    fn classes() -> ClassTable {
        let objects = [
            "Camera",
            "DynamicGrass",
            "DynamicGrassReplicator",
            "DynamicShapeReplicator",
            "GameObject",
            "Sky",
            "Sun",
        ];
        let e: Vec<ClassEntry> = objects
            .iter()
            .enumerate()
            .map(|(i, n)| ClassEntry {
                class_type: 0,
                server_id: i as u32,
                name: n.as_bytes().to_vec(),
            })
            .collect();
        ClassTable::new(&e, 1)
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn player() -> GameObjectUpdate {
        let mut attributes: [Option<Vec<u8>>; 8] = Default::default();
        attributes[1] = Some(b"hat.png".to_vec());
        GameObjectUpdate {
            name: Some(b"G3DPlayer_alice".to_vec()),
            shape: Some(b"body.dts".to_vec()),
            transform: Some(Transform {
                position: [10.0, 20.0, 1.5],
                axis: [0.0, 0.0, 1.0],
                angle: 0.5,
            }),
            movetype: Some(b"list".to_vec()),
            movement: Some(Movement::List(vec![[1, 2, 3, 4, 5, 6, 7, 8]])),
            chat: Some(b"hi".to_vec()),
            attributes,
            nickname: Some((b"Alice".to_vec(), [255, 0, 0, 255])),
            flags: Some(0x18),
            ..Default::default()
        }
    }

    /// A ghost section with one create per (index, class id, payload).
    fn ghost_section(entries: &[(u32, u32, BitWriter)], deletes: &[u32]) -> BitWriter {
        let mut w = BitWriter::new();
        w.flag(true);
        w.int(12, 4);
        for i in deletes {
            w.flag(true);
            w.int(*i, 12);
            w.flag(true);
        }
        for (i, class, p) in entries {
            w.flag(true);
            w.int(*i, 12);
            w.flag(false);
            w.int(*class, 12);
            w.int(p.bit_len() as u32, LENGTH_BITS);
            w.append(p);
        }
        w.flag(false);
        w
    }

    #[test]
    fn a_game_object_round_trips_every_section() {
        let mut w = BitWriter::new();
        player().write(&mut w);
        let bytes = w.into_bytes();
        let got = GameObjectUpdate::read(&mut BitReader::new(&bytes), None).unwrap();
        assert_eq!(got, player());
    }

    #[test]
    fn an_empty_update_is_thirteen_zero_flags() {
        let mut w = BitWriter::new();
        GameObjectUpdate::default().write(&mut w);
        assert_eq!(w.bit_len(), 13);
        assert_eq!(w.into_bytes(), [0, 0]);
    }

    #[test]
    fn ghosts_are_created_updated_and_deleted() {
        let t = classes();
        let mut g = Ghosts::new();
        let mut p = BitWriter::new();
        player().write(&mut p);
        let mut sun = BitWriter::new();
        sun.flag(true);
        sun.pstring_alone(b"TheSun");
        for x in [0.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.3, 0.3, 0.3, 1.0] {
            sun.f32(x);
        }
        let mut odd = BitWriter::new();
        odd.int(0x3ff, 10);
        let w = ghost_section(&[(5, 4, p), (6, 6, sun), (7, 1, odd)], &[]);
        let bytes = w.into_bytes();
        let ch = g.read_ghosts(&mut BitReader::new(&bytes), &t).unwrap();
        assert_eq!(
            ch,
            [
                GhostChange::Created(5),
                GhostChange::Created(6),
                GhostChange::Created(7)
            ]
        );
        assert_eq!(g.ghosts[&5], GhostState::GameObject(player()));
        assert!(
            matches!(&g.ghosts[&6], GhostState::Sun { name, direction: [_, d, _], .. } if name == b"TheSun" && *d == -1.0)
        );
        assert_eq!(g.ghosts[&7], GhostState::Other(Some("DynamicGrass".into())));

        // A chat update merges; the list movement still reads.
        let u = GameObjectUpdate {
            chat: Some(b"bye".to_vec()),
            movement: Some(Movement::List(vec![])),
            ..Default::default()
        };
        let mut p = BitWriter::new();
        u.write(&mut p);
        let bytes = ghost_section(&[(5, 4, p)], &[7]).into_bytes();
        let ch = g.read_ghosts(&mut BitReader::new(&bytes), &t).unwrap();
        assert_eq!(ch, [GhostChange::Deleted(7), GhostChange::Updated(5)]);
        let GhostState::GameObject(o) = &g.ghosts[&5] else {
            panic!()
        };
        assert_eq!(o.chat.as_deref(), Some(&b"bye"[..]));
        assert_eq!(o.nickname.as_ref().unwrap().0, b"Alice");
        assert!(!g.ghosts.contains_key(&7));
    }

    #[test]
    fn the_control_object_must_be_a_known_game_object() {
        let t = classes();
        let mut g = Ghosts::new();
        let mut w = BitWriter::new();
        w.flag(true);
        w.flag(true);
        w.u32(1234);
        w.int(5, 12);
        GameObjectUpdate::default().write(&mut w);
        w.flag(false);
        w.flag(false);
        let game = w.into_bytes();
        assert_eq!(
            g.read_game(&mut BitReader::new(&game)),
            Err(GhostError::UnknownGhost(5))
        );

        let mut p = BitWriter::new();
        player().write(&mut p);
        let bytes = ghost_section(&[(5, 4, p)], &[]).into_bytes();
        g.read_ghosts(&mut BitReader::new(&bytes), &t).unwrap();
        let got = g.read_game(&mut BitReader::new(&game)).unwrap();
        assert_eq!(
            got,
            ServerGame {
                server_time: Some(1234),
                control: Some(5),
                camera: None,
                field_of_view: None
            }
        );
        assert_eq!(g.control, Some(5));
    }

    #[test]
    fn reads_the_servers_game_object_sun_and_sky_payloads() {
        // Payloads written by the Seihai gserver's encoder (research note §8).
        let go = hex("6d887574de1b5979938ddce9c2c81df6bb2f1d431300000000000000000000f00700000000000000000000f00700000000e8c0c8ddffffff7f2400");
        let mut r = BitReader::new(&go);
        let u = GameObjectUpdate::read(&mut r, None).unwrap();
        assert_eq!(r.position(), 471);
        assert_eq!(u.name.as_deref(), Some(&b"G3DPlayer_lab"[..]));
        assert_eq!(u.shape.as_deref(), Some(&b"labcube.3ds"[..]));
        assert_eq!(
            u.transform,
            Some(Transform {
                position: [0.0, 0.0, 1.0],
                axis: [0.0, 0.0, 1.0],
                angle: 0.0
            })
        );
        assert_eq!(u.nickname, Some((b"lab".to_vec(), [255; 4])));
        assert_eq!(u.flags, Some(0x48));

        let t = classes();
        let sun = hex("1d987ab2699ef8b1699ef8b1699ef86d6666fa696666fa696666fa010000fcd1ccccf4d1ccccf4999999f5010000fc01");
        let sky = hex(concat!(
            "1d984e2a0100f4874c69938a6cb368b749456610ed36a9c8769368b7494536b668b74945e6eb68b7494516080000e0473333e3a79999e90700000020",
            "00a09999d9473333e3c7ccccec0700000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000"
        ));
        let mut sun_w = BitWriter::new();
        let mut rr = BitReader::new(&sun);
        for _ in 0..379 {
            sun_w.flag(rr.flag().unwrap());
        }
        let mut sky_w = BitWriter::new();
        let mut rr = BitReader::new(&sky);
        for _ in 0..956 {
            sky_w.flag(rr.flag().unwrap());
        }
        let mut sky_alone = BitReader::new(&sky);
        let s = read_sky(&mut sky_alone).unwrap();
        assert_eq!(
            sky_alone.position(),
            956,
            "the Sky reader takes exactly its payload"
        );
        assert_eq!(s.visible_distance, Some(500.0));
        assert_eq!(
            s.textures.as_deref(),
            Some(&b"sky_fr,sky_rt,sky_bk,sky_lf,sky_up,sky_dn"[..])
        );
        assert_eq!(s.fog_colour, Some([0.5, 0.6, 0.8]));
        assert_eq!(s.solid_colour, Some([0.4, 0.6, 0.9]));

        let mut g = Ghosts::new();
        let bytes = ghost_section(&[(0, 6, sun_w), (1, 5, sky_w)], &[]).into_bytes();
        g.read_ghosts(&mut BitReader::new(&bytes), &t).unwrap();
        let GhostState::Sun { name, ambient, .. } = &g.ghosts[&0] else {
            panic!("a Sun")
        };
        assert_eq!((&name[..], ambient[2]), (&b"Sun"[..], 0.35));
        assert!(
            matches!(&g.ghosts[&1], GhostState::Sky(s) if s.name.as_deref() == Some(&b"Sky"[..]))
        );
    }

    /// All 19 protocol-24 classes in the Seihai gserver's order.
    fn all_classes() -> ClassTable {
        let objects = [
            "Camera",
            "DynamicGrass",
            "DynamicGrassReplicator",
            "DynamicShapeReplicator",
            "GameObject",
            "Sky",
            "Sun",
            "TerrainBlock",
            "WaterBlock",
            "fxSunLight",
        ];
        let events = [
            "BigDataChunkEvent",
            "BigDataStartEvent",
            "ConnectionMessageEvent",
            "GhostAlwaysObjectEvent",
            "NetStringEvent",
            "RemoteCommandEvent",
            "RequestReloadEvent",
            "TerrainChunkEvent",
            "TerrainRequestUpdatesEvent",
        ];
        let mut e: Vec<ClassEntry> = objects
            .iter()
            .enumerate()
            .map(|(i, n)| ClassEntry {
                class_type: 0,
                server_id: i as u32,
                name: n.as_bytes().to_vec(),
            })
            .collect();
        e.extend(events.iter().enumerate().map(|(i, n)| ClassEntry {
            class_type: 1,
            server_id: i as u32,
            name: n.as_bytes().to_vec(),
        }));
        ClassTable::new(&e, 1)
    }

    #[test]
    fn plays_a_whole_server_session_to_the_control_object() {
        use crate::torque::net::events::{mission_start_answer, Events};
        // The five server packets from the research note (Seihai gserver
        // encoder), read the way the client reads every data packet:
        // header and rates, game section, events, ghosts.
        let server_packets = [
            concat!(
                "05084002080108000d004ab8b835569a421aeccdd95ebc001880203062007b4107a69e6c9a277e6c9a277e6c9a277e9b99997e9a99997e9a99997e00",
                "00007f3433337d3433337d6666667d0000007f4600d7411b621d9df74656de642377ba307287fdee4bc7d00400000000000000000000fc0100000000",
                "000000000000fc01000000003a3072f7ffffff1f0900"
            ),
            "09104006080908400d084ab8b835569a421aeccdd97eff0260008201",
            "0d18400e989300002000000000",
            "1120401e0800",
            "1528403e0800",
        ];
        let t = all_classes();
        let mut link = Link::new(0x400);
        let (mut events, mut ghosts) = (Events::new(), Ghosts::new());
        let mut w = link.data(None, Some(Rate::CLIENT_512_MAX)).unwrap();
        ClientGame::before_control().write(&mut w);
        events.write(&mut w, link.last_sent(), &t, 450 * 8);
        let mut answered = Vec::new();
        let mut controls = Vec::new();
        for (i, hexed) in server_packets.iter().enumerate() {
            let bytes = hex(hexed);
            let got = link.receive(&bytes).unwrap();
            for n in &got.notifies {
                assert!(n.delivered, "packet {}: our {} lost", i + 1, n.sequence);
                events.notify(*n);
            }
            let mut data = got.data.unwrap();
            assert_eq!(data.sequence, i as u32 + 1);
            let game = ghosts.read_game(&mut data.body).unwrap();
            controls.push(game.control);
            for ev in events.read(&mut data.body, &t).unwrap() {
                if let Some(answer) = events
                    .command(&ev)
                    .and_then(|c| mission_start_answer(&c.name))
                {
                    answered.push(answer.to_vec());
                    events.send_command(answer, &[]);
                }
            }
            ghosts.read_ghosts(&mut data.body, &t).unwrap();
            assert!(
                data.body.remaining() < 8,
                "packet {} read to its end",
                i + 1
            );
            // Our reply, so the server's acks have something to cover.
            let mut w = link.data(None, None).unwrap();
            ClientGame::before_control().write(&mut w);
            events.write(&mut w, link.last_sent(), &t, 450 * 8);
        }
        assert_eq!(
            answered,
            [
                b"MissionStartPhase1Ack".to_vec(),
                b"MissionStartPhase2Ack".to_vec()
            ]
        );
        assert_eq!(controls, [None, None, Some(1), None, None]);
        assert_eq!(ghosts.control, Some(1));
        assert!(matches!(&ghosts.ghosts[&0], GhostState::Sun { .. }));
        let GhostState::GameObject(me) = &ghosts.ghosts[&1] else {
            panic!("ghost 1 is a GameObject")
        };
        assert_eq!(me.name.as_deref(), Some(&b"G3DPlayer_lab"[..]));
    }

    #[test]
    fn the_client_game_section_before_control_matches_the_512_client() {
        // The captured second data packet: header, two clear rate flags,
        // then first person 1, a zero position, no object, no fov.
        let mut link = Link::new(0x400);
        link.data(None, Some(Rate::CLIENT_512_MAX));
        let mut w = link.data(None, None).unwrap();
        ClientGame::before_control().write(&mut w);
        let real = hex("0900000800000000000000000000000000");
        let n = w.bit_len();
        assert_eq!(n, 25 + 2 + 100, "the real client writes 100 game bits");
        assert_eq!(&w.bytes()[..n / 8], &real[..n / 8]);
    }
}
