//! The player's actions beyond the walk and the sword, ported from
//! `seihai-mae` onto core's session: the spin attack, the weapon key (D)
//! with the built-in Bomb and Bow, the lay mode that drops items, and the
//! item list (Q) that picks the weapon D fires.
//!
//! - **Spin.** S again during a swing, with the has-spin bit (0x40) in the
//!   server's STATUS: core's `OfflineSession::start_swing` turns the swing
//!   into the spin. The pose goes out as `spin`.
//! - **Weapon key.** D fires the weapon the item list picked: its GS1
//!   script gets `weaponfired` on core's script host, as in `seihai-mae`.
//!   The built-in weapons (`PLO_DEFAULTWEAPON`) run core's own scripts:
//!   the Bomb (and Superbomb, Joltbomb) puts a bomb in front, the Bow
//!   shoots an arrow. `PLI_BOMBADD` and `PLI_ARROWADD` go to the server,
//!   then the new BOMBSCOUNT or ARROWSCOUNT, as the real 2.05 sends them.
//!   A server weapon with a GS2 script (6.x) does not run here yet.
//! - **Lay mode.** A with S held picks the first item the player has, A
//!   with S held again picks the next, A alone lays it in front (rupees,
//!   bombs, arrows, a heart, or a glove, sword, shield or the spin), and
//!   an arrow, D or S leaves the mode. The item lies on the floor until it
//!   is picked up or times out; online it goes to the server as
//!   `PLI_ITEMADD` by the build's rules.
//! - **Lift and throw.** A lifts what the player faces (a bush, pot or
//!   rock the glove can take), opens a chest from below, or grabs a wall;
//!   A or S while carrying throws. The others see CARRYSPRITE and the
//!   `lift` / `carry` / `carrystill` GANI, then `PLI_THROWCARRIED`.
//! - **Other players.** Their bombs, explosions, arrows and throws come
//!   from core's network thread (`Live::level_objects`, `Live::throws`)
//!   and run in the level as the player's own do.
//! - **Item list.** Q opens and closes the list of the player's weapons
//!   (built-in first, then the server's). The arrows move the selection,
//!   which picks the weapon at once; Q, Esc, Enter, D, S and A close it.
//!   While it is open the player does not walk, swing or act.

use super::Game;
use seihai_core::client::{OwnStats, OwnWeapons};
use seihai_core::offline::Interaction;
use seihai_core::protocol::ClientVersion;
use seihai_core::protocol::{ChatKind, ChatLine, LevelObject};
use seihai_core::scripting::engine::host::{SessionScripts, WeaponScript};

/// The STATUS bit that allows the spin attack.
pub const HAS_SPIN: u8 = 0x40;

/// The item list (Q).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemList {
    pub open: bool,
    /// The weapon names, built-in first.
    pub entries: Vec<String>,
    pub selected: Option<usize>,
}

/// A key the open item list takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKey {
    /// Up or left: the entry before.
    Back,
    /// Down or right: the entry after.
    Next,
    /// Q, Esc, Enter, D, S or A.
    Close,
}

/// What [`Game`] keeps for the actions.
pub(super) struct Actions {
    pub list: ItemList,
    /// The weapon D fires, by name.
    pub weapon: Option<String>,
    /// Core's GS1 host, for the weapons' scripts (set with the build).
    pub scripts: Option<SessionScripts>,
    /// The weapons as last taken from the network thread.
    pub weapons: OwnWeapons,
    /// The arrows and bombs the server last heard of.
    pub told: (Option<i32>, Option<i32>),
}

impl Actions {
    pub fn new() -> Actions {
        Actions {
            list: ItemList::default(),
            weapon: None,
            scripts: None,
            weapons: OwnWeapons::default(),
            told: (None, None),
        }
    }
}

