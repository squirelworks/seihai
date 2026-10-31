//! Walking, jumping, the follow camera and the ground under the player,
//! after Graal_analysis `graal3d/client-behaviour.md`. One call to
//! [`Walker::tick`] is one 50 ms client tick.
//!
//! The real client tests each step against every face of the level mesh;
//! how it decides was not read. Seihai stands the player on the highest
//! face under it that is no more than [`STEP`] above its feet, and refuses
//! a step where there is none (a wall, or off the level).
//!
//! Two seams are kept for the faithful versions (see the crate's
//! `ARCHITECTURE.md`, tasks G3D-1 and G3D-2):
//! - [`Collider`]: what the walker asks of the level. [`Ground`] is
//!   Seihai's stand-in; a port of the real face test implements the same
//!   trait and replaces it in `main.rs` without touching [`Walker`].
//! - [`CameraRig`]: the follow camera's numbers. Distance and turn rate
//!   are the client's; height, look point and field of view are stand-ins.

use crate::props::Player;
use crate::render::V3;
use crate::xfile::XFile;

/// Walking speed per tick, and running after [`RUN_AFTER`] ticks.
pub const WALK: f32 = 8.0;
pub const RUN: f32 = 12.0;
pub const RUN_AFTER: u32 = 25;
/// The world's half width on x and z.
pub const BOUND: f32 = 4000.0;
/// Upward speeds per tick of a jump.
pub const JUMP_UP: [f32; 7] = [10.0, 10.0, 9.0, 8.0, 6.0, 3.0, 1.0];
/// Fall speed once the table is used up.
pub const FALL: f32 = 10.0;
/// The highest ledge a step climbs, and the body height a surface above
/// that blocks (Seihai's choices).
pub const STEP: f32 = 16.0;
pub const BODY: f32 = 50.0;
/// Follow camera distance (the client's).
pub const CAM_DIST: f32 = 220.0;
/// Follow camera turn per tick, in degrees (the client's).
pub const CAM_TURN: f32 = 5.0;
/// Camera height over the player's feet, and the point it looks at.
/// TODO(G3D-2): stand-ins; read the real values from the client's camera
/// routine and replace them here.
pub const CAM_HEIGHT: f32 = 70.0;
pub const LOOK_HEIGHT: f32 = 25.0;

/// The follow camera's numbers in one place, so a faithful set can
/// replace the stand-ins without touching the camera code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraRig {
    pub distance: f32,
    pub turn: f32,
    pub height: f32,
    pub look_height: f32,
    /// Horizontal field of view in degrees. TODO(G3D-2): stand-in.
    pub fov: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        CameraRig { distance: CAM_DIST, turn: CAM_TURN, height: CAM_HEIGHT, look_height: LOOK_HEIGHT, fov: 70.0 }
    }
}

/// What walking asks of the level. TODO(G3D-1): [`Ground`] is a stand-in;
/// a port of the client's per-face test implements this trait.
pub trait Collider {
    /// The highest surface at (x, z) no higher than `max_y`.
    fn height(&self, x: f32, z: f32, max_y: f32) -> Option<f32>;
    /// Whether something solid at (x, z) is in the way of a body whose
    /// feet are below `lo` and whose head is at `hi`.
    fn blocked(&self, x: f32, z: f32, lo: f32, hi: f32) -> bool;
}

impl Collider for Ground {
    fn height(&self, x: f32, z: f32, max_y: f32) -> Option<f32> {
        Ground::height(self, x, z, max_y)
    }

    fn blocked(&self, x: f32, z: f32, lo: f32, hi: f32) -> bool {
        Ground::blocked(self, x, z, lo, hi)
    }
}

/// Upward-facing level triangles, for standing on.
#[derive(Default)]
pub struct Ground {
    tris: Vec<[V3; 3]>,
}

impl Ground {
    pub fn of(level: &XFile) -> Ground {
        let mut tris = Vec::new();
        for m in &level.meshes {
            for (t, _) in m.triangles() {
                let p = t.map(|i| m.vertices[i as usize]);
                // Skip walls: faces whose normal is nearly flat.
                let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
                let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
                let n = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if len > 0.0 && (n[1] / len).abs() > 0.5 {
                    tris.push(p);
                }
            }
        }
        Ground { tris }
    }

