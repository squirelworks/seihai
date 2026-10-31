//! The player model: eight meshes, each in its own frame, moved by a
//! keyframe animation whose time is prop 5.
//!
//! The key values below were read from the client binary (the table it
//! builds at start-up before loading the meshes). Times are in animation
//! units: 0 is standing, 1–41 is one walk cycle, 40 is the jump pose.
//! Rotation keys are Direct3D RM quaternions `(s, x, y, z)`, applied in
//! Direct3D's row-vector convention. The model faces −z.
//!
//! Not read from the binary: the frame tree. The hand meshes are in their
//! arm's coordinates (they hang off the arm's end), so each hand is drawn
//! in its arm's frame; every other part is a child of the player's frame.
//! Keys are interpolated linearly (positions) and by slerp (rotations).

use crate::render::{compose, Mat, IDENTITY, V3};

/// Which colour prop tints a part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tint {
    /// Body and head keep their materials (props 8 and 9 swap textures).
    None,
    Arm,
    Hand,
    Shoe,
}

pub struct Part {
    /// The mesh file; for body and head, the default (props 6 and 7
    /// replace it).
    pub mesh: &'static str,
    pub parent: Option<usize>,
    pub tint: Tint,
    /// `(time, x, y, z)`.
    pub pos: &'static [[f32; 4]],
    /// `(time, s, x, y, z)`.
    pub rot: &'static [[f32; 5]],
}

pub const HEAD: usize = 0;
pub const BODY: usize = 1;

const ARM_R_POS: [[f32; 4]; 6] = [
    [0.0, -6.8, 30.5, -0.6],
    [1.0, -6.1, 30.0, -2.5],
    [11.0, -6.3, 30.0, -2.6],
    [21.0, -6.0, 30.0, -2.0],
    [31.0, -6.3, 30.0, -2.6],
    [41.0, -6.1, 30.0, -2.5],
];
const ARM_L_POS: [[f32; 4]; 6] = [
    [0.0, 6.8, 30.5, -0.6],
    [1.0, 6.1, 30.0, -2.5],
    [11.0, 6.3, 30.0, -2.6],
    [21.0, 6.0, 30.0, -2.0],
    [31.0, 6.3, 30.0, -2.6],
    [41.0, 6.1, 30.0, -2.5],
];
const ARM_R_ROT: [[f32; 5]; 6] = [
    [0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.8, 0.0, 0.5916, 0.1],
    [11.0, 1.0, 0.0, 0.0, 0.0],
    [21.0, 0.9, 0.0, -0.4358, 0.0],
    [31.0, 1.0, 0.0, 0.0, 0.0],
    [41.0, 0.8, 0.0, 0.5916, 0.1],
];
const ARM_L_ROT: [[f32; 5]; 6] = [
    [0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.9, 0.0, 0.4358, 0.0],
    [11.0, 1.0, 0.0, 0.0, 0.0],
    [21.0, 0.8, 0.0, -0.5916, 0.1],
    [31.0, 1.0, 0.0, 0.0, 0.0],
    [41.0, 0.9, 0.0, 0.4358, 0.0],
];
// The binary's keys at 11 and 31 are all-zero quaternions; drawn as no
// turn, which is what the arms do at those times.
const FOOT_R_ROT: [[f32; 5]; 6] = [
    [0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.9, 0.4358, 0.0, 0.0],
    [11.0, 0.0, 0.0, 0.0, 0.0],
    [21.0, 0.9, -0.4358, 0.0, 0.0],
    [31.0, 0.0, 0.0, 0.0, 0.0],
    [41.0, 0.9, 0.4358, 0.0, 0.0],
];
const FOOT_L_ROT: [[f32; 5]; 6] = [
    [0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.9, -0.4358, 0.0, 0.0],
    [11.0, 0.0, 0.0, 0.0, 0.0],
    [21.0, 0.9, 0.4358, 0.0, 0.0],
    [31.0, 0.0, 0.0, 0.0, 0.0],
    [41.0, 0.9, -0.4358, 0.0, 0.0],
];

pub const PARTS: [Part; 8] = [
    Part { mesh: "head.x", parent: None, tint: Tint::None, pos: &[[0.0, 0.0, 34.0, 3.5], [1.0, 0.0, 33.0, 0.5], [40.0, 0.0, 33.0, 0.5]], rot: &[] },
    Part {
        mesh: "body.x",
        parent: None,
        tint: Tint::None,
        pos: &[[0.0, 0.0, 4.0, 4.0], [1.0, 0.0, 4.0, 6.0], [40.0, 0.0, 4.0, 6.0]],
        rot: &[[0.0, 1.0, 0.0, 0.0, 0.0], [1.0, 0.9, 0.1, 0.0, 0.0], [40.0, 0.9, 0.1, 0.0, 0.0]],
    },
    Part { mesh: "armright.x", parent: None, tint: Tint::Arm, pos: &ARM_R_POS, rot: &ARM_R_ROT },
    Part { mesh: "armleft.x", parent: None, tint: Tint::Arm, pos: &ARM_L_POS, rot: &ARM_L_ROT },
    Part { mesh: "handright.x", parent: Some(2), tint: Tint::Hand, pos: &[], rot: &[] },
    Part { mesh: "handleft.x", parent: Some(3), tint: Tint::Hand, pos: &[], rot: &[] },
    Part { mesh: "footright.x", parent: None, tint: Tint::Shoe, pos: &[[0.0, -6.0, 9.0, 0.0]], rot: &FOOT_R_ROT },
    Part { mesh: "footleft.x", parent: None, tint: Tint::Shoe, pos: &[[0.0, 6.0, 9.0, 0.0]], rot: &FOOT_L_ROT },
];