/// The scripts of `w`: core's for the built-in weapons, the server's GS1
/// script for the others (`None` for one with no GS1 script).
fn weapon_scripts(w: &OwnWeapons) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for &id in &w.defaults {
        let Some(name) = seihai_core::client::default_weapon_name(id) else {
            continue;
        };
        let script = seihai_core::bombs::builtin_script(id)
            .or_else(|| seihai_core::arrows::builtin_script(id).map(str::to_string));
        if let Some(script) = script {
            out.push((name, String::new(), script));
        }
    }
    for n in &w.named {
        if let Some(script) = n.script.as_ref().filter(|s| !s.trim().is_empty()) {
            if !out.iter().any(|(name, _, _)| *name == n.name) {
                out.push((
                    n.name.clone(),
                    n.image.clone().unwrap_or_default(),
                    script.clone(),
                ));
            }
        }
    }
    out
}

impl Game {
    /// The build's weapon scripts host ([`Game::set_version`]).
    pub(super) fn set_action_version(&mut self, version: ClientVersion) {
        self.actions.scripts = Some(SessionScripts::new(version));
        self.actions.weapons = OwnWeapons::default();
        self.take_weapons();
    }

    /// The server's counters into the session, so the weapons and the lay
    /// mode spend what the player has: bombs, arrows, gralats, the glove
    /// and the has-spin bit. Only a counter the server changed since
    /// `was` is taken: the others may be behind what the player spent.
    pub(super) fn take_own_counters(&mut self, was: &OwnStats, own: &OwnStats) {
        let s = &mut self.session;
        if let Some(n) = own.bombs.filter(|_| own.bombs != was.bombs) {
            s.bombs = i32::from(n);
            self.actions.told.1 = Some(s.bombs);
        }
        if let Some(n) = own.arrows.filter(|_| own.arrows != was.arrows) {
            s.darts = i32::from(n);
            self.actions.told.0 = Some(s.darts);
        }
        if let Some(n) = own.rupees.filter(|_| own.rupees != was.rupees) {
            s.rupees = n.min(i32::MAX as u32) as i32;
        }
        if let Some(g) = own
            .glove_power
            .filter(|_| own.glove_power != was.glove_power)
        {
            s.glove_power = g;
        }
        if let Some(st) = own.status.filter(|_| own.status != was.status) {
            s.combat.set_has_spin(st & HAS_SPIN != 0);
        }
    }

    /// The status display's counters: the server's, with the bombs,
    /// arrows and gralats the player spent since.
    pub(super) fn hud_values(&self) -> crate::hud::HudValues {
        let mut v = crate::hud::HudValues::of(&self.drawn_own);
        let s = &self.session;
        if self.actions.told.1.is_some() {
            v.bombs = s.bombs;
        }
        if self.actions.told.0.is_some() {
            v.arrows = s.darts;
        }
        if self.drawn_own.rupees.is_some() {
            v.rupees = s.rupees;
        }
        v
    }

    /// The weapons from the network thread: the item list's entries and
    /// the scripts D runs. True when they changed.
    pub(super) fn take_weapons(&mut self) -> bool {
        let now = self
            .weapons
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if now == self.actions.weapons && self.actions.list.entries == now.names() {
            return false;
        }
        self.actions.list.entries = now.names();
        if let Some(sc) = self.actions.scripts.as_mut() {
            let scripts = weapon_scripts(&now);
            let refs: Vec<WeaponScript> = scripts
                .iter()
                .map(|(name, image, script)| WeaponScript {
                    name,
                    image,
                    script,
                })
                .collect();
            sc.sync_weapons(&mut self.session, &refs);
        }
        self.actions.weapons = now;
        // A weapon that is gone is not fired; with none picked, the last
        // one added is, as each add selects it in 2.05.
        let entries = &self.actions.list.entries;
        if !self
            .actions
            .weapon
            .as_ref()
            .is_some_and(|w| entries.contains(w))
        {
            self.actions.weapon = entries.last().cloned();
        }
        self.actions.list.selected = self
            .actions
            .weapon
            .as_ref()
            .and_then(|w| entries.iter().position(|e| e == w));
        true
    }

    /// The item list, for the view to draw.
    pub fn item_list(&self) -> &ItemList {
        &self.actions.list
    }

    /// The weapon D fires.
    pub fn weapon(&self) -> Option<&str> {
        self.actions.weapon.as_deref()
    }

