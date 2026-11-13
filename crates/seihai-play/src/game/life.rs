//! Being hit, health and death in the shared loop (#445), as `seihai-mae`
//! runs them: other players' hits are judged on the player's own side
//! (core's `OfflineSession::receive_all`), the hearts follow the server's
//! CURPOWER, the death runs on core's 50 ms life clock, and a movement or
//! action key respawns. What the real clients send for each goes out
//! through [`Live::send`](seihai_core::client::Live): the hearts report
//! (CURPOWER and the hurt or dead GANI), the kill claim, and the dead bit
//! of STATUS.

use super::Game;
use seihai_core::client::{IncomingHits, OwnStats};
use seihai_core::life::LifeEvent;
use seihai_core::offline::RespawnTo;
use seihai_core::protocol::ClientVersion;
use std::time::Instant;

/// The game's side of health and death.
pub(super) struct LifeState {
    /// The build, once [`Game::set_version`] named it: no hits are judged
    /// and nothing is sent before.
    pub version: Option<ClientVersion>,
    /// When the life clock last ran.
    pub clock: Instant,
    /// The server's STATUS without the dead and paused bits: what the dead
    /// bit is sent with.
    pub status_base: u8,
    /// The hide bits last sent in STATUS (core's `hide_status`).
    pub hide_told: u8,
}

impl Default for LifeState {
    fn default() -> Self {
        LifeState {
            version: None,
            clock: Instant::now(),
            status_base: 0,
            hide_told: 0,
        }
    }
}

impl Game {
    /// The server's hearts, heart containers, AP and STATUS into the
    /// session: only what changed since `was`, so a hit taken here is not
    /// undone by an older CURPOWER.
    pub(super) fn take_own_power(&mut self, was: &OwnStats, own: &OwnStats) {
        let changed = |a: Option<u8>, b: Option<u8>| b.filter(|_| a != b);
        self.session.apply_server_power(
            changed(was.max_power, own.max_power),
            changed(was.cur_power, own.cur_power),
            changed(was.alignment, own.alignment),
        );
        if let Some(s) = own.status {
            self.life.status_base = seihai_core::offline::server_status_base(s);
        }
    }

