//! Combat, the way the 2.05 client runs it: the victim's client judges.
//!
//! Sources: `decomp-findings-2026-10-24/04-sword-hitbox.md` (who decides a
//! sword hit and the reach rule), `01-death-respawn.md` (hurt state, claim),
//! `constants-205.tsv`, and Graal_analysis `protocol/packet-layouts.md`
//! "Hits" and `02-weapons-combat.md` (the ids and layouts of 26/40 and
//! 36/46). The numbers come from the version profile's [`CombatRules`];
//! nothing here names a version. A build without a row judges no hits and
//! sends none. `documentation/combat.md` lists what is not known.
//!
//! How a sword hit happens: the attacker does not send "I hit you". Its
//! client broadcasts its position, direction and sprite as it swings (the
//! sprite is `dir + 4 * S` with S 6 to 10 while the sword is out), and every
//! other client tests whether *it* is in the box in front of that swing.
//! [`judge_sword`] is that test, run on the receiving side. Packet hits
//! (`PLO_HURTPLAYER` 40, `PLO_HITOBJECTS` 46) carry their own power and are
//! judged by [`judge_hurtplayer`].

use crate::offline::OfflineSession;
use crate::protocol::{ClientVersion, CombatRules, OtherPlayer};

/// `PLI_HURTPLAYER` (26), client to server:
/// `GSHORT victim, GCHAR (dx+4)*16, GCHAR (dy+4)*16, GCHAR power` (half
/// hearts), with an optional `GINT3` NPC id.
pub const PLI_HURTPLAYER: u8 = 26;
/// `PLO_HURTPLAYER` (40): the same with the attacker's id in front.
pub const PLO_HURTPLAYER: u8 = 40;
/// `PLO_HITOBJECTS` (46): `GSHORT attacker, GCHAR power*2, GCHAR x*2,
/// GCHAR y*2` and an optional `GINT3` NPC id.
pub const PLO_HITOBJECTS: u8 = 46;
/// The dead bit of STATUS (prop 18): other players can see it.
pub const STATUS_DEAD: u8 = 0x08;

/// Where a player stands and which way it faces. Tiles; `dir` is 0 up,
/// 1 left, 2 down, 3 right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub dir: u8,
}

/// Why a hit does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reject {
    /// The build has no combat rules, or the packet is not in its id set.
    NotInBuild,
    /// The attacker's sprite is not a sword-swing frame.
    NotSwinging,
    /// The victim is not 0 to 4 tiles ahead of the attacker.
    OutOfReach,
    /// The victim is 2 or more tiles to the side.
    OffLine,
    /// The victim's AP is exactly the immune value (100).
    Immune,
    /// The victim is dead (or its hearts are 0).
    Dead,
    /// The victim is in hurt mode (mode 3, 40 ticks).
    AlreadyHurt,
    /// The victim is swimming (mode 10).
    Swimming,
    /// Packet hit from farther than the profile's range.
    TooFar,
    /// The hit came from an NPC. Its position is not known to Seihai.
    NpcSource,
    /// The attacker's sword power was never sent.
    UnknownPower,
    /// The hit takes nothing (power 0).
    NoDamage,
    /// Id 46 is decoded but not applied: where a player's centre is, for its
    /// "within 2 tiles" test, is not in the notes.
    CentreUnknown,
}

/// Whether the player can be hurt right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vulnerability {
    Hurtable,
    Dead,
    Hurt,
    Swimming,
}

impl Vulnerability {
    fn check(self) -> Result<(), Reject> {
        match self {
            Vulnerability::Hurtable => Ok(()),
            Vulnerability::Dead => Err(Reject::Dead),
            Vulnerability::Hurt => Err(Reject::AlreadyHurt),
            Vulnerability::Swimming => Err(Reject::Swimming),
        }
    }
}

/// The player on the receiving end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Victim {
    pub x: f32,
    pub y: f32,
    /// Alignment points.
    pub ap: u8,
    pub state: Vulnerability,
}

/// A player swinging a sword, as the victim sees it from the props.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swinger {
    pub pose: Pose,
    /// SPRITE (17) as received: `dir + 4 * S`.
    pub sprite: u8,
    /// SWORDPOWER (8) as last received, if ever.
    pub sword_power: Option<u8>,
}

fn half_tiles(h: u8) -> f32 {
    f32::from(h) / 2.0
}

/// `(forward, sideways)` distance of the point `(tx, ty)` from `a`: forward
/// is along the way `a` faces (negative behind it), sideways is the
/// absolute offset across it. Tiles.
pub fn forward_and_side(a: Pose, tx: f32, ty: f32) -> (f32, f32) {
    let (dx, dy) = (tx - a.x, ty - a.y);
    match a.dir % 4 {
        0 => (-dy, dx.abs()),
        1 => (-dx, dy.abs()),
        2 => (dy, dx.abs()),
        _ => (dx, dy.abs()),
    }
}

/// The sword box: 0 to 4 tiles ahead (both ends count), under 2 tiles to the
/// side.
pub fn in_sword_box(rules: &CombatRules, a: Pose, tx: f32, ty: f32) -> Result<(), Reject> {
    let (fwd, side) = forward_and_side(a, tx, ty);
    if fwd < half_tiles(rules.reach_min_half) || fwd > half_tiles(rules.reach_max_half) {
        return Err(Reject::OutOfReach);
    }
    if side >= half_tiles(rules.lateral_below_half) {
        return Err(Reject::OffLine);
    }
    Ok(())
}

/// The sprite number S (animation) in a SPRITE value, and the direction.
pub fn sprite_parts(sprite: u8) -> (u8, u8) {
    (sprite / 4, sprite % 4)
}

/// True when `sprite` is one of the sword-swing frames.
pub fn is_swing_sprite(rules: &CombatRules, sprite: u8) -> bool {
    let s = sprite_parts(sprite).0;
    s >= rules.swing_sprite_first && s <= rules.swing_sprite_last
}

/// Half hearts a sword hit takes: the placeholder rule (see
/// [`CombatRules::sword_half_per_power`]), at least one.
pub fn sword_half_hearts(rules: &CombatRules, power: u8) -> i32 {
    (i32::from(power) * i32::from(rules.sword_half_per_power)).max(1)
}

