# Defense proposals — recovered Graal hacks / trainers

**Date:** 2026-10-31  
**Audience:** operators of Seihai-stack worlds (`seihai-gserver` / `seihai-gserver-v2`, `seihai-listserver`, staff RC/NC) and maintainers of `seihai-servertools`.  
**Method:** static analysis only (Drive listing, archive inventory, readmes, PE/strings hints). No hacks executed; no exploit PoCs.

## Confidence and gaps

| Item | Level | Notes |
|---|---|---|
| Overall threat picture for **Drive `Hacks/Trainers`** | **High** | Filenames + readmes + AutoIt strings + howto texts are consistent |
| **Version faker / CRC patch / packet logger** (adjacent client archives, not under `Hacks/` alone) | **Medium** | Inferred from names and Graal history; binaries not deeply RE’d here |
| Classic protocol / login / RC risks (from Seihai docs + `Graal_analysis`) | **High** | Documented in-repo |
| Sibling `INVENTORY.md` / `THREAT_TAXONOMY.md` | **Missing at write time** | First-pass plan; re-check when those land |
| Deep behaviour of every RpgWarrior EXE (Miner, Digger, Prayer, GTS, …) | **Medium–Low** | Classified by name + shared AutoIt pattern; per-tool RE incomplete |
| Network-level packet injectors / memory editors beyond these archives | **Unknown** | Not present in the `Hacks` folder listing seen so far |

**Sources used**