    /// Judge the hits the network thread handed over. True when one was
    /// taken (the hearts went down and hurt mode started).
    pub(super) fn take_hits(&mut self) -> bool {
        let got: IncomingHits =
            std::mem::take(&mut *self.hits.lock().unwrap_or_else(|e| e.into_inner()));
        if got.is_empty() || self.life.version.is_none() || self.session.current_level.is_none() {
            return false;
        }
        // The client's second-of-day clock (UTC) for its once-a-second
        // limit per attacker.
        let second = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| (d.as_secs() % 86_400) as u32);
        let mut took = false;
        for r in self.session.receive_all(
            &got.others,
            got.own_cell,
            got.nickname.as_deref(),
            second,
            &got.swings,
            &got.hits,
        ) {
            match r.outcome {
                Ok(half) => {
                    took = true;
                    eprintln!("hit by {:?} ({:?}): {half} half hearts", r.attacker, r.kind);
                }
                Err(why) => eprintln!("hit by {:?} ({:?}) not taken: {why:?}", r.attacker, r.kind),
            }
        }
        if took {
            self.set_pose(self.session.walking);
            self.dirty = true;
        }
        took
    }

    /// Run the life clock to `now`: the death timer, and the messages for
    /// what happened (a death, the dead bit, a respawn, the hearts report).
    pub(super) fn tick_life(&mut self, now: Instant) {
        let Some(version) = self.life.version else {
            return;
        };
        let ms = now.saturating_duration_since(self.life.clock).as_millis() as u64;
        self.life.clock = now;
        if self.session.current_level.is_none() {
            return;
        }
        let events = self.session.tick_life(ms);
        self.life_events(version, events);
    }

    /// A movement or action key while dead: respawn, if the death allows
    /// it yet. True when the player came back.
    pub fn respawn(&mut self) -> bool {
        let Some(version) = self.life.version else {
            return false;
        };
        if !self.session.life.can_respawn() {
            return false;
        }
        let online = !self.offline;
        let Some(to) = self.session.respawn(online) else {
            return false;
        };
        eprintln!("respawn: {to:?}");
        // Online the level is asked for again at the respawn point, as an
        // ordinary link. On the same level the server only moves the
        // player, and the real client walks at once (2.05, 2026-10-29).
        let dest = match &to {
            RespawnTo::Saved(e) => Some(e.level.clone()),
            RespawnTo::Default { .. } => self.level_here.clone(),
            RespawnTo::InPlace => None,
        };
        if let (Some(dest), true) = (dest, online) {
            let same = self
                .level_here
                .as_deref()
                .is_some_and(|h| h.eq_ignore_ascii_case(&dest));
            let (x, y) = (self.session.player_x, self.session.player_y);
            if same {
                self.session.arrive(x, y);
                self.ask_link(&dest, x, y, false);
            } else {
                self.session.pending_link = Some((dest, x, y));
                self.follow_link();
            }
        } else if let (RespawnTo::Saved(e), false) = (&to, online) {
            let here = self.session.level_name();
            if !here.is_some_and(|h| h.eq_ignore_ascii_case(&e.level)) {
                let (level, x, y) = (e.level.clone(), e.x, e.y);
                if let Err(err) = self.enter_offline(&level, Some((x, y))) {
                    eprintln!("respawn: {level}: {err}");
                }
            }
            self.session.arrive(e.x, e.y);
        }
        let events = self.session.life.take_events();
        self.life_events(version, events);
        self.set_pose(false);
        self.dirty = true;
        true
    }

    /// Hiding under a carried object, or coming out, goes to the server as
    /// STATUS with the hidden bit (0x02) set or cleared, once the level is
    /// in, as Mae sends it (`pause_message_bits`). The position goes with
    /// the next pose tick.
    pub(super) fn send_hide_status(&mut self) {
        let bits = self.session.hide_status();
        if bits == self.life.hide_told || self.session.current_level.is_none() {
            return;
        }
        self.life.hide_told = bits;
        let Some(version) = self.life.version else {
            return;
        };
        if self.offline {
            return;
        }
        let (status, _) =
            seihai_core::offline::pause_message_bits(version, self.life.status_base, bits);
        self.send
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(seihai_core::life::status_message(status));
    }

    /// Send what the real client sends for these life events, with the
    /// hearts report, in one frame (as the real 2.00 sends a death).
    fn life_events(&mut self, version: ClientVersion, events: Vec<LifeEvent>) {
        let mut msgs = Vec::new();
        for ev in events {
            if let LifeEvent::Died { attacker } = ev {
                eprintln!("died (attacker {attacker:?})");
                if let Some(sc) = self.actions.scripts.as_mut() {
                    sc.player_dies(&mut self.session);
                }
                self.set_pose(false);
                self.dirty = true;
            }
            msgs.extend(seihai_core::life::messages_for(
                ev,
                version,
                self.life.status_base,
            ));
        }
        // A killing hit's report waits for the death itself: the real 2.00
        // sends the claim, CURPOWER 0 and GANI `dead` together (lab,
        // 2026-10-30).
        if version.death_rules().is_some() && !self.session.death_pending() {
            if let Some(report) = self.session.take_power_report() {
                if let Some(m) = report.message(version.profile().ui.ganis) {
                    msgs.push(m);
                }
            }
        }
        if !msgs.is_empty() && !self.offline {
            self.send
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend(msgs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Event, Game};
    use seihai_core::assets::Level;
    use seihai_core::client::{IncomingHits, Live};
    use seihai_core::combat::{HurtPlayer, IncomingHit};
    use seihai_core::protocol::ClientVersion;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    const V: ClientVersion = ClientVersion::V205;

    /// An online game on a level, with the build set and the network
    /// thread's half kept.
    fn game() -> (Game, Live, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        let mut g = Game::new(rx, None, true).unwrap();
        g.offline = false;
        let live = Live::default();
        g.attach(live.clone());
        g.set_version(V);
        g.session.current_level = Some(Level::from_board(&[0; 4096]));
        g.session.arrive(30.0, 30.0);
        g.level_here = Some("lab.nw".into());
        (g, live, tx)
    }

    /// Player 2 next to the player, as the server places them.
    fn attacker() -> Vec<seihai_core::protocol::OtherPlayer> {
        let mut st = seihai_core::protocol::SessionState::for_version(V);
        // PLO_OTHERPLPROPS (8) for player 2: X (15) and Y (16) in half tiles.
        st.apply_body(&[
            8 + 0x20,
            0x20,
            2 + 0x20,
            15 + 0x20,
            62 + 0x20,
            16 + 0x20,
            60 + 0x20,
            b'\n',
        ]);
        st.others
    }

    fn hurt(half: i32) -> IncomingHits {
        IncomingHits {
            others: attacker(),
            hits: vec![IncomingHit::Hurt(HurtPlayer {
                attacker: 2,
                dx: 0.0,
                dy: 0.0,
                half,
                npc: None,
            })],
            ..Default::default()
        }
    }

    /// A `PLO_HURTPLAYER` takes the hearts down, starts hurt mode, and the
    /// hearts report goes to the server.
    #[test]
    fn a_hit_takes_hearts_and_reports_them() {
        let (mut g, live, _tx) = game();
        let before = g.session.hearts_half();
        *live.hits.lock().unwrap() = hurt(2);
        g.tick(0, 0, Instant::now());
        assert_eq!(g.session.hearts_half(), before - 2);
        assert!(g.session.combat.is_hurt());
        assert!(!live.send.lock().unwrap().is_empty(), "the hearts report");
        assert!(live.hits.lock().unwrap().is_empty(), "taken once");
    }

    /// The HUD shows the hearts a hit took, without waiting for a
    /// CURPOWER the server does not echo.
    #[test]
    fn the_hud_hearts_drop_with_a_hit() {
        let (mut g, live, _tx) = game();
        let before = g.hud_values().half_hearts;
        assert_eq!(before, g.session.hearts_half());
        *live.hits.lock().unwrap() = hurt(2);
        g.tick(0, 0, Instant::now());
        assert_eq!(g.hud_values().half_hearts, before - 2);
        assert_eq!(g.hud_values().max_half_hearts, g.session.max_hearts);
    }

    /// Before the build is set nothing is judged.
    #[test]
    fn no_build_no_hits() {
        let (tx, rx) = mpsc::channel::<Event>();
        let mut g = Game::new(rx, None, true).unwrap();
        drop(tx);
        g.session.current_level = Some(Level::from_board(&[0; 4096]));
        let before = g.session.hearts_half();
        *g.hits.lock().unwrap() = hurt(2);
        assert!(!g.take_hits());
        assert_eq!(g.session.hearts_half(), before);
    }

    /// A killing hit: the death runs on the life clock, and once it allows
    /// a respawn an arrow key brings the player back with hearts.
    #[test]
    fn a_killing_hit_then_a_key_respawns() {
        let (mut g, live, _tx) = game();
        let t0 = Instant::now();
        // One heart left: the next hit kills.
        g.session.apply_server_power(None, Some(2), None);
        *live.hits.lock().unwrap() = hurt(2);
        g.tick(0, 0, t0);
        let mut t = t0;
        for _ in 0..200 {
            t += Duration::from_millis(50);
            g.tick(0, 0, t);
            if g.session.life.can_respawn() {
                break;
            }
        }
        assert!(g.session.life.is_dead());
        assert!(g.session.life.can_respawn());
        let sent = live.send.lock().unwrap().len();
        assert!(sent > 0, "the death went to the server");
        g.tick(1, 0, t + Duration::from_millis(50));
        assert!(!g.session.life.is_dead());
        assert!(g.session.hearts_half() > 0);
        assert!(live.link.lock().unwrap().is_some(), "the level asked again");
    }
}
