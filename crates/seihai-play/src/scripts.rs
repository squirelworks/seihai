//! The server NPCs' GS2 client scripts (#447): each `PLO_NPCBYTECODE` the
//! level brought runs on core's VM for the build (`Machine::for_version`).
//!
//! The scripts are started when their level arrives (the top-level code,
//! then `onCreated`, then `onPlayerEnters`), and tick on the 50 ms frame
//! the VM counts its timers in. What they change goes through core's
//! [`SessionScripts`]: each NPC with a script has an object in its script
//! world, tagged with the NPC's place in the level, so the effects land on
//! that NPC (its image, visibility, place, message) or on the player (flags,
//! props, `setlevel`). Player flags that are not `local.` go to the server
//! as the clients send them (`PLI_FLAGSET` / `PLI_FLAGDEL`).
//!
//! The player's GS2 weapons (6.x: `PLO_NPCWEAPONSCRIPT`, core's
//! `OwnWeapons::bytecode`) run here too, each on an object of its own that
//! stands at the player and acts as the player (a bomb or arrow a weapon
//! makes is the player's). A weapon starts with `onCreated` when the
//! server gives it, gets `onPlayerEnters` on each level, `onWeaponFired`
//! when D fires it, and `onPlayerChats`; it stops when the server takes
//! it away. Level changes leave weapons running.
//!
//! Every run reads the player through core's `PlayerSnap` (`player.x`,
//! `player.chat`, the counters), and `player.chat = …` says it.
//!
//! A machine runs one thing at a time: while a run sleeps (`sleep`) or
//! waits (`waitfor`), the events that come for its NPC wait for it to end,
//! in the order they came. A `waitfor` on an event of the NPC's own that
//! comes in time resumes with 1; one that times out resumes with 0.

use std::collections::{HashMap, VecDeque};

use seihai_core::offline::{OfflineSession, FOOT_DX, FOOT_DY};
use seihai_core::protocol::ClientVersion;
use seihai_core::scripting::engine::host::SessionScripts;
use seihai_core::scripting::engine::world::{ObjId, ObjSpec};
use seihai_core::scripting::engine::{Era, Val};
use seihai_core::scripting::gs2::exec::{EffectHost, Machine, Outcome, PlayerSnap};
use seihai_core::scripting::gs2::Script;

/// The VM's frame, in milliseconds (core's `TICK`).
pub const FRAME_MS: u64 = 50;

/// A GS2 weapon for [`ClientScripts::set_weapons`]: its name, image, the
/// script's bytes and the script as read.
pub type GsWeapon = (String, String, Vec<u8>, Result<Script, String>);

/// What a machine's current run is waiting for.
#[derive(Debug, Clone, PartialEq)]
enum Wait {
    /// Nothing runs: events start at once.
    Idle,
    /// `sleep`: seconds left before the run goes on.
    Sleep(f64),
    /// `waitfor`: the event (without its `on`) and the seconds left.
    Event { event: String, left: f64 },
}

/// Whose script a run is.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Owner {
    /// A server NPC, by its id, which finds it in the level's list.
    Npc(u32),
    /// One of the player's weapons, by name.
    Weapon(String),
}

/// One NPC's or weapon's script.
struct Running {
    owner: Owner,
    /// The bytes the script came from, so a script sent again with other
    /// bytes restarts.
    code: Option<Vec<u8>>,
    obj: ObjId,
    machine: Machine,
    host: EffectHost,
    wait: Wait,
    /// Events that came while the run slept or waited.
    queued: VecDeque<(String, Vec<Val>)>,
    /// The player's feet were on the NPC last frame.
    touching: bool,
}

/// The scripts of the level on screen.
pub struct ClientScripts {
    version: ClientVersion,
    /// Core's GS1 host, used for its script world (one object per NPC that
    /// runs a script) and for applying effects to the session.
    pub host: SessionScripts,
    running: Vec<Running>,
    /// The NPC scripts of gmap cells the walk left, by lower-case level
    /// name, kept with their state for when it comes back (in the order
    /// of their objects, which core's world parks with them).
    parked: HashMap<String, Vec<Running>>,
    /// Parked cells that have been out of the neighbour ring since they
    /// were parked: they owe `onPlayerEnters` when they run again.
    left_ring: std::collections::HashSet<String>,
    /// Milliseconds not yet counted into a frame.
    acc_ms: u64,
    /// Lines a run logged (calls with no command behind them), for the
    /// caller to show or drop.
    pub log: Vec<String>,
    /// What the player said last, for `player.chat`.
    pub chat: String,
    /// The player's nickname and account, for `player.nick` and
    /// `player.account`.
    pub nick: String,
    pub account: String,
}

impl ClientScripts {
    pub fn new(version: ClientVersion) -> Self {
        ClientScripts {
            version,
            host: SessionScripts::new(version),
            running: Vec::new(),
            parked: HashMap::new(),
            left_ring: std::collections::HashSet::new(),
            acc_ms: 0,
            log: Vec::new(),
            chat: String::new(),
            nick: String::new(),
            account: String::new(),
        }
    }

    /// How many scripts are running.
    pub fn len(&self) -> usize {
        self.running.len()
    }

    pub fn is_empty(&self) -> bool {
        self.running.is_empty()
    }

    /// True if NPC `npc_id` has a script running.
    pub fn runs(&self, npc_id: u32) -> bool {
        self.running.iter().any(|r| r.owner == Owner::Npc(npc_id))
    }