    /// Q: open the item list, or close it.
    pub fn toggle_item_list(&mut self) {
        let l = &mut self.actions.list;
        l.open = !l.open;
        if l.open && l.selected.is_none() && !l.entries.is_empty() {
            l.selected = Some(0);
        }
        self.dirty = true;
    }

    /// A key while the item list is open. Moving the selection picks
    /// that weapon at once.
    pub fn item_list_key(&mut self, k: ListKey) {
        let l = &mut self.actions.list;
        if !l.open {
            return;
        }
        let n = l.entries.len();
        match k {
            ListKey::Close => l.open = false,
            ListKey::Back if n > 0 => {
                l.selected = Some(l.selected.map_or(0, |i| i.saturating_sub(1)))
            }
            ListKey::Next if n > 0 => {
                l.selected = Some(l.selected.map_or(0, |i| (i + 1).min(n - 1)))
            }
            _ => {}
        }
        if let Some(name) = l.selected.and_then(|i| l.entries.get(i)) {
            self.actions.weapon = Some(name.clone());
        }
        self.dirty = true;
    }

    /// True while the item list is open: the player does not walk, swing
    /// or act.
    pub fn list_open(&self) -> bool {
        self.actions.list.open
    }

    /// D: fire the picked weapon. In the lay mode D leaves it instead.
    /// False when nothing was fired.
    pub fn fire(&mut self) -> bool {
        if self.session.current_level.is_none() || self.list_open() {
            return false;
        }
        if self.session.stop_laying() {
            self.dirty = true;
            return false;
        }
        if self.session.life.is_dead() {
            self.respawn();
            return false;
        }
        if self.session.carried.is_some() {
            return false;
        }
        let Some(name) = self.actions.weapon.clone() else {
            return false;
        };
        let Some(sc) = self.actions.scripts.as_mut() else {
            return false;
        };
        sc.sync_player(&self.session);
        sc.world.weapon_fired(&name);
        sc.apply(&mut self.session);
        self.after_actions();
        self.dirty = true;
        true
    }

    /// A, with `sword_held` when S is down: in the lay mode, the next item
    /// (S held) or lay the item (A alone). Else A works on what the player
    /// faces ([`Game::interact`]): it lifts a bush, pot or rock, throws
    /// what is carried, opens a chest or grabs a wall; and when that did
    /// nothing with S held, the lay mode starts. Returns the item laid.
    pub fn act(&mut self, sword_held: bool) -> Option<u8> {
        if self.session.current_level.is_none() || self.list_open() {
            return None;
        }
        self.dirty = true;
        if self.session.laying.is_some() {
            if sword_held {
                self.session.next_lay_item();
                return None;
            }
            let laid = self.session.lay_item();
            self.after_actions();
            return laid;
        }
        // A key while dead respawns, once the death allows it.
        if self.session.life.is_dead() {
            self.respawn();
            return None;
        }
        if self.interact() == Interaction::Nothing && sword_held {
            self.session.start_laying();
        }
        None
    }

    /// Work on what the player faces (core's `OfflineSession::interact`),
    /// and tell the server as a GANI build does: the carried object
    /// (CARRYSPRITE) on a lift, `PLI_THROWCARRIED` and empty hands on a
    /// throw, `PLI_OPENCHEST` for a chest, and the board change a lift
    /// left. The pose (`lift`, `carry`, `carrystill`) goes with the walk.
    pub fn interact(&mut self) -> Interaction {
        let got = self.session.interact();
        let mut msgs = Vec::new();
        match &got {
            Interaction::Lifted { kind } => msgs.push(seihai_core::net::carry_sprite_message(
                seihai_core::throwing::carry_sprite(*kind),
            )),
            Interaction::Thrown { .. } => {
                msgs.push(seihai_core::net::throw_carried_message());
                msgs.push(seihai_core::net::carry_sprite_message(None));
            }
            Interaction::Chest { x, y, got, .. } => {
                if let (Ok(x), Ok(y)) = (u8::try_from(*x), u8::try_from(*y)) {
                    msgs.push(vec![20 + 0x20, x + 0x20, y + 0x20, b'\n']);
                }
                self.chat.push((
                    ChatLine {
                        kind: ChatKind::Admin,
                        from: "Chest".into(),
                        text: format!("You got {got}."),
                        born_ms: 0,
                    },
                    std::time::Instant::now(),
                ));
            }
            Interaction::Grabbed | Interaction::Nothing => {}
        }
        if !self.offline && !msgs.is_empty() {
            self.send
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend(msgs);
        }
        self.send_board_changes();
        self.set_pose(self.session.walking);
        self.dirty = true;
        got
    }

