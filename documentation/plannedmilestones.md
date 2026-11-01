# Planned milestones

What remains after the current gmaps / script engine / differential testing work. Update this when a milestone lands.

## Now (Chiaki)

- [x] Gmaps. Ids 49, 185 and 186, cell props 43/44, and the crossing behaviour captured in the lab. (aed001e)
- [x] Script engine. The NPC and weapon script vocabulary, with a per-version delta table already mined from the builds. (c42e8fa)
- [x] Differential testing. Run the same scripted session against the real Wine client and Seihai in the lab, and diff the wire frames. (f8127f4)

## Graal3D (February 2000)

Separate side project (`GR3DV002`): its own client and server protocol, not the
2D gserver path and not the Torque 3D of 3.x–5.x. User notes:
`documentation/graal3d.md`. Task ids and module map:
`crates/seihai-g3d/ARCHITECTURE.md`. Wire and formats:
Graal_analysis `graal3d/`.

**Worlds are not old 2D levels.** There is no `.nw` / `.graal` / GANI / tile /
NPC path. A world is DirectX text `.x` meshes plus local BMPs (textures and
player parts are never sent on the wire). Hosting today means the original
Wine `GServer.exe` or Graal_analysis `graal3d/reference-server/`;
`seihai-gserver` does not speak this protocol.

### Done

