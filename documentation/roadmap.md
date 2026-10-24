# Seihai Roadmap

Five layers. The first four are largely understood; the fifth is where the unknowns are. Start with gmaps, then the script engine, then differential testing.

## 1. Data-driven client core

Single code path. Versions are profile rows, not branches.

- [ ] Profile rows set framing, login token, id sets, field layouts and timers. All 26 rows come from the Graal_analysis TOML.
- [ ] CI check that fails when the tables and the code drift apart.
- [ ] Per-version replay tests. Each profile gets the real captured frames from the lab and the decompiles. Sixty-odd real frames already exist; every profile must decode and re-encode them byte for byte.

## 2. Wire parity, by version

- [x] Gmaps. Ids 49 and 186, cell props 43/44, and the crossing behaviour captured in the lab: grid model and `.gmap` reader, edge crossing and wrapped coordinate, cell-prop frame gated by profile (with 46 or the 2.22 tamper check in front), 186 applied at its cell, neighbour levels for collision and drawing, `PLI_ADJACENTLEVEL`. Still open under gmaps: 185 board heights, the `gr.x`/`gr.y` flags, neighbours' NPCs/links/chests, cached-level crossings on a gmap, the 171/172 map screens, and a run against a real server. See `documentation/protocol.md`, "Gmaps".
- [x] Cached level links. The 30-plus-46 form: a level the player has left is kept with its `PLO_LEVELMODTIME`, and a link back into it sends `PLI_LEVELWARPMOD` (30) with that mod time, then 46 and the props in the same frame (2.31), or the tamper check (2.22), before the server answers; the server's data-free answer is filled from the cache. Gated by `levels.cached_links`. Still open: the real client's disk cache (`GWEBL001`), cached crossings on a gmap, and a lab run of Seihai itself. See `documentation/protocol.md`, "Level changes".
- [x] Script-initiated warps. `setlevel`/`setlevel2` from NPC scripts: offline the client loads the level as before; online it sends the link request (`PLI_LEVELWARP`, or `PLI_LEVELWARPMOD` with a cached copy, by profile) and loads the level on the server's answer. A lab run of Seihai confirmed the frames on 2.31 and 2.22. Still open: what a real client sends (its level NPC scripts never ran in the lab; see `documentation/script-warps.md`). See `documentation/protocol.md`, "Script-initiated warps".
- [ ] Remaining fields. The unresolved id 24 and 46 meanings, and the field order for id 178 when a real service sends it.
- [ ] Early builds. A real 2.00-era listserver reply, which is still unobserved.

## 3. Gameplay the client decides on its own

- [x] Health and death. The 2.05 death sequence from the profile (`life.death`): 1.5 s timer, input lock to tick 27, dead bit in STATUS from tick 28, respawn hearts by AP, respawn at the saved level entry point, and the victim-sent `PLI_CLAIMPKER` (14). The same-name-group rule is in (2.05 only; later builds save on first entry and gmap-list crossings). Still open: the end-of-respawn save, the gmap list's order and timing, the dead gani, whether the real client sends CURPOWER on damage, the order of the respawn frame, and a lab run. See `documentation/health-death.md`.
- [x] Combat. Sword hits are judged by the victim's client (`life.combat` in the profile): the attacker's swing sprite (S 6 to 10), the victim 0 to 4 tiles ahead (both ends count) and under 2 to the side, AP 100 immune, no hit on a dead, hurt (40 ticks) or swimming player; `PLO_HURTPLAYER` (40) judged by range and capped at one heart; Seihai's own swing on the S key broadcasts the swing sprite. The damage per sword hit is the attacker's sword power in half hearts (±20, heals only at AP 40+, hurt mode always), behind `noplayerkilling`, guild, attacker-status and once-a-second gates. Still open: id 46 is decoded but not applied (a player's centre is not in the notes), NPC-sourced hits, knockback, the per-player field in the packet path, explosions, script `hitplayer`/`hitobjects` senders, the real swing frame times, and a lab run. See `documentation/combat.md`.
- [ ] Swimming, emotes and chat. Head-only swim blit is done; emotes are local only. Chat popups and the buddy, ignore and mute files remain.
- [x] Damage formula and respawn "same name group" rule (item 7), from the 2.05, 2.17.1, 2.22 and 2.31 clients. Open points are in `documentation/damage-respawn.md`.
- [x] Health and death. The 2.05 death sequence from the profile (`life.death`): 1.5 s timer, input lock to tick 27, dead bit in STATUS from tick 28, respawn hearts by AP, respawn at the saved level entry point, and the victim-sent `PLI_CLAIMPKER` (14). Still open: the damage formula and the same-name-group rule (item 7), the dead gani, whether the real client sends CURPOWER on damage, the order of the respawn frame, and a lab run. See `documentation/health-death.md`.
- [x] Combat. Sword hits are judged by the victim's client (`life.combat` in the profile): the attacker's swing sprite (S 6 to 10), the victim 0 to 4 tiles ahead (both ends count) and under 2 to the side, AP 100 immune, no hit on a dead, hurt (40 ticks) or swimming player; `PLO_HURTPLAYER` (40) judged by range and capped at one heart; Seihai's own swing on the S key broadcasts the swing sprite. The damage per sword hit is a labelled placeholder (item 7). Still open: id 46 is decoded but not applied (a player's centre is not in the notes), NPC-sourced hits, knockback, explosions, script `hitplayer`/`hitobjects` senders, the real swing frame times, and a lab run. See `documentation/combat.md`.
- [x] Chat popups, buddy, ignore and mute. Chat bubbles over the speaker go down after 3 s (a placeholder, see `documentation/chat-lists.md`); `buddyaccounts.txt`, `ignoredaccounts.txt` and `mutedaccounts.txt` are loaded from and saved to the settings directory; ignore and mute hide the player's bubble, chat, toalls and PMs; `/mute` sends `PLI_MUTEPLAYER` (43) where the profile has it. Still open: the F7-style popup menu with the mouse, History, Profile, status, the listserver buddy list (2.17 and later), and PMs from players the client has never seen. Swimming (head-only blit) and local emotes were done earlier.
- [ ] Unresolved rules. The damage formula and the respawn "same name group" rule.

