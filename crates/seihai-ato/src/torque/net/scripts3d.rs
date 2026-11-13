//! The weapon scripts a Graal3D server sends over BigData (#463), run on
//! core's GS2 VM, and the `events` text they make the client send back.
//!
//! A `weaponscript` transfer holds GS2 bytecode: the seihai-gserver-v2
//! lab server compiles its `weapon3d` files, and a real 5.12 client runs
//! what it gets that way (lab 12). Source text is compiled here with
//! core's compiler as a fallback; the real client does not run source.
//!
//! Each weapon runs its top-level code and then `onCreated` when its script
//! comes. A transfer for a weapon (`s1` set) runs its `onReceiveText(s2,
//! s3, text)`. What the scripts do that the server must hear goes into one
//! `events` text (`graal3d/torque-engine-4x.md` in Graal_analysis):
//!
//! - a player flag or a `client.` variable set: `s <name>=<value>`;
//! - `triggerserver(kind, name, params…)`: `t kind,name,params…`.
//!
//! `local.` flags stay on the client. An unset flag has no line of its own
//! in the text, so it is only logged. Sleeps and timers run on the VM's
//! 50 ms frame, as for the 2D NPC scripts.

use super::graal_text::{Applied, ClientEvents};
use seihai_core::protocol::ClientVersion;
use seihai_core::scripting::engine::world::Effect;
use seihai_core::scripting::engine::{Era, Val};
use seihai_core::scripting::gs2::exec::{EffectHost, Host, Machine, Outcome, Root};
use seihai_core::scripting::gs2::Script;
use std::collections::BTreeMap;

/// The VM's frame, in milliseconds.
const FRAME_MS: u64 = 50;

/// The client whose VM rules a Torque protocol's scripts run under.
pub fn vm_version(protocol: u32) -> ClientVersion {
    match protocol {
        24 => ClientVersion::V512,
        21 => ClientVersion::V420R6,
        _ => ClientVersion::V3153,
    }
}

/// What a script asked to tell the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToServer {
    Flag {
        name: String,
        value: String,
    },
    /// `triggerserver`'s arguments as text.
    Trigger(Vec<String>),
}

/// Core's effect host, with the calls a 3D weapon makes to the server.
struct Host3d {
    inner: EffectHost,
    out: Vec<ToServer>,
}

impl Host for Host3d {
    fn command(&mut self, name: &str, args: &[Val]) -> Option<Val> {
        if name.eq_ignore_ascii_case("triggerserver") {
            self.out
                .push(ToServer::Trigger(args.iter().map(Val::text).collect()));
            return Some(Val::Num(0.0));
        }
        self.inner.command(name, args)
    }

    fn get_prop(&mut self, root: Root, name: &str) -> Option<Val> {
        self.inner.get_prop(root, name)
    }

    fn set_prop(&mut self, root: Root, name: &str, value: &Val) -> bool {
        // `client.` variables go to the server; the VM keeps the value too.
        if name.len() > 7 && name[..7].eq_ignore_ascii_case("client.") {
            self.out.push(ToServer::Flag {
                name: name.to_string(),
                value: value.text(),
            });
            return false;
        }
        self.inner.set_prop(root, name, value)
    }

    fn log(&mut self, line: String) {
        self.inner.log(line);
    }
}

/// What a machine's run waits for.
#[derive(Debug, Clone, PartialEq)]
enum Wait {
    Idle,
    Sleep(f64),
}

struct Running {
    machine: Machine,
    host: Host3d,
    wait: Wait,
}

/// The weapons' scripts that run, by weapon name.
pub struct Weapons3d {
    version: ClientVersion,
    running: BTreeMap<String, Running>,
    acc_ms: u64,
    /// What the scripts asked to send, not yet sent.
    pub events: ClientEvents,
    /// Lines for the log, without the `3d: ` prefix.
    pub log: Vec<String>,
}

/// A weapon script's bytes as a script: bytecode, or source compiled here.
pub fn script_of(bytes: &[u8]) -> Result<(Script, bool), String> {
    match Script::parse(bytes) {
        Ok(s) => Ok((s, false)),
        Err(e) => {
            let src = std::str::from_utf8(bytes).map_err(|_| format!("not GS2 bytecode: {e}"))?;
            seihai_core::scripting::gs2::compile::compile_gs2(src)
                .map(|s| (s, true))
                .map_err(|c| format!("not bytecode ({e}) and does not compile: {c:?}"))
        }
    }
}

impl Weapons3d {
    pub fn new(version: ClientVersion) -> Self {
        Weapons3d {
            version,
            running: BTreeMap::new(),
            acc_ms: 0,
            events: ClientEvents::default(),
            log: Vec::new(),
        }
    }

