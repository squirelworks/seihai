//! Selected player property encodings.

/// Position scaled the classic way (pos * 2 as u8).
pub fn encode_pos_classic(pos: f32) -> u8 {
    (pos * 2.0) as u8
}

/// Higher-precision position (PLPROP_X2/Y2/Z2).
/// `abs(pos * 16) << 1`, low bit = sign.
pub fn encode_pos_precise(pos: f32) -> u16 {
    let mut val = ((pos.abs() * 16.0) as u16) << 1;
    if pos < 0.0 {
        val |= 1;
    }
    val
}

pub fn decode_pos_precise(val: u16) -> f32 {
    let sign = if val & 1 != 0 { -1.0 } else { 1.0 };
    sign * ((val >> 1) as f32) / 16.0
}

/// One decoded player property from a prop stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerProp {
    /// Property id (`PLPROP_*`).
    pub id: u8,
    /// Raw value bytes as they appear on the wire (after the id byte).
    pub raw: Vec<u8>,
}

impl PlayerProp {
    /// First value byte as a GCHAR (wire byte - 0x20).
    pub fn gchar(&self) -> Option<u8> {
        self.raw.first().map(|b| b.wrapping_sub(0x20))
    }
}

/// Byte length of a GSTRING (GCHAR length + bytes) at `data[0..]`.
fn gstring_len(data: &[u8]) -> Option<usize> {
    let n = data.first()?.wrapping_sub(0x20) as usize;
    Some(1 + n)
}

/// Byte length of one player prop value for `id`, or `None` if the id
/// is unknown or the value runs past the end.
///
/// Encodings follow the cleaned PLPROP table (2.05 forms where they
/// differ from later clients).
fn prop_len(id: u8, data: &[u8]) -> Option<usize> {
    let c = |i: usize| data.get(i).map(|b| b.wrapping_sub(0x20) as usize);
    let len = match id {
        // str
        0 | 12 | 20 | 21 | 34 | 35 | 37..=41 | 46..=49 | 52 | 54..=75 | 82 => gstring_len(data)?,
        // c
        1 | 2 | 4..=7 | 15..=19 | 22 | 23 | 26 | 32 | 33 | 43..=45 | 50 | 53 | 81 => 1,
        // i3
        3 | 24 | 27..=29 | 31 | 36 | 76 => 3,
        // s
        14 | 25 | 78..=80 => 2,
        // i5
        30 => 5,
        // sword: c(power + 30) + image, or c(power) with no image
        8 => {
            let v = c(0)?;
            if v > 4 {
                1 + gstring_len(data.get(1..)?)?
            } else {
                1
            }
        }
        // shield: c(power + 10) + image, or c(power) with no image
        9 => {
            let v = c(0)?;
            if v > 3 {
                1 + gstring_len(data.get(1..)?)?
            } else {
                1
            }
        }
        // 2.05 bow: c(10 + len) + image, or c(bowpower)
        10 => {
            let v = c(0)?;
            if v >= 10 {
                1 + (v - 10)
            } else {
                1
            }
        }
        // head: c(n) for n < 100, else c(len + 100) + image
        11 => {
            let v = c(0)?;
            if v >= 100 {
                1 + (v - 100)
            } else {
                1
            }
        }
        // 5 color indexes
        13 => 5,
        // attach type + npc id
        42 => 4,
        // pconnected: no payload
        51 => 0,
        _ => return None,
    };
    if len > data.len() {
        return None;
    }
    Some(len)
}

/// Walk a `(GCHAR id, value)*` player prop stream.
///
/// Stops at the first unknown id or truncated value, like the 2.05
/// client does; everything decoded before that point is returned.
pub fn parse_player_props(payload: &[u8]) -> Vec<PlayerProp> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < payload.len() {
        let id = payload[i].wrapping_sub(0x20);
        let rest = &payload[i + 1..];
        let Some(n) = prop_len(id, rest) else { break };
        out.push(PlayerProp {
            id,
            raw: rest[..n].to_vec(),
        });
        i += 1 + n;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prop_stream_position_and_sprite() {
        // NICKNAME "ab", X = 30.5 (61), Y = 12 (24), SPRITE = dir 2 + 4*1, ALIGNMENT 53
        let mut p = vec![0x20, 0x22, b'a', b'b'];
        p.extend_from_slice(&[0x2f, 0x20 + 61, 0x30, 0x20 + 24, 0x31, 0x20 + 6, 0x40, 0x55]);
        let props = parse_player_props(&p);
        let ids: Vec<u8> = props.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![0, 15, 16, 17, 32]);
        assert_eq!(props[1].gchar(), Some(61));
        assert_eq!(props[3].gchar().map(|s| s % 4), Some(2));
    }

    #[test]
    fn prop_stream_stops_at_unknown() {
        // prop 77 has no known encoding: stop, keep what came before.
        let p = [0x2f, 0x20 + 10, 0x20 + 77, 0x21];
        let props = parse_player_props(&p);
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn classic_roundtrip() {
        assert_eq!(encode_pos_classic(30.0), 60);
        assert_eq!(encode_pos_classic(0.5), 1);
    }

    #[test]
    fn precise_roundtrip() {
        let e = encode_pos_precise(30.0);
        assert!((decode_pos_precise(e) - 30.0).abs() < 0.1);
        let n = encode_pos_precise(-5.0);
        assert!(decode_pos_precise(n) < 0.0);
    }
}
