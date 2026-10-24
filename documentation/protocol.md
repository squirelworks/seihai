# Protocol & Networking

This document describes the wire protocol used by Graal Reborn listservers and gservers, derived from public sources (primarily GServer-v2 and OpenGraal connection code) and cleaned observations of client 2.05 rev 2 behaviour. No decompile material.

## Connection Sequence

1. Client connects to the listserver (2.05: `www.graalonline.com:14921`, list name `newmain`; later clients use 14922/14923).
2. Client sends account credentials. In 2.05 the first line is client type `0` + key (`Random & 0x7F`) + version token (`GNW03081`) + length-prefixed account and password.
3. Listserver returns the list of registered gservers.
4. User selects a server; client opens a second TCP connection to that gserver.
5. Client and gserver exchange login, level, and entity packets for the duration of the session.
6. Client may keep the listserver connection open for profile updates.

## Packet Framing

- Logical message: `chr(id+32)` + payload + `"\n"`.
- Several messages are zlib-compressed together under a `u16be` length prefix.
- Binary payloads may be wrapped behind a raw-data escape.

## Encryption generations (cleaned)

| Gen | Builds | Behaviour |
|-----|--------|-----------|
| 1 | through ~1.1.9 | No compression |
| 2 | ~1.21–1.39 | zlib only |
| 3 | ~1.41 onward (including 2.05) | zlib + one LCG-inserted byte per client→server message after the first |
| 4/5 | later clients | full-frame partial XOR with the same LCG family |

**2.05 gen-3 insertion:** iterator seeded `0x04A80B38`, stepped `it = it * 0x08088405 + key`. Inserts `chr(((it>>16)&0x7F)+32)` at 1-based position `(it & 0xFFFF) mod len + 1` (len includes the trailing newline). Position can be 1 (junk byte before the id). Server→client is plain. Login frame is clean.

## Key Packet Categories

### Listserver ↔ Client
- Account verification
- Server list population
- Player profile updates

### Gserver ↔ Client
- Login / version handshake
- Level name + tile data
- Player position / movement (half-tile props 15/16 in 2.05; precise ×16 in later clients)
- NPC create / update / destroy
- Chat messages (including `toall:` prefix → id 6)
- Weapon / item / flag updates
- Sound and effect triggers

### Peer-to-peer UDP (2.05)
- Fixed port 14899. IP/port exchanged via player props 30/31.
- Datagram = sequence byte + payload, XOR’d with low byte of `(peerId ^ ownId)`. No zlib, no gen-3.
- Id 21: remote hit (from GS1 `hitplayer`).
- Id 24: entered level at x,y (3×3 neighbourhood).
- Id 23: receive-only in 2.05 (rewritten as current-level prop).

Exact IDs and layouts are recorded in `../shworkingpapers/protocol/` as they are confirmed.

## Login-time server messages (ids 10, 25, 39, 41, 44, 47, 156, 168, 171, 172, 174, 179, 180, 182)

Every public 2.22 gserver tried on 2026-10-24 sent these around the level
data. `client/src/protocol/extras.rs` decodes them into `SessionState`.
Layouts come from gs2lib's `IEnums.h` and Graal_analysis `packet-ids.md`, and
each was checked against the captured payload shapes (four gservers).

| id | name | payload | `SessionState` field |
|---|---|---|---|
| 10 | ISLEADER | empty | `is_leader` |
| 25 | SIGNATURE | GCHAR (73 on gs2 servers, 41 on GServer-RS). The real client quits on 1, quits on 0 unless the host string is `127.0.0.1`, and ignores anything else (`signature_action`; reported, not enforced) | `signature` |
| 39 | LEVELMODTIME | GINT5 Unix time of the level file | `level_mod_time` |
| 41 | STARTMESSAGE | HTML or RTF welcome page, trimmed; empty on one server | `start_message` |
| 44 | HASNPCSERVER | empty | `has_npc_server` |
| 47 | STAFFGUILDS | comma list, items optionally quoted | `staff_guilds` |
| 156 | SETACTIVELEVEL | level name | `active_level` |
| 168 | UNKNOWN168 | empty | none (accepted, no effect) |
| 171, 172 | BIGMAP, MINIMAP | `txt,png,x,y` (at least 4 comma fields, x and y floats) | `bigmap`, `minimap` |
| 174 | GHOSTICON | GCHAR, above 0 = show (servers sent 0, once as a raw `0x00`) | `ghost_icon` |
| 179 | RPGWINDOW | quoted comma list (welcome line, credits line) | `rpg_window` |
| 180 | STATUSLIST | comma list of status names | `status_list` |
| 182 | LISTPROCESSES | empty | `process_list_requests` |

Deliberately not done: Seihai does not answer 182 with a process list (the
real client sends `PLI_PROCESSLIST`; Seihai never reports the user's
processes). A version only acts on an id its profile lists, so 2.05 still drops
171 and 172.

Seen on the wire but not in the 2.22 client's server set, so they stay
ignored: 82 SERVERTEXT (empty), 190 SERVERLISTCONNECTED (empty), 194
CLEARWEAPONS (empty).

