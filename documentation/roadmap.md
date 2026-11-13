# Seihai Roadmap

Five layers. The first four are largely understood; the fifth is where the unknowns are. Start with gmaps, then the script engine, then differential testing.

## 1. Data-driven client core

Single code path. Versions are profile rows, not branches.

- [x] Profile rows set framing, login token, id sets, field layouts and timers. Every row is checked against the Graal_analysis TOML (`profile_toml.tsv`, test `rows_match_version_profile_toml`).
- [x] CI check that fails when the tables and the code drift apart (the TOML check above, and the `version-chain.md`/`.tsv` staleness tests in `chain.rs`).
- [ ] Per-version replay tests. Each profile gets the real captured frames from the lab and the decompiles. Sixty-odd real frames already exist; every profile must decode and re-encode them byte for byte.

## 2. Wire parity, by version

- [x] Gmaps. Ids 49 and 186, cell props 43/44, and the crossing behaviour captured in the lab: grid model and `.gmap` reader, edge crossing and wrapped coordinate, cell-prop frame gated by profile (with 46 or the 2.22 tamper check in front), 186 applied at its cell, neighbour levels for collision and drawing, `PLI_ADJACENTLEVEL`. Cached-level crossings on a gmap are settled by the lab capture: the real client re-enters a level it holds with the ordinary crossing props, not the 30-plus-46 form, and Seihai matches (pinned by a test). Still open under gmaps: the `gr.x`/`gr.y` flags (see `gr-flags.md`), and, with no capture, 185 board heights, neighbours' NPCs/links/chests, a non-zero `PLI_ADJACENTLEVEL` mod time, the 171/172 map screens, and a run against a real server. See `documentation/protocol.md`, "Gmaps".
- [x] Cached level links. The 30-plus-46 form: a level the player has left is kept with its `PLO_LEVELMODTIME`, and a link back into it sends `PLI_LEVELWARPMOD` (30) with that mod time, then 46 and the props in the same frame (2.31), or the tamper check (2.22), before the server answers; the server's data-free answer is filled from the cache. Gated by `levels.cached_links`. Still open: the real client's disk cache (`GWEBL001`), cached crossings on a gmap, and a lab run of Seihai itself. See `documentation/protocol.md`, "Level changes".
- [x] Script-initiated warps. `setlevel`/`setlevel2` from NPC scripts: offline the client loads the level as before; online it sends the link request (`PLI_LEVELWARP`, or `PLI_LEVELWARPMOD` with a cached copy, by profile) and loads the level on the server's answer. A lab run of Seihai confirmed the frames on 2.31 and 2.22. Still open: what a real client sends (its level NPC scripts never ran in the lab; see `documentation/script-warps.md`). See `documentation/protocol.md`, "Script-initiated warps".
- [x] Remaining fields. Id 24 is the sender's weapon-script image list (the byte 9 is its full-list marker); id 46 is a content-free level-switch notice; id 178 is read with the clients' own comma-text reader. What a real service sends for 178 is still unobserved (`documentation/remaining-fields.md`).
- [ ] Early builds. A real 2.00-era listserver reply, which is still unobserved.

## 3. Gameplay the client decides on its own

