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
- Asset loaders from the 2.22 data folder that work for every client.
- File transfer caching rules the real clients follow.
- Version chain. Each profile plus a per-step handler-change table, so adding a version means adding a row and a small delta.
- Capture viewer and layout explorer fed by the .tsv tables.
- Remaining fields. The unresolved id 24 and 46 meanings, and the field order for id 178 when a real service sends it.
- Early builds. A real 2.00-era listserver reply, which is still unobserved.
- Public-server checks. Read-only first, and only with approval each time.

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
