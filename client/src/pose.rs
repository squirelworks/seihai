//! Pose animations of the player: the sword swing, the hurt pose and sitting
//! (item 22).
//!
//! `idle.gani`, `walk.gani` and `swim.gani` are stepped by walking
//! (`offline`). The three poses here are *driven by state that already
//! exists* rather than by a clock of their own, so what is drawn can never
//! disagree with what the other players are sent:
//!
//! * **Sword** follows the swing of [`crate::combat::CombatState`]: swing
//!   step `i` (the sprite value S = 6 + `i` that goes on the wire) shows step
//!   `i` of `sword.gani`. A GANI `PLAYSOUND` cue after a step is queued as a
//!   sound event when that step is entered, so `sword.wav` plays at the start
//!   of the swing. When the swing ends the pose is gone and the player is
//!   drawn as idle again, which is what the file's `SETBACKTO idle` says.
//! * **Hurt** follows hurt mode: the first [`HURT_POSE_MS`] of the 2 s
//!   (`hurt_ticks`) in which nothing can hurt the player again. The sprite
//!   others see (S = 32) lasts the whole of hurt mode and is not changed here.
//! * **Sit** is shown while [`OfflineSession::sitting`] is set. Nothing sets
//!   it yet (no key is bound; how the real clients start the sit pose is not
//!   documented).
//!
//! What the real clients do for the hurt pose (a file name, how long it is
//! shown) is not documented, so `hurt.gani` and the 500 ms are Seihai's own
//! convention. `documentation/default-assets.md` says so as well.
//!
//! A pose is shown only when its GANI is loaded (`OfflineSession::poses`),
//! so a build without GANIs, or a data folder without `sword.gani`, draws the
//! plain idle figure as before.

use crate::assets::{Gani, GaniSound};
use crate::offline::OfflineSession;
use crate::sound::SoundEvent;

/// How long the hurt pose is shown at the start of hurt mode.
pub const HURT_POSE_MS: u64 = 500;

/// The loaded pose animations.
#[derive(Debug, Clone, Default)]
pub struct PoseGanis {
    pub sword: Option<Gani>,
    pub hurt: Option<Gani>,
    pub sit: Option<Gani>,
}

/// Which pose is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseKind {
    Sword,
    Hurt,
    Sit,
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
        }
    }

    /// The pose a file name belongs to (case-insensitive).
    pub fn from_file(name: &str) -> Option<PoseKind> {
        [PoseKind::Sword, PoseKind::Hurt, PoseKind::Sit].into_iter().find(|k| k.file().eq_ignore_ascii_case(name))
    }
}

fn pick(kind: PoseKind, gani: &Option<Gani>, i: usize) -> Option<PoseFrame<'_>> {
    gani.as_ref().filter(|g| g.frame_count() > 0).map(|g| PoseFrame { kind, gani: g, frame: i.min(g.frame_count() - 1) })
}

impl OfflineSession {
    /// Attach a pose animation.
    pub fn set_pose_gani(&mut self, kind: PoseKind, gani: Gani) {
        match kind {
            PoseKind::Sword => self.poses.sword = Some(gani),
            PoseKind::Hurt => self.poses.hurt = Some(gani),
            PoseKind::Sit => self.poses.sit = Some(gani),
        }
    }

    /// The pose to draw now, if any. Not while swimming or dead, and only
    /// for a pose whose GANI is loaded. A swing wins over hurt, hurt over
    /// sitting.
    pub fn current_pose(&self) -> Option<PoseFrame<'_>> {
        if self.in_water || self.life.is_dead() {
            return None;
        }
        if let Some(i) = self.combat.swing_frame() {
            if let Some(p) = pick(PoseKind::Sword, &self.poses.sword, i) {
                return Some(p);
            }
        }
        if self.combat.hurt_elapsed_ms().is_some_and(|ms| ms < HURT_POSE_MS) {
            if let Some(p) = pick(PoseKind::Hurt, &self.poses.hurt, 0) {
                return Some(p);
            }
        }
        if self.sitting {
            return pick(PoseKind::Sit, &self.poses.sit, 0);
        }
        None
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