    /// The highest surface at (x, z) no higher than `max_y`.
    pub fn height(&self, x: f32, z: f32, max_y: f32) -> Option<f32> {
        self.surfaces(x, z).filter(|&y| y <= max_y + 1e-3).fold(None, |b: Option<f32>, y| Some(b.map_or(y, |b| b.max(y))))
    }

    /// Whether a surface at (x, z) lies between `lo` and `hi` (exclusive
    /// of `lo`): something solid in the way of a body standing at `lo`.
    pub fn blocked(&self, x: f32, z: f32, lo: f32, hi: f32) -> bool {
        self.surfaces(x, z).any(|y| y > lo + 1e-3 && y <= hi)
    }

    fn surfaces(&self, x: f32, z: f32) -> impl Iterator<Item = f32> + '_ {
        self.tris.iter().filter_map(move |t| {
            let (a, b, c) = (t[0], t[1], t[2]);
            let d = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if d.abs() < 1e-6 {
                return None;
            }
            let l0 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / d;
            let l1 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / d;
            let l2 = 1.0 - l0 - l1;
            if l0 < -1e-4 || l1 < -1e-4 || l2 < -1e-4 {
                return None;
            }
            Some(l0 * a[1] + l1 * b[1] + l2 * c[1])
        })
    }
}

/// The keys held this tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Jump {
    /// On the ground (or falling off a ledge when `fall` > 0).
    None,
    /// Climbing: index into [`JUMP_UP`].
    Up(usize),
    /// Coming down: ticks since the top.
    Down(usize),
}

/// The follow camera.
#[derive(Debug, Clone, Copy)]
pub struct FollowCam {
    pub pos: V3,
    /// Where it looks, in degrees about y (0 = +z, 90 = +x).
    pub yaw: f32,
    pub rig: CameraRig,
}

impl FollowCam {
    /// Behind a player facing `angle`.
    pub fn behind(p: &Player) -> FollowCam {
        let (s, c) = (p.angle as f32).to_radians().sin_cos();
        let rig = CameraRig::default();
        FollowCam { pos: [p.x - s * rig.distance, p.y + rig.height, p.z - c * rig.distance], yaw: p.angle as f32, rig }
    }

    pub fn tick(&mut self, p: &Player) {
        let (dx, dz) = (p.x - self.pos[0], p.z - self.pos[2]);
        let d = (dx * dx + dz * dz).sqrt();
        if d > 1e-3 {
            // Close the gap by ((d − 220) / 220)² / 10 of it per tick.
            let gap = d - self.rig.distance;
            let k = ((gap / self.rig.distance).powi(2) / 10.0).min(1.0);
            let step = gap.signum() * gap.abs().max(1.0) * k.max(0.02);
            self.pos[0] += dx / d * step.clamp(-d, d);
            self.pos[2] += dz / d * step.clamp(-d, d);
        }
        self.pos[1] += (p.y + self.rig.height - self.pos[1]) * 0.3;
        let want = dx.atan2(dz).to_degrees();
        let mut turn = (want - self.yaw + 540.0).rem_euclid(360.0) - 180.0;
        let max = if turn.abs() > 90.0 { self.rig.turn * 3.0 } else { self.rig.turn };
        turn = turn.clamp(-max, max);
        self.yaw = (self.yaw + turn).rem_euclid(360.0);
    }

    pub fn target(&self, p: &Player) -> V3 {
        let (s, c) = self.yaw.to_radians().sin_cos();
        let d = self.rig.distance;
        [self.pos[0] + s * d, p.y + self.rig.look_height, self.pos[2] + c * d]
    }
}

/// The local player's movement state.
pub struct Walker {
    walk_ticks: u32,
    jump: Jump,
    jump_held: bool,
    back_dir: Option<f32>,
    fall: f32,
    rng: u32,
}

impl Walker {
    pub fn new(seed: u32) -> Walker {
        Walker { walk_ticks: 0, jump: Jump::None, jump_held: false, back_dir: None, fall: 0.0, rng: seed | 1 }
    }