    /// The keys let go this frame: A (`a_down` false) ends a grab, and an
    /// arrow let go ends a push or a pull.
    pub fn keys_up(&mut self, a_down: bool, arrow_let_go: bool) {
        if self.session.pushpull.grab.is_some() && !a_down && self.session.release_grab() {
            self.dirty = true;
        }
        if arrow_let_go && self.session.arrow_released() {
            self.dirty = true;
        }
    }

    /// What other players did that the network thread handed over: their
    /// bombs, explosions and arrows, and the objects they threw, which fly
    /// from where they stood and can hit the player.
    pub(super) fn take_others_actions(&mut self) -> bool {
        let objects =
            std::mem::take(&mut *self.level_objects.lock().unwrap_or_else(|e| e.into_inner()));
        let throws = std::mem::take(&mut *self.throws.lock().unwrap_or_else(|e| e.into_inner()));
        if objects.is_empty() && throws.is_empty() {
            return false;
        }
        self.session.present_players = self.drawn_others.iter().map(|o| o.id).collect();
        for obj in objects {
            match obj {
                LevelObject::Bomb { sender, bomb } => {
                    self.session.receive_bomb(&bomb, Some(sender))
                }
                LevelObject::BombDel { x, y } => self.session.remove_bomb(x, y),
                LevelObject::Explosion { sender, explosion } => {
                    self.session.receive_explosion(&explosion, Some(sender))
                }
                LevelObject::Arrow { sender, arrow } => {
                    self.session.receive_arrow(&arrow, Some(sender))
                }
                _ => {}
            }
        }
        for t in throws {
            self.session.throw_remote(t.sprite, t.x, t.y, t.dir, t.id);
        }
        true
    }

    /// The actions' 50 ms clock, run with the pose clock: the weapon
    /// scripts' timers (the Bow's shot), the bomb fuses, the arrows and the
    /// items on the floor. True when something moved.
    pub(super) fn tick_actions(&mut self, ms: u64) -> bool {
        let mut moved = false;
        if let Some(sc) = self.actions.scripts.as_mut() {
            sc.tick(&mut self.session, ms);
        }
        moved |= self.session.tick_bombs(ms);
        moved |= self.session.tick_arrows(ms, &|_, _| None);
        moved |= !self.session.floor_items.is_empty();
        self.session.tick_items(ms, !self.offline);
        moved |= !self.session.leaps.is_empty();
        moved |= !self.session.thrown.is_empty();
        self.session.tick_throw_with(ms, &|_, _| None);
        // Another player's throw hurt the player: the hurt pose goes out.
        if self.session.take_throw_hurt() {
            self.set_pose(self.session.walking);
            moved = true;
        }
        self.session.take_landings();
        self.session.take_pelts();
        self.after_actions();
        moved
    }

