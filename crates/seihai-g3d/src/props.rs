//! Graal3D player properties.
//!
//! The same ids travel in both directions and inside UDP. A props payload
//! is a run of `c(prop id)` + value with no separators. See Graal_analysis
//! `graal3d/protocol.md`, "Player properties".

use crate::codec::{c, put_sn, put_str, Reader};

/// One property and its value.
#[derive(Debug, Clone, PartialEq)]
pub enum Prop {
    /// 0: player id, 1–200.
    Id(u8),
    /// 1–3: position, in half-unit steps from −4000.
    X(f32),
    Y(f32),
    Z(f32),
    /// 4: facing in degrees, sent as `degrees / 2`.
    Angle(u16),
    /// 5: the walk / jump tick counter.
    Anim(u16),
    /// 6–9: model and texture file names.
    BodyModel(String),
    HeadModel(String),
    BodyTexture(String),
    HeadTexture(String),
    /// 10, 11, 17: 24-bit RGB.
    ArmColour(u32),
    ShoeColour(u32),
    /// 12: nickname, optionally `nick (guild)`.
    Nick(String),
    /// 13: chat text.
    Chat(String),
    /// 14: level file.
    Level(String),
    /// 15: UDP port, 0 for none.
    UdpPort(u32),
    /// 16: IPv4 address, filled in by the server.
    Address([u8; 4]),
    HandColour(u32),
    /// 50: the player is (1) or is not (0) in your level.
    InLevel(bool),
    /// 51: the player left.
    Left,
}

impl Prop {
    pub fn id(&self) -> u8 {
        match self {
            Prop::Id(_) => 0,
            Prop::X(_) => 1,
            Prop::Y(_) => 2,
            Prop::Z(_) => 3,
            Prop::Angle(_) => 4,
            Prop::Anim(_) => 5,
            Prop::BodyModel(_) => 6,
            Prop::HeadModel(_) => 7,
            Prop::BodyTexture(_) => 8,
            Prop::HeadTexture(_) => 9,
            Prop::ArmColour(_) => 10,
            Prop::ShoeColour(_) => 11,
            Prop::Nick(_) => 12,
            Prop::Chat(_) => 13,
            Prop::Level(_) => 14,
            Prop::UdpPort(_) => 15,
            Prop::Address(_) => 16,
            Prop::HandColour(_) => 17,
            Prop::InLevel(_) => 50,
            Prop::Left => 51,
        }
    }

    /// The value alone, without the id (as in a UDP position datagram).
    pub fn put_value(&self, out: &mut Vec<u8>) {
        match self {
            Prop::Id(v) => out.push(c(*v)),
            Prop::X(v) | Prop::Y(v) | Prop::Z(v) => put_sn(out, pos_to_wire(*v), 2),
            Prop::Angle(d) => out.push(c(((d % 360) / 2) as u8)),
            Prop::Anim(v) => put_sn(out, *v as u32, 2),
            Prop::BodyModel(s)
            | Prop::HeadModel(s)
            | Prop::BodyTexture(s)
            | Prop::HeadTexture(s) => put_str(out, s.as_bytes()),
            Prop::Nick(s) | Prop::Chat(s) => put_str(out, &cut_text(s)),
            Prop::Level(s) => put_str(out, s.as_bytes()),
            Prop::ArmColour(v) | Prop::ShoeColour(v) | Prop::HandColour(v) => {
                out.extend([18, 12, 6, 0].map(|s| ((v >> s) & 63) as u8 + 32))
            }
            Prop::UdpPort(v) => put_sn(out, *v, 3),
            Prop::Address(a) => {
                // The in-memory address word read as a little-endian
                // integer, then 4 bits and 4 × 7 bits.
                let v = u32::from_le_bytes(*a);
                out.push(((v >> 28) & 0x0f) as u8 + 32);
                put_sn(out, v & 0x0fff_ffff, 4);
            }
            Prop::InLevel(b) => out.push(c(*b as u8)),
            Prop::Left => {}
        }
    }

    pub fn put(&self, out: &mut Vec<u8>) {
        out.push(c(self.id()));
        self.put_value(out);
    }
}