/// The victim's judgement of a sword swing. `Ok` is the half hearts it
/// takes. The attacker must be on a swing frame, the victim hurtable, its
/// AP not immune, and in the box in front of the attacker.
pub fn judge_sword(rules: &CombatRules, s: &Swinger, v: &Victim) -> Result<i32, Reject> {
    if !is_swing_sprite(rules, s.sprite) {
        return Err(Reject::NotSwinging);
    }
    v.state.check()?;
    if v.ap == rules.immune_ap {
        return Err(Reject::Immune);
    }
    let pose = Pose { dir: sprite_parts(s.sprite).1, ..s.pose };
    in_sword_box(rules, pose, v.x, v.y)?;
    let power = s.sword_power.ok_or(Reject::UnknownPower)?;
    Ok(sword_half_hearts(rules, power))
}

/// A decoded `PLO_HURTPLAYER` (40).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HurtPlayer {
    pub attacker: u16,
    /// Knockback, tiles (-4 to 4).
    pub dx: f32,
    pub dy: f32,
    /// Half hearts, as sent (not yet capped).
    pub half: i32,
    /// The hit came from this NPC, not from the player.
    pub npc: Option<u32>,
}

/// A decoded `PLO_HITOBJECTS` (46).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitObjects {
    pub attacker: u16,
    pub half: i32,
    /// The hit's centre, tiles.
    pub x: f32,
    pub y: f32,
    pub npc: Option<u32>,
}

/// A packet hit waiting for the game loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncomingHit {
    Hurt(HurtPlayer),
    Objects(HitObjects),
}

fn gc(b: u8) -> i32 {
    i32::from(b) - 0x20
}

/// `GSHORT` at the start of `b`.
fn id_of(b: &[u8]) -> Option<(u16, &[u8])> {
    let (hi, lo) = (*b.first()?, *b.get(1)?);
    Some((((gc(hi).clamp(0, 127) as u16) << 7) | gc(lo).clamp(0, 127) as u16, &b[2..]))
}

fn npc_of(rest: &[u8]) -> Option<u32> {
    let t = rest.get(3..6)?;
    let v = (gc(t[0]).clamp(0, 127) << 14) | (gc(t[1]).clamp(0, 127) << 7) | gc(t[2]).clamp(0, 127);
    Some(v as u32)
}

/// Decode `PLO_HURTPLAYER` (40): `GSHORT attacker, c dx, c dy, c power`
/// (+ `GINT3 npc` when 6 or more bytes follow the id). Knockback is
/// `(byte - 0x60) / 16`, power `(byte - 32)` half hearts.
pub fn parse_hurtplayer(payload: &[u8]) -> Option<HurtPlayer> {
    let (attacker, rest) = id_of(payload)?;
    if rest.len() < 3 {
        return None;
    }
    Some(HurtPlayer {
        attacker,
        dx: (i32::from(rest[0]) - 0x60) as f32 / 16.0,
        dy: (i32::from(rest[1]) - 0x60) as f32 / 16.0,
        half: gc(rest[2]).max(0),
        npc: if rest.len() >= 6 { npc_of(rest) } else { None },
    })
}

/// Decode `PLO_HITOBJECTS` (46): `GSHORT attacker, c power*2, c x*2, c y*2`
/// (+ `GINT3 npc`).
pub fn parse_hitobjects(payload: &[u8]) -> Option<HitObjects> {
    let (attacker, rest) = id_of(payload)?;
    if rest.len() < 3 {
        return None;
    }
    Some(HitObjects {
        attacker,
        half: gc(rest[0]).max(0),
        x: gc(rest[1]).max(0) as f32 / 2.0,
        y: gc(rest[2]).max(0) as f32 / 2.0,
        npc: if rest.len() >= 6 { npc_of(rest) } else { None },
    })
}

fn within(a: (f32, f32), b: (f32, f32), half_tiles_range: u8) -> bool {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    let r = half_tiles(half_tiles_range);
    dx * dx + dy * dy <= r * r
}

/// The victim's judgement of a `PLO_HURTPLAYER`. It applies only when the
/// victim can be hurt and the attacker is within the profile's range
/// (10 tiles); the power is capped at one heart. `attacker_at` is where the
/// attacker is, in the victim's coordinates, or `None` when it is not known.
/// A hit from an NPC has no known position and is refused. Unlike the
/// sword, nothing in the notes makes AP 100 immune to this packet, so AP is
/// not checked here.
pub fn judge_hurtplayer(rules: &CombatRules, h: &HurtPlayer, attacker_at: Option<(f32, f32)>, v: &Victim) -> Result<i32, Reject> {
    if h.npc.is_some() {
        return Err(Reject::NpcSource);
    }
    v.state.check()?;
    let at = attacker_at.ok_or(Reject::TooFar)?;
    if !within(at, (v.x, v.y), rules.hurtplayer_range_half) {
        return Err(Reject::TooFar);
    }
    let half = h.half.min(i32::from(rules.packet_hit_cap_half));
    if half <= 0 {
        return Err(Reject::NoDamage);
    }
    Ok(half)
}

/// What the attacker knows about a player it might hit, from that player's
/// props.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    /// STATUS (18) as last received.
    pub status: Option<u8>,
    /// SPRITE (17) as last received.
    pub sprite: Option<u8>,
}

/// The attacker's side of the same rules, for the hits it sends: nothing
/// goes out that the victim would refuse, where the attacker can tell. It
/// can see the dead bit (STATUS 0x08), hurt mode (sprite S = 32) and the
/// distance. It cannot see swimming, so that is left to the victim.
pub fn attacker_may_send(rules: &CombatRules, from: (f32, f32), t: &Target) -> Result<(), Reject> {
    if t.status.is_some_and(|s| s & STATUS_DEAD != 0) {
        return Err(Reject::Dead);
    }
    if t.sprite.is_some_and(|s| sprite_parts(s).0 == rules.hurt_sprite) {
        return Err(Reject::AlreadyHurt);
    }
    if !within(from, (t.x, t.y), rules.hurtplayer_range_half) {
        return Err(Reject::TooFar);
    }
    Ok(())
}

