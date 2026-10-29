//! Pose animations of the player: the sword swing, the hurt pose, sitting
//! (item 22), and the idle, walking and swimming figure.
//!
//! The poses here are *driven by state that already exists* rather than by a
//! clock of their own, so what is drawn can never disagree with what the
//! other players are sent:
//!
//! * **Walk** and **swim** show the frame `offline` has stepped to: one GANI
//!   frame per step taken (every 50 ms on foot, 100 ms swimming), the same
//!   stepping that plays the footstep cues. **Idle** shows `idle.gani`'s first
//!   step when the player is not walking. The sprite value the other players
//!   are sent is not touched.
//! * **Sword** follows the swing of [`crate::combat::CombatState`]: swing
//!   step `i` (the sprite value S = 6 + `i` that goes on the wire) shows step
//!   `i` of `sword.gani`. A GANI `PLAYSOUND` cue after a step is queued as a
//!   sound event when that step is entered, so `sword.wav` plays at the start
//!   of the swing. When the swing ends the pose is gone and the player is
//!   drawn as idle again, which is what the file's `SETBACKTO idle` says.
//! * **Hurt** follows hurt mode. The 2.05 client sets the `hurt` gani when
//!   hurt mode starts (Graal_analysis 01-death-respawn). Seihai steps it one
//!   step per [`GANI_STEP_MS`]: a `LOOP` file repeats for the whole hurt mode,
//!   any other plays once (for at least [`HURT_POSE_MS`]) and gives way. The
//!   sprite others see (S = 32) lasts the whole of hurt mode and is not
//!   changed here.
//! * **Dead** shows `dead.gani` while the player is dead, one step per tick
//!   of the death timer, held on its last step. The 2.05 client plays the
//!   local `dead` gani on death. Seihai's default set has its own: the hurt
//!   pose, kneeling, then lying on the back.
//! * **Push**, **grab** and **pull** follow [`crate::offline::PushPull`]:
//!   the push pose after 17 blocked steps in a row (one step of `push.gani`
//!   per blocked step), the grab pose while A holds a wall, and the pull pose
//!   (one step of `pull.gani` per tick, looping) while the arrow away from
//!   the wall is held. The 2.05 client plays its `push`, `grab` and `pull`
//!   ganis then. Before the push pose (and before a ledge jump) the walk
//!   pose keeps stepping while the arrow is held: 2.05 sets the walk gani
//!   on every walk tick whether the step moved or not, so the player runs in
//!   place against the wall. A ledge jump keeps the walk pose: 2.05 plays no
//!   other gani in the air.
//! * **Sit** is shown while [`OfflineSession::sitting`] is set and the player
//!   is not walking. The session sets it when the feet are on a chair tile
//!   (arrays.dat array 4, tile type 3: benches, chairs, couches). How the
//!   real client starts the sit pose was not read; this trigger is inferred
//!   (see `documentation/sitting.md`).
//!
//! How long the real client shows each step of the hurt and dead ganis was
//! not read (Seihai does not parse per-step waits), so the 50 ms step and the
//! 500 ms minimum are Seihai's own convention.
//!
//! A pose is shown only when its GANI is loaded (`OfflineSession::poses`),
//! so a build without GANIs, or a data folder without `sword.gani`, draws the
//! plain idle figure as before.

use crate::assets::{Gani, GaniSound};
use crate::offline::OfflineSession;
use crate::sound::SoundEvent;

/// How long the hurt pose is shown at the start of hurt mode.
pub const HURT_POSE_MS: u64 = 500;

/// How long one GANI step is shown when the file gives no time of its own
/// (Seihai does not read per-step waits).
pub const GANI_STEP_MS: u64 = 50;

/// The loaded pose animations.
#[derive(Debug, Clone, Default)]
pub struct PoseGanis {
    pub sword: Option<Gani>,
    pub hurt: Option<Gani>,
    pub sit: Option<Gani>,
    pub idle: Option<Gani>,
    /// Standing and walking with an object held over the head.
    pub carrystill: Option<Gani>,
    pub carry: Option<Gani>,
    /// Shown while the player is dead.
    pub dead: Option<Gani>,
    /// Running against a wall, grabbing one, and pulling.
    pub push: Option<Gani>,
    pub grab: Option<Gani>,
    pub pull: Option<Gani>,
}