/// Position on the wire: `v × 2 + 8000`, clamped to two 7-bit bytes.
pub fn pos_to_wire(v: f32) -> u32 {
    ((v * 2.0).round() as i32 + 8000).clamp(0, 0x3fff) as u32
}

/// Nickname and chat: at most 200 bytes, a longer line cut and given `...`.
pub fn cut_text(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    if b.len() <= 200 {
        return b.to_vec();
    }
    let mut v = b[..197].to_vec();
    v.extend_from_slice(b"...");
    v
}

fn text(b: &[u8]) -> String {
    // Latin-1, as the 2000 client read it.
    b.iter().map(|&x| x as char).collect()
}

fn colour(r: &mut Reader) -> Option<u32> {
    let b = r.bytes(4)?;
    Some(
        b.iter()
            .fold(0u32, |v, &x| (v << 6) | (x.wrapping_sub(32) & 63) as u32),
    )
}

/// Read one prop. `None` at the end, or on an id this table does not have
/// (the rest of the run can't be framed then).
pub fn read(r: &mut Reader) -> Option<Prop> {
    let id = r.c()?;
    Some(match id {
        0 => Prop::Id(r.c()?),
        1 => Prop::X(wire_to_pos(r.sn(2)?)),
        2 => Prop::Y(wire_to_pos(r.sn(2)?)),
        3 => Prop::Z(wire_to_pos(r.sn(2)?)),
        4 => Prop::Angle(r.c()? as u16 * 2),
        5 => Prop::Anim(r.sn(2)? as u16),
        6 => Prop::BodyModel(text(r.str()?)),
        7 => Prop::HeadModel(text(r.str()?)),
        8 => Prop::BodyTexture(text(r.str()?)),
        9 => Prop::HeadTexture(text(r.str()?)),
        10 => Prop::ArmColour(colour(r)?),
        11 => Prop::ShoeColour(colour(r)?),
        12 => Prop::Nick(text(r.str()?)),
        13 => Prop::Chat(text(r.str()?)),
        14 => Prop::Level(text(r.str()?)),
        15 => Prop::UdpPort(r.sn(3)?),
        16 => {
            let hi = (r.c()? & 0x0f) as u32;
            let v = (hi << 28) | r.sn(4)?;
            Prop::Address(v.to_le_bytes())
        }
        17 => Prop::HandColour(colour(r)?),
        50 => Prop::InLevel(r.c()? != 0),
        51 => Prop::Left,
        _ => return None,
    })
}

pub fn wire_to_pos(v: u32) -> f32 {
    (v as f32 - 8000.0) / 2.0
}

/// Every prop of a run, stopping at the first unknown id.
pub fn read_all(payload: &[u8]) -> Vec<Prop> {
    let mut r = Reader::new(payload);
    let mut out = Vec::new();
    while !r.done() {
        match read(&mut r) {
            Some(p) => out.push(p),
            None => break,
        }
    }
    out
}

/// What is known of one player.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub angle: u16,
    pub anim: u16,
    pub body_model: String,
    pub head_model: String,
    pub body_texture: String,
    pub head_texture: String,
    pub arm_colour: u32,
    pub shoe_colour: u32,
    pub hand_colour: u32,
    pub nick: String,
    pub chat: String,
    pub level: String,
    pub udp_port: u32,
    pub address: Option<[u8; 4]>,
    pub in_level: bool,
}

impl Default for Player {
    /// The look from the shipped `graal3dconfig.txt`.
    fn default() -> Self {
        Player {
            id: 0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            angle: 0,
            anim: 0,
            body_model: "body.x".into(),
            head_model: "head.x".into(),
            body_texture: "body.bmp".into(),
            head_texture: "man.bmp".into(),
            arm_colour: 0xFF0000,
            shoe_colour: 0x800000,
            hand_colour: 0xFDCC47,
            nick: String::new(),
            chat: String::new(),
            level: "level1.x".into(),
            udp_port: 0,
            address: None,
            in_level: false,
        }
    }
}

/// The ids a client sends about itself, in send order.
pub const CLIENT_SENT: [u8; 15] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 17];