- Drive folder [Hacks](https://drive.google.com/drive/folders/1BAaQCiSUIfU8xBPj2oAR6qVKEJH9X1TK) → `Trainers/` (+ `AutoPrograms by RpgWarrior/`)
- Local `/workspace/recovered-hacks/` (partial downloads + `_download_manifest.json`)
- Adjacent client archives: v4 packet-logger builds, v5 CRC / “Version 5.120 Faker Hack”
- `/workspace/recovered-drive-tools/SEIHAI_SERVERTOOLS_MAP.md`, `seihai-servertools` design/checks
- `documentation/secure-login.md`, public-go / kick notes; `Graal_analysis/tools/server-tools-2001.md` (password-stealer ListServer)

---

## Threat summary (operator view)

Recovered material clusters into five operator-relevant classes:

1. **Client-side automation (AutoIt / SendKeys bots)** — AFK chat, random walk, water/dig/mine/prayer loops, “auto key”, load scripts. They drive the real client UI; the server mostly sees “normal” packets with unnatural timing/volume.
2. **Modified clients** — CRC integrity bypass + version-token fakery; packet loggers baked into client builds. Used to join servers that gate on version / tamper checks, or to study/alter traffic.
3. **Script / appearance macro files** — e.g. `bomb.txt` with chat + `setnick`/`setbody` spam (annoyance / social engineering, not a remote exploit by itself).
4. **Benign-looking utilities mixed in** — `capslock.zip` is a generic CapsLock toggler (noise in the archive; low game threat).
5. **Stack-adjacent historical threats** (not in `Hacks/` but critical for operators) — cleartext classic passwords; `localhost=true` open login; RC/NC staff channel; malware “listserver” that steals saved client credentials; missing listserver gserver allow-list.

---

## Technique cards

### T1 — AFK / chat bots (Zorg AFK Bot, AFK Scroll Bot, AutoIdle / AutoS / AutoD)

**How it works (high level)**  
External Windows programs focus the Graal chat field and inject keystrokes on a timer (fixed phrases every ~3s, or scrolling text). Goal: farm online-time / stay “active” without a player.

**Detection**

- Chat periodicity: same account sending N messages with near-fixed inter-arrival (e.g. 2.5–3.5s ±ε) for minutes.
- Phrase repetition / character-scroll patterns (one growing string).
- Online-time gain with near-zero movement, combat, or level changes.
- Optional: client idle vs. chat-only activity score.

**Prevention / hardening**

- Server policy: cap messages per minute per account/IP; progressive mute → kick → temp ban.
- Online-time / AFK rewards only for **meaningful activity** (movement + interaction thresholds), never for chat alone.
- Captcha / staff challenge on reward claim (rare, for high-value worlds).
- NPC scripts that grant AFK rewards should not trust client-side timers alone.

**Response**

- Mute → warn → jail/kick; snapshot chat log + timestamps.
- If farming a specific NPC: disable or rebalance that reward path.
- Document phrase samples for future rules (do not publish as a “bot guide”).

**seihai-servertools fit**

- Docs: operator policy section under `docs/lab/` or new `docs/ops/hardening.md`.
- Future `check` / ops helpers: sample log parsers (offline) for “periodic chat” — keep in `lat-core` or a small `ops` module; **enforcement lives in gserver**, not in the GUI alone.

---

### T2 — Activity automation (RpgWarrior: Water Get, Digger, Miner, Prayer, Random Walker, Auto Key, GTS)

**How it works (high level)**  
Compiled AutoIt-style tools: operator stands at a resource (well, dig site, …), starts the EXE, focuses Graal; the tool sends repeated key/mouse events (equip, face, use, walk). Same class as T1 but targeting gameplay actions instead of chat.

**Detection**

- Action-rate anomalies: use-item / interact / dig packets far above human max for sustained periods.
- Movement entropy: random-walker shows high turn rate with low goal-seeking; miners show tiny spatial variance + high interact rate.
- Session fingerprints: long sessions with one action type dominating (>95% of C→S game actions).
- Correlate IP → many accounts with identical timing profiles.

**Prevention / hardening**

- **Authoritative cooldowns** on server for gather/prayer/dig/water (client delay is not enough).
- Daily / hourly caps on resource yields per account and per IP.
- Anti-macro soft signals: require occasional facing change, inventory open, or movement between gathers.
- Rate-limit PLI action ids that correspond to gather loops (tune from lab captures of legitimate play — Seihai stress doc shows healthy C→S peaks; bots are flatter and longer).

**Response**

- Soft: silent yield nerf / “tired” state. Hard: kick + temp ban.
- Wipe ill-gotten stack if economy is at risk; restore from last good account snapshot if you keep them.

**seihai-servertools fit**

- World checker (`lat-core/check.rs`): flag NPC scripts that grant unbounded resources with no server-side cooldown comments/config keys (heuristic docs + optional lint).
- Setup wizard: when going public, warn if `maxplayers` / accounts look like a bot farm test world left open.

---

### T3 — Macro / appearance spam loaders (`GraalLoad` + script files like `bomb.txt`)

**How it works (high level)**  
Loader feeds a text script of chat lines and `/`-style or `set*` appearance commands into the client at high speed (countdown spam, nick/body flicker). Harassment and chat flood more than protocol break.

**Detection**

- Burst of `setnick` / appearance props or chat lines with sub-human intervals.
- Rapid prop churn without movement.

**Prevention**

- Rate-limit appearance prop updates and nick changes (e.g. 1 nick / 30s).
- Chat flood controls (T1).
- Filter empty / punctuation-only / countdown-only spam if it becomes a problem.

**Response**

- Mute + revert nick/appearance to last good values; escalate if targeted harassment.

---

### T4 — Version fakery + CRC / integrity patch

**Evidence**  
`Graal v5.00r7[Windows][CRC Patch][Version 5.120 Faker Hack].{exe,rar}` and CRC-only variants in client archive trees (`Main Client Application Archive` / v5).

**How it works (high level)**  
Patches the official client so integrity checks pass and/or the **version token** presented at login claims a different build (e.g. 5.12 while binary is 5.00r7). Lets a modified or outdated client onto servers that only check the token string / a weak CRC.

**Detection**

- Log login token + secondary signals: feature bits, first-packet shapes, prop sets, zlib level habits (Seihai already profiles these). Mismatch → quarantine.
- If you ship a Seihai-only world: require `SEI00001` secure login (`documentation/secure-login.md`) so classic fakers cannot present a sealed login without speaking the handshake.
- Optional: challenge–response after login (server asks for a capability only real 5.12 has); fail closed for high-security worlds.

**Prevention / hardening**

- **Do not trust the version string alone.** Bind privileges to server-side allowlists of known-good behaviours.
- Prefer Seihai `require_secure_login` for staff / economy worlds (blocks all classic clients — product decision).
- Keep a reject list of known-bad tokens only as a weak extra; focus on protocol shape.
- Listserver: record client version + IP + account for audits.

**Response**

- Disconnect with generic “outdated or modified client”; ban account if repeated.
- Rotate any temporary “version allow” lists after incidents.

**seihai-servertools fit**

- `setup-core` / go-public docs: explain version policy options when joining a listserver.
- Future: host-listserver wizard documents `require_secure_gserver_link` / account hashing (already partly in design).

---

### T5 — Packet logger / modified clients (protocol observation & potential injection)

**Evidence**  
`Graal v4.20r8[Windows][+Packet Logger v1/v2]` archives next to stock v4 clients.

**How it works (high level)**  
A client build that logs (and historically often can alter) the framed/ciphered game stream. Logging alone aids reverse engineering; the same class of patch is the usual base for **forged packets** (illegal warps, item claims, staff-looking messages) if the server trusts the client.

**Detection**

- Impossible state transitions: position delta beyond speed limits; claiming chests/items out of range; sending RC/NC ids on a player login; unknown message ids (GServer-v2 already kicks after repeated unknowns — good).
- Prop/values outside schema; negative/overflow counts.
- Sequence / cipher desync attempts (malformed multi-line frames) — Seihai client notes already stress one-message-per-line discipline; servers should drop and count.

**Prevention / hardening (highest priority for a real gserver)**

| Control | Where |
|---|---|
| **Server-authoritative movement** (speed, collision, level bounds) | gserver |
| **Server-authoritative inventory / chests / combat** | gserver + NPC server |
| Reject player connections that send **RC/NC-only message ids** | gserver |
| Strict parse: unknown id budget, max frame size, max messages/sec | gserver |
| Separate auth for RC (type 6) and NC; no default `super`/`super` in production | gserver + `seihai-servertools` checks |
| Secure login for Seihai (`SEI00001`) | gserver + listserver |
| Do not put secrets in client-visible scripts | world content / LAT review |

**Response**

- Immediate disconnect + IP throttle; freeze economy actions pending review.
- Diff account inventory vs last snapshot; roll back if needed.
- Capture server-side packet log for that session (operator-controlled).

**seihai-servertools fit**

- Expand `setup-core::checks` for public profile:
  - Block empty / default staff passwords if detectable in lab accounts files.
  - Warn if RC-capable staff accounts remain from lab.
  - Refuse `localhost=true` on non-loopback (already implemented).
- LAT upload path (`lat-core`): refuse upload of levels/NPC scripts that call dangerous patterns once you define a denylist (e.g. unrestricted `setplayerprop` abuse — policy TBD with world owners).
- Document: “never run untrusted classic ListServer.exe” (stealer — see below).

---

### T6 — Credential theft & unsafe hosting (stack-adjacent)

**Threats**

- Classic login: account/password in clear on the wire (zlib only) — `secure-login.md`.
- Historical **Win32 ListServer by “Rick” (2003)** steals `HKCU\Software\Graal\Client` saved logins — `Graal_analysis/tools/server-tools-2001.md`. **Do not run.**
- `localhost=true` with public bind ⇒ anyone logs in as anyone — already **Block** in `setup-core` checks.
- Listserver without gserver allow-list: rogue gservers can register — called out in go-public design; keep gserver registration port firewalled until allow-list ships.
- Default / lab accounts left on a public world.

**Detection**

- Unexpected listserver registrations; sudden staff logins from new IPs.
- Players reporting password reuse compromise.

**Prevention**

- Ship / require `seihai-listserver` with hashed accounts; host wizard already refuses open mode on outside binds.
- Publish Ed25519 fingerprint for `SEI00001`; pin with `--server-key` for staff.
- Firewall: expose only client ports; lock gserver↔listserver port to known peers.
- Force password change on first public publish; wipe lab accounts (`check_server_folder` already surfaces lab leftovers).

**Response**

- Rotate listserver and staff passwords; invalidate sessions; audit `staff` line and RC rights.
- If stealer ListServer was ever run on an admin PC: treat all saved Graal client passwords as burned.

---

### T7 — Malicious or “hacked” world content (`hackedstaff.nw` class)

**How it works (high level)**  
Level/NPC content named or designed to grant staff-like powers, spawn items, or break local rules when the **server executes scripts trustingly**. Filename alone is not proof; treat as a content-review hazard.

**Detection / prevention**

- LAT / `seihai-servertools check`: parse GS1; flag calls that escalate rights, warp without checks, or give items without quest flags (policy list maintained by the world).
- Dual-control: production uploads only via RC accounts with `w` rights; log every put/delete.
- Conflict check on upload (design already: re-download before put).

---

## Priority patch / build order (Seihai stack)

Ordered by operator impact vs. effort:

1. **Keep and extend go-public safety checks** (`setup-core/checks.rs`) — already blocks the worst misconfigs; add lab-account wipe reminders and “RC default password” warnings when detectable.
2. **Gserver: rate limits + authoritative cooldowns** for chat, gather, appearance — defeats T1–T3 without client changes.
3. **Gserver: movement / inventory / chest authority** — primary defense against T5 forgeries.
4. **Listserver gserver allow-list + hashed accounts** — finish what go-public design already assumes.
5. **Offer `SEI00001` secure login** on public Seihai-friendly worlds; document `require_secure_login` tradeoff.
6. **LAT world linter** in `lat-core/check.rs` for dangerous script patterns + missing asset refs.
7. **Ops runbooks** in `seihai-servertools/docs/ops/` (this report can be linked or copied).
8. **Monitoring queries** (log-based) for periodic chat, gather loops, version mismatches — export examples for owners.

---

## Concrete controls mapped to `seihai-servertools`

| Defense | Implement in |
|---|---|
| Refuse unsafe public bind / localhost / UPnP combos | `crates/setup-core/src/checks.rs` (**exists**) |
| Host-listserver: no open accounts on `0.0.0.0` | `setup-core` listserver checks (**exists**); docs reinforce firewall |
| Pre-public “bot/farm lab leftovers” finding | `check_server_folder` — list unexpected accounts, high `online` stats files |
| Operator hardening doc + checklist | `docs/ops/hardening.md` (new) + link from README |
| World script / link / gmap safety | `crates/lat-core/src/check.rs` |
| LAT upload audit trail | `lat-core` + later `seihai-rc` file put logging |
| Packet/rate policy | **Not** in servertools — track as requirements on `seihai-gserver` / fork |
| Secure login toggles | Document in setup wizard copy when gserver grows TOML flags |

**Do not** place recovered trainer binaries, packet loggers, or CRC patches into `seihai-servertools`. Keep under `recovered-hacks/`; analyse offline; rewrite only clean-room defenses.

---

## Suggested monitoring signals (log / metrics)

Use server logs or a thin sidecar; thresholds are starting points — calibrate in lab.

| Signal | Suggest alert |
|---|---|
| Chat messages / account / min | > 20 sustained 5+ min |
| Identical message repeat count | ≥ 10 in 5 min |
| Interact/use actions / min at fixed tile | > 40 for 10+ min |
| Position samples with speed > configured max | any |
| Unknown message ids | > 3 / session (align with GServer-v2) |
| Login version token ∉ allowlist | info; kick if strict mode |
| RC login from non-staff IP | page owner |
| New gserver registration | page listserver host |
| Failed passwords / account from many IPs | lockout |

---

## What this pass did *not* cover

- Full disassembly of version-faker / packet-logger binaries.
- Runtime behaviour of every RpgWarrior tool (Miner may still have been mid-download).
- Non-`Hacks` Drive malware beyond the documented stealer ListServer.
- iOS / modern GraalOnline anti-cheat (out of scope for classic Seihai stack).

When sibling `INVENTORY.md` / `THREAT_TAXONOMY.md` appear, merge IDs into these cards and bump confidence.