    fn coin(&mut self) -> bool {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng & 1 == 1
    }

    /// Move `p` one tick. `cam` gives the view direction.
    pub fn tick(&mut self, p: &mut Player, cam: &FollowCam, inp: Input, ground: &dyn Collider) {
        // Facing: the camera-to-player direction.
        let (dx, dz) = (p.x - cam.pos[0], p.z - cam.pos[2]);
        let view = if dx * dx + dz * dz > 1.0 { dx.atan2(dz).to_degrees() } else { p.angle as f32 };
        p.angle = (view.rem_euclid(360.0).round() as u16) % 360;

        let only_back = inp.back && !inp.forward && !inp.left && !inp.right;
        if only_back {
            self.back_dir.get_or_insert(view);
        } else {
            self.back_dir = None;
        }
        let (mut mx, mut mz) = (0.0f32, 0.0f32);
        let (s, c) = view.to_radians().sin_cos();
        if inp.forward {
            mx += s;
            mz += c;
        }
        if inp.back {
            let (s2, c2) = self.back_dir.unwrap_or(view).to_radians().sin_cos();
            mx -= s2;
            mz -= c2;
        }
        if inp.right {
            mx += c;
            mz -= s;
        }
        if inp.left {
            mx -= c;
            mz += s;
        }
        let len = (mx * mx + mz * mz).sqrt();
        if len > 1e-3 {
            if self.walk_ticks == 0 {
                p.anim = if self.coin() { 20 } else { 0 };
            }
            self.walk_ticks += 1;
            let speed = if self.walk_ticks > RUN_AFTER { RUN } else { WALK };
            let (nx, nz) = (p.x + mx / len * speed, p.z + mz / len * speed);
            if nx.abs() <= BOUND && nz.abs() <= BOUND && !ground.blocked(nx, nz, p.y + STEP, p.y + BODY) {
                match ground.height(nx, nz, p.y + STEP) {
                    Some(h) if self.jump == Jump::None && self.fall == 0.0 && h > p.y - STEP => {
                        p.x = nx;
                        p.z = nz;
                        p.y = h;
                    }
                    Some(_) => {
                        p.x = nx;
                        p.z = nz;
                    }
                    None => {}
                }
            }
            if self.jump == Jump::None {
                // One walk cycle is animation time 1 to 40.
                p.anim = p.anim % 40 + 1;
            }
        } else {
            if self.walk_ticks > 0 && self.jump == Jump::None {
                // Back to the standing pose (Seihai's choice; where the
                // real client resets the counter was not traced).
                p.anim = 0;
            }
            self.walk_ticks = 0;
        }

        // Jumping and falling.
        let floor = ground.height(p.x, p.z, p.y + 0.5);
        match self.jump {
            Jump::None => {
                if inp.jump && !self.jump_held && floor.is_some_and(|h| p.y - h < 0.5) {
                    self.jump = Jump::Up(0);
                    p.anim = 40;
                } else if let Some(h) = floor {
                    if p.y > h + 0.5 {
                        self.fall = (self.fall + 2.0).min(FALL);
                        p.y = (p.y - self.fall).max(h);
                    } else {
                        p.y = h;
                        self.fall = 0.0;
                    }
                }
            }
            Jump::Up(i) => {
                if !inp.jump {
                    self.jump = Jump::Down(0);
                } else {
                    p.y += JUMP_UP[i];
                    self.jump = if i + 1 < JUMP_UP.len() { Jump::Up(i + 1) } else { Jump::Down(0) };
                }
            }
            Jump::Down(i) => {
                let v = JUMP_UP.iter().rev().nth(i).copied().unwrap_or(FALL);
                let h = floor.unwrap_or(f32::MIN);
                if p.y - v <= h {
                    p.y = h;
                    self.jump = Jump::None;
                } else {
                    p.y -= v;
                    self.jump = Jump::Down(i + 1);
                }
            }
        }
        self.jump_held = inp.jump;
    }
}