    /// The weapons with a script running.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.running.keys().map(String::as_str)
    }

    /// Act on what a server transfer did: start, stop or call a weapon.
    /// `flags` are the player's flags so far, for the scripts to read.
    pub fn applied(
        &mut self,
        a: &Applied,
        script: Option<&[u8]>,
        flags: &BTreeMap<String, String>,
    ) {
        match a {
            Applied::WeaponScript { weapon, .. } => {
                if let Some(bytes) = script {
                    self.start(weapon, bytes, flags);
                }
            }
            Applied::WeaponDeleted(w) => {
                if self.running.remove(w).is_some() {
                    self.log.push(format!("weapon {w} stopped"));
                }
            }
            Applied::ReceiveText {
                weapon,
                kind,
                option,
                text,
            } => {
                let args = [kind, option, text].map(|s| Val::Str(s.clone()));
                self.event(weapon, "onReceiveText", &args);
            }
            _ => {}
        }
    }

    /// Start (or restart) `weapon` with script `bytes`.
    pub fn start(&mut self, weapon: &str, bytes: &[u8], flags: &BTreeMap<String, String>) {
        let (script, compiled) = match script_of(bytes) {
            Ok(s) => s,
            Err(e) => {
                self.log
                    .push(format!("weapon {weapon} script not run: {e}"));
                return;
            }
        };
        let mut inner = EffectHost::new(Era::for_version(self.version), 0);
        inner.flags = flags
            .iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
            .collect();
        let mut r = Running {
            machine: Machine::for_version(script, self.version),
            host: Host3d {
                inner,
                out: Vec::new(),
            },
            wait: Wait::Idle,
        };
        let out = r.machine.run_top_level(&mut r.host);
        self.running.insert(weapon.to_string(), r);
        self.log.push(format!(
            "weapon {weapon} script runs{}",
            if compiled {
                " (source, compiled here)"
            } else {
                ""
            }
        ));
        self.ended(weapon, out);
        self.event(weapon, "onCreated", &[]);
    }

    /// Run `event` on `weapon` now, unless its run sleeps.
    fn event(&mut self, weapon: &str, event: &str, args: &[Val]) {
        let Some(r) = self.running.get_mut(weapon) else {
            return;
        };
        if r.wait != Wait::Idle {
            self.log
                .push(format!("weapon {weapon}: {event} dropped while it sleeps"));
            return;
        }
        let out = r.machine.call_event(event, args, &mut r.host);
        self.ended(weapon, out);
    }

    /// Advance the clock by `ms`: sleeps end, timers run.
    pub fn tick(&mut self, ms: u64) {
        self.acc_ms += ms;
        while self.acc_ms >= FRAME_MS {
            self.acc_ms -= FRAME_MS;
            let names: Vec<String> = self.running.keys().cloned().collect();
            for name in names {
                let Some(r) = self.running.get_mut(&name) else {
                    continue;
                };
                let resumed = match &mut r.wait {
                    Wait::Idle => None,
                    Wait::Sleep(left) => {
                        *left -= FRAME_MS as f64 / 1000.0;
                        (*left <= 1e-9).then(|| r.machine.resume(&mut r.host))
                    }
                };
                if let Some(out) = resumed {
                    r.wait = Wait::Idle;
                    self.ended(&name, out);
                }
                let Some(r) = self.running.get_mut(&name) else {
                    continue;
                };
                if r.wait == Wait::Idle {
                    let ran = r.machine.tick(&mut r.host);
                    for (_, out) in ran {
                        self.ended(&name, out);
                    }
                }
            }
        }
    }

    /// After a run: note how it ended, and take what it did.
    fn ended(&mut self, weapon: &str, out: Outcome) {
        let Some(r) = self.running.get_mut(weapon) else {
            return;
        };
        match out {
            Outcome::Sleep(t) => r.wait = Wait::Sleep(t),
            Outcome::Stopped(why) => self.log.push(format!("weapon {weapon} stopped: {why}")),
            _ => {}
        }
        for line in std::mem::take(&mut r.host.inner.log) {
            self.log.push(format!("weapon {weapon}: {line}"));
        }
        let mut sent = std::mem::take(&mut r.host.out);
        for e in std::mem::take(&mut r.host.inner.effects) {
            match e {
                Effect::SetFlag { name, value } => sent.push(ToServer::Flag {
                    name,
                    value: value.unwrap_or_default(),
                }),
                Effect::UnsetFlag { name } => self.log.push(format!(
                    "weapon {weapon}: unset {name} (not sent in 3D mode)"
                )),
                other => self
                    .log
                    .push(format!("weapon {weapon}: {other:?} (not done in 3D)")),
            }
        }
        for s in sent {
            match s {
                ToServer::Flag { name, value } => {
                    self.log
                        .push(format!("weapon {weapon}: sends s {name}={value}"));
                    self.events.set(&name, &value);
                }
                ToServer::Trigger(args) => {
                    let line = args.join(",");
                    self.log.push(format!("weapon {weapon}: sends t {line}"));
                    self.events.trigger_line(&line);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::scripting::gs2::compile::compile_gs2;

    fn bytecode(src: &str) -> Vec<u8> {
        compile_gs2(src).unwrap().write()
    }

    fn lines(w: &mut Weapons3d) -> Vec<String> {
        let evs = w.events.take(0);
        let mut asm = super::super::events::BigDataAssembler::default();
        let mut out = Vec::new();
        for e in &evs {
            if let Some(Ok(d)) = asm.feed(e) {
                assert_eq!(d.kind, b"events");
                out.extend(String::from_utf8(d.text).unwrap().lines().map(String::from));
            }
        }
        out
    }

    #[test]
    fn a_weapon_runs_on_created_and_its_flag_and_trigger_go_to_the_server() {
        let mut w = Weapons3d::new(ClientVersion::V512);
        let src = "function onCreated() {\n  client.speed = 3;\n  triggerserver(\"gui\", \"-Move3D\", \"g3dpos\", 1, 2, 3);\n}\n";
        w.start("-Move3D", &bytecode(src), &BTreeMap::new());
        assert_eq!(w.names().collect::<Vec<_>>(), ["-Move3D"]);
        assert_eq!(
            lines(&mut w),
            ["s client.speed=3", "t gui,-Move3D,g3dpos,1,2,3"]
        );
        assert!(w.events.is_empty());
    }

    #[test]
    fn source_text_is_compiled_and_junk_is_logged() {
        let mut w = Weapons3d::new(ClientVersion::V512);
        w.start(
            "-A",
            b"function onCreated() { client.a = 1; }",
            &BTreeMap::new(),
        );
        assert!(
            w.log.iter().any(|l| l.contains("compiled here")),
            "{:?}",
            w.log
        );
        assert_eq!(lines(&mut w), ["s client.a=1"]);
        w.start("-B", &[0xFF, 0xFE, 0x00], &BTreeMap::new());
        assert!(w.log.iter().any(|l| l.contains("-B script not run")));
        assert_eq!(w.names().count(), 1);
    }

    #[test]
    fn receive_text_goes_to_the_weapon_and_delete_stops_it() {
        let mut w = Weapons3d::new(ClientVersion::V512);
        let src = "function onReceiveText(kind, option, text) {\n  client.got = kind @ \"/\" @ option @ \"/\" @ text;\n}\n";
        w.start("-T", &bytecode(src), &BTreeMap::new());
        w.applied(
            &Applied::ReceiveText {
                weapon: "-T".into(),
                kind: "k".into(),
                option: "o".into(),
                text: "hello".into(),
            },
            None,
            &BTreeMap::new(),
        );
        assert_eq!(lines(&mut w), ["s client.got=k/o/hello"]);
        w.applied(&Applied::WeaponDeleted("-T".into()), None, &BTreeMap::new());
        assert_eq!(w.names().count(), 0);
    }

    #[test]
    fn a_sleep_ends_on_the_clock() {
        let mut w = Weapons3d::new(ClientVersion::V512);
        let src = "function onCreated() {\n  sleep(0.1);\n  client.woke = 1;\n}\n";
        w.start("-S", &bytecode(src), &BTreeMap::new());
        assert!(lines(&mut w).is_empty());
        w.tick(50);
        assert!(lines(&mut w).is_empty());
        w.tick(60);
        assert_eq!(lines(&mut w), ["s client.woke=1"]);
    }

    #[test]
    fn a_flag_the_server_already_set_is_not_sent_again() {
        let mut w = Weapons3d::new(ClientVersion::V512);
        let mut flags = BTreeMap::new();
        flags.insert("Door".to_string(), "open".to_string());
        let src = "function onCreated() {\n  set(\"door=open\");\n  set(\"lamp=on\");\n}\n";
        w.start("-F", &bytecode(src), &flags);
        assert_eq!(lines(&mut w), ["s lamp=on"]);
    }

    #[test]
    fn protocols_pick_their_clients_vm() {
        assert_eq!(vm_version(24), ClientVersion::V512);
        assert_eq!(vm_version(21), ClientVersion::V420R6);
        assert_eq!(vm_version(19), ClientVersion::V3153);
    }
}