/// The `PLI_HURTPLAYER` (26) message for `victim`, or why it is not sent.
/// `knock` is the knockback in tiles (-4 to 4). The power is capped at the
/// profile's limit, and the id must be in the build's send set.
pub fn hurtplayer_message(
    version: ClientVersion,
    from: (f32, f32),
    t: &Target,
    knock: (f32, f32),
    half: i32,
) -> Result<Vec<u8>, Reject> {
    let p = version.profile();
    let Some(rules) = p.life.combat else {
        return Err(Reject::NotInBuild);
    };
    if !p.protocol.pli.contains(PLI_HURTPLAYER) {
        return Err(Reject::NotInBuild);
    }
    attacker_may_send(&rules, from, t)?;
    let half = half.min(i32::from(rules.packet_hit_cap_half));
    if half <= 0 {
        return Err(Reject::NoDamage);
    }
    let kb = |k: f32| (((k.clamp(-4.0, 3.9375) + 4.0) * 16.0).round() as u8).wrapping_add(0x20);
    let mut m = vec![PLI_HURTPLAYER + 0x20];
    m.extend_from_slice(&crate::net::gshort(t.id));
    m.push(kb(knock.0));
    m.push(kb(knock.1));
    m.push(half as u8 + 0x20);
    m.push(b'\n');
    Ok(m)
}

/// The swing and the hurt state of one session.
#[derive(Debug, Clone, Default)]
pub struct CombatState {
    rules: Option<CombatRules>,
    /// Index into the swing frames (0 is the first, step 9) and the
    /// milliseconds spent on it.
    swing: Option<(u8, u64)>,
    hurt_left: u16,
    hurt_acc_ms: u64,
}

/// One client tick, in milliseconds.
const TICK_MS: u64 = crate::life::TICK_MS;

impl CombatState {
    pub fn new(rules: Option<CombatRules>) -> Self {
        CombatState { rules, ..Self::default() }
    }

    pub fn rules(&self) -> Option<&CombatRules> {
        self.rules.as_ref()
    }

    /// Take the rules of another build. Ends a swing and a hurt in progress.
    pub fn set_rules(&mut self, rules: Option<CombatRules>) {
        *self = CombatState::new(rules);
    }

    pub fn is_hurt(&self) -> bool {
        self.hurt_left > 0
    }

    pub fn is_swinging(&self) -> bool {
        self.swing.is_some()
    }

    /// Start the swing. False if the build has no combat rules or a swing
    /// is already out.
    pub fn start_swing(&mut self) -> bool {
        if self.rules.is_none() || self.swing.is_some() {
            return false;
        }
        self.swing = Some((0, 0));
        true
    }

    pub fn cancel_swing(&mut self) -> bool {
        self.swing.take().is_some()
    }

    /// Hurt mode starts (40 ticks).
    pub fn start_hurt(&mut self) {
        if let Some(r) = &self.rules {
            self.hurt_left = r.hurt_ticks;
            self.hurt_acc_ms = 0;
        }
    }

    /// The SPRITE (17) value to send for a player facing `dir`: the swing
    /// frame while the sword is out, the hurt sprite while hurt, otherwise
    /// plain idle (S = 0, as every walk message sends).
    pub fn sprite(&self, dir: u8) -> u8 {
        let Some(r) = &self.rules else { return dir % 4 };
        let s = match (self.swing, self.hurt_left) {
            (Some((i, _)), _) => r.swing_sprite_first + i,
            (None, h) if h > 0 => r.hurt_sprite,
            _ => 0,
        };
        (dir % 4).wrapping_add(s.wrapping_mul(4))
    }

    /// Advance by `ms`. True when the sprite others see changed (a swing
    /// frame, the end of the swing, the end of hurt mode).
    pub fn tick(&mut self, ms: u64) -> bool {
        let Some(r) = self.rules else { return false };
        let before = self.sprite(0);
        if let Some((i, acc)) = self.swing {
            let acc = acc + ms;
            let per = u64::from(r.swing_step_ticks.max(1)) * TICK_MS;
            let frames = r.swing_sprite_last - r.swing_sprite_first + 1;
            let step = (acc / per) as u8;
            self.swing = if i + step >= frames { None } else { Some((i + step, acc % per)) };
        }
        if self.hurt_left > 0 {
            self.hurt_acc_ms += ms;
            let ticks = (self.hurt_acc_ms / TICK_MS) as u16;
            self.hurt_acc_ms %= TICK_MS;
            self.hurt_left = self.hurt_left.saturating_sub(ticks);
        }
        self.sprite(0) != before
    }
}

/// Where a hit came from, for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Sword,
    HurtPlayer,
    HitObjects,
}

/// What became of one hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitReport {
    pub kind: HitKind,
    pub attacker: Option<u16>,
    /// Half hearts taken, or why not.
    pub outcome: Result<i32, Reject>,
}

impl OfflineSession {
    /// The player as the judge of a hit sees them.
    pub fn victim(&self) -> Victim {
        let state = if self.life.is_dead() || self.hearts_half() <= 0 {
            Vulnerability::Dead
        } else if self.combat.is_hurt() {
            Vulnerability::Hurt
        } else if self.in_water {
            Vulnerability::Swimming
        } else {
            Vulnerability::Hurtable
        };
        Victim { x: self.player_x, y: self.player_y, ap: self.alignment, state }
    }

    /// A hit that was judged good: take the hearts, name the attacker for a
    /// claim, start hurt mode. Returns the half hearts taken.
    fn take_hit(&mut self, half: i32, attacker: Option<u16>) -> Result<i32, Reject> {
        if !self.damage_half(half, attacker) {
            return Err(Reject::Dead);
        }
        self.combat.start_hurt();
        Ok(half)
    }

    /// A player's props showed a sword swing: judge it from this side.
    pub fn receive_sword(&mut self, id: Option<u16>, s: &Swinger) -> HitReport {
        let outcome = match self.combat.rules().copied() {
            None => Err(Reject::NotInBuild),
            Some(r) => judge_sword(&r, s, &self.victim()).and_then(|h| self.take_hit(h, id)),
        };
        HitReport { kind: HitKind::Sword, attacker: id, outcome }
    }

