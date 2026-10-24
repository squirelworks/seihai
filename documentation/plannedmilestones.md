# Planned milestones

What remains after the current gmaps / script engine / differential testing work. Update this when a milestone lands.

## Now (Chiaki)

- [x] Gmaps. Ids 49, 185 and 186, cell props 43/44, and the crossing behaviour captured in the lab. (aed001e)
- [x] Script engine. The NPC and weapon script vocabulary, with a per-version delta table already mined from the builds. (c42e8fa)
- [x] Differential testing. Run the same scripted session against the real Wine client and Seihai in the lab, and diff the wire frames. (f8127f4)

## Next

- Damage and respawn leftovers: knockback, the per-player field in the packet path, bombs/arrows/baddies powers, the end-of-respawn save (`documentation/damage-respawn.md`).
- Audio playback. GANI frames carry a sound field; nothing consumes it.
- Chat popups, buddy/ignore/mute files.
- Damage formula and the respawn "same name group" rule.
- File transfer leftovers: the `GWEBL001` level cache, the login set of `UPDATEFILE`s and the file-index scan order (`documentation/file-cache.md`).
- Early builds. A real 2.00-era listserver reply, which is still unobserved. Skipped (item 13): nothing new observed.
- Public-server checks. Approved (read-only, then chat, warps and file transfer), but the build box could not reach any listserver port on 2026-10-25, so nothing was observed. Checklist and port facts in `documentation/public-checks.md`; run it from a network that reaches ports 14921 to 14923.

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
