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

#[cfg(test)]
mod tests {
    use super::*;

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
