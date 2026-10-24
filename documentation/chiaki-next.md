# Next work for Chiaki

This is for after the current differential testing PR is open and merged. Do not start any of this until that PR is done. Do not interrupt the current job.

## Order

Do these in order. Open the PR for one item, then start the next. Do not wait for merge. The reviewer will catch up.

### 1. Cached level links

The 30-plus-46 form. When the client has the level file already, it sends id 30 then id 46 on entry instead of requesting the full level. Confirm the exact payload against the lab captures and wire it in. Gate it by version profile.

### 2. Script-initiated warps

`setlevel` and `setlevel2` from NPC scripts. The script engine already exists. Online, these currently do nothing. Decode the command, send the level request, and load the new level. Test against a lab level that has an NPC with a scripted warp.

### 3. Health and death

- 1.5 s timer after taking damage before the player can act again.
- Input lock while dead.
- Dead status bit.
- Respawn hearts by AP.
- Victim-sent claim packet.

Use the existing offline session hooks. Confirm the packet ids and payloads against the lab captures and the decompile notes.

### 4. Combat

Sword hits are judged by the victim's client.

- Reach of 0–4 tiles.
- Lateral limit under 2.
- AP 100 immune.

Implement the check on the receiving side. Do not send a hit the victim would reject.

### 5. Audio playback

GANI frames already carry a sound field. Nothing plays it. Add a simple audio path that plays the named sound when a GANI frame with a sound is stepped. Use the sounds from the 2.22 data folder. Keep it optional so the client still runs without audio files.

### 6. Chat popups, buddy, ignore, mute

- Chat popups above the speaker for a few seconds.
- Buddy list file.
- Ignore list file.
- Mute list file.

Load and save the lists from the settings directory. Apply them to chat and PM display.

### 7. Damage formula and respawn "same name group"

These rules are still unresolved. Check the decompile notes and lab captures. If the rule is clear, implement it. If not, write a short note in `documentation/` saying what is known and what is still open, and leave the code alone.

### 8. Asset loaders from the 2.22 data folder

Loaders for levels, tiles, gani animations, images and sounds that work for every client version. The 2.22 folder is the reference. Point the loaders at it and confirm that 1.39, 2.05, 2.17, 2.22 and 2.31 all find their files.

### 9. File transfer caching rules

The rules the real clients follow for what to keep, what to re-request, and when a local file is "up to date". Implement the same rules so Seihai does not re-download files it already has.

### 10. Version chain

Each profile plus a per-step handler-change table. Adding a version means adding a row and a small delta, not new logic. Start from the existing profile table and add the delta table.

### 11. Capture viewer and layout explorer

A small tool that reads the .tsv tables and a capture file and shows the decoded frames. Not a full GUI. A CLI that prints the decoded fields is enough for the first pass.

### 12. Remaining fields

- Id 24 meaning (still unknown).
- Id 46 meaning beyond the observed GCHAR 1.
- Id 178 field order when a real service sends it (lab text was synthetic).

If a lab capture or decompile note settles one of these, wire it. If not, leave a note and move on.

### 13. Early builds

A real 2.00-era listserver reply is still unobserved. If one appears, decode it and add the parser. Otherwise leave it.

### 14. Public-server checks

Read-only first. Only with explicit approval each time. Do not connect to a public server without being told to.

## Rules

- One PR per item.
- Tests must pass.
- Release build must work.
- Update `documentation/plannedmilestones.md` when an item is done.
- If something is blocked on a missing capture or an unresolved rule, write a short note and skip it. Do not invent the missing data.
- Do not wait for merge. Open the PR and continue to the next item.

### 15. Differential testing divergences

From Seihai #24. Fix these in one PR or split them if they are independent. Confirm each against a lab capture before changing the wire.

- Rectangle level links. Seihai walks through them and never sends the level warp. The real client does. Edge links work.
- Login messages. Seihai sends no login player props, no file update (id 34) and no process list (id 44).
- 2.22 position flags. Seihai never sends the `gr.x` and `gr.y` flags. The real client sends them at login and every step.
- 2.31 early id 46. Seihai sends it before the login burst, and twice. The server then warps the player to the online start level first. The early 46 is the likely cause, but that is not proven.
- Smaller ones. Asset file requests, walk update fields, walk speed (about 10 tiles per second against 7.8), frame bundling, adjacent level order and the 2.22 crossing prop order.

Do not invent the missing payloads. If a lab capture does not show the exact bytes, leave a note and skip that item.
