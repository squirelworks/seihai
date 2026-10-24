# Planned milestones

What remains after the current gmaps / script engine / differential testing work. Update this when a milestone lands.

## Now (Chiaki)

- [x] Gmaps. Ids 49, 185 and 186, cell props 43/44, and the crossing behaviour captured in the lab. (aed001e)
- [x] Script engine. The NPC and weapon script vocabulary, with a per-version delta table already mined from the builds. (c42e8fa)
- [x] Differential testing. Run the same scripted session against the real Wine client and Seihai in the lab, and diff the wire frames. (f8127f4)

## Next

- Audio playback. GANI frames carry a sound field; nothing consumes it.
- Health and death. The 1.5 s timer, input lock, dead status bit, respawn hearts by AP, and the victim-sent claim packet.
- Combat. Sword hits are judged by the victim's client, with a reach of 0–4 tiles, a lateral limit under 2, and AP 100 immune.
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