    /// True if weapon `name` has a GS2 script running.
    pub fn runs_weapon(&self, name: &str) -> bool {
        self.running
            .iter()
            .any(|r| matches!(&r.owner, Owner::Weapon(w) if w == name))
    }

    /// The weapons with a GS2 script running, by name.
    pub fn weapons(&self) -> Vec<&str> {
        self.running
            .iter()
            .filter_map(|r| match &r.owner {
                Owner::Weapon(w) => Some(w.as_str()),
                Owner::Npc(_) => None,
            })
            .collect()
    }

    /// A new level is on screen: the old level's scripts stop, and so do
    /// the parked gmap cells'. The weapons go on.
    pub fn clear(&mut self) {
        self.running.retain(|r| matches!(r.owner, Owner::Weapon(_)));
        self.host.world.clear_level();
        self.parked.clear();
        self.left_ring.clear();
        self.host.world.retain_parked(|_| false);
        self.acc_ms = 0;
    }

    /// The walk left gmap cell `level`: its NPC scripts stop running but
    /// keep their state (variables, timers, a sleep in progress), with
    /// their objects in core's world, until [`Self::unpark`] brings them
    /// back. The weapons go on.
    pub fn park(&mut self, level: &str) {
        let mut npcs = Vec::new();
        let mut i = 0;
        while i < self.running.len() {
            if matches!(self.running[i].owner, Owner::Npc(_)) {
                npcs.push(self.running.remove(i));
            } else {
                i += 1;
            }
        }
        // Core parks the level's objects in id order; keep the same order.
        npcs.sort_by_key(|r| r.obj);
        self.host.world.park_level(level);
        let key = level.to_ascii_lowercase();
        self.left_ring.remove(&key);
        self.parked.insert(key, npcs);
        self.acc_ms = 0;
    }

    /// The walk came back into gmap cell `level`: its parked scripts run
    /// again where they were, never with `onCreated`. They get
    /// `onPlayerEnters` only if the cell left the neighbour ring meanwhile
    /// ([`ClientScripts::mark_left_ring`]). The real 6.03r7 fires it when
    /// the cell comes back into the ring, not on entering it; scripts
    /// parked here do not run until the player is in the cell, so it comes
    /// then (seihai#497 lab, 2026-11-20). Returns how many came back.
    pub fn unpark(&mut self, session: &mut OfflineSession, level: &str) -> usize {
        let key = level.to_ascii_lowercase();
        let enters = self.left_ring.remove(&key);
        let Some(mut npcs) = self.parked.remove(&key) else {
            return 0;
        };
        let base = self.host.world.len();
        if !self.host.world.unpark_level(level) {
            return 0;
        }
        let back = self.host.world.len() - base;
        if back != npcs.len() {
            // Not the objects these runs were parked with: start over.
            for k in 0..back {
                Self::release_npc_object(&mut self.host.world, base + k);
            }
            return 0;
        }
        for (k, r) in npcs.iter_mut().enumerate() {
            r.obj = base + k;
            r.host.obj = r.obj;
            r.touching = false;
        }
        let first = self.running.len();
        self.running.extend(npcs);
        self.retag(session);
        self.host.sync_player(session);
        let mut came_back = 0;
        for i in first..self.running.len() {
            self.sync(i, session);
            self.seed_flags(i);
            if enters {
                self.event(i, "PlayerEnters", &[]);
            }
            came_back += 1;
        }
        self.apply(session);
        came_back
    }

    /// Keep only the parked cells `keep` says yes to (lower-case names):
    /// the cells of the player's gmap. The real 6.03r7 keeps a cell's NPC
    /// state after the player has gone two cells away and back
    /// (seihai#497 lab, 2026-11-20).
    pub fn retain_parked(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.parked.retain(|n, _| keep(n));
        self.left_ring.retain(|n| self.parked.contains_key(n));
        self.host.world.retain_parked(keep);
    }

    /// Note the parked cells that are not neighbours of the player's cell
    /// now (`near` says which are, lower-case names): coming back, they
    /// get `onPlayerEnters`.
    pub fn mark_left_ring(&mut self, mut near: impl FnMut(&str) -> bool) {
        for n in self.parked.keys() {
            if !near(n) {
                self.left_ring.insert(n.clone());
            }
        }
    }

    /// The gmap cells with scripts parked, lower-case.
    pub fn parked_levels(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.parked.keys().map(String::as_str).collect();
        v.sort_unstable();
        v
    }

    /// Stop NPC `npc_id`'s script, so a new one can start.
    pub fn stop(&mut self, npc_id: u32) {
        let removed: Vec<ObjId> = self
            .running
            .iter()
            .filter(|r| r.owner == Owner::Npc(npc_id))
            .map(|r| r.obj)
            .collect();
        for obj in removed {
            self.remove_npc_object(obj);
        }
        self.running.retain(|r| r.owner != Owner::Npc(npc_id));
    }

    /// Core's remove_npc closes a deleted level-list gap. Stopping or
    /// replacing a runtime does not change that list, so restore the
    /// surviving tags after releasing its object and queued ID events.
    fn remove_npc_object(&mut self, obj: ObjId) {
        Self::release_npc_object(&mut self.host.world, obj);
    }

    fn release_npc_object(
        world: &mut seihai_core::scripting::engine::world::ScriptWorld,
        obj: ObjId,
    ) {
        let Some(tag) = world.obj(obj).map(|o| o.tag) else {
            return;
        };
        let tags: Vec<_> = world
            .ids()
            .into_iter()
            .filter_map(|id| world.obj(id).map(|o| (id, o.tag)))
            .collect();
        world.remove_npc(tag);
        for (id, tag) in tags {
            if let Some(o) = world.obj_mut(id) {
                o.tag = tag;
            }
        }
    }