## 4. Content and scripting

- [x] Script engine. A first working GS1 interpreter in `client/src/scripting/engine/`: lexer, parser, flat-node runtime, deterministic 50 ms tick/timeout scheduler, the command surface with per-version vocabulary gating, level NPCs and weapons hooked into the offline session, unknown commands logged and ignored. Still open: `board[]`/`tiles[]` reads, string lists, `with`/`join`, online NPC script props, weapon-key input and timed `move`. See `documentation/seihaiscript.md`, "Implemented GS1 engine".
- [x] Audio playback. A sound engine (`client/src/sound/`) behind a `SoundOut` trait: WAV decoder (PCM and IMA ADPCM), 8-voice cap, `+` loop prefix, pan, once-per-frame-entry GANI `PLAYSOUND` cues from `walk.gani`/`swim.gani`, missing files logged once. Null backend by default; the device backend is the `audio-rodio` feature, not built or run yet. `--no-sound` and `sound=0`. Still open: GS1 `play`, built-in effects, volume falloff, music, fetching missing sounds. See `documentation/audio.md`.
- [x] Assets. `AssetResolver` (`client/src/assets/resolve.rs`) finds levels, GANIs, images, sounds and music in the 2.22 data folder for every version profile; `.graal` levels read links, baddies, NPCs, chests and signs; `seihai --check-assets` loads what each build uses and the `#[ignore]` tests do the same on the real folder for 1.39, 2.05, 2.17, 2.22 and 2.31. Real `.graal` levels (two supplied, 3113 in a player world) all parse; sign text fixed. Still open: sheets of other widths (1.39's own `pics1.png` is 1856 wide), GIF and MNG, tiledef matching, GANI sprite sheets the folder does not ship. See `documentation/assets.md`.
- [x] File transfer and caching. `FileRequester`/`FileSession` (`client/src/filecache.rs`): a missing file is a `WANTFILE`, a local one an `UPDATEFILE` with its mtime (once per name per connection, four of each kind in flight, the rest queued), replies settle names, a failed name is not asked again, downloads are saved in place or by the clients' folder rules with the server's mtime, and only when `save_downloads` is on. Gated by the profile (`pli` 34, `files.modtime`, `files.download_maps`). Still open: the `GWEBL001` level cache, the login set of `UPDATEFILE`s, the file-index scan order, the save folders of 1.39/2.00 and 2.1 to 2.16, and a lab run. See `documentation/file-cache.md`.

## 5. Behaviour over time

- [x] Version chain (`client/src/protocol/chain.rs`). Each build is a step with a small delta against the build before; `PROFILES` is the resolved chain and equals the old hand-written rows. A generated handler-change table (`seihai --version-chain`, `documentation/version-chain.md`/`.tsv`, checked by a test) lists what changed at each step with its source and what is unknown. Adding a version is one step, not new logic. Still open: the unknown deltas of 2.1 to 2.16 and 2.3 (no binaries), and per-step layouts that are not profile fields. See `documentation/version-chain.md`.
- [x] Differential testing. Tools, lab runner and first results are in: a capture-to-frame-sequence decoder (`difftest trace`), a differ with a ranked report (`difftest diff`), and a private-namespace scenario runner (`tools/difftest/lab.sh`). First run: 2.22 and 2.31, login, gmap crossing, chat and door/warp, real client against Seihai. Ranked divergences are in `documentation/difftest.md`. Not yet covered: the real 2.17 client (its launcher needs its own UI steps), the server-side warp scenario, and running it in CI.
- [ ] Public-server checks. Read-only first, and only with approval each time.
- [x] Tooling. A capture viewer and a layout explorer fed by the .tsv tables: `seihai capture show|layout|tables`, with `documentation/packet-layouts.tsv` as the field-layout table (`documentation/capture-viewer.md`). Not done: a GUI, live capture, and layouts for the ids the tables do not list yet.

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