/// The keys around `t` and how far between them, clamped at the ends.
fn between<const N: usize>(keys: &[[f32; N]], t: f32) -> Option<(&[f32; N], &[f32; N], f32)> {
    let first = keys.first()?;
    if t <= first[0] {
        return Some((first, first, 0.0));
    }
    for w in keys.windows(2) {
        if t <= w[1][0] {
            let span = w[1][0] - w[0][0];
            return Some((&w[0], &w[1], if span > 0.0 { (t - w[0][0]) / span } else { 0.0 }));
        }
    }
    let last = keys.last()?;
    Some((last, last, 0.0))
}

fn quat(k: &[f32; 5]) -> [f32; 4] {
    let q = [k[1], k[2], k[3], k[4]];
    let l = q.iter().map(|v| v * v).sum::<f32>().sqrt();
    if l < 1e-6 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        q.map(|v| v / l)
    }
}

fn slerp(a: [f32; 4], mut b: [f32; 4], t: f32) -> [f32; 4] {
    let mut d: f32 = (0..4).map(|i| a[i] * b[i]).sum();
    if d < 0.0 {
        b = b.map(|v| -v);
        d = -d;
    }
    let (wa, wb) = if d > 0.9995 {
        (1.0 - t, t)
    } else {
        let th = d.acos();
        let s = th.sin();
        (((1.0 - t) * th).sin() / s, (t * th).sin() / s)
    };
    let q = [0, 1, 2, 3].map(|i| a[i] * wa + b[i] * wb);
    let l = q.iter().map(|v| v * v).sum::<f32>().sqrt();
    q.map(|v| v / l)
}

/// Rotation of a unit quaternion `(s, x, y, z)` as Direct3D applies it to
/// row vectors: the matrix is the transpose of the column-vector one.
fn rot_matrix(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [w, x, y, z] = q;
    let m = [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
        [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
        [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
    ];
    [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]]
}

/// A part's own frame at time `t`, relative to its parent.
pub fn local(p: &Part, t: f32) -> Mat {
    let pos: V3 = match between(p.pos, t) {
        Some((a, b, k)) => [1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * k),
        None => [0.0; 3],
    };
    let r = match between(p.rot, t) {
        Some((a, b, k)) => rot_matrix(slerp(quat(a), quat(b), k)),
        None => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    [
        [r[0][0], r[0][1], r[0][2], pos[0]],
        [r[1][0], r[1][1], r[1][2], pos[1]],
        [r[2][0], r[2][1], r[2][2], pos[2]],
    ]
}

/// Every part's frame at time `t`, under `player` (the player's frame).
pub fn pose(player: &Mat, t: f32) -> [Mat; 8] {
    let mut out = [IDENTITY; 8];
    for (i, p) in PARTS.iter().enumerate() {
        let parent = p.parent.map_or(*player, |k| out[k]);
        out[i] = compose(&parent, &local(p, t));
    }
    out
}

/// The animation time for prop 5.
pub fn time(anim: u16) -> f32 {
    anim as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::transform;

    #[test]
    fn standing_pose() {
        let m = pose(&IDENTITY, 0.0);
        assert_eq!(transform(&m[HEAD], [0.0; 3]), [0.0, 34.0, 3.5]);
        assert_eq!(transform(&m[BODY], [0.0; 3]), [0.0, 4.0, 4.0]);
        // A hand sits in its arm's frame.
        assert_eq!(transform(&m[4], [0.0; 3]), [-6.8, 30.5, -0.6]);
    }

    #[test]
    fn walking_leans_forward_and_swings() {
        // The model faces −z: walking tips the body's top that way.
        let m = pose(&IDENTITY, 1.0);
        let top = transform(&m[BODY], [0.0, 25.0, 0.0]);
        assert!(top[2] < 6.0, "{top:?}");
        // Halfway between keys 1 and 11 the arm is between them.
        let a = transform(&local(&PARTS[2], 6.0), [0.0; 3]);
        assert!((a[0] - -6.2).abs() < 1e-4 && (a[2] - -2.55).abs() < 1e-4, "{a:?}");
        // The feet swing opposite ways.
        let r = transform(&pose(&IDENTITY, 1.0)[6], [0.0, -8.0, 0.0]);
        let l = transform(&pose(&IDENTITY, 1.0)[7], [0.0, -8.0, 0.0]);
        assert!((r[2] + l[2]).abs() < 1e-3 && r[2].abs() > 1.0, "{r:?} {l:?}");
    }
}