## Layouts read from the 2.17.1, 2.22 and 2.31 decompiles

`client/src/protocol/layouts.rs` decodes the ids the 2.22 dispatcher handles
that the capture-backed set above did not cover. Source: Graal_analysis
`client-internals/decomp-findings-2026-10-24-b/` (`plo_layouts.tsv`). Which
versions handle an id is the profile's `plo` set; the tests compare every
2.00 to 2.31 row with the per-build tables read from the binaries
(`decomp_plo_handled.tsv`, `decomp_pli_sent.tsv`).

| id | name | payload | gate | `SessionState` field |
|---|---|---|---|---|
| 49 | PLAYERWARP2 | GCHAR x*2, y*2, z (unused), gmapX, gmapY, name; a `.gmap` name is kept in `gmap_name` instead of becoming the level | 2.1 test builds, 2.17+ | `player_warp2`, `gmap_name`, position |
| 75 | PROFILE | repeated GCHAR n + n chars; first = account | 1.39+ | `profile` |
| 150 | NPCDEL2 | GCHAR n, level name, GINT3 id | 1.42+ | `last_npc_del`, `npc_dels` |
| 151 | HIDENPCS | none; removes ids 1 to 999 of the active level | 1.42+ | `hide_npcs` |
| 153 / 154 / 155 | SAY2, FREEZEPLAYER2, UNFREEZEPLAYER | text / none / none | 1.42+, 2.00+ | `say2`, `frozen` |
| 165, 166, 189 | MOVE, SHOWIMGNPC, MOVE2 | GINT3 npc id + rest (id above 0 for 165 and 189) | 2.03+, 2.04+, 2.31 | `npc_moves`, `npc_showimgs` |
| 169, 173 | UPGRADEURL, GHOSTTEXT | text | 2.1 test builds+, 2.17+ | `upgrade_url`, `ghost_text` |
| 170 | GHOSTMODE | GCHAR, above 0 = on | 2.17+ | `ghost_mode` |
| 175 | SHOOT | GSHORT player id + rest | 2.17+ | `shoots` |
| 176, 177 | KINGDOMSMODE, NEWWORLDMODE | none | 2.17+ | `server_mode` |
| 178 | SERVERWARP | comma text, at least 4 fields; fields 1 to 3 used | 2.17+ | `server_warp` (meanings unresolved; not followed) |
| 181 | (unnamed) | GSHORT player id, GCHAR toggle | 2.17+ | `player_toggle` |
| 183 | HASPROCESSRUNNING | names; the client answers PLI 45 | 2.17+ | `process_running_requests` (never answered) |
| 184 | TAKESCREENSHOT | file name, `.png` only | 2.17+ | `screenshot_requests` (no screenshot taken) |
| 185 | BOARDHEIGHTS | GCHAR mapX, mapY, blockX, blockY, w-1, h-1, then w*h cells of 2 bytes: `(b0-0x20-50) + (b1-0x20)/128` | 2.17+ | `board_heights` |
| 186 | BOARDMODIFY2 | GCHAR mapX, mapY, x, y, w, h, then w*h GSHORT tiles, then an unread trailer (ignored; a payload one byte short of `6+2*w*h` is dropped silently; a block past column 63 is clipped) | 2.22+ | `board_modify2` |

Ids Seihai answers or sends only on request: none of the above. Seihai still
does not answer 182 or 183 (process list and "is X running"), and takes no
screenshot on 184.

## Tamper check (client to server id 39)

`client/src/protocol/tampercheck.rs`. Off by default; `--send-tampercheck`
turns it on for 2.17, 2.22 and 2.31 (GServer-v2 ignores the packet, and four
public 2.22 servers did not kick a client that never sent it for 20 s).

- Payload: 10 raw bytes, word A then word B, each 5 bytes:
  `((v>>28)&0x0F)+0x20`, then `((v>>21)+0x20)&0xFF`, `((v>>14)+0x20)&0xFF`,
  `((v>>7)+0x20)&0xFF`, `(v&0x7F)+0x20`. Not GINT5 and not masked, so middle
  bytes can fall below 0x20. A payload with a `0x0a` is not sent (it would
  split the line; the real send helper drops it too).
- The words are reflected CRC-32 values over ranges of the client's own
  memory image (`crc_chain`). Seihai has no such image, so the source is a
  `TamperImage`; the default is the build's constant in its profile row:
  2.17 `ab637cbe`/`3e48dc0c`, 2.22 `5dcb77f6`/`bc736311`, 2.31
  `9ecd4bc0`/`ecea65e2`. The tests reproduce the six lab byte strings from
  them (2.17 `2a7bad195e231243d82c`, 2.22 `250e4d0f962b03ede631`, 2.31
  `291655b7602e87c9eb82`).
- Position: the second message of the first frame after login, behind
  `PLI_LANGUAGE`.
- Re-sends, as the real builds do: 2.17 never; 2.22 on every level change
  (the real client's level-change routine; the lab saw it on each gmap edge
  crossing). Seihai re-sends on a level change after the first level, and has
  no gmap edge crossing yet. 2.31 every 3.2 s: a 50 ms timer counts modulo
  64 and sends at count 31 (`TamperSender::advance`, driven from the game loop).