    /// A `PLO_HURTPLAYER`. `attacker_at` is where the attacker is, in this
    /// player's coordinates.
    pub fn receive_hurtplayer(&mut self, h: &HurtPlayer, attacker_at: Option<(f32, f32)>) -> HitReport {
        let outcome = match self.combat.rules().copied() {
            None => Err(Reject::NotInBuild),
            Some(r) => judge_hurtplayer(&r, h, attacker_at, &self.victim()).and_then(|half| self.take_hit(half, Some(h.attacker))),
        };
        HitReport { kind: HitKind::HurtPlayer, attacker: Some(h.attacker), outcome }
    }

    /// A `PLO_HITOBJECTS`. The notes say it hurts a player whose centre is
    /// within 2 tiles of `(x, y)` on each axis, but not where a player's
    /// centre is, so Seihai decodes it and applies nothing.
    pub fn receive_hitobjects(&mut self, h: &HitObjects) -> HitReport {
        HitReport { kind: HitKind::HitObjects, attacker: Some(h.attacker), outcome: Err(Reject::CentreUnknown) }
    }

    /// Judge everything that arrived: `swings` (ids of players whose props
    /// showed a swing) and packet `hits`, against `others` (the players in
    /// the level, from their props) and `own_cell` (our gmap cell, so a
    /// player in another cell is placed relative to us). Reports every hit,
    /// taken or refused, in order: swings first, then packets.
    pub fn receive_all(
        &mut self,
        others: &[OtherPlayer],
        own_cell: Option<(i32, i32)>,
        swings: &[u16],
        hits: &[IncomingHit],
    ) -> Vec<HitReport> {
        let mut out = Vec::new();
        for id in swings {
            let Some(o) = others.iter().find(|o| o.id == *id) else { continue };
            let Some(sprite) = o.sprite else { continue };
            let (x, y) = o.position_from(own_cell);
            let s = Swinger { pose: Pose { x, y, dir: o.dir }, sprite, sword_power: o.sword_power };
            out.push(self.receive_sword(Some(o.id), &s));
        }
        for h in hits {
            match h {
                IncomingHit::Hurt(h) => {
                    let at = others.iter().find(|o| o.id == h.attacker).map(|o| o.position_from(own_cell));
                    out.push(self.receive_hurtplayer(h, at));
                }
                IncomingHit::Objects(h) => out.push(self.receive_hitobjects(h)),
            }
        }
        out
    }

    /// Start a sword swing on the player's own key press. Not while dead,
    /// swimming, or already swinging, and not with no sword.
    pub fn start_swing(&mut self) -> bool {
        if self.life.is_dead() || self.in_water || self.sword_power == 0 || self.warp_pending.is_some() {
            return false;
        }
        self.combat.start_swing()
    }

    /// Advance the swing and hurt mode by `ms`. True when the sprite others
    /// see changed. A death ends the swing.
    pub fn tick_combat(&mut self, ms: u64) -> bool {
        let cancelled = self.life.is_dead() && self.combat.cancel_swing();
        self.combat.tick(ms) || cancelled
    }