    /// The new level is in: `onPlayerEnters` on each weapon.
    pub fn level_entered(&mut self, session: &mut OfflineSession) {
        self.host.sync_player(session);
        for i in 0..self.running.len() {
            if matches!(self.running[i].owner, Owner::Weapon(_)) {
                self.sync(i, session);
                self.seed_flags(i);
                self.event(i, "PlayerEnters", &[]);
            }
        }
        self.apply(session);
    }

    /// The player's GS2 weapons are now `weapons` (name, image, the
    /// script's bytes and the script as read): a new one starts
    /// (`onCreated`, then `onPlayerEnters` if a level is in), one sent
    /// again with other bytes starts again, and one that is gone stops.
    /// Returns how many started.
    pub fn set_weapons(&mut self, session: &mut OfflineSession, weapons: &[GsWeapon]) -> usize {
        let world = &mut self.host.world;
        self.running.retain(|r| match &r.owner {
            Owner::Weapon(w) => {
                let keep = weapons
                    .iter()
                    .any(|(n, _, code, _)| n == w && r.code.as_ref() == Some(code));
                if !keep {
                    world.remove_weapon(w);
                }
                keep
            }
            Owner::Npc(_) => true,
        });
        let mut started = 0;
        for (name, image, code, script) in weapons {
            let Ok(script) = script else {
                self.log.push(format!(
                    "weapon {name}: script not read: {}",
                    script.as_ref().err().map_or("", |e| e)
                ));
                continue;
            };
            if self.runs_weapon(name) {
                continue;
            }
            let obj = self.host.world.add(ObjSpec::weapon(name, image, ""));
            let i = self.push(Owner::Weapon(name.clone()), Some(code.clone()), obj, script);
            self.host.sync_player(session);
            self.sync(i, session);
            self.seed_flags(i);
            let out = {
                let r = &mut self.running[i];
                r.machine.run_top_level(&mut r.host)
            };
            self.ended(i, out);
            self.event(i, "Created", &[]);
            if session.current_level.is_some() {
                self.event(i, "PlayerEnters", &[]);
            }
            started += 1;
        }
        self.apply(session);
        started
    }

    /// D fired weapon `name`: `onWeaponFired` on its script. False if it
    /// has no GS2 script running.
    pub fn weapon_fired(&mut self, session: &mut OfflineSession, name: &str) -> bool {
        let Some(i) = self
            .running
            .iter()
            .position(|r| matches!(&r.owner, Owner::Weapon(w) if w == name))
        else {
            return false;
        };
        self.host.sync_player(session);
        self.sync(i, session);
        self.seed_flags(i);
        self.event(i, "WeaponFired", &[]);
        self.apply(session);
        true
    }

    /// A machine for `owner` on world object `obj`.
    fn push(&mut self, owner: Owner, code: Option<Vec<u8>>, obj: ObjId, script: &Script) -> usize {
        let mut host = EffectHost::new(Era::for_version(self.version), obj);
        host.resolve_host = self.version.profile().scripts.resolve_host;
        self.running.push(Running {
            owner,
            code,
            obj,
            machine: Machine::for_version(script.clone(), self.version),
            host,
            wait: Wait::Idle,
            queued: VecDeque::new(),
            touching: false,
        });
        self.running.len() - 1
    }

    /// Start the scripts a level brought, by the NPC's server id. A script
    /// whose NPC is not on the level, that did not parse, or that is
    /// already running is left out. Returns how many started.
    pub fn start(
        &mut self,
        session: &mut OfflineSession,
        scripts: &[(u32, Result<Script, String>)],
    ) -> usize {
        self.retag(session);
        let mut started = 0;
        for (id, script) in scripts {
            let Ok(script) = script else { continue };
            if self.runs(*id) {
                continue;
            }
            let Some((index, n)) = session.current_level.as_ref().and_then(|l| {
                l.npcs
                    .iter()
                    .enumerate()
                    .find(|(_, n)| n.server_id == Some(*id))
            }) else {
                continue;
            };
            let spec = ObjSpec::npc(&n.image, n.x as f64, n.y as f64, "").with_tag(index);
            let visible = n.visible;
            let obj = self.host.world.add(spec);
            if let Some(o) = self.host.world.obj_mut(obj) {
                o.visible = visible;
            }
            let i = self.push(Owner::Npc(*id), None, obj, script);
            self.host.sync_player(session);
            self.sync(i, session);
            self.seed_flags(i);
            let out = {
                let r = &mut self.running[i];
                r.machine.run_top_level(&mut r.host)
            };
            self.ended(i, out);
            self.event(i, "Created", &[]);
            self.event(i, "PlayerEnters", &[]);
            started += 1;
        }
        self.apply(session);
        started
    }

