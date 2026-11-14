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

use std::collections::VecDeque;

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

    /// A new level is on screen: the old level's scripts stop. The
    /// weapons go on.
    pub fn clear(&mut self) {
        self.running.retain(|r| matches!(r.owner, Owner::Weapon(_)));
        self.host.world.clear_level();
        self.acc_ms = 0;
    }

    /// Stop NPC `npc_id`'s script, so a new one can start.
    pub fn stop(&mut self, npc_id: u32) {
        self.running.retain(|r| r.owner != Owner::Npc(npc_id));
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
    pub fn set_weapons(
        &mut self,
        session: &mut OfflineSession,
        weapons: &[GsWeapon],
    ) -> usize {
        self.running.retain(|r| match &r.owner {
            Owner::Weapon(w) => weapons
                .iter()
                .any(|(n, _, code, _)| n == w && r.code.as_ref() == Some(code)),
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
                if self.running[i].wait == Wait::Idle {
                    let r = &mut self.running[i];
                    let ran = r.machine.tick(&mut r.host);
                    for (_, out) in ran {
                        self.ended(i, out);
                    }
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
    fn retag(&mut self, session: &OfflineSession) {
        let Some(level) = session.current_level.as_ref() else {
            return;
        };
        let world = &mut self.host.world;
        self.running.retain(|r| {
            let Owner::Npc(id) = r.owner else {
                return true;
            };
            let at = level.npcs.iter().position(|n| n.server_id == Some(id));
            match (at, world.obj_mut(r.obj)) {
                (Some(index), Some(o)) => {
                    o.tag = index;
                    true
                }
                _ => false,
            }
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
    }
}