/// Which pose is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseKind {
    Sword,
    Hurt,
    Sit,
    Idle,
    Walk,
    Swim,
    CarryStill,
    Carry,
    Dead,
    Push,
    Grab,
    Pull,
}

/// A pose step ready to draw.
#[derive(Debug, Clone, Copy)]
pub struct PoseFrame<'a> {
    pub kind: PoseKind,
    pub gani: &'a Gani,
    /// Step index inside `gani` (already limited to its length).
    pub frame: usize,
}

impl PoseKind {
    /// The GANI file of the pose.
    pub fn file(self) -> &'static str {
        match self {
            PoseKind::Sword => "sword.gani",
            PoseKind::Hurt => "hurt.gani",
            PoseKind::Sit => "sit.gani",
            PoseKind::Idle => "idle.gani",
            PoseKind::Walk => "walk.gani",
            PoseKind::Swim => "swim.gani",
            PoseKind::CarryStill => "carrystill.gani",
            PoseKind::Carry => "carry.gani",
            PoseKind::Dead => "dead.gani",
            PoseKind::Push => "push.gani",
            PoseKind::Grab => "grab.gani",
            PoseKind::Pull => "pull.gani",
        }
    }

    /// The pose a file name belongs to (case-insensitive). `walk.gani` and
    /// `swim.gani` are not here: they load as the stepped motion ganis.
    pub fn from_file(name: &str) -> Option<PoseKind> {
        [
            PoseKind::Sword,
            PoseKind::Hurt,
            PoseKind::Sit,
            PoseKind::Idle,
            PoseKind::CarryStill,
            PoseKind::Carry,
            PoseKind::Dead,
            PoseKind::Push,
            PoseKind::Grab,
            PoseKind::Pull,
        ]
            .into_iter()
            .find(|k| k.file().eq_ignore_ascii_case(name))
    }
}

fn pick(kind: PoseKind, gani: &Option<Gani>, i: usize) -> Option<PoseFrame<'_>> {
    gani.as_ref().filter(|g| g.frame_count() > 0).map(|g| PoseFrame { kind, gani: g, frame: i.min(g.frame_count() - 1) })
}

/// Step `i` of a looping pose, wrapped to its length.
fn cycle(kind: PoseKind, gani: &Option<Gani>, i: usize) -> Option<PoseFrame<'_>> {
    gani.as_ref().filter(|g| g.frame_count() > 0).map(|g| PoseFrame { kind, gani: g, frame: i % g.frame_count() })
}

/// The step of `hurt.gani` `ms` into hurt mode: one step per
/// [`GANI_STEP_MS`]. A `LOOP` GANI repeats for the whole hurt mode; any
/// other is shown once (at least [`HURT_POSE_MS`]) and then gives way, as
/// its `SETBACKTO` says.
fn hurt_step(hurt: &Option<Gani>, ms: u64) -> Option<PoseFrame<'_>> {
    let g = hurt.as_ref().filter(|g| g.frame_count() > 0)?;
    let n = g.frame_count() as u64;
    let step = ms / GANI_STEP_MS;
    if g.looped {
        return pick(PoseKind::Hurt, hurt, (step % n) as usize);
    }
    (ms < (n * GANI_STEP_MS).max(HURT_POSE_MS)).then(|| PoseFrame { kind: PoseKind::Hurt, gani: g, frame: step.min(n - 1) as usize })
}

fn pick_player<'a>(kind: PoseKind, p: &'a crate::assets::GaniPlayer) -> Option<PoseFrame<'a>> {
    (p.gani.frame_count() > 0).then(|| PoseFrame { kind, gani: &p.gani, frame: p.frame.min(p.gani.frame_count() - 1) })
}