impl Player {
    pub fn apply(&mut self, p: &Prop) {
        match p.clone() {
            Prop::Id(v) => self.id = v,
            Prop::X(v) => self.x = v,
            Prop::Y(v) => self.y = v,
            Prop::Z(v) => self.z = v,
            Prop::Angle(v) => self.angle = v,
            Prop::Anim(v) => self.anim = v,
            Prop::BodyModel(v) => self.body_model = v,
            Prop::HeadModel(v) => self.head_model = v,
            Prop::BodyTexture(v) => self.body_texture = v,
            Prop::HeadTexture(v) => self.head_texture = v,
            Prop::ArmColour(v) => self.arm_colour = v,
            Prop::ShoeColour(v) => self.shoe_colour = v,
            Prop::Nick(v) => self.nick = v,
            Prop::Chat(v) => self.chat = v,
            Prop::Level(v) => self.level = v,
            Prop::UdpPort(v) => self.udp_port = v,
            Prop::Address(a) => self.address = Some(a),
            Prop::HandColour(v) => self.hand_colour = v,
            Prop::InLevel(b) => self.in_level = b,
            Prop::Left => {}
        }
    }

    /// The prop with this id, as this player holds it.
    pub fn prop(&self, id: u8) -> Option<Prop> {
        Some(match id {
            0 => Prop::Id(self.id),
            1 => Prop::X(self.x),
            2 => Prop::Y(self.y),
            3 => Prop::Z(self.z),
            4 => Prop::Angle(self.angle),
            5 => Prop::Anim(self.anim),
            6 => Prop::BodyModel(self.body_model.clone()),
            7 => Prop::HeadModel(self.head_model.clone()),
            8 => Prop::BodyTexture(self.body_texture.clone()),
            9 => Prop::HeadTexture(self.head_texture.clone()),
            10 => Prop::ArmColour(self.arm_colour),
            11 => Prop::ShoeColour(self.shoe_colour),
            12 => Prop::Nick(self.nick.clone()),
            13 => Prop::Chat(self.chat.clone()),
            14 => Prop::Level(self.level.clone()),
            15 => Prop::UdpPort(self.udp_port),
            16 => Prop::Address(self.address?),
            17 => Prop::HandColour(self.hand_colour),
            _ => return None,
        })
    }

    /// The UDP position datagram body: props 1–5 back to back, no ids.
    pub fn position_values(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(9);
        for id in 1..=5 {
            self.prop(id).unwrap().put_value(&mut v);
        }
        v
    }

    /// Apply a UDP position body (9 bytes; 8 or fewer are ignored).
    pub fn apply_position_values(&mut self, b: &[u8]) -> bool {
        if b.len() <= 8 {
            return false;
        }
        let mut r = Reader::new(b);
        let (Some(x), Some(y), Some(z), Some(a), Some(n)) =
            (r.sn(2), r.sn(2), r.sn(2), r.c(), r.sn(2))
        else {
            return false;
        };
        self.x = wire_to_pos(x);
        self.y = wire_to_pos(y);
        self.z = wire_to_pos(z);
        self.angle = a as u16 * 2;
        self.anim = n as u16;
        true
    }
}

/// The client's per-tick diff: the props in [`CLIENT_SENT`] whose encoding
/// changed since the last send.
pub struct Differ {
    last: Vec<Option<Vec<u8>>>,
}

impl Differ {
    /// The state before the first send. The real client's first props
    /// line (lab capture against the reference server) carries the whole
    /// look, props 6–12, and the UDP port (15), even at their shipped
    /// values; position, chat and hand colour only follow once they differ
    /// from the defaults. So 6–12 and 15 start unset.
    pub fn new() -> Self {
        let base = Player::default();
        let last = (0..=17u8)
            .map(|id| match id {
                6..=12 | 15 => None,
                _ => base.prop(id).map(|p| enc(&p)),
            })
            .collect();
        Differ { last }
    }