- [x] `seihai-g3d` plays online: login, props, level receive, UDP peers, chat,
  software draw of textured `.x` levels and the eight-part player model,
  walk / jump / follow camera (`documentation/graal3d.md`; #331, #332).
- [x] Plain-behaviour notes that unblock client fidelity: collision
  (`graal3d/collision.md`, Graal_analysis #164) and camera / FOV
  (`graal3d/camera.md`, #163). Live Wine confirmation of both still wants
  native `d3drm.dll` (G3D-8).

### M1 — Client fidelity (`seihai-g3d`)

Analysis for the first two is ready; code still uses stand-ins.

- [ ] **G3D-1** Faithful per-face collision (`world::Collider` from
  `collision.md`).
- [ ] **G3D-2** Camera height, look point and field 0.5 / ~53.13° from
  `camera.md` (replace `CameraRig` stand-ins).
- [ ] **G3D-4** F1 hardware options and F2 player look; write
  `graal3dconfig.txt`.
- [ ] **G3D-6** Draw without `--assets` (coloured boxes; name missing files).
- [ ] **G3D-7** Connect window and launcher "Graal3D" entry.

### M2 — Admin and real-client lab

- [ ] **G3D-5** Admin session (`--admin`): account / player lists and
  kick / message; account-record layout still needs a lab capture.
- [ ] **G3D-8** Two real `Graal3D.exe` clients under Wine with native
  `d3drm.dll`; capture UDP and match Seihai.

### M3 — Hosting (Rust Graal3D server)

- [ ] **G3D-9** Port the Python reference server into a **separate** Rust
  binary (same `GR3DV002` facts as `protocol.md` / `formats.md`). Not a
  stage of `seihai-gserver` (different codec, ids, 3D floats, peer UDP, no
  board / NPC / file-transfer game). Home crate: prefer next to
  `seihai-g3d` in this workspace unless packaging says otherwise.
- Done when: `seihai-g3d` and, where Wine + `d3drm` allow, the real client
  log into the Rust host on loopback; level text and prop / UDP rules match
  the reference server's lab session.

### M4 — World content and assets

New worlds are authored as `.x` + BMP, not converted from 2D boards.

- [ ] Document the authoring contract in `documentation/graal3d.md` (or a
  short `graal3d-worlds.md`): text `.x` only, materials and texture file
  names, BMP bit depths the client reads, player mesh set, `startlevel.x`
  naming, accounts / guilds beside the server.
- [ ] Optional built-in or generated **default Graal3D asset root** (shipped
  demo meshes / textures under a clear licence, or a generator), distinct
  from the 2D `assets/default/` set; `--assets` / `--check` path for it.
- [ ] Keep `--level-info` on `.x` as the smoke check; add a small world-folder
  check (every texture named by a level or player prop exists) once M3 or
  servertools wants it.
- Out of scope here: the 2D offline level editor (`tools/level-editor`) and
  LAT `.nw` editing — they stay 2D-only unless a later milestone adds an
  `.x` editor (Squirrel decides).

### M5 — Stack knock-ons (after M3)

Only pieces that actually touch February 2000 Graal3D:

| Area | Effect | Milestone |
|---|---|---|
| `seihai-servertools` | Lab start / stop for a Graal3D host; optional `.x`+BMP world check | Add a Graal3D lab kind once M3 exists (`seihai-servertools` `docs/NEXT.md`) |
| `seihai-gserver` | None on the wire | Keep stages 1–7 2D / gen-5 only; PLAN notes G3D-9 is a sibling host |
| `seihai-listserver` | Feb 2000 client/server never talk to the list | No work for this era. HQ tag `3 ` / level 4 stays **Torque-era** (with Ato Stage 4), not this milestone |
| `seihai-rc` / `seihai-npcserver` | No RC/NC and no NPC scripting in either Graal3D binary | No milestone |
| `seihai-ato` Stage 4 | Different 3D (Torque on UDP port+1) | **Held** as G3D-3 / `documentation/ato-3d-path.md` until a Torque capture exists |

### Held (not February 2000)

- [ ] **G3D-3 / Ato Stage 4** Torque 3D servers for 3.x–5.x. Needs a connect
  capture and a Torque-speaking lab host. Do not invent wire bytes.

## Next
- Asset generation track (items 17–21): [x] item 17, inventory of the playerworld archives (`documentation/asset-inventory.md` and `.tsv`: counts by type per world, starter-set candidates, reader pass rates); [x] item 18, reference sheet measurements (`documentation/asset-reference.md` and `.tsv`: tile id mapping checked against the real 2048x512 sheet and against real levels, palette types, PNG failure cases, body/head/sword/shield/GANI/sound layouts, the foot-test and 128-column constraints a default set must meet; numbers only, no art; found `assets::tile_src_rect` uses a different, unused mapping), [x] item 19, a self-contained default asset root `assets/default/` (47.8 KB, 17 files at item 19, all original and generated by `tools/gen_default_assets.py` from code: 2048x512 tile sheet and type table, body/head/sword/shield/sprites/state sheets, idle/walk/swim GANIs, footstep and splash WAVs, two starter levels; `documentation/default-assets.md`; contract test `client/tests/default_assets.rs`), [x] item 20, the set compiled into the binary and wired in as the last resolver layer (a missing, empty or partial `--assets` falls back file by file; `--check-assets` with no `--assets` checks the built-in set and passes for 1.39, 2.05, 2.17, 2.22 and 2.31; per-server download cache `<cache root>/<host>_<port>[~name]/` searched first, written by downloads only, never the built-in set; built-in files are asked for with `WANTFILE` and no mod time; `walk.gani` and the step sounds are asked for at the first steps only when no copy exists; built-in levels offline only; chests and signs drawn from the board, no placeholder boxes; `documentation/file-cache.md`, `default-assets.md`, `client/tests/builtin_wiring.rs`), [ ] item 21 optional generator tool (mostly covered by `tools/gen_default_assets.py`). See `documentation/chiaki-next.md`.
- GANI expansion (item 22): [x] `sword.gani` (5 steps, `sword.wav` after step 0, `SETBACKTO idle`), `sit.gani` (1 step) and `hurt.gani` (1 step) in the default set, generated by `tools/gen_default_assets.py` (body rows 9 to 12, sword cells placed by the GANI, synthesised `sword.wav`; the set is 54.7 KB in 21 files, original, `--check` passes); the S key plays the sword animation in step with the swing and queues its sound, an accepted hit shows the hurt pose for 500 ms (offline session included), then back to idle; `Gani::setback_to`, `pose.rs`, `render::draw_gani_frame`; `client/tests/poses.rs`. Open: no key for sit; the walk poses are still not drawn for the local player; other players' swings and hurts are not drawn with the GANIs; the hurt file name and duration are Seihai's convention (`documentation/default-assets.md`, "Poses").
- Script to GANI and images (item 23, stacked on #54): [x] `setimg` / `setgif` / `setimgpart` change the NPC picture (part trimmed when drawn), `hide` / `show` toggle it, `showimg` / `hideimg` / `changeimgpart` keep a numbered image list per NPC (index replaces, same file moves, `hideimg` deletes) that the renderer draws, `setcharani` / `setani` / `showcharacter` draw an NPC as a GANI (idle <-> walk, 100 ms a step, `LOOP` and `SETBACKTO`), `setimg x.gani` as Seihai's own convention; new pictures and animations are loaded by the host; `render::draw_npcs`, `npcart.rs`, `client/tests/npc_images.rs` (12 tests + an ignored smoke test over real scripts through `SEIHAI_SCRIPT_CORPUS`; `documentation/npc-images.md`). Not done: weapon-script `showimg` (no image list on the weapon), image layers other than over, screen-layer images, NPC directions. The plan text had `showimg` / `hideimg` toggling the NPC's visible flag; the command reference says they put up and delete numbered images, which is what is done.
- Remaining fields (item 12): merged (#39). Ids 24, 46 and 178 are in; what is still open is listed in `documentation/remaining-fields.md`. Notes are in Graal_analysis batch g.
- Differential-testing divergences (item 15), PRs #41 to #48 (all merged; #42 was closed when its base branch was deleted and its content landed as #44), one per divergence, measured in the private lab (`documentation/difftest.md`): [x] login frame (image checks, process list, login props; `documentation/login-burst.md`), [x] rectangle level links (`documentation/link-geometry.md`), [x] 2.22 position flags (`documentation/gr-flags.md`), [x] early id 46 and the early id 24 (`documentation/early-enter-level.md`), [x] adjacent level order and 2.22 crossing prop order (`documentation/gmap-order.md`), [-] asset requests, walk fields, frame bundling: confirmed against the captures and skipped with reasons (`documentation/difftest-leftovers.md`); [x] walk speed (`documentation/walk-pace.md`).
- Damage and respawn leftovers: [x] knockback (5 ticks of 1 tile, 12 body-box probes, same in 2.05 rev 2, 2.17.1, 2.22, 2.31), the packet-path gates of ids 40 and 46 (AP 100, once-a-second stamp, guild test, power 0, id 46 box), see `documentation/damage-respawn.md`; skipped with a stated capture: the per-player field of packet 40 (an owner link, always true for plain players), bomb/arrow/baddy powers, the end-of-respawn save, CURPOWER on damage.
- File transfer leftovers: [x] save folders by build (1.33 to 1.41 flat `levels/webgifs`; 2.00 and 2.01 without `hats`; both 2.1 test builds have no `maps` rule), the empty-stream guard (a zero-length file is never saved), the login `UPDATEFILE` set checked on the traces (only `wbowi1.png` differs; the weapon ids are evidence only), see `documentation/file-cache.md`; skipped with a stated capture: `GWEBL001` level cache, file-index scan order, pre-2.1 re-send of `UPDATEFILE`, case folding.
- Early builds. A real 2.00-era listserver reply was observed on 2026-10-25 (public port 14923: one plain zlib frame, 4 untagged rows, no status messages; see `documentation/public-checks.md`). Item 13 can start from it.
- Public-server checks. The list checks ran on the user's machine on 2026-10-25: 14921 refused, 14923 answers a classic login with the legacy list, 14922 answers the gen-5 login (2.22, 2.31) and stays silent to a classic one (2.17). The 2026-10-25 matrix (19 profiles x both ports) showed the reply follows the login type: all 15 classic profiles (1.41 to 2.17) get a list on 14923 only, the gen-5 profiles (2.22, 2.3, 2.31) on both; defaults and port statuses follow it (`documentation/port-rule.md`). The game-server checks (login, chat, warps, files) are still to do. Results, open points and the checklist are in `documentation/public-checks.md`.
- Level editor (item 24): [x] tracking checklist copied to `documentation/level-editor.md`; [x] first slice, `tools/level-editor` CLI: opens a `.nw` (GLEVNW01 text), `list` (summary, `--rows` dump), `paint` a rectangle of one tile id (bounds and id < 4096 checked), `save` (untouched levels byte-identical; after a paint the BOARD rows are rewritten canonically and every other line is kept; temp file and rename), `--version` 2.22 (default) and 2.17.1, other versions say "not yet" with a reason; 18 tests. Not done: binary levels, any GUI, selection/undo/stamps, link/sign/chest/baddy/NPC editing, 2.05 and older profiles.

## Done

- Head-only swim blit
- Emotes local only (no SHOWIMG send)
- GANI parser and stepper
- Basic script stubs (message, say, setimg, move)
- File transfer (raw data packet, level save)
- Chat send and receive
- Player list, overlays, screenshots, chests, signs
- Warps (level links)
- Other players' positions and logouts
- Tamper check sender
- PLI 46 on level entry
- SERVERWARP follow (default on, settings option)
- Script-initiated warps: `setlevel`/`setlevel2` online use the link request (PLI 0, or 30 with a cached copy, by profile); the new level loads on the server's answer. Seihai lab-tested on 2.31 and 2.22; the real client's own form is not captured (`documentation/script-warps.md`)
- Cached level links: PLI 30 (`LEVELWARPMOD`) with the mod time from a per-session level cache, 46 and the props in the same frame, gated by profile
- Health and death: 1.5 s death timer, input lock, dead status bit from tick 28, respawn hearts by AP and at the saved entry point, victim-sent `PLI_CLAIMPKER` (14), gated by `life.death` in the profile (`documentation/health-death.md`)
- Audio playback: GANI `PLAYSOUND` cues play once per frame entry from `walk.gani` and `swim.gani`, through a `SoundOut` trait with a null backend, a WAV decoder (PCM, IMA ADPCM), the 8-voice cap, pan and the `+` loop prefix. `--no-sound` and `sound=0`. The device backend is the `audio-rodio` feature and is not yet built or run (`documentation/audio.md`)
- Combat: the victim's client judges sword swings (reach 0 to 4 tiles, side under 2, AP 100 immune, no hit while dead, hurt or swimming) and `PLO_HURTPLAYER` (40); hurt mode (40 ticks); the S key swings and broadcasts the swing sprite; hits that the victim would refuse are not sent; gated by `life.combat` in the profile (`documentation/combat.md`)
- Damage and respawn rules: a sword hit takes the attacker's sword power in half hearts (limited to ±20; negative heals only at AP 40 or more; hurt mode always), gated by `noplayerkilling`, the `(guild)` tag, attacker STATUS bits and a once-a-second limit per attacker; the respawn entry is saved on the first entry or when crossing the gmap list, plus a name-group test in 2.05 only. Derived from the 2.05, 2.17.1, 2.22 and 2.31 clients; no lab run (`documentation/damage-respawn.md`)
- Chat popups, buddy, ignore and mute lists: chat bubbles over the speaker for 3 s (a placeholder; the one documented number is the 3 s of script-set chat text), `buddyaccounts.txt`, `ignoredaccounts.txt` and `mutedaccounts.txt` in the settings directory (one account per line, case-insensitive, atomic save), ignore and mute hide chat, bubbles, toalls and PMs, `/mute` sends `PLI_MUTEPLAYER` (43) only where the profile has it (`documentation/chat-lists.md`)
- Asset loaders from the 2.22 data folder: one `AssetResolver` finds levels, GANIs, images, sounds and music by name (several folders per kind, case-insensitive, extension and format rules from the version profile, names cannot leave the folder), `.graal` levels now read their links, baddies, NPCs, chests and signs (and GR-V1.04 uses 13-bit codes), a tiledefs parser, and `seihai --check-assets` / `SEIHAI_DATA_222` tests that load the files of 1.39, 2.05, 2.17, 2.22 and 2.31 from the folder. Not done: a `.graal` sample from a real folder (none ships), tile sheets of other widths, GIF/MNG decoding, tiledef matching (`documentation/assets.md`)
- File transfer caching rules: four `WANTFILE` and four `UPDATEFILE` in flight with the rest queued, one `UPDATEFILE` per local file per connection, replies settle names, a failed name is not asked again, the server's mtime is set on the saved file, the clients' download folders (`sounds`, `maps`, `levels/<kind>`), saving gated by `save_downloads` (the real `saveimages`). Derived from 2.00 to 2.31 and checked against the 2.17, 2.22 and 2.31 captures; no new lab run (`documentation/file-cache.md`)
- Version chain: every build is a step (label, token, date, basis) plus a small delta against the build before it, resolved at compile time into the same 26 `PROFILES` rows (checked byte for byte against the hand-written rows). A generated per-step handler-change table lists the server ids a build gains or loses a handler for, client ids it sends, prop ids, framing, files, levels, rules and UI switches, with each step's source and what is still unknown; `seihai --version-chain [--chain-format text|md|tsv] [--version V]`, `documentation/version-chain.md` and `.tsv`. Adding a version is one step in `chain.rs`. Based on #33 (file cache) for `files.download_maps`. Not done: per-step payload layouts and timings that are not profile fields; the unknown deltas of 2.1 to 2.16 and 2.3 stay marked unknown
- Capture viewer and layout explorer: `seihai capture show FILE` prints a pcap or a difftest trace with packet names, decoded fields, raw hex for what is undecoded, filters (direction, id or name, time, frame, connection) and text, tsv or json output; `seihai capture layout ID` prints a packet's field layout along the version chain and marks unknown fields; both read `packet-layouts.tsv` (built in, or from `--tables DIR`). Login frames are redacted and output files are private (`documentation/capture-viewer.md`)
- Remaining fields: id 24 is the sender's weapon-script image list and the byte 9 is its full-list marker (the text is decoded and applied per player for 2.17 and later, nothing draws it; the 2.22 sender now follows the client's free-running counter); id 46 is a literal-1 level-switch notice that the server ignores (table row settled, behaviour unchanged); id 178 is split with the clients' own comma-text reader (white space splits unquoted items, `P Name` loses its prefix, the name goes in the window title). Still open: the sender's own field formatting for images, drawing them, and what a real service sends for 178 (`documentation/remaining-fields.md`)
- Real `.graal` levels: two supplied `GR-V1.03` levels and a 3113-level player world run through the reader (all parse, sections end exactly at the end of each file); sign text now uses the packed alphabet instead of the retracted `char + 89` rule; links with `playerx+N` / `playery+N` arrivals are kept (offsets stored); baddies at x = 0 confirmed; `seihai --level-info`; `SEIHAI_GRAAL_LEVELS` and `SEIHAI_PLAYERWORLD` `#[ignore]` tests. Not done: `HEIGHTS` (no real file has one), whether a client evaluates link offsets (`documentation/assets.md`)
- `.zelda` (Z3-V1) levels: a strict reader (`client/src/assets/zelda.rs`) for `Z3-V1.03` and `.04` (board, decimal links with negative arrivals meaning "keep the player's coordinate", computers with `.04` verse text, signs), selected by magic whatever the extension; settled against the Java Zelda Online source, the 1st Graal and 1.39 clients and 139 distinct real files (663 copies, all read to the end); `levels.zelda` in the profile (true for every build), `.zelda` in the resolver, `seihai --level-info` and `--check-assets` scans for Z3-V1. Also fixes the old section reader on Z3-V1 (decimal links were dropped, `1.03` computers misread) and reads a link arrival of `-1` as "keep". Not done: which build tries `.zelda` for a bare name, sign glyph codes 82 to 88 in the Java alphabet, baddy simulation, a `Z3-V1.02` sample (`documentation/assets.md`)