    /// Advance the clock by `ms`: each 50 ms frame resumes the sleeps and
    /// waits that are over, runs the timers (`onTimeout`, `scheduleevent`)
    /// and the events that waited. Then `onPlayerTouchsMe` for the NPCs the
    /// player's feet just stepped onto.
    pub fn tick(&mut self, session: &mut OfflineSession, ms: u64) {
        self.retag(session);
        self.acc_ms += ms;
        while self.acc_ms >= FRAME_MS {
            self.acc_ms -= FRAME_MS;
            let frame = FRAME_MS as f64 / 1000.0;
            for i in 0..self.running.len() {
                self.sync(i, session);
                let r = &mut self.running[i];
                let out = match &mut r.wait {
                    Wait::Idle => None,
                    Wait::Sleep(left) => {
                        *left -= frame;
                        (*left <= 1e-9).then(|| r.machine.resume(&mut r.host))
                    }
                    Wait::Event { left, .. } => {
                        *left -= frame;
                        (*left <= 1e-9).then(|| r.machine.resume_waitfor(false, &mut r.host))
                    }
                };
                if let Some(out) = out {
                    self.running[i].wait = Wait::Idle;
                    self.ended(i, out);
                }
                // Count every elapsed frame, even while a run sleeps or waits.
                // The VM queues due timers and dispatches only when its run ends.
                let r = &mut self.running[i];
                let ran = r.machine.tick(&mut r.host);
                for (_, out) in ran {
                    self.ended(i, out);
                }
                self.drain_queue(i);
            }
            self.apply(session);
        }
        self.touch(session);
    }

    /// The player said `text`: `onPlayerChats` on every script, with
    /// `player.chat` reading it.
    pub fn player_chats(&mut self, session: &mut OfflineSession, text: &str) {
        self.chat = text.to_string();
        self.retag(session);
        self.host.sync_player(session);
        for i in 0..self.running.len() {
            self.sync(i, session);
            self.seed_flags(i);
            self.event(i, "PlayerChats", &[]);
        }
        self.apply(session);
    }

    /// A key went down: `onKeyPressed(code, key, scan)` on every script,
    /// NPCs and weapons alike, as the real 4.20r8 and 6.03r7 pass every
    /// key (`crate::keycodes`, lab key sweep 2026-11-19). The GS2 weapons
    /// of 4.x to 6.x read them as `params` (221 of the weapons in
    /// Graal_analysis `npc-library/gs2-weapon-corpus-2026-11-14.md`).
    pub fn key_pressed(&mut self, session: &mut OfflineSession, press: &crate::keycodes::Press) {
        self.event_on_all(
            session,
            "KeyPressed",
            &[
                Val::Num(f64::from(press.code)),
                Val::Str(press.key.clone()),
                Val::Num(f64::from(press.scan)),
            ],
        );
    }

    /// A mouse click at `(x, y)` in level tiles: `onMouseDown(modifier)`
    /// (`left`, `right`, `double` or `middle`) on every script, NPCs and
    /// weapons alike, wherever the click lands, with `mousex`/`mousey` at
    /// the click. The real 6.03r7 passes only the button and gives every
    /// NPC the click, not only the one under it (seihai#497 lab,
    /// 2026-11-20; seihai-core#136). Returns how many scripts got it.
    pub fn mouse_down(
        &mut self,
        session: &mut OfflineSession,
        x: f32,
        y: f32,
        modifier: &str,
    ) -> usize {
        if self.running.is_empty() {
            return 0;
        }
        self.retag(session);
        self.host.sync_player(session);
        let mouse = (f64::from(x), f64::from(y));
        let args = [Val::Str(modifier.to_string())];
        for i in 0..self.running.len() {
            self.running[i].host.mouse = mouse;
            self.sync(i, session);
            self.seed_flags(i);
            self.event(i, "MouseDown", &args);
        }
        self.apply(session);
        self.running.len()
    }

    /// The player took damage: `onPlayerHurt` on every script.
    pub fn player_hurt(&mut self, session: &mut OfflineSession) {
        self.event_on_all(session, "PlayerHurt", &[]);
    }

    /// The player died: `onPlayerDies` on every script.
    pub fn player_dies(&mut self, session: &mut OfflineSession) {
        self.event_on_all(session, "PlayerDies", &[]);
    }

    /// `on{event}` on every running script, each with a fresh view of the
    /// player.
    fn event_on_all(&mut self, session: &mut OfflineSession, event: &str, args: &[Val]) {
        if self.running.is_empty() {
            return;
        }
        self.retag(session);
        self.host.sync_player(session);
        for i in 0..self.running.len() {
            self.sync(i, session);
            self.seed_flags(i);
            self.event(i, event, args);
        }
        self.apply(session);
    }

    /// `onPlayerTouchsMe` on each visible NPC the player's feet have just
    /// entered.
    fn touch(&mut self, session: &mut OfflineSession) {
        let (fx, fy) = (
            f64::from(session.player_x + FOOT_DX),
            f64::from(session.player_y + FOOT_DY),
        );
        let mut any = false;
        for i in 0..self.running.len() {
            if !matches!(self.running[i].owner, Owner::Npc(_)) {
                continue;
            }
            let inside = self.host.world.obj(self.running[i].obj).is_some_and(|o| {
                let (x, y, w, h) = o.rect();
                o.visible && fx >= x && fx < x + w && fy >= y && fy < y + h
            });
            let was = std::mem::replace(&mut self.running[i].touching, inside);
            if inside && !was {
                if !any {
                    self.host.sync_player(session);
                }
                self.sync(i, session);
                self.seed_flags(i);
                self.event(i, "PlayerTouchsMe", &[]);
                any = true;
            }
        }
        if any {
            self.apply(session);
        }
    }

    /// Run `on{event}` on script `i` now, or after its run if one sleeps
    /// or waits. A `waitfor` on this event ends with 1 instead.
    fn event(&mut self, i: usize, event: &str, args: &[Val]) {
        let r = &mut self.running[i];
        match &r.wait {
            Wait::Idle => {
                let out = r
                    .machine
                    .call_event(&format!("on{event}"), args, &mut r.host);
                self.ended(i, out);
            }
            Wait::Event { event: e, .. } if e.eq_ignore_ascii_case(event) => {
                r.wait = Wait::Idle;
                let out = r.machine.resume_waitfor(true, &mut r.host);
                self.ended(i, out);
            }
            _ => r.queued.push_back((event.to_string(), args.to_vec())),
        }
    }