- 2.3 has no binary and no captured value, so nothing is sent for it.
- Not known: whether any server enforces the value.

## 2.00 login

Type 0, the key byte `(key & 0x7F) + 0x20`, the token `GNW24011`, then the
account and password as length-biased strings, in one plain zlib frame, then
gen 3. Account and password are not hidden. No PLI 37 (the row has no
language), and the send set stops at 34 (no 35 to 39). The row and a wire test
(`v200_login_line`) cover this. The lab gserver only needs `GNW24011` in its
allowed versions. 2.00's listserver client side (lab, 2026-10-24): one plain
zlib frame of `GCHAR 0` + `newmain` + LF, no account, password, token or key,
then no second listserver connection; Connect goes straight to the gserver. What
a real 2.00-era listserver sends back is unobserved (the lab fed it the 2.17-style
two plain frames, which it parsed). Seihai does not send this frame: the 2.00
listserver flow here is still the 2.05 one, unverified. Note only.

Still unknown or not implemented, for lack of a capture or a decompile: what
`x` and `y` in 171/172 position (always 0 in the captures); the SERVERWARP
field meanings; what 2.31's PLI 24 (value 9) is for (Seihai does not send it)
and what PLI 46 (always `GCHAR 1`) tells a server; the 2.31 extra step in 156;
whether a server checks the tamper value; gmap support (49, 185 and 186 are
decoded but not applied).

## PLI 46 (ENTERLEVEL) and PLO 186 (BOARDMODIFY2)

Settled by the lab captures in Graal_analysis #33.

- **PLI 46, client to server, 2.3 and 2.31 only** (the profile's send set has
  it; the code reads the set, not the version). The payload is always
  `GCHAR 1`. A real 2.31 sends it at login and on every level entry (7 times in
  the lab: login plus 6 gmap crossings), as the first message of the frame,
  immediately before the props message that carries the cell prop (43/44) and
  the wrapped coordinate. It is not a one-shot, and 2.31 does not re-send the
  tamper check (39) on a crossing. Seihai sends it once after the post-login
  frame and again whenever the server puts the player in a different level
  (`EnterLevelSender`). Seihai has no gmaps and sends no cell prop, so the 46
  goes out alone and the next walk update is the props frame that follows it;
  `Connection::send_enter_level` can put props messages behind it in the same
  frame. Not tested in the lab: doors and warps.
- **PLO 186:** header and `w*h` tiles are applied at the position. Bytes after the
  tiles are ignored (a whole second record there is not applied). One byte short
  of `6+2*w*h` is dropped silently, with no disconnect and no partial apply. A
  block past column 63 is clipped. `BoardModify2::apply_to` does that on a
  64x64 board. The session does not call it: it keeps the last decoded record
  only, because applying one needs the gmap cell the player is in (`mapX`/`mapY`
  other than the current cell were never tested), and Seihai has no gmaps.

## Implementation Notes

- Prefer a single `Packet` / buffer class that can be shared by both listserver and gserver connections.
- Keep the protocol layer free of rendering or scripting dependencies so it can be unit-tested with recorded captures.
- Version negotiation: advertise a widely accepted token (`GNW03081` for 2.05-era servers, later tokens for modern Reborn).

## Next Steps

- Implement gen-3 encode/decode against a local capture.
- Implement login + server-list flow against a local listserver.
- Implement level-name and basic player-move packets.

## Lab findings 2026-10-24

Real clients (2.17, 2.22, 2.31) against a local lab. No public server.

- **Tampercheck (PLI 39):** 10 raw bytes, per-build constant. 2.17 `2a7bad195e231243d82c`, 2.22 `250e4d0f962b03ede631`, 2.31 `291655b7602e87c9eb82`. Sent after language. 2.22 re-sends on every gmap-edge cross; 2.31 re-sends about every 3.2 s. Seihai sends them with `--send-tampercheck` (see above).
- **2.17 listserver:** one plain zlib frame with two messages (`id 0` + `newmain`, then `id 1` + account/password). Server answers with list, then status/site/donation. Gen-3 after that. Seihai 2.17 matched this flow.
- **2.22 gmap crossing:** no level-warp packet. Client sets `gr.x`/`gr.y` to the global position (local + 64 × cell). When it wraps, the same props message carries local X/Y = 0 and GMAPLEVELX/Y (props 43/44). Server answers with the new level name, then the gmap name. Seihai currently treats the gmap name in `PLO_PLAYERWARP2` as a level and asks for `name.gmap.nw`.
- **Buddy / ignore:** local files (`buddyaccounts.txt`, `ignoredaccounts.txt`). No listserver traffic in this lab. Mute is `PLI 43` with player id + 1/0.
- **Gen-5 XOR:** all 17 real 2.22 client zlib frames (lengths not multiples of 4) decode with the one-step-per-group rule. No bz2 client send was seen.
- **Missing binaries:** no 2.1.x (2.1–2.16) or 2.3 binaries are recoverable. Those rows stay inferred. Alias 2.1–2.16 to 2.05 or 2.17, and 2.3 to 2.31.