    /// The SPRITE (17) value for the player's own walk and swing messages.
    pub fn sprite_value(&self) -> u8 {
        self.combat.sprite(self.player_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::life::LifeEvent;
    use crate::protocol::{ClientVersion, SessionState, COMBAT_205, DEATH_205, PROFILES};

    const R: CombatRules = COMBAT_205;

    /// SPRITE value of swing frame `s` facing `dir`.
    fn sw(s: u8, dir: u8) -> u8 {
        dir + 4 * s
    }

    fn at(x: f32, y: f32, dir: u8) -> Pose {
        Pose { x, y, dir }
    }

    fn victim(x: f32, y: f32) -> Victim {
        Victim { x, y, ap: 50, state: Vulnerability::Hurtable }
    }

    fn swinger(dir: u8, sprite: u8) -> Swinger {
        Swinger { pose: at(20.0, 20.0, dir), sprite, sword_power: Some(1) }
    }

    // ---- the box: reach 0..=4 ahead, under 2 to the side ----

    #[test]
    fn reach_runs_from_0_to_4_tiles_inclusive_in_every_direction() {
        // (dir, step along the facing axis as (dx, dy) per tile)
        for (dir, ux, uy) in [(0u8, 0.0, -1.0), (1, -1.0, 0.0), (2, 0.0, 1.0), (3, 1.0, 0.0)] {
            let a = at(20.0, 20.0, dir);
            let ok = |t: f32| in_sword_box(&R, a, 20.0 + ux * t, 20.0 + uy * t);
            assert_eq!(ok(0.0), Ok(()), "dir {dir}: 0 tiles counts");
            assert_eq!(ok(2.0), Ok(()));
            assert_eq!(ok(4.0), Ok(()), "dir {dir}: 4 tiles counts");
            assert_eq!(ok(4.5), Err(Reject::OutOfReach), "dir {dir}");
            assert_eq!(ok(-0.5), Err(Reject::OutOfReach), "dir {dir}: behind the attacker");
        }
    }

    #[test]
    fn the_side_limit_is_strictly_under_2_tiles() {
        // Facing east: forward is x, sideways is y.
        let a = at(20.0, 20.0, 3);
        assert_eq!(in_sword_box(&R, a, 22.0, 21.5), Ok(()));
        assert_eq!(in_sword_box(&R, a, 22.0, 18.5), Ok(()), "either side");
        assert_eq!(in_sword_box(&R, a, 22.0, 22.0), Err(Reject::OffLine), "exactly 2 is out");
        assert_eq!(in_sword_box(&R, a, 22.0, 18.0), Err(Reject::OffLine));
        assert_eq!(in_sword_box(&R, a, 22.0, 21.99), Ok(()));
        // Facing north: sideways is x.
        let n = at(20.0, 20.0, 0);
        assert_eq!(in_sword_box(&R, n, 21.5, 17.0), Ok(()));
        assert_eq!(in_sword_box(&R, n, 22.0, 17.0), Err(Reject::OffLine));
    }

    #[test]
    fn reach_is_tested_before_the_side() {
        let a = at(20.0, 20.0, 3);
        assert_eq!(in_sword_box(&R, a, 30.0, 30.0), Err(Reject::OutOfReach));
    }

    // ---- the whole judgement ----

    #[test]
    fn only_sprites_6_to_10_are_a_swing() {
        let v = victim(22.0, 20.0);
        for s in 0..40u8 {
            let r = judge_sword(&R, &swinger(3, sw(s, 3)), &v);
            if (6..=10).contains(&s) {
                assert_eq!(r, Ok(1), "S {s}");
            } else {
                assert_eq!(r, Err(Reject::NotSwinging), "S {s}");
            }
        }
    }

    #[test]
    fn the_direction_is_the_one_in_the_sprite() {
        // The pose says north, the sprite says east: the sprite is what was sent.
        let s = Swinger { pose: at(20.0, 20.0, 0), sprite: sw(8, 3), sword_power: Some(1) };
        assert_eq!(judge_sword(&R, &s, &victim(22.0, 20.0)), Ok(1));
        assert_eq!(judge_sword(&R, &s, &victim(18.0, 20.0)), Err(Reject::OutOfReach), "behind an east-facing swing");
        assert_eq!(judge_sword(&R, &s, &victim(20.0, 18.0)), Err(Reject::OffLine), "where a north-facing swing would have hit");
    }

    #[test]
    fn ap_100_cannot_be_hit_by_a_sword_and_99_can() {
        let mut v = victim(22.0, 20.0);
        v.ap = 100;
        assert_eq!(judge_sword(&R, &swinger(3, sw(8, 3)), &v), Err(Reject::Immune));
        v.ap = 99;
        assert_eq!(judge_sword(&R, &swinger(3, sw(8, 3)), &v), Ok(1));
        v.ap = 0;
        assert_eq!(judge_sword(&R, &swinger(3, sw(8, 3)), &v), Ok(1));
    }

    #[test]
    fn a_player_who_cannot_be_hurt_is_refused_first() {
        for (state, why) in [
            (Vulnerability::Dead, Reject::Dead),
            (Vulnerability::Hurt, Reject::AlreadyHurt),
            (Vulnerability::Swimming, Reject::Swimming),
        ] {
            let mut v = victim(22.0, 20.0);
            v.state = state;
            assert_eq!(judge_sword(&R, &swinger(3, sw(8, 3)), &v), Err(why));
        }
    }

    #[test]
    fn the_damage_is_the_labelled_placeholder() {
        // Sword power times `sword_half_per_power`, at least a half heart.
        let v = victim(22.0, 20.0);
        for (power, half) in [(0u8, 1), (1, 1), (2, 2), (4, 4)] {
            let s = Swinger { sword_power: Some(power), ..swinger(3, sw(8, 3)) };
            assert_eq!(judge_sword(&R, &s, &v), Ok(half));
        }
        let unknown = Swinger { sword_power: None, ..swinger(3, sw(8, 3)) };
        assert_eq!(judge_sword(&R, &unknown, &v), Err(Reject::UnknownPower));
    }

    // ---- packets ----

    /// A wire GSHORT.
    fn gs(v: u16) -> [u8; 2] {
        [0x20 + (v >> 7) as u8, 0x20 + (v & 127) as u8]
    }

    fn p40(attacker: u16, dx: i32, dy: i32, half: u8, npc: Option<u32>) -> Vec<u8> {
        let mut p = gs(attacker).to_vec();
        p.push((0x20 + (dx + 4) * 16) as u8);
        p.push((0x20 + (dy + 4) * 16) as u8);
        p.push(0x20 + half);
        if let Some(n) = npc {
            p.extend_from_slice(&[0x20 + (n >> 14) as u8, 0x20 + ((n >> 7) & 127) as u8, 0x20 + (n & 127) as u8]);
        }
        p
    }

    #[test]
    fn hurtplayer_decodes_the_layout() {
        let h = parse_hurtplayer(&p40(300, -2, 1, 2, None)).unwrap();
        assert_eq!(h.attacker, 300);
        assert_eq!((h.dx, h.dy, h.half, h.npc), (-2.0, 1.0, 2, None));
        let h = parse_hurtplayer(&p40(7, 3, -4, 1, Some(1234))).unwrap();
        assert_eq!((h.dx, h.dy, h.half, h.npc), (3.0, -4.0, 1, Some(1234)));
        assert_eq!(parse_hurtplayer(&[0x20, 0x21, 0x60, 0x60]), None, "no power byte");
        assert_eq!(parse_hurtplayer(&[]), None);
    }

    #[test]
    fn hitobjects_decodes_the_layout() {
        let mut p = gs(5).to_vec();
        p.extend_from_slice(&[0x20 + 2, 0x20 + 41, 0x20 + 25]);
        let h = parse_hitobjects(&p).unwrap();
        assert_eq!((h.attacker, h.half, h.x, h.y, h.npc), (5, 2, 20.5, 12.5, None));
        p.extend_from_slice(&[0x20, 0x20, 0x20 + 9]);
        assert_eq!(parse_hitobjects(&p).unwrap().npc, Some(9));
    }

    fn hurt(attacker: u16, half: i32) -> HurtPlayer {
        HurtPlayer { attacker, dx: 0.0, dy: 0.0, half, npc: None }
    }

    #[test]
    fn hurtplayer_needs_the_attacker_within_10_tiles() {
        let v = victim(20.0, 20.0);
        let go = |x: f32, y: f32| judge_hurtplayer(&R, &hurt(1, 2), Some((x, y)), &v);
        assert_eq!(go(30.0, 20.0), Ok(2), "10 tiles away counts");
        assert_eq!(go(30.5, 20.0), Err(Reject::TooFar));
        assert_eq!(go(20.0, 9.5), Err(Reject::TooFar));
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 2), None, &v), Err(Reject::TooFar), "attacker not in the level");
    }

    #[test]
    fn hurtplayer_power_is_capped_at_one_heart() {
        let v = victim(20.0, 20.0);
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 9), Some((20.0, 20.0)), &v), Ok(2));
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 1), Some((20.0, 20.0)), &v), Ok(1));
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 0), Some((20.0, 20.0)), &v), Err(Reject::NoDamage));
    }

    #[test]
    fn hurtplayer_ignores_ap_and_refuses_the_unhurtable_and_npcs() {
        let mut v = victim(20.0, 20.0);
        v.ap = 100;
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 2), Some((20.0, 20.0)), &v), Ok(2), "the notes make AP 100 immune to the sword only");
        v.state = Vulnerability::Dead;
        assert_eq!(judge_hurtplayer(&R, &hurt(1, 2), Some((20.0, 20.0)), &v), Err(Reject::Dead));
        let from_npc = HurtPlayer { npc: Some(4), ..hurt(1, 2) };
        assert_eq!(judge_hurtplayer(&R, &from_npc, Some((20.0, 20.0)), &victim(20.0, 20.0)), Err(Reject::NpcSource));
    }

    // ---- the attacker's side ----

    fn target(x: f32, y: f32) -> Target {
        Target { id: 300, x, y, status: None, sprite: None }
    }

    #[test]
    fn the_attacker_does_not_send_what_the_victim_would_refuse() {
        let from = (20.0, 20.0);
        assert_eq!(attacker_may_send(&R, from, &target(25.0, 20.0)), Ok(()));
        assert_eq!(attacker_may_send(&R, from, &target(31.0, 20.0)), Err(Reject::TooFar));
        let dead = Target { status: Some(0x04 | STATUS_DEAD), ..target(21.0, 20.0) };
        assert_eq!(attacker_may_send(&R, from, &dead), Err(Reject::Dead));
        let alive = Target { status: Some(0x14), ..target(21.0, 20.0) };
        assert_eq!(attacker_may_send(&R, from, &alive), Ok(()));
        let hurt = Target { sprite: Some(sw(32, 2)), ..target(21.0, 20.0) };
        assert_eq!(attacker_may_send(&R, from, &hurt), Err(Reject::AlreadyHurt));
    }

    #[test]
    fn hurtplayer_message_bytes_and_gating() {
        let v = ClientVersion::V205;
        let m = hurtplayer_message(v, (20.0, 20.0), &target(21.0, 20.0), (0.0, -1.0), 2).unwrap();
        // 26 + 0x20, GSHORT 300, (0+4)*16 = 64, (-1+4)*16 = 48, power 2.
        let mut want = vec![26 + 0x20];
        want.extend_from_slice(&gs(300));
        want.extend_from_slice(&[0x20 + 64, 0x20 + 48, 0x20 + 2, b'\n']);
        assert_eq!(m, want);
        // And it decodes the way the victim reads it.
        let back = parse_hurtplayer(&m[1..m.len() - 1]).unwrap();
        assert_eq!((back.attacker, back.dx, back.dy, back.half), (300, 0.0, -1.0, 2));
        // The cap, the refusals, and a build without the rules.
        let big = hurtplayer_message(v, (20.0, 20.0), &target(21.0, 20.0), (0.0, 0.0), 9).unwrap();
        assert_eq!(big[big.len() - 2], 0x20 + 2);
        assert_eq!(hurtplayer_message(v, (20.0, 20.0), &target(40.0, 20.0), (0.0, 0.0), 2), Err(Reject::TooFar));
        assert_eq!(hurtplayer_message(v, (20.0, 20.0), &target(21.0, 20.0), (0.0, 0.0), 0), Err(Reject::NoDamage));
        for old in [ClientVersion::V133, ClientVersion::V139, ClientVersion::V200, ClientVersion::V204] {
            assert_eq!(hurtplayer_message(old, (20.0, 20.0), &target(21.0, 20.0), (0.0, 0.0), 2), Err(Reject::NotInBuild), "{old:?}");
        }
    }

    // ---- profiles ----

    #[test]
    fn combat_follows_the_profile() {
        for p in PROFILES {
            assert_eq!(p.life.combat.is_some(), p.life.death.is_some(), "{}: combat and death rows go together", p.label);
            if let Some(c) = p.life.combat {
                assert!(c.swing_sprite_first <= c.swing_sprite_last, "{}", p.label);
                assert!(c.reach_min_half <= c.reach_max_half, "{}", p.label);
            }
        }
        for v in [ClientVersion::V133, ClientVersion::V139, ClientVersion::V200, ClientVersion::V204] {
            assert!(v.combat_rules().is_none(), "{}", v.label());
        }
        for v in [ClientVersion::V205, ClientVersion::V217, ClientVersion::V222, ClientVersion::V231] {
            assert_eq!(v.combat_rules(), Some(COMBAT_205), "{}", v.label());
        }
        assert_eq!(COMBAT_205.immune_ap, 100);
        assert_eq!((COMBAT_205.reach_min_half, COMBAT_205.reach_max_half, COMBAT_205.lateral_below_half), (0, 8, 4));
    }

    // ---- the session: damage, hurt mode, death ----

    fn session() -> OfflineSession {
        let mut s = OfflineSession::new();
        s.life.set_rules(Some(DEATH_205));
        s.combat.set_rules(Some(COMBAT_205));
        s.player_x = 22.0;
        s.player_y = 20.0;
        s.set_max_hearts_half(12);
        s.set_hearts_half(12);
        s
    }

    #[test]
    fn a_sword_hit_takes_hearts_and_starts_hurt_mode() {
        let mut s = session();
        let r = s.receive_sword(Some(9), &swinger(3, sw(8, 3)));
        assert_eq!(r.outcome, Ok(1));
        assert_eq!(s.hearts_half(), 11);
        assert!(s.combat.is_hurt());
        // Hurt for 40 ticks: the same swing again does nothing.
        let r = s.receive_sword(Some(9), &swinger(3, sw(9, 3)));
        assert_eq!(r.outcome, Err(Reject::AlreadyHurt));
        assert_eq!(s.hearts_half(), 11);
        s.tick_combat(39 * 50);
        assert!(s.combat.is_hurt());
        s.tick_combat(50);
        assert!(!s.combat.is_hurt(), "40 ticks, 2 s");
        assert_eq!(s.receive_sword(Some(9), &swinger(3, sw(10, 3))).outcome, Ok(1));
        assert_eq!(s.hearts_half(), 10);
    }

    #[test]
    fn a_rejected_hit_changes_nothing() {
        let mut s = session();
        s.alignment = 100;
        assert_eq!(s.receive_sword(Some(9), &swinger(3, sw(8, 3))).outcome, Err(Reject::Immune));
        s.alignment = 50;
        s.player_x = 30.0; // 10 tiles ahead: out of reach
        assert_eq!(s.receive_sword(Some(9), &swinger(3, sw(8, 3))).outcome, Err(Reject::OutOfReach));
        s.player_x = 22.0;
        s.player_y = 23.0; // 3 tiles to the side
        assert_eq!(s.receive_sword(Some(9), &swinger(3, sw(8, 3))).outcome, Err(Reject::OffLine));
        assert_eq!(s.hearts_half(), 12);
        assert!(!s.combat.is_hurt());
    }

    #[test]
    fn swimming_players_cannot_be_hit() {
        let mut s = session();
        s.in_water = true;
        assert_eq!(s.receive_sword(Some(9), &swinger(3, sw(8, 3))).outcome, Err(Reject::Swimming));
        assert_eq!(s.hearts_half(), 12);
    }

    #[test]
    fn the_killing_hit_names_the_attacker_for_the_claim() {
        let mut s = session();
        s.set_hearts_half(1);
        assert_eq!(s.receive_sword(Some(321), &swinger(3, sw(8, 3))).outcome, Ok(1));
        assert_eq!(s.hearts_half(), 0);
        assert_eq!(s.tick_life(50), vec![LifeEvent::Died { attacker: Some(321) }]);
        let msgs = crate::life::messages_for(LifeEvent::Died { attacker: Some(321) }, ClientVersion::V205, 0);
        assert_eq!(msgs, vec![crate::life::claim_message(321)]);
    }

    #[test]
    fn a_dead_player_cannot_be_hit_again() {
        let mut s = session();
        s.set_hearts_half(1);
        s.receive_sword(Some(1), &swinger(3, sw(8, 3)));
        s.tick_life(50);
        assert!(s.life.is_dead());
        s.tick_combat(2_000); // hurt mode over: only death is left to refuse it
        assert!(!s.combat.is_hurt());
        assert_eq!(s.receive_sword(Some(2), &swinger(3, sw(8, 3))).outcome, Err(Reject::Dead));
        assert_eq!(s.receive_hurtplayer(&hurt(2, 2), Some((22.0, 20.0))).outcome, Err(Reject::Dead));
    }

    #[test]
    fn hearts_at_zero_before_the_death_tick_count_as_dead() {
        let mut s = session();
        s.set_hearts_half(0);
        assert_eq!(s.victim().state, Vulnerability::Dead);
    }

    #[test]
    fn a_hurtplayer_packet_hurts_through_the_session() {
        let mut s = session();
        let r = s.receive_hurtplayer(&hurt(4, 5), Some((25.0, 20.0)));
        assert_eq!((r.kind, r.outcome), (HitKind::HurtPlayer, Ok(2)));
        assert_eq!(s.hearts_half(), 10);
        assert!(s.combat.is_hurt());
    }

    #[test]
    fn hitobjects_is_decoded_and_not_applied() {
        let mut s = session();
        let h = HitObjects { attacker: 1, half: 2, x: 22.0, y: 20.0, npc: None };
        assert_eq!(s.receive_hitobjects(&h).outcome, Err(Reject::CentreUnknown));
        assert_eq!(s.hearts_half(), 12);
    }

    #[test]
    fn no_rules_no_hits() {
        let mut s = OfflineSession::new();
        s.life.set_rules(Some(DEATH_205));
        assert_eq!(s.receive_sword(Some(1), &swinger(3, sw(8, 3))).outcome, Err(Reject::NotInBuild));
        assert_eq!(s.receive_hurtplayer(&hurt(1, 2), Some((0.0, 0.0))).outcome, Err(Reject::NotInBuild));
        assert!(!s.start_swing());
        assert_eq!(s.sprite_value(), s.player_dir);
    }

    // ---- receive_all over real players ----

    fn other(id: u16, x: f32, y: f32, dir: u8, sprite: u8, power: Option<u8>) -> OtherPlayer {
        OtherPlayer {
            id,
            x,
            y,
            dir,
            nick: None,
            chat: None,
            chat_timer: Default::default(),
            account: None,
            cell: None,
            sprite: Some(sprite),
            status: None,
            ap: None,
            sword_power: power,
        }
    }

    #[test]
    fn receive_all_judges_swings_and_packets_in_order() {
        let mut s = session();
        let others = [other(10, 20.0, 20.0, 3, sw(7, 3), Some(2)), other(11, 22.0, 24.0, 0, sw(7, 0), Some(1))];
        let hits = [IncomingHit::Hurt(hurt(11, 2)), IncomingHit::Objects(HitObjects { attacker: 11, half: 2, x: 22.0, y: 20.0, npc: None })];
        let out = s.receive_all(&others, None, &[10, 11, 99], &hits);
        let kinds: Vec<_> = out.iter().map(|r| (r.kind, r.attacker, r.outcome)).collect();
        assert_eq!(
            kinds,
            vec![
                (HitKind::Sword, Some(10), Ok(2)),
                (HitKind::Sword, Some(11), Err(Reject::AlreadyHurt)),
                (HitKind::HurtPlayer, Some(11), Err(Reject::AlreadyHurt)),
                (HitKind::HitObjects, Some(11), Err(Reject::CentreUnknown)),
            ],
            "an unknown id (99) is skipped"
        );
    }

    #[test]
    fn a_player_in_another_gmap_cell_is_placed_relative_to_us() {
        // We stand at x 2 of cell (1, 0). The attacker is at x 62 of cell
        // (0, 0), facing east: that is x -2 in our cell, 4 tiles behind us
        // along its facing, so the hit lands (the edge of the reach).
        let mut s = session();
        s.player_x = 2.0;
        let mut o = other(10, 62.0, 20.0, 3, sw(7, 3), Some(1));
        o.cell = Some((0, 0));
        assert_eq!(s.receive_all(&[o.clone()], Some((1, 0)), &[10], &[])[0].outcome, Ok(1));
        // Without the cell offset the same numbers are 60 tiles apart.
        let mut s = session();
        s.player_x = 2.0;
        assert_eq!(s.receive_all(&[o], None, &[10], &[])[0].outcome, Err(Reject::OutOfReach));
    }

    // ---- our own swing ----

    #[test]
    fn our_swing_sends_frames_6_to_10_then_goes_idle() {
        let mut s = session();
        s.player_dir = 3;
        assert_eq!(s.sprite_value(), 3);
        assert!(s.start_swing());
        assert_eq!(s.sprite_value(), sw(6, 3));
        assert!(!s.start_swing(), "one swing at a time");
        let mut seen = vec![s.sprite_value()];
        // swing_step_ticks (2) of 50 ms per frame.
        for _ in 0..20 {
            if s.tick_combat(50) {
                seen.push(s.sprite_value());
            }
        }
        assert_eq!(seen, vec![sw(6, 3), sw(7, 3), sw(8, 3), sw(9, 3), sw(10, 3), 3]);
        assert!(!s.combat.is_swinging());
        assert!(s.start_swing(), "and again once it is over");
    }

    #[test]
    fn the_swing_needs_a_sword_and_a_live_dry_player() {
        let mut s = session();
        s.sword_power = 0;
        assert!(!s.start_swing());
        s.sword_power = 1;
        s.in_water = true;
        assert!(!s.start_swing());
        s.in_water = false;
        s.set_hearts_half(1);
        s.damage_half(1, None);
        s.tick_life(0);
        assert!(!s.start_swing(), "dead");
    }

    #[test]
    fn dying_ends_the_swing_and_hurt_mode_shows_the_hurt_sprite() {
        let mut s = session();
        s.player_dir = 2;
        assert!(s.start_swing());
        s.damage_half(12, Some(1));
        s.tick_life(0);
        assert!(s.tick_combat(0), "the sprite changed");
        assert!(!s.combat.is_swinging());
        let mut h = session();
        h.player_dir = 1;
        h.receive_sword(Some(2), &swinger(3, sw(8, 3)));
        assert_eq!(h.sprite_value(), 1 + 4 * 32);
        assert!(h.tick_combat(2_000), "hurt mode ends, the sprite goes back");
        assert_eq!(h.sprite_value(), 1);
    }

    #[test]
    fn walk_message_carries_the_sprite() {
        assert_eq!(crate::net::position_message(10.5, 3.0, 2), crate::net::position_message_sprite(10.5, 3.0, 2));
        let m = crate::net::position_message_sprite(10.5, 3.0, sw(8, 3));
        assert_eq!(m[5], 17 + 0x20);
        assert_eq!(m[6], 0x20 + 35);
    }

    // ---- the session state reading the wire ----

    fn plo8(id: u16, props: &[u8]) -> Vec<u8> {
        let mut m = vec![8 + 0x20];
        m.extend_from_slice(&gs(id));
        m.extend_from_slice(props);
        m.push(b'\n');
        m
    }

    #[test]
    fn other_props_feed_the_victims_side() {
        let mut st = SessionState::for_version(ClientVersion::V205);
        // SWORDPOWER (8) = 2 with image "s.png", X 40 (20 tiles), Y 40,
        // SPRITE swing frame 8 facing east, STATUS 0x14, ALIGNMENT 77.
        let mut props = vec![8 + 0x20, 0x20 + 32, 0x20 + 5];
        props.extend_from_slice(b"s.png");
        props.extend_from_slice(&[15 + 0x20, 0x20 + 40, 16 + 0x20, 0x20 + 40, 17 + 0x20, 0x20 + sw(8, 3)]);
        props.extend_from_slice(&[18 + 0x20, 0x20 + 0x14, 32 + 0x20, 0x20 + 77]);
        st.apply_body(&plo8(300, &props));
        let o = &st.others[0];
        assert_eq!((o.sword_power, o.sprite, o.status, o.ap), (Some(2), Some(sw(8, 3)), Some(0x14), Some(77)));
        assert_eq!((o.x, o.y, o.dir), (20.0, 20.0, 3));
        assert_eq!(st.swings, vec![300]);
        // An idle update later keeps what it knew and adds no swing.
        st.swings.clear();
        st.apply_body(&plo8(300, &[17 + 0x20, 0x20 + 3]));
        assert!(st.swings.is_empty());
        assert_eq!(st.others[0].sword_power, Some(2));
        assert_eq!(st.others[0].sprite, Some(3));
    }

    #[test]
    fn packet_hits_are_decoded_by_profile() {
        let m40 = |id: u16| {
            let mut m = vec![40 + 0x20];
            m.extend_from_slice(&p40(id, 1, 0, 2, None));
            m.push(b'\n');
            m
        };
        let mut st = SessionState::for_version(ClientVersion::V205);
        st.apply_body(&m40(12));
        let mut m46 = vec![46 + 0x20];
        m46.extend_from_slice(&gs(12));
        m46.extend_from_slice(&[0x20 + 2, 0x20 + 40, 0x20 + 40, b'\n']);
        st.apply_body(&m46);
        assert!(matches!(st.hits[0], IncomingHit::Hurt(h) if h.attacker == 12 && h.half == 2));
        assert!(matches!(st.hits[1], IncomingHit::Objects(h) if h.attacker == 12 && h.x == 20.0));
        // A build without the rules decodes nothing and counts the ids as unhandled.
        for v in [ClientVersion::V133, ClientVersion::V139, ClientVersion::V200, ClientVersion::V204] {
            let mut st = SessionState::for_version(v);
            st.apply_body(&m40(12));
            st.apply_body(&m46);
            assert!(st.hits.is_empty(), "{}", v.label());
            let mut sw_props = vec![17 + 0x20, 0x20 + sw(8, 3)];
            sw_props.extend_from_slice(&[15 + 0x20, 0x20 + 40]);
            st.apply_body(&plo8(5, &sw_props));
            assert!(st.swings.is_empty(), "{}: no swing is noticed", v.label());
        }
    }
}