    /// Run the events that waited while script `i` was busy.
    fn drain_queue(&mut self, i: usize) {
        while self.running[i].wait == Wait::Idle {
            let Some((event, args)) = self.running[i].queued.pop_front() else {
                break;
            };
            self.event(i, &event, &args);
        }
    }

    /// Note how a run of script `i` ended.
    fn ended(&mut self, i: usize, out: Outcome) {
        let r = &mut self.running[i];
        match out {
            Outcome::Sleep(t) => r.wait = Wait::Sleep(t),
            Outcome::WaitFor { event, timeout, .. } => {
                r.wait = Wait::Event {
                    event,
                    left: timeout,
                }
            }
            Outcome::Stopped(why) => {
                let who = match &r.owner {
                    Owner::Npc(id) => format!("npc {id}"),
                    Owner::Weapon(w) => format!("weapon {w}"),
                };
                self.log.push(format!("{who}: {why}"))
            }
            Outcome::Done | Outcome::NoSuchEvent => {}
        }
        self.log.append(&mut r.host.log);
    }

    /// Copy the NPC's place into script `i` before a run: the level's
    /// NPC is where the scripts' last moves and the server's updates put it.
    /// A weapon stands at the player. The player goes in too.
    fn sync(&mut self, i: usize, session: &OfflineSession) {
        let r = &mut self.running[i];
        r.host.player = PlayerSnap {
            x: f64::from(session.player_x),
            y: f64::from(session.player_y),
            dir: f64::from(session.player_dir % 4),
            chat: self.chat.clone(),
            nick: self.nick.clone(),
            account: self.account.clone(),
            hearts: f64::from(session.player_health) / 100.0 * f64::from(session.max_hearts) / 2.0,
            fullhearts: f64::from(session.max_hearts) / 2.0,
            rupees: f64::from(session.rupees),
            bombs: f64::from(session.bombs),
            darts: f64::from(session.darts),
        };
        let Some(o) = self.host.world.obj_mut(r.obj) else {
            return;
        };
        match r.owner {
            Owner::Weapon(_) => {
                o.x = f64::from(session.player_x);
                o.y = f64::from(session.player_y);
            }
            Owner::Npc(_) => {
                if let Some(n) = session
                    .current_level
                    .as_ref()
                    .and_then(|l| l.npcs.get(o.tag))
                {
                    o.x = f64::from(n.x);
                    o.y = f64::from(n.y);
                }
            }
        }
        r.host.x = o.x;
        r.host.y = o.y;
    }

    /// Give script `i` the player's flags as the session has them, so a
    /// `set` of a flag the server already set sends nothing.
    fn seed_flags(&mut self, i: usize) {
        self.running[i].host.flags = self.host.world.core.flags.clone();
    }

    /// The level's NPC list changed (an NPC came or went): each script's
    /// object follows its NPC's new place, and a script whose NPC is gone
    /// stops.
    pub(crate) fn retag(&mut self, session: &OfflineSession) {
        let Some(level) = session.current_level.as_ref() else {
            return;
        };
        let world = &mut self.host.world;
        self.running.retain(|r| {
            let Owner::Npc(id) = r.owner else {
                return true;
            };
            let Some(index) = level.npcs.iter().position(|n| n.server_id == Some(id)) else {
                Self::release_npc_object(world, r.obj);
                return false;
            };
            let Some(o) = world.obj_mut(r.obj) else {
                return false;
            };
            o.tag = index;
            true
        });
    }

    /// Hand every run's effects to the session, and the player flags that
    /// changed to `send` for the server.
    fn apply(&mut self, session: &mut OfflineSession) {
        for r in &mut self.running {
            if let (Some(o), Owner::Npc(_)) = (self.host.world.obj_mut(r.obj), &r.owner) {
                o.x = r.host.x;
                o.y = r.host.y;
            }
            self.host.world.core.effects.append(&mut r.host.effects);
        }
        self.host.apply(session);
        self.host.unhandled.clear();
    }