    /// What the actions send: the bombs and arrows put, the items laid,
    /// then the new counts.
    fn after_actions(&mut self) {
        let mut msgs = self.session.take_bomb_sends();
        msgs.extend(self.session.take_arrow_sends());
        msgs.extend(self.session.take_item_messages(!self.offline));
        let s = &self.session;
        let told = self.actions.told;
        let arrows = told.0.filter(|&n| n != s.darts).map(|_| s.darts);
        let bombs = told.1.filter(|&n| n != s.bombs).map(|_| s.bombs);
        if arrows.is_some() || bombs.is_some() {
            let to_u8 = |n: i32| n.clamp(0, 99) as u8;
            msgs.push(seihai_core::net::count_props_message(
                arrows.map(to_u8),
                bombs.map(to_u8),
            ));
            self.actions.told = (arrows.or(told.0), bombs.or(told.1));
        }
        if self.offline || msgs.is_empty() {
            return;
        }
        self.send
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(msgs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Event;
    use seihai_core::assets::Level;
    use seihai_core::client::level::{Arrival, Source};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    /// An online game on build `v` with an empty level, the player at
    /// (30, 30) facing down.
    fn game_on(v: ClientVersion) -> Game {
        let (tx, rx) = mpsc::channel();
        let mut g = Game::new(rx, None, false).unwrap();
        g.set_version(v);
        let a = Arrival {
            name: "test.nw".into(),
            source: Source::Board,
            start: Some((30.0, 30.0)),
            server_npcs: 0,
            server_npc_props: Vec::new(),
            file_npcs: 0,
            scripts: Vec::new(),
        };
        tx.send(Event::Level(Box::new(Level::from_board(&[0; 4096])), a))
            .unwrap();
        g.poll();
        g.session.player_dir = 2;
        g
    }

    fn sent(g: &Game) -> Vec<Vec<u8>> {
        std::mem::take(&mut *g.send.lock().unwrap())
    }

    /// The server's counters: 5 bombs, 10 arrows, and `status`.
    fn counters(g: &mut Game, status: u8) {
        *g.own.lock().unwrap() = OwnStats {
            bombs: Some(5),
            arrows: Some(10),
            rupees: Some(30),
            status: Some(status),
            ..Default::default()
        };
        g.tick(0, 0, Instant::now());
    }

    fn weapons(g: &mut Game, defaults: &[u8]) {
        *g.weapons.lock().unwrap() = OwnWeapons {
            defaults: defaults.to_vec(),
            named: Vec::new(),
        };
        g.tick(0, 0, Instant::now());
    }

    /// Run the 50 ms clocks for `ticks` ticks.
    fn run(g: &mut Game, ticks: u64) {
        let t0 = Instant::now();
        for i in 1..=ticks {
            g.tick_poses(t0 + Duration::from_millis(50 * i));
        }
    }

    /// S again during the swing spins, with the has-spin bit; without it,
    /// it does not.
    #[test]
    fn s_twice_spins_with_the_status_bit() {
        let mut g = game_on(ClientVersion::V205);
        counters(&mut g, 0);
        assert!(g.swing());
        assert!(!g.swing(), "no spin without the bit");
        run(&mut g, 40);
        counters(&mut g, HAS_SPIN);
        assert!(g.swing());
        assert!(g.swing(), "the spin");
        assert!(g.session.combat.spin_elapsed_ms().is_some());
        assert_eq!(crate::render::own_figure(&g.session).pose, "spin");
        assert_eq!(g.pose.lock().unwrap().unwrap().gani, Some("spin"));
        run(&mut g, 80);
        assert!(g.session.combat.spin_elapsed_ms().is_none());
        assert_eq!(g.pose.lock().unwrap().unwrap().gani, None);
    }

    /// The item list shows the built-in weapons, its arrows pick the one D
    /// fires, and D with the Bomb puts a bomb in front: `PLI_BOMBADD`,
    /// then BOMBSCOUNT 4.
    #[test]
    fn the_item_list_picks_the_bomb_and_d_puts_it() {
        let mut g = game_on(ClientVersion::V205);
        counters(&mut g, 0);
        weapons(&mut g, &[8, 7]);
        assert_eq!(g.item_list().entries, ["Bomb", "Bow"]);
        assert_eq!(g.weapon(), Some("Bow"), "the last one added");
        g.toggle_item_list();
        assert!(g.list_open());
        assert!(!g.fire(), "not with the list open");
        g.item_list_key(ListKey::Back);
        assert_eq!(g.weapon(), Some("Bomb"));
        g.item_list_key(ListKey::Close);
        sent(&g);
        assert!(g.fire());
        assert_eq!(g.session.placed_bombs.len(), 1);
        assert_eq!(g.session.bombs, 4);
        let out = sent(&g);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0][0], 0x24, "PLI_BOMBADD");
        assert_eq!(out[1], seihai_core::net::count_props_message(None, Some(4)));
        assert_eq!(g.hud_values().bombs, 4);
        // The fuse burns down and the bomb goes off.
        run(&mut g, 70);
        assert!(g.session.placed_bombs.is_empty());
    }