- [x] Health and death. The 2.05 death sequence from the profile (`life.death`): 1.5 s timer, input lock to tick 27, dead bit in STATUS from tick 28, respawn hearts by AP, respawn at the saved level entry point, and the victim-sent `PLI_CLAIMPKER` (14). The same-name-group rule is in (2.05 only; later builds save on first entry and gmap-list crossings). Still open: the end-of-respawn save, the gmap list's order and timing, the dead gani, whether the real client sends CURPOWER on damage, the order of the respawn frame, and a lab run. See `documentation/health-death.md`.
- [x] Combat. Sword hits are judged by the victim's client (`life.combat` in the profile): the attacker's swing sprite (S 6 to 10), the victim 0 to 4 tiles ahead (both ends count) and under 2 to the side, AP 100 immune, no hit on a dead, hurt (40 ticks) or swimming player; `PLO_HURTPLAYER` (40) judged by range and capped at one heart; Seihai's own swing on the S key broadcasts the swing sprite. The damage per sword hit is the attacker's sword power in half hearts (±20, heals only at AP 40+, hurt mode always), behind `noplayerkilling`, guild, attacker-status and once-a-second gates. Still open: id 46 is decoded but not applied (a player's centre is not in the notes), NPC-sourced hits, knockback, the per-player field in the packet path, explosions, script `hitplayer`/`hitobjects` senders, the real swing frame times, and a lab run. See `documentation/combat.md`.
- [x] Damage formula and respawn "same name group" rule (item 7), from the 2.05, 2.17.1, 2.22 and 2.31 clients. Open points are in `documentation/damage-respawn.md`.
- [x] Chat popups, buddy, ignore and mute. Chat bubbles over the speaker go down after 3 s (a placeholder, see `documentation/chat-lists.md`); `buddyaccounts.txt`, `ignoredaccounts.txt` and `mutedaccounts.txt` are loaded from and saved to the settings directory; ignore and mute hide the player's bubble, chat, toalls and PMs; `/mute` sends `PLI_MUTEPLAYER` (43) where the profile has it. Still open: the F7-style popup menu with the mouse, History, Profile, status, the listserver buddy list (2.17 and later), and PMs from players the client has never seen. Swimming (head-only blit) and local emotes were done earlier.

## 4. Content and scripting

- [x] Script engine. A first working GS1 interpreter in `crates/seihai-core/src/scripting/engine/`: lexer, parser, flat-node runtime, deterministic 50 ms tick/timeout scheduler, the command surface with per-version vocabulary gating, level NPCs and weapons hooked into the offline session, unknown commands logged and ignored. Still open: `board[]`/`tiles[]` reads, string lists, `with`/`join`, online NPC script props, weapon-key input and timed `move`. See `documentation/seihaiscript.md`, "Implemented GS1 engine".
- [x] Audio playback. A sound engine (`crates/seihai-core/src/sound/`) behind a `SoundOut` trait: WAV decoder (PCM and IMA ADPCM), 8-voice cap, `+` loop prefix, pan, once-per-frame-entry GANI `PLAYSOUND` cues from `walk.gani`/`swim.gani`, missing files logged once. Null backend by default; the device backend is seihai-core's `audio` feature, shared by every client. `--no-sound` and `sound=0`. Still open: GS1 `play`, built-in effects, volume falloff, music, fetching missing sounds. See `documentation/audio.md`.
- [x] Assets. `AssetResolver` (`crates/seihai-core/src/assets/resolve.rs`) finds levels, GANIs, images, sounds and music in the 2.22 data folder for every version profile; `.graal` levels read links, baddies, NPCs, chests and signs; `seihai --check-assets` loads what each build uses and the `#[ignore]` tests do the same on the real folder for 1.39, 2.05, 2.17, 2.22 and 2.31. Real `.graal` levels (two supplied, 3113 in a player world) all parse; sign text fixed. Still open: sheets of other widths (1.39's own `pics1.png` is 1856 wide), GIF and MNG, tiledef matching, GANI sprite sheets the folder does not ship. See `documentation/assets.md`.
- [x] File transfer and caching. `FileRequester`/`FileSession` (`crates/seihai-core/src/filecache.rs`): a missing file is a `WANTFILE`, a local one an `UPDATEFILE` with its mtime (once per name per connection, four of each kind in flight, the rest queued), replies settle names, a failed name is not asked again, downloads are saved in place or by the clients' folder rules with the server's mtime, and only when `save_downloads` is on. Gated by the profile (`pli` 34, `files.modtime`, `files.download_maps`). Still open: the `GWEBL001` level cache, the login set of `UPDATEFILE`s, the file-index scan order, the save folders of 1.39/2.00 and 2.1 to 2.16, and a lab run. See `documentation/file-cache.md`.

## 5. Behaviour over time

- [x] Version chain (`crates/seihai-core/src/protocol/chain.rs`). Each build is a step with a small delta against the build before; `PROFILES` is the resolved chain and equals the old hand-written rows. A generated handler-change table (`seihai --version-chain`, `documentation/version-chain.md`/`.tsv`, checked by a test) lists what changed at each step with its source and what is unknown. Adding a version is one step, not new logic. Still open: the unknown deltas of 2.1 to 2.16 and 2.3 (no binaries), and per-step layouts that are not profile fields. See `documentation/version-chain.md`.
- [x] Differential testing. Tools, lab runner and first results are in: a capture-to-frame-sequence decoder (`difftest trace`), a differ with a ranked report (`difftest diff`), and a private-namespace scenario runner (`tools/difftest/lab.sh`). First run: 2.22 and 2.31, login, gmap crossing, chat and door/warp, real client against Seihai. Ranked divergences are in `documentation/difftest.md`. Not yet covered: the real 2.17 client (its launcher needs its own UI steps), the server-side warp scenario, and running it in CI.
- [ ] Public-server checks. Approved for read-only checks, then chat, warps and file transfer, with a few normal lines and no uploads. The build box cannot reach the listserver ports, so the user ran the list checks on their own machine on 2026-10-25: 14921 refused, 14923 answers a classic login with the legacy list, 14922 answers the gen-5 login; the game-server checks are still to do. Results, open points and the checklist are in `documentation/public-checks.md`.
- [x] Tooling. A capture viewer and a layout explorer fed by the .tsv tables: `seihai capture show|layout|tables`, with `documentation/packet-layouts.tsv` as the field-layout table (`documentation/capture-viewer.md`). Not done: a GUI, live capture, and layouts for the ids the tables do not list yet.

## One program, one start window

`seihai` is the one program that ships: Mae, Ato, Issho and G3D are linked into it as libraries, so the code they share ships once (13.8 MB, against 57 MB for the five separate programs). With no arguments it opens one start window for every line. Tracking issue: #469.

- [x] Shared start window (`seihai-play::start`): every version grouped by client, then Graal3D; the server list; Join runs that client, logged in at once. A `3 ` server joined with a 3.x to 5.x build gets `--graal3d`. A lost connection comes back to the window.
- [x] Each client's whole program is its library's `app::main`; `seihai` runs it in-process. `seihai mae|ato|issho|g3d`, or the program named `seihai-ato` (and so on), runs a client on its own. The crates' binaries are for development and are not shipped.
- [x] Release profile: strip, LTO, one codegen unit. release.yml ships only `seihai`.
- [x] Mae's own window hands a 6.x Join to Issho.
- [x] Mae's version picker folds into the shared window (#470): offline play, the tutorial, the offline-only builds and per-version asset folders are in the shared window; inside `seihai`, Mae with no arguments and a lost connection (also after `--online`) come back to it. Mae's own window stays for the `seihai-mae` development binary.
- [ ] Lab run on Windows 10, Linux and macOS (#471).
- [ ] Ato's start window and server list reuse `seihai-play::start`.

## Issho (6.x)

`seihai-issho` is the 6.x client line (the `quattro_lib` builds). It runs the game loop Ato runs, from `seihai-play`, and does not depend on Ato. Tracking issue: #440.

- [x] Game loop. Log in with core's client session, draw the level the server sends flat, walk it with the arrows, cross links and gmap edges, show the other players and the server's board and NPC changes, and play the walk's sounds through core's `SoundEngine`, asking the server for missing ones with `WANTFILE` (`--no-sound` turns it off). `--level` plays level files offline; `--no-window` only logs. Ato's `game` and `render` moved to `seihai-play` so both lines share them.
- [ ] Lab run against the lab server stack, every 6.x row, in a headless VM (#441).
- [x] Start window and server list; the launcher and mae hand 6.x picks to Issho (#442). Issho has no start window of its own: the shared one hands a 6.x pick over, logged in at once; a pick Issho refuses comes back to the start window with a note.
- [x] Characters from ganis, and the renderer moved to `seihai-draw` (#443).
- [x] The 6.x window size and HUD (#444): 1024 x 768, and the default status display (`seihai-play::hud`), the 2.05 routine on the 6.x window, which a lab run of the real 6.00r2 and 6.03r7 put on the same pixels. Counters from the server's player props (core's `client::OwnStats`).
- [x] Chat, sword, items and the rest of mae's flat play, in the shared loop (#445). The leftovers are in #497.
  - [x] Chat: a chat line (Tab or Enter), the bubbles over the players and the player's own, toalls, PMs and admin lines (Ato gets them too).
  - [x] Sword: S swings (core's swing, the build's combat rules), the player drawn in `sword.gani`, the cut in front of it from the build's `arrays.dat` sent to the server as a board change, and the swing shown to the others (the GANI prop on a GANI build, the sprite otherwise).
  - [x] Mae's actions (`seihai-play::game::actions`, Ato gets them too): the spin attack (S again during a swing, with the server's has-spin STATUS bit), the weapon key D (the built-in Bomb, Superbomb, Joltbomb and Bow run core's scripts and send `PLI_BOMBADD` / `PLI_ARROWADD` and the new counts; a server weapon's GS1 script gets `weaponfired`), the lay mode that drops items (A with S held picks, A lays, sent as `PLI_ITEMADD`), and the item list on Q that picks the weapon D fires. Bombs, blasts, arrows and floor items are drawn from `sprites.png`, `bomb1.png` and `bomb2.png` (`seihai-play::effects`). A lifts, throws (S while carrying too), opens chests and grabs, sending CARRYSPRITE, `PLI_THROWCARRIED` and `PLI_OPENCHEST`, with the carry pose; other players' bombs, explosions, arrows and throws come in through core's `Live::level_objects` and `Live::throws`. Not yet: GS2 weapon scripts (6.x), hiding under a carried object, other players' carried objects drawn over them, and baddies.
  - [x] Being hit, health and death (`seihai-play::game::life`; Ato gets them too, since it sets the build in #487): other players' swings and `PLO_HURTPLAYER` / `PLO_HITOBJECTS` (core's `Live::hits`) judged on the player's side with core's `receive_all`, the hearts from the server's CURPOWER, the hurt and dead poses, the death on core's life clock with the kill claim, the dead bit and the hearts report sent as Mae sends them, and a respawn on an arrow, S, A or D key.
  - [x] Signs, sitting and ledges: a sign core opens on the walk shows three lines a page (Up back, the other game keys on, Escape closes; the keys stay off the player until let go), the player sits on a chair (`sit.gani`), and ledge jumps, pulls and a hit's knockback run on the 50 ms clock and send the position.
  - [x] Other players' floor items: the server's `PLO_ITEMADD` / `PLO_ITEMDEL` (core's `Live::items`, 0.1.4) join and leave the level, and walking over one picks it up.
  - [ ] The push and pull poses.
- [ ] Sound (#446).
  - [x] The 6.x effect formats (core's content sniffing: A-law, mu-law, MPEG in WAV, MP3, Ogg Vorbis), the sounds the server sends, and a lab check of `WANTFILE` (#441).
  - [ ] GS2 script `play`: core's GS2 `EffectHost` has no `play` yet (GS1 scripts' `play` already sounds).
  - [ ] Music: core's `SoundEngine` skips anything that is not `.wav`; the 6.x worlds use `.mp3` and `.mid`.
- [x] GS2 client scripts, clean-room in seihai-core (#447).
  - [x] The VM in seihai-core, one profile per build (`scripting::gs2::exec::Machine::for_version`).
  - [x] The client runtime in `seihai-play::scripts`: the server NPCs' scripts start when their level arrives (top level, `onCreated`, `onPlayerEnters`), tick on the 50 ms frame (`onTimeout`, `scheduleevent`, `sleep`, `waitfor`), and run `onPlayerTouchsMe` and `onPlayerChats`. Their effects land through core's `SessionScripts` (NPC image, visibility, place, `message` drawn as a bubble, `setlevel`, player props), and player flags go to the server as `PLI_FLAGSET` / `PLI_FLAGDEL`. Issho and Ato (since #487) turn it on through `Game::set_version`.
  - [ ] Weapon scripts, scripts sent after the level (an NPC that comes online later), gmap neighbours' scripts kept across crossings, `player.chat` rewrites, and the rest of the events (#497). A lab run with #441.
- [x] The 6.00r4 profile row and `resolveHost` (#448).

## Remaining

- Differential-testing divergences (item 15): rectangle links, login props, 2.22 flags, early 46, walk speed. Running in the private lab.
- Level editor: notes in Graal_analysis `tools/level-editor/` (2.22 reference, version deltas, Seihai tracking checklist); the checklist, the first CLI slice (open, list, paint, save) and a window (palette, layers 0 and 1, stamps, object markers, gmap grid, save) are in `documentation/level-editor.md` and `tools/level-editor/`.
- Public-server checks: list checks done from the user's machine (2026-10-25); game-server checks still to do. Results and checklist in `documentation/public-checks.md`.
- Early builds: real 2.00-era listserver reply still unobserved.
- Graal3D (February 2000): planned as its own track in `documentation/plannedmilestones.md` (M1 client fidelity G3D-1/2/4/6/7, M2 admin + real-client lab G3D-5/8, M3 Rust host G3D-9, M4 `.x`+BMP world content, M5 servertools lab knock-on). `seihai-g3d` already plays against the original / reference servers (#331, #332). Worlds are DirectX `.x` + local BMPs, not 2D levels; `seihai-gserver` does not speak `GR3DV002`. Torque 3D (3.x–5.x) stays held as G3D-3 / Ato Stage 4. Task detail: `crates/seihai-g3d/ARCHITECTURE.md`.
- Walking collision: the real six-probe body-box routine is documented in `documentation/walking-collision-note.md` (cross-link). The earlier SDL single-point test is superseded.

## Completed (prior phases)

### Phase 1 — Protocol & Minimal Client
- [x] Packet framing notes + classic login line
- [x] Packet ids locked against lab captures
- [x] Listserver login against a real server
- [x] Gserver connection + version handshake
- [x] Level name / basic tile data reception
- [x] Player position codecs (classic ×2, precise ×16)
- [x] Simple chat

### Phase 2 — Assets & Rendering
- [x] Text level loader (GLEVNW01)
- [x] Binary loaders (.graal, .zelda): `.zelda` (Z3-V1.03/.04) has its own strict reader, profile gate and `--level-info` / `--check-assets` support; see `assets.md`
- [x] Tile-type table (text or arrays.dat)
- [x] ASCII / PPM board view
- [x] Sprite sheet / tileset loading
- [x] Tile renderer (software, minifb window)
- [x] Player sprite compositing (4 directions)
- [x] Camera follow

### Phase 3 — Seihaiscript GS1 Subset
- [x] Bytecode parser + walker
- [x] Minimal interpreter (arithmetic, comparison, jump)
- [x] Command stubs (`message`, `say`, `setimg`, `move`, …)

### Phase 4 — Polish & Compatibility
- [x] Gani parser + stepper