    /// The messages for the player flags the scripts changed.
    pub fn take_server_messages(&mut self, session: &mut OfflineSession) -> Vec<Vec<u8>> {
        std::mem::take(&mut session.flags_out)
            .into_iter()
            .map(|(name, value)| {
                seihai_core::protocol::flagset::flag_message(&name, value.as_deref())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::assets::Level;
    use seihai_core::protocol::npcprops::NpcProps;
    use seihai_core::scripting::gs2::compile::compile_gs2;

    const V: ClientVersion = ClientVersion::V600R2;

    /// Timers count elapsed frames while the run is suspended, then dispatch
    /// after it ends. This is the pinned VM contract, not native-client evidence.
    #[test]
    fn timers_due_during_sleep_and_waitfor_dispatch_on_resume() {
        let mut resumed_messages = Vec::new();
        for suspension in [
            "sleep(0.15);",
            "waitfor(this, \"playerchats\", 0.15);",
            "sleep(0.05); sleep(0.1);",
            "sleep(0.05); waitfor(this, \"playerchats\", 0.1);",
        ] {
            let mut s = session_with_npc(7);
            let mut sc = ClientScripts::new(V);
            let src = format!(
                "function onCreated() {{ this.order = \"\"; setTimer(0.1); scheduleevent(0.1, \"Probe\"); {suspension} }}\n\
                 function onTimeout() {{ this.order @= \"timeout,\"; }}\n\
                 function onProbe() {{ message(this.order @ \"probe\"); }}"
            );
            sc.start(&mut s, &script(&src));
            sc.tick(&mut s, 100);
            assert_eq!(
                npc(&s).message,
                "",
                "{suspension}: no dispatch while suspended"
            );
            sc.tick(&mut s, 50);
            resumed_messages.push((suspension, npc(&s).message.clone()));
        }
        assert!(
            resumed_messages
                .iter()
                .all(|(_, msg)| msg == "timeout,probe"),
            "{resumed_messages:?}"
        );
    }

    #[test]
    fn a_successful_waitfor_keeps_elapsed_timer_frames() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { setTimer(0.1); scheduleevent(0.1, \"Probe\"); this.caught = waitfor(this, \"playerchats\", 0.5); }\n\
                   function onTimeout() { this.order = \"timeout,\"; }\n\
                   function onProbe() { message(this.order @ this.caught); }";
        sc.start(&mut s, &script(src));
        sc.tick(&mut s, 50);
        sc.player_chats(&mut s, "wake");
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "timeout,1");
    }

    /// Internal adapter/VM ordering: an event satisfies the wait without
    /// dispatching already-due timers or replacing the suspended continuation.
    #[test]
    fn a_successful_waitfor_preserves_overdue_timers_and_queued_events() {
        for continuation in ["", "sleep(0.05);"] {
            let mut s = session_with_npc(7);
            let mut sc = ClientScripts::new(V);
            let src = format!(
                "function onCreated() {{ setTimer(0.1); scheduleevent(0.1, \"Probe\");
                   temp.caught = waitfor(this, \"playerchats\", 0.5);
                   this.order = \"resume\" @ temp.caught @ \",\"; message(this.order);
                   {continuation} this.order @= \"continued,\"; }}
                 function onTimeout() {{ this.order @= \"timeout,\"; message(this.order); }}
                 function onProbe() {{ this.order @= \"probe,\"; message(this.order); }}
                 function onPlayerEnters() {{ message(this.order @ \"enters\"); }}
                 function onPlayerChats() {{ message(\"unexpected chat handler\"); }}"
            );
            sc.start(&mut s, &script(&src));
            // Both VM timers are due, but onCreated still waits and the
            // adapter's startup PlayerEnters event is queued behind it.
            sc.tick(&mut s, 100);
            assert_eq!(npc(&s).message, "", "{continuation}");
            sc.player_chats(&mut s, "wake");
            assert_eq!(npc(&s).message, "resume1,", "{continuation}");
            // The next frame finishes any resumed sleep, dispatches overdue
            // VM timers, then drains the adapter's queued event. The matching
            // chat satisfied waitfor and must not also run its event handler.
            sc.tick(&mut s, 50);
            assert_eq!(
                npc(&s).message,
                "resume1,continued,timeout,probe,enters",
                "{continuation}"
            );
            sc.tick(&mut s, 50);
            assert_eq!(
                npc(&s).message,
                "resume1,continued,timeout,probe,enters",
                "{continuation}: nothing should replay"
            );
        }
    }

    #[test]
    fn due_timers_do_not_interrupt_a_sleeping_event_or_reorder_queued_events() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { this.order = \"\"; setTimer(0.1); scheduleevent(0.1, \"Probe\"); }\n\
                   function onTimeout() { sleep(0.1); this.order @= \"timeout,\"; }\n\
                   function onProbe() { this.order @= \"probe,\"; }\n\
                   function onPlayerChats() { message(this.order @ \"chat\"); }";
        sc.start(&mut s, &script(src));
        sc.tick(&mut s, 100);
        sc.player_chats(&mut s, "queued");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "timeout,probe,chat");
    }

    #[test]
    fn elapsed_frames_are_counted_once_and_keep_fractional_time() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { this.n = 0; setTimer(0.1); }\n\
                   function onTimeout() { this.n++; message(this.n); setTimer(0.1); }";
        sc.start(&mut s, &script(src));
        sc.tick(&mut s, 49);
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 1);
        assert_eq!(npc(&s).message, "1");
        sc.tick(&mut s, 100);
        assert_eq!(npc(&s).message, "2");
    }

    /// A level with server NPC `id` at (10, 10), 2 by 2 tiles.
    fn session_with_npc(id: u32) -> OfflineSession {
        let mut s = OfflineSession::default();
        s.current_level = Some(Level::from_board(&[0; 4096]));
        s.take_server_npc(
            id,
            &NpcProps {
                image: Some("block.png".into()),
                x: Some(10.0),
                y: Some(10.0),
                ..Default::default()
            },
        );
        s.player_x = 30.0;
        s.player_y = 30.0;
        s
    }

    fn npc(s: &OfflineSession) -> &seihai_core::assets::Npc {
        &s.current_level.as_ref().unwrap().npcs[0]
    }

    fn script(src: &str) -> Vec<(u32, Result<Script, String>)> {
        vec![(7, Ok(compile_gs2(src).unwrap()))]
    }

    /// The top level runs, then `onCreated`, then `onPlayerEnters`, and
    /// their effects land on the NPC with the script.
    #[test]
    fn a_level_starts_its_scripts_in_order() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "this.order = \"top\";\n\
                   function onCreated() { this.order @= \",created\"; }\n\
                   function onPlayerEnters() { this.order @= \",enters\"; message(this.order); }\n";
        assert_eq!(sc.start(&mut s, &script(src)), 1);
        assert_eq!(npc(&s).message, "top,created,enters");
        // A second arrival of the same script does not start it again.
        assert_eq!(sc.start(&mut s, &script(src)), 0);
        assert_eq!(sc.len(), 1);
    }

    /// `onKeyPressed` gets the key code and text as `params`;
    /// `onPlayerHurt` and `onPlayerDies` run on every script.
    #[test]
    fn key_hurt_and_death_events_reach_the_scripts() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onKeyPressed() { message(params[0] @ \":\" @ params[1]); }\n\
                   function onPlayerHurt() { message(\"hurt\"); }\n\
                   function onPlayerDies() { message(\"dies\"); }\n";
        sc.start(&mut s, &script(src));
        let a = crate::keycodes::press(egui::Key::A, egui::Modifiers::NONE).unwrap();
        sc.key_pressed(&mut s, &a);
        assert_eq!(npc(&s).message, "65:a");
        sc.player_hurt(&mut s);
        assert_eq!(npc(&s).message, "hurt");
        sc.player_dies(&mut s);
        assert_eq!(npc(&s).message, "dies");
    }

    /// A script whose NPC is not on the level, or that did not parse,
    /// does not start.
    #[test]
    fn scripts_without_their_npc_are_left_out() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let scripts = vec![
            (
                8,
                Ok(compile_gs2("function onCreated() { message(\"x\"); }").unwrap()),
            ),
            (7, Err("bad".to_string())),
        ];
        assert_eq!(sc.start(&mut s, &scripts), 0);
        assert!(sc.is_empty());
        assert_eq!(npc(&s).message, "");
    }

    /// `setTimer` runs `onTimeout` after its frames, and a `sleep` goes on
    /// after its time, both on the 50 ms frame.
    #[test]
    fn timers_and_sleeps_run_on_the_frame() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { setTimer(0.1); }\n\
                   function onTimeout() { message(\"a\"); sleep(0.1); message(\"b\"); }\n";
        sc.start(&mut s, &script(src));
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "a");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "a");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "b");
    }

    /// An event that comes while a run sleeps waits for the run to end.
    #[test]
    fn events_wait_for_a_sleeping_run() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { this.n = 0; sleep(0.1); this.n = 1; }\n\
                   function onPlayerChats() { message(\"n=\" @ this.n); }\n";
        sc.start(&mut s, &script(src));
        sc.player_chats(&mut s, "hi");
        assert_eq!(npc(&s).message, "");
        sc.tick(&mut s, 100);
        assert_eq!(npc(&s).message, "n=1");
    }

    /// A key runs `onKeyPressed(code, key, scan)` with the numbers the
    /// real clients give (lab key sweep, 2026-11-19).
    #[test]
    fn a_key_runs_on_key_pressed_with_the_real_numbers() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onKeyPressed(code, key, scan) { message(code @ \",\" @ key @ \",\" @ scan); }\n";
        sc.start(&mut s, &script(src));
        let a = crate::keycodes::press(egui::Key::A, egui::Modifiers::NONE).unwrap();
        sc.key_pressed(&mut s, &a);
        assert_eq!(npc(&s).message, "65,a,30");
        let alt7 = crate::keycodes::press(egui::Key::Num7, egui::Modifiers::ALT).unwrap();
        sc.key_pressed(&mut s, &alt7);
        assert_eq!(npc(&s).message, "1079,7,8");
    }

    /// Stepping onto the NPC runs `onPlayerTouchsMe` once, until the feet
    /// leave and come back.
    #[test]
    fn touching_the_npc_runs_its_event_once() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onPlayerTouchsMe() { this.n++; message(\"touched \" @ this.n); }\n";
        sc.start(&mut s, &script(src));
        // The feet at (10.5, 10.5), inside the NPC's 2 by 2 tiles.
        s.player_x = 10.5 - FOOT_DX;
        s.player_y = 10.5 - FOOT_DY;
        sc.tick(&mut s, 0);
        assert_eq!(npc(&s).message, "touched 1");
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "touched 1");
        s.player_x = 30.0;
        sc.tick(&mut s, 50);
        s.player_x = 10.5 - FOOT_DX;
        sc.tick(&mut s, 50);
        assert_eq!(npc(&s).message, "touched 2");
    }

    /// A player flag a script sets goes to the server as `PLI_FLAGSET`; a
    /// `local.` flag stays in the client and sends nothing.
    #[test]
    fn player_flags_go_to_the_server() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { set(\"quest=2\"); set(\"local.seen\"); }\n";
        sc.start(&mut s, &script(src));
        assert_eq!(
            sc.take_server_messages(&mut s),
            vec![seihai_core::protocol::flagset::flag_message(
                "quest",
                Some("2")
            )]
        );
    }

    /// Moving `this.x` moves the level's NPC, and a later run starts from
    /// where the server put it.
    #[test]
    fn the_script_moves_its_npc() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { this.x += 2; }\n\
                   function onPlayerChats() { this.y = this.x + 1; }\n";
        sc.start(&mut s, &script(src));
        assert_eq!((npc(&s).x, npc(&s).y), (12.0, 10.0));
        s.take_server_npc(
            7,
            &NpcProps {
                x: Some(20.0),
                ..Default::default()
            },
        );
        sc.player_chats(&mut s, "hi");
        assert_eq!((npc(&s).x, npc(&s).y), (20.0, 21.0));
    }

    /// A gmap cell the walk leaves keeps its scripts' state: coming back
    /// runs `onPlayerEnters` on the same run, not `onCreated` again.
    #[test]
    fn a_gmap_cell_left_and_entered_again_keeps_its_scripts_state() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onCreated() { this.entries = 0; }\n\
                   function onPlayerEnters() { this.entries += 1; this.x = 10 + this.entries; }\n";
        sc.start(&mut s, &script(src));
        assert_eq!(npc(&s).x, 11.0);
        // Out to the next cell and back: the cell stayed a neighbour, so
        // its scripts go on with no onPlayerEnters (6.03r7, A -> B -> A).
        sc.park("a.nw");
        assert!(!sc.runs(7));
        assert_eq!(sc.parked_levels(), ["a.nw"]);
        sc.mark_left_ring(|n| n == "a.nw");
        assert_eq!(sc.unpark(&mut s, "A.nw"), 1);
        assert!(sc.runs(7));
        assert_eq!(npc(&s).x, 11.0);
        // Two cells away and back: still the same run, and now one
        // onPlayerEnters (6.03r7, A -> B -> C -> B -> A).
        for back in [12.0, 13.0] {
            sc.park("a.nw");
            sc.mark_left_ring(|_| false);
            sc.mark_left_ring(|n| n == "a.nw");
            assert_eq!(sc.unpark(&mut s, "a.nw"), 1);
            assert_eq!(npc(&s).x, back);
        }
        // Starting the kept scripts again does not start a second run.
        assert_eq!(sc.start(&mut s, &script(src)), 0);
        assert_eq!(sc.len(), 1);
        // Nothing parked under another name; a cell outside the gmap is
        // dropped, and a level change drops them all.
        assert_eq!(sc.unpark(&mut s, "b.nw"), 0);
        sc.park("a.nw");
        sc.retain_parked(|n| n == "b.nw");
        assert_eq!(sc.unpark(&mut s, "a.nw"), 0);
        sc.start(&mut s, &script(src));
        sc.park("a.nw");
        sc.clear();
        assert!(sc.parked_levels().is_empty());
        assert_eq!(sc.unpark(&mut s, "a.nw"), 0);
    }

    /// A click runs `onMouseDown` on every NPC, wherever it lands, with
    /// the button as its only parameter and the place in `mousex`/`mousey`
    /// (6.03r7 lab; seihai-core#136).
    #[test]
    fn a_click_runs_on_mouse_down_with_mousex_and_mousey() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        let src = "function onMouseDown(m) {\n\
                     if (m == \"right\") this.x = 20; else this.x = mousex;\n\
                     this.y = mousey;\n\
                   }\n";
        sc.start(&mut s, &script(src));
        // Far from the NPC: it still gets the click.
        assert_eq!(sc.mouse_down(&mut s, 47.5, 45.5, "left"), 1);
        assert_eq!((npc(&s).x, npc(&s).y), (47.5, 45.5));
        assert_eq!(sc.mouse_down(&mut s, 31.0, 26.5, "right"), 1);
        assert_eq!((npc(&s).x, npc(&s).y), (20.0, 26.5));
    }

    /// When the server deletes the NPC its script stops.
    #[test]
    fn a_deleted_npc_stops_its_script() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        sc.start(&mut s, &script("function onTimeout() {}"));
        assert!(sc.runs(7));
        s.drop_server_npc(7);
        sc.tick(&mut s, 50);
        assert!(sc.is_empty());
        assert!(
            sc.host.world.ids().is_empty(),
            "a deleted runtime must release its live host object"
        );
    }

    #[test]
    fn replacing_a_script_keeps_one_live_object() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        for _ in 0..5 {
            sc.stop(7);
            sc.start(&mut s, &script("function onCreated() {}"));
        }
        assert_eq!(
            sc.host.world.ids().len(),
            1,
            "replacement must not orphan world objects"
        );
    }

    #[test]
    fn replacing_and_removing_weapons_releases_live_objects() {
        let mut s = session_with_npc(7);
        let mut sc = ClientScripts::new(V);
        for code in 0..5 {
            sc.set_weapons(
                &mut s,
                &[(
                    "Tool".into(),
                    String::new(),
                    vec![code],
                    Ok(compile_gs2("function onCreated() {}").unwrap()),
                )],
            );
            assert_eq!(sc.host.world.ids().len(), 1);
        }
        sc.clear();
        assert_eq!(
            sc.host.world.ids().len(),
            1,
            "level changes preserve the weapon"
        );
        sc.set_weapons(&mut s, &[]);
        assert!(sc.host.world.ids().is_empty());
    }

    #[test]
    fn stopping_an_earlier_npc_keeps_later_tags_and_effects_aligned() {
        let mut s = session_with_npc(7);
        s.take_server_npc(
            8,
            &NpcProps {
                image: Some("b.png".into()),
                x: Some(20.0),
                y: Some(20.0),
                ..Default::default()
            },
        );
        let mut sc = ClientScripts::new(V);
        let script = compile_gs2("function onPlayerChats() { message(\"B only\"); }").unwrap();
        sc.start(&mut s, &[(7, Ok(script.clone())), (8, Ok(script))]);
        sc.stop(7);
        let ids = sc.host.world.ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(sc.host.world.obj(ids[0]).unwrap().tag, 1);
        sc.player_chats(&mut s, "hi");
        let npcs = &s.current_level.as_ref().unwrap().npcs;
        assert!(npcs[0].message.is_empty());
        assert_eq!(npcs[1].message, "B only");
    }
}
