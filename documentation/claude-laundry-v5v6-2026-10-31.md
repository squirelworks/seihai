# Claude Laundry List — v5 and v6 support

Date: 2026-10-31. Continue from main at `327371f`.

Public Reborn game servers can allow v5 and v6 clients, so Seihai should speak those versions.

> **Correction (2026-10-31).** The listserver does not check the version; each game server's `allowedversions.txt` does. Two public lists are on record: Graal the Adventure allows `2.22, 5.07` (5.07 is 5.00r7), and a gs2 server allows `2.22, 1.41r1 - iPhone Belote 1.5`, a range that covers every 5.x/6.x token in gs2lib's table. Neither allows 5.24 (`G3D15010` is not in the table), and 6.036 has no token. No v5/v6 login has been made to a public server. Details in Graal_analysis `server-notes/public-allowed-versions.md`. Ato stopped at 3.153 because there is no 3.x Torque capture (only 5.12's). The flat path is enough for 3.x until that capture exists. Do not invent Torque bytes for 3.04 or 3.153.

## Why Ato is stuck

- 3.04 (TGE protocol 9) and 3.153 (Graal3D protocol 19) open Torque only for a server whose listed name starts with `3 `, on UDP at port + 1.
- GServer-v2 has no Torque side, so the lab cannot produce a 3.x exchange.
- The only live Torque handshake on record is 5.12's.
- See `documentation/ato-3d-path.md`. Leave the 3D path held until a real 3.x capture exists.

## v5 and v6 facts already in Graal_analysis

- `protocol/v5x-clients.md` — 5.00r7, 5.12, 5.24, 5.30, 5.31. Delphi launcher + MSVC engine DLL. Torque-derived base classes.
- `protocol/v6x-clients.md` — 6.00r2 and later. Same wrapper family as 5.30/5.31.
- `protocol/v5v6-lab-play.md` — lab runs. The client treats the first server as "Login" and draws no characters until it receives id 178 (SERVERWARP). After the warp it draws the player, HUD, and other players.
- `version-profiles/lab-v5-2026-10-30.md` — 5.00r7 and 5.12 lab notes.

## Work order

### 1. Profile rows for the accepted builds

Add profile rows for these builds (the original wording said listserver.graal.in lists them as acceptable; see the correction above):

- 5.00r7, 5.12, 5.24r4, 5.30r3, 5.31r4
- 6.00r2, 6.03r7, 6.036 (the ones named in the notes)

Each row needs: login token, framing (gen-5 or later), handled ids, and the id 178 requirement. Do not invent tokens. If a token is missing, mark the row and move on.

### 2. Id 178 as a required step

After the login burst, if the profile says the client is in Login mode, send id 178 with the host and port of the real game server. The client then reconnects and draws. Gate it by profile. Add a test that the bytes match the lab capture.

### 3. Login shape for v5/v6

The notes say the first connection is a Login server. Capture or read the exact frames 5.00r7 and 5.12 send. If Seihai's gen-5 login is already accepted, say so and keep it. If not, add the shape that the capture shows.

### 4. File transfer and the PNG-as-name bug

5.24 tries to save PNG data as a file name when the server's version table is wrong. The lab fix was renaming `G3D26090` to `G3D15010` in a copy of libgs2lib. Document the rule and make Seihai request files the way 5.24 expects.

### 5. Drawing after the warp

Once id 178 has been sent and the second connection is up, draw the player, the HUD, and other players. Use the existing flat path. Do not open Torque.

### 6. Lab check

Run 5.00r7 and 5.12 against the lab with the new profiles. Confirm login, the id 178 reconnect, and that the player is visible. Note any packet Seihai does not yet handle.

### 7. 6.x once 5.x works

Repeat the same steps for the 6.x builds that the notes cover. If a 6.x binary is wrapped and cannot be read, leave a note and use the lab capture only.

## Rules

- Do not invent wire bytes. If a capture is missing, leave a note.
- Do not start the 3.x Torque path. It stays held.
- Keep every existing profile. A classic or Torque client still works.
- If a public server check is needed, do it read-only and only if the network can reach the listserver.