/// The walking bob added to a drawn player's height:
/// `|sin(3.14 × 2 × (anim − 1) / 40)| × 3`, with the program's 3.14.
#[allow(clippy::approx_constant)]
pub fn bob(anim: u16, walking: bool) -> f32 {
    if !walking {
        return 0.0;
    }
    (3.14f32 * 2.0 * (anim as f32 - 1.0) / 40.0).sin().abs() * 3.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xfile::Mesh;

    fn flat_with_step() -> Ground {
        // A big floor at 0, a 10-high step and a 40-high ledge in +z.
        let quad = |x0: f32, z0: f32, x1: f32, z1: f32, y: f32| -> Vec<[f32; 3]> {
            vec![[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]]
        };
        let mut m = Mesh::default();
        for (i, q) in [quad(-1000.0, -1000.0, 1000.0, 1000.0, 0.0), quad(-50.0, 100.0, 50.0, 200.0, 10.0), quad(-50.0, 300.0, 50.0, 400.0, 40.0)]
            .into_iter()
            .enumerate()
        {
            m.vertices.extend(q);
            let b = i as u32 * 4;
            m.faces.push(vec![b, b + 1, b + 2, b + 3]);
        }
        Ground::of(&XFile { meshes: vec![m] })
    }

    #[test]
    fn ground_height() {
        let g = flat_with_step();
        assert_eq!(g.height(0.0, 0.0, 5.0), Some(0.0));
        assert_eq!(g.height(0.0, 150.0, 50.0), Some(10.0));
        assert_eq!(g.height(0.0, 150.0, 5.0), Some(0.0));
        assert_eq!(g.height(5000.0, 0.0, 5.0), None);
    }

    fn walk(p: &mut Player, w: &mut Walker, g: &Ground, inp: Input, n: usize) {
        let cam = FollowCam { pos: [p.x, p.y + CAM_HEIGHT, p.z - CAM_DIST], yaw: 0.0, rig: CameraRig::default() };
        for _ in 0..n {
            w.tick(p, &cam, inp, g);
        }
    }

    #[test]
    fn walks_runs_climbs_and_stops_at_walls() {
        let g = flat_with_step();
        let mut p = Player::default();
        let mut w = Walker::new(7);
        let fwd = Input { forward: true, ..Input::default() };
        walk(&mut p, &mut w, &g, fwd, 1);
        assert_eq!((p.x, p.z, p.angle), (0.0, 8.0, 0));
        walk(&mut p, &mut w, &g, fwd, 24);
        assert_eq!(p.z, 200.0);
        walk(&mut p, &mut w, &g, fwd, 1);
        assert_eq!(p.z, 212.0, "running after 25 ticks");
        // The 10-high step was climbed on the way.
        let mut q = Player { z: 96.0, ..Player::default() };
        walk(&mut q, &mut Walker::new(1), &g, fwd, 1);
        assert_eq!((q.z, q.y), (104.0, 10.0));
        // The 40-high ledge stops the player.
        let mut r = Player { z: 296.0, ..Player::default() };
        walk(&mut r, &mut Walker::new(1), &g, fwd, 3);
        assert_eq!(r.z, 296.0);
    }

    #[test]
    fn jump_follows_the_table() {
        let g = flat_with_step();
        let mut p = Player::default();
        let mut w = Walker::new(3);
        let jump = Input { jump: true, ..Input::default() };
        let mut heights = Vec::new();
        for _ in 0..16 {
            walk(&mut p, &mut w, &g, jump, 1);
            heights.push(p.y);
        }
        assert_eq!(&heights[..8], &[0.0, 10.0, 20.0, 29.0, 37.0, 43.0, 46.0, 47.0]);
        assert_eq!(heights[8], 46.0);
        assert_eq!(*heights.last().unwrap(), 0.0);
        assert_eq!(p.anim, 40);
    }

    #[test]
    fn camera_keeps_its_distance() {
        let p = Player { x: 0.0, z: 0.0, ..Player::default() };
        let mut c = FollowCam { pos: [0.0, 70.0, -600.0], yaw: 0.0, rig: CameraRig::default() };
        for _ in 0..400 {
            c.tick(&p);
        }
        let d = (c.pos[0].powi(2) + c.pos[2].powi(2)).sqrt();
        assert!((d - CAM_DIST).abs() < 15.0, "{d}");
    }
}