impl OfflineSession {
    /// Attach a pose animation.
    pub fn set_pose_gani(&mut self, kind: PoseKind, gani: Gani) {
        match kind {
            PoseKind::Sword => self.poses.sword = Some(gani),
            PoseKind::Hurt => self.poses.hurt = Some(gani),
            PoseKind::Sit => self.poses.sit = Some(gani),
            PoseKind::Idle => self.poses.idle = Some(gani),
            PoseKind::CarryStill => self.poses.carrystill = Some(gani),
            PoseKind::Carry => self.poses.carry = Some(gani),
            PoseKind::Dead => self.poses.dead = Some(gani),
            PoseKind::Push => self.poses.push = Some(gani),
            PoseKind::Grab => self.poses.grab = Some(gani),
            PoseKind::Pull => self.poses.pull = Some(gani),
            // Loaded with `set_walk_gani` / `set_swim_gani`.
            PoseKind::Walk | PoseKind::Swim => {}
        }
    }

    /// The player stopped walking (no arrow key is held): back to idle.
    /// True if the pose changed.
    pub fn stop_walking(&mut self) -> bool {
        std::mem::take(&mut self.walking)
    }

    /// The pose to draw now, if any: only for a pose whose GANI is loaded.
    /// Dead: none. A swing wins over hurt, hurt over sitting, sitting over
    /// the motion poses: swimming (the `swim.gani` frame), walking (the
    /// `walk.gani` frame) or idle (`idle.gani`, first step). Swimming has no
    /// sword, hurt or sit pose.
    pub fn current_pose(&self) -> Option<PoseFrame<'_>> {
        // Dead: `dead.gani`, one step per 50 ms tick of the death timer,
        // held on its last step.
        if self.life.is_dead() {
            return pick(PoseKind::Dead, &self.poses.dead, usize::from(self.life.timer()));
        }
        if self.in_water {
            return self.swim_gani.as_ref().and_then(|p| pick_player(PoseKind::Swim, p));
        }
        if let Some(i) = self.combat.swing_frame() {
            if let Some(p) = pick(PoseKind::Sword, &self.poses.sword, i) {
                return Some(p);
            }
        }
        if let Some(p) = self.combat.hurt_elapsed_ms().and_then(|ms| hurt_step(&self.poses.hurt, ms)) {
            return Some(p);
        }
        // Holding a wall with A: the grab pose, or the pull while the arrow
        // away from it is held. Without those ganis the player stands.
        if let Some(g) = self.pushpull.grab {
            let p = if g.pulling() {
                cycle(PoseKind::Pull, &self.poses.pull, self.pushpull.pull_ticks as usize)
            } else {
                pick(PoseKind::Grab, &self.poses.grab, 0)
            };
            return p.or_else(|| pick(PoseKind::Idle, &self.poses.idle, 0));
        }
        // Holding an object: the carry poses (the 2.05 client plays its
        // carrystill and carry ganis then). The carry walk shows the step the
        // walk has reached.
        if self.carried.is_some() {
            if self.walking {
                let step = self.walk_gani.as_ref().map_or(0, |p| p.frame);
                if let Some(p) = pick(PoseKind::Carry, &self.poses.carry, step) {
                    return Some(p);
                }
            }
            if let Some(p) = pick(PoseKind::CarryStill, &self.poses.carrystill, 0) {
                return Some(p);
            }
        }
        if self.sitting && !self.walking {
            return pick(PoseKind::Sit, &self.poses.sit, 0);
        }
        if self.push_pose() {
            let step = self.pushpull.blocked_steps as usize;
            if let Some(p) = cycle(PoseKind::Push, &self.poses.push, step) {
                return Some(p);
            }
        }
        if self.walking {
            if let Some(p) = self.walk_gani.as_ref().and_then(|p| pick_player(PoseKind::Walk, p)) {
                return Some(p);
            }
        }
        pick(PoseKind::Idle, &self.poses.idle, 0)
    }

    /// Queue the sound cues of a step of `sword.gani`, once, at its offset
    /// from the player.
    pub(crate) fn queue_sword_cues(&mut self, step: usize) {
        let cues: Vec<GaniSound> = match self.poses.sword.as_ref().and_then(|g| g.frames.get(step)) {
            Some(f) => f.sounds.clone(),
            None => return,
        };
        for c in cues {
            self.sound_events.push(SoundEvent { file: c.file, x: self.player_x + c.x, y: self.player_y + c.y });
        }
    }
}