    /// Take these props of `p` as already known to the server (it just
    /// sent them in packet 3), so they are not echoed back.
    pub fn mark_sent(&mut self, p: &Player, ids: impl IntoIterator<Item = u8>) {
        for id in ids {
            if let (Some(slot), Some(prop)) = (self.last.get_mut(id as usize), p.prop(id)) {
                *slot = Some(enc(&prop));
            }
        }
    }

    /// The changed props of `p`, encoded as one props run, or `None`.
    pub fn diff(&mut self, p: &Player) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        for id in CLIENT_SENT {
            let prop = p.prop(id).unwrap();
            let e = enc(&prop);
            if self.last[id as usize].as_ref() != Some(&e) {
                out.extend_from_slice(&e);
                self.last[id as usize] = Some(e);
            }
        }
        (!out.is_empty()).then_some(out)
    }
}

impl Default for Differ {
    fn default() -> Self {
        Self::new()
    }
}

fn enc(p: &Prop) -> Vec<u8> {
    let mut v = Vec::new();
    p.put(&mut v);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_props_line_matches_the_capture() {
        // The real client's first line for nickname `Carol` on UDP port
        // 57542, from the lab run against the reference server.
        let p = Player {
            nick: "Carol".into(),
            udp_port: 57542,
            ..Player::default()
        };
        let mut d = Differ::new();
        let line = d.diff(&p).unwrap();
        assert_eq!(
            line,
            b"&&body.x'&head.x((body.bmp)'man.bmp*_P  +@   ,%Carol/#af"
        );
        assert_eq!(d.diff(&p), None);
    }

    #[test]
    fn server_set_props_are_not_echoed() {
        let mut p = Player::default();
        let mut d = Differ::new();
        d.diff(&p);
        p.x = 30.0;
        p.angle = 270;
        d.mark_sent(&p, [1, 4]);
        assert_eq!(d.diff(&p), None);
        p.x = 38.0;
        assert_eq!(d.diff(&p).unwrap(), b"!_,");
    }

    #[test]
    fn movement_and_chat_line() {
        // A captured client line: position, angle, animation and chat.
        let props = read_all(b"!^t\"^`#^\x89$M% #-,hello from A");
        let mut p = Player::default();
        props.iter().for_each(|x| p.apply(x));
        assert_eq!((p.x, p.y, p.z, p.angle, p.anim), (10.0, 0.0, 20.5, 90, 3));
        assert_eq!(p.chat, "hello from A");
        let mut again = Vec::new();
        props.iter().for_each(|x| x.put(&mut again));
        assert_eq!(again, b"!^t\"^`#^\x89$M% #-,hello from A");
    }

    #[test]
    fn full_props_from_the_server() {
        // Packet 2 to the second player: `c(1)`, then player 1's full props.
        let run = b"! !!^`\"^`#^`$\xa7%  &&body.x'&head.x((body.bmp)'man.bmp*_P  +@   ,%Alice- .,startlevel.x/#F\x870 (  \x9f1_<Q'";
        let props = read_all(&run[1..]);
        let mut p = Player::default();
        props.iter().for_each(|x| p.apply(x));
        assert_eq!(p.id, 1);
        assert_eq!(p.nick, "Alice");
        assert_eq!(p.level, "startlevel.x");
        assert_eq!(p.udp_port, 54119);
        assert_eq!(p.address, Some([127, 0, 0, 1]));
        assert_eq!(
            (p.arm_colour, p.shoe_colour, p.hand_colour),
            (0xFF0000, 0x800000, 0xFDCC47)
        );
        assert_eq!(p.angle, 270);
        let mut again = Vec::new();
        props.iter().for_each(|x| x.put(&mut again));
        assert_eq!(&again[..], &run[1..]);
    }

    #[test]
    fn udp_position() {
        let p = Player {
            x: 10.0,
            z: 4.5,
            angle: 90,
            anim: 3,
            ..Player::default()
        };
        let v = p.position_values();
        assert_eq!(v.len(), 9);
        let mut q = Player::default();
        assert!(q.apply_position_values(&v));
        assert_eq!((q.x, q.z, q.angle, q.anim), (10.0, 4.5, 90, 3));
        assert!(!q.apply_position_values(&v[..8]));
    }
}