    /// D with the Bow shoots an arrow after the draw: `PLI_ARROWADD`, then
    /// ARROWSCOUNT 9. With no arrows it does nothing.
    #[test]
    fn d_with_the_bow_shoots() {
        let mut g = game_on(ClientVersion::V205);
        counters(&mut g, 0);
        weapons(&mut g, &[7]);
        sent(&g);
        assert!(g.fire());
        run(&mut g, 10);
        assert_eq!(g.session.darts, 9);
        let out = sent(&g);
        assert!(
            out.iter().any(|m| m[0] == 0x20 + 9),
            "PLI_ARROWADD: {out:?}"
        );
        assert!(out.contains(&seihai_core::net::count_props_message(Some(9), None)));
        g.session.darts = 0;
        g.actions.told.0 = Some(0);
        assert!(g.fire());
        run(&mut g, 10);
        assert!(!sent(&g).iter().any(|m| m[0] == 0x20 + 9));
    }

    /// A with S held picks an item to drop, A lays it in front, and it
    /// lies on the floor; an arrow in the lay mode leaves it.
    #[test]
    fn a_with_s_held_drops_an_item() {
        let mut g = game_on(ClientVersion::V205);
        counters(&mut g, 0);
        assert_eq!(g.act(true), None);
        assert!(g.session.laying.is_some());
        let picked = g.session.laying;
        let laid = g.act(false);
        assert_eq!(laid, picked);
        assert!(g.session.laying.is_none());
        assert_eq!(g.session.floor_items.len(), 1);
        // The lay mode again, and an arrow leaves it without a step.
        g.act(true);
        let x = g.session.player_x;
        g.walk(1, 0, Instant::now() + Duration::from_secs(1));
        assert!(g.session.laying.is_none());
        assert_eq!(g.session.player_x, x);
    }

    /// Carrying, the others see the carry pose; S throws what is carried:
    /// `PLI_THROWCARRIED` and empty hands go out, and the object flies.
    #[test]
    fn s_while_carrying_throws() {
        let mut g = game_on(ClientVersion::V603R7);
        g.session.carried = Some(seihai_core::offline::Carried {
            kind: 0,
            tiles: [[0, 0], [0, 0]],
        });
        g.set_pose(false);
        assert_eq!(g.pose.lock().unwrap().unwrap().gani, Some("carrystill"));
        assert_eq!(crate::render::own_figure(&g.session).pose, "carrystill");
        sent(&g);
        assert!(!g.swing(), "no swing while carrying");
        assert!(g.session.carried.is_none());
        assert_eq!(g.session.thrown.len(), 1);
        let out = sent(&g);
        assert_eq!(out[0], seihai_core::net::throw_carried_message());
        assert_eq!(out[1], seihai_core::net::carry_sprite_message(None));
    }

    /// Another player's bomb and arrow, as the network thread hands them
    /// over, go into the level; the bomb burns down there.
    #[test]
    fn other_players_bombs_and_arrows_arrive() {
        let mut g = game_on(ClientVersion::V205);
        g.level_objects.lock().unwrap().extend([
            LevelObject::Bomb {
                sender: 4,
                bomb: seihai_core::bombs::BombAdd {
                    x: 20.0,
                    y: 20.0,
                    power: 1,
                    fuse: 40,
                    owned: true,
                },
            },
            LevelObject::Arrow {
                sender: 4,
                arrow: seihai_core::arrows::ArrowAdd {
                    x: 10.0,
                    y: 10.0,
                    dir: 3,
                    replaces: false,
                    owned: true,
                    sprite: seihai_core::arrows::ARROW_SPRITE + 3,
                    kind: 0,
                },
            },
        ]);
        g.tick(0, 0, Instant::now());
        assert_eq!(g.session.placed_bombs.len(), 1);
        assert_eq!(g.session.arrows.len(), 1);
        assert!(g.level_objects.lock().unwrap().is_empty());
        run(&mut g, 60);
        assert!(g.session.placed_bombs.is_empty());
    }
}
