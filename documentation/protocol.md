# Protocol & Networking

This document describes the wire protocol used by Graal Reborn listservers and gservers, derived from public sources (primarily GServer-v2 and OpenGraal connection code) and cleaned observations of client 2.05 rev 2 behaviour. No decompile material.

## Connection Sequence

1. Client connects to the listserver (the 2.05 build carries `www.graalonline.com:14921`, list name `newmain`, now dead; Seihai dials `listserver.graal.in` on 14923 for the profiles that send the classic login, 1.41 to 2.17, and 14922 for the gen-5 login, 2.22 and later). On the public service (observed 2026-10-25, every profile) 14921 is refused, 14923 answers both login types (a classic login gets a plain-frame legacy list), and 14922 answers only the gen-5 login (a classic login gets no reply); see `port-rule.md` and `public-checks.md`.
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

**Gen-5 step count: keep Seihai's.** The iterator steps once for every 4-byte group the body has, a partial last group included (rounded up), at most 12 groups for an uncompressed (`0x02`) body and 4 for zlib (`0x04`) or bz2 (`0x06`). Every lab 2.22 frame chains with this count, client and server, and every real 2.22 zlib send (17 of them, none a multiple of 4 bytes) decodes with it. The one exception is 5.12, which on compressed bodies steps per whole group only (rounded down) and leaves a partial last group in the clear (`Gen5::round_down`, see `v5v6-profiles.md`).

The third-party codecs agree with Seihai for 2.22 to 6.x but have no 5.12 case: pyReborn, reborn-protocol and pygserver always round up, as GServer-v2 does, so none of them stays in step with a real 5.12 (cross-check of 2026-11-01; nothing copied). Don't "fix" the step count toward any of them. Change it only if a new capture shows a frame that doesn't chain. These tests pin it in `protocol/gen5.rs`:

- `lab_222_first_client_frame_is_rebuilt_byte_for_byte`: the 2.22 client's first frame, built from its two messages through `choose_comp`, `encode` and `write_frame` (the `net` send path), matches the capture byte for byte. It also shows that a whole-groups-only count leaves the frame's last byte unXORed.
- `lab_222_short_server_frame_is_rebuilt_byte_for_byte`, `lab_222_server_chain`: a 7-byte uncompressed frame between zlib frames, chained by the exported iterator states.
- `round_down_on_compressed_only`: 5.12's count, on compressed bodies only.

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

Seihai answers 182 with `PLI_PROCESSLIST` (44) holding only `"seihai"`, as
the real client answers it (`login.rs`, `login-burst.md`; seen on public
servers on 2026-10-30, `public-gserver-2026-10-30.md`). It never reports the
user's processes. A version only acts on an id its profile lists, so 2.05 still drops
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
| 49 | PLAYERWARP2 | GCHAR x*2, y*2, z (unused), gmapX, gmapY, name; a `.gmap` name starts a gmap session (`gmap`, see "Gmaps") instead of becoming the level | 2.1 test builds, 2.17+ | `player_warp2`, `gmap_name`, `gmap`, position |
| 75 | PROFILE | repeated GCHAR n + n chars; first = account | 1.39+ | `profile` |
| 150 | NPCDEL2 | GCHAR n, level name, GINT3 id | 1.42+ | `last_npc_del`, `npc_dels` |
| 151 | HIDENPCS | none; removes ids 1 to 999 of the active level | 1.42+ | `hide_npcs` |
| 153 / 154 / 155 | SAY2, FREEZEPLAYER2, UNFREEZEPLAYER | text / none / none | 1.42+, 2.00+ | `say2`, `frozen` |
| 165, 166, 189 | MOVE, SHOWIMGNPC, MOVE2 | GINT3 npc id + rest (id above 0 for 165 and 189) | 2.03+, 2.04+, 2.31 | `npc_moves`, `npc_showimgs` |
| 169, 173 | UPGRADEURL, GHOSTTEXT | text | 2.1 test builds+, 2.17+ | `upgrade_url`, `ghost_text` |
| 170 | GHOSTMODE | GCHAR, above 0 = on | 2.17+ | `ghost_mode` |
| 175 | SHOOT | GSHORT player id + rest | 2.17+ | `shoots` |
| 176, 177 | KINGDOMSMODE, NEWWORLDMODE | none | 2.17+ | `server_mode` |
| 178 | SERVERWARP | comma text read like the clients' list reader (quotes; white space splits unquoted items), at least 4 fields: 0 unused, 1 name, 2 host, 3 port | 2.17+ | `server_warp` (followed: see "Server warp" below) |
| 32 | SHOWIMG | GSHORT sender id + the sender's PLI 24 payload: an image-record text from 2.17 on (a real client's one byte 9 is its full-list marker); see `remaining-fields.md` | 2.05+ | `showimg_relays`, `last_showimg`, `player_images` (2.17+; nothing drawn) |
| 181 | (unnamed) | GSHORT player id, GCHAR toggle | 2.17+ | `player_toggle` |
| 183 | HASPROCESSRUNNING | names; the client answers PLI 45 | 2.17+ | `process_running_requests` (never answered) |
| 184 | TAKESCREENSHOT | file name, `.png` only | 2.17+ | `screenshot_requests` (no screenshot taken) |
| 185 | BOARDHEIGHTS | GCHAR mapX, mapY, blockX, blockY, w-1, h-1, then w*h cells of 2 bytes: `(b0-0x20-50) + (b1-0x20)/128` | 2.17+ | `board_heights` |
| 186 | BOARDMODIFY2 | GCHAR mapX, mapY, x, y, w, h, then w*h GSHORT tiles, then an unread trailer (ignored; a payload one byte short of `6+2*w*h` is dropped silently; a block past column 63 is clipped) | 2.22+ | `board_modify2` |

Ids Seihai answers or sends only on request: none of the above. Seihai does
not answer 183 ("is X running"), and takes no screenshot on 184. (182 is
answered with a process list of `"seihai"` only; see above.)

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
  crossing). Seihai re-sends on a level change after the first level, and on a gmap
  edge crossing when the profile re-sends (see "Gmaps"). 2.31 every 3.2 s: a 50 ms timer counts modulo
  64 and sends at count 31 (`TamperSender::advance`, driven from the game loop).
- 2.3 has no binary and no captured value, so nothing is sent for it.
- Enforcement, what is known (read-only evidence, 2026-10-31): no server
  seen so far needs the packet. Four public 2.22 servers did not kick a
  client that never sent it for 20 s (2026-10-24), and three more (GServer-RS
  and two gs2) gave a full login without it (`public-gserver-2026-10-30.md`).
  GServer-v2 ignores id 39. Whether a server checks the *value* is still not
  known: that needs a client that sends a wrong one, which is not a
  read-only check. The build box still cannot resolve the listserver
  (2026-10-31), so no new public check was made from it.

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
field 1's use beyond a name; what PLI 24 (value 9, 2.22 and 2.31) is for and what
PLI 46 (always `GCHAR 1`) tells a server; the 2.31 extra step in 156;
whether a server checks the tamper value.

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
  (`EnterLevelSender`). At a gmap crossing it goes out in the crossing frame,
  in front of the props with the cell prop; see "Gmaps". Elsewhere it goes out
  alone and the next walk update is the props frame that follows it;
  `Connection::send_enter_level` can put props messages behind it in the same
  frame. Doors and warps: see "Level changes" below.
- **PLO 186:** header and `w*h` tiles are applied at the position. Bytes after the
  tiles are ignored (a whole second record there is not applied). One byte short
  of `6+2*w*h` is dropped silently, with no disconnect and no partial apply. A
  block past column 63 is clipped. `BoardModify2::apply_to` does that on a
  64x64 board, and the session now calls it: the block goes onto the level at its
  `mapX`/`mapY` cell (see "Gmaps").

## Level changes, server warp and PLI 24 (lab batch c)

Real 2.22 and 2.31 clients against GServer-v2 and a local listserver
reimplementation, loopback only (Graal_analysis lab note, 2026-10-24).

**Level changes (PLI 46 and the tamper check).** 2.31 sends 46 on every level
entry, not only gmap crossings: door or edge links, and a server-side warp.
Where it sits depends on who started the move:

| move | client frames |
|---|---|
| link, level cached | one frame: 30 (`LEVELWARPMOD`), 46, props; before the server answers |
| link, level not cached | one frame: 0 (`LEVELWARP`); the server answers (14, 6, ...); then one frame: 46, props |
| server-side warp | the server sends 14 + 6; the first client frame after it starts with 46; props follow only if the position changed |

2.22 never sends 46. It sends the tamper check (39) in the same frame as every
client-initiated link request, right after the 0 or 30, and not after a
server-side warp. 2.31 does not re-send 39 on a link or warp (its 39 is
periodic).

Seihai (code: `net::EnterLevelSender`, `net::link_request_frame`,
`Connection::send_link`, `net::enter_level_extras`,
`protocol::levelcache`):

- **Level cache.** A level counts as cached once the server has sent its
  board (101) and its mod time (`PLO_LEVELMODTIME`, 39) and the player has
  left it (off a gmap). The cache is in memory for one login; the real
  client's disk cache (`GWEBL001`, Graal_analysis `gwebl001-cache.md`) is
  not read or written. What the real client counts as cached beyond
  "in memory or in its web-level cache" is not isolated by the lab, which
  only revisited levels inside one session.
- **Cached link.** For a profile with `levels.cached_links` (2.05, and
  every build from 2.1 on as an inference; not 1.33 to 2.04) and a cached
  destination other than the current level, the request is
  `PLI_LEVELWARPMOD` (30): `GINT5 mod time, GCHAR x*2, GCHAR y*2, name`,
  as GServer-v2 `msgPLI_LEVELWARP` reads it. The same frame then holds, in
  order: the tamper check (2.22, with `--send-tampercheck`), or 46 and
  the props at the destination (2.31, 2.3). The 46 is therefore not sent
  again when the server names the level.
- **Answer.** The server's answer to a current mod time has no 14, board,
  mod time, links or signs (lab). Seihai takes the position from the link
  request, and fills the level's board, links, signs and mod time from the
  cache. If the server sends them anyway (a stale copy, or a server that
  ignores the mod time) they replace the cached ones. A request with no
  answer within 4 s drops that cache entry.
- Not cached (or a profile without cached links): it asks with 0, the "not
  cached" row.
- A new level name after the first one owes a 46 (profiles with 46 in their
  send set only). Once the server's position is known, the 46 goes out as the
  first message of one frame; after Seihai's own link request the position
  props follow in the same frame, and after a server-side warp the 46 goes
  alone (Seihai does not track whether the position changed).
- The tamper check (with `--send-tampercheck`) now rides in the link-request
  frame for the profile whose rule is "on level change" (2.22), and is no
  longer sent after the server's answer, so a server-side warp does not
  trigger it.
- Level changes started by a level script (`setlevel`, `setlevel2`) go out as
  the same request as a link the player walked into (see "Script-initiated
  warps" below); the real client's own form is not captured.
- Not done: cached links on a gmap (a gmap keeps its own
  per-level slots and uses 35); the disk cache; a lab run of Seihai's own
  30 frame.

**Script-initiated warps (`setlevel`, `setlevel2`).** A client-side NPC script
can change level with `setlevel2 level,x,y` (level name, then the new position
in tiles) or `setlevel level` (the older form: the player's current position,
which a script may have set first with `playerx = ...; playery = ...`). Both
end the script run. Seihai's engine hands the host a `SetLevel` effect and the
host turns it into the session's level request, `pending_link`, the same value a
walked-into link produces. Online, Seihai then sends that request through the
link path (`net::start_online_link`):

- the request is `PLI_LEVELWARP` (0) with the position and level name, or
  `PLI_LEVELWARPMOD` (30) when the level is cached and the profile has
  `levels.cached_links`; everything else in the frame (tamper check on 2.22,
  46 and the props on 2.31 with the cached form) is exactly what a link
  request carries, by profile;
- the session is in the warp-pending state until the server's answer: walking
  and links stop, and a second script warp in that time is dropped (nothing is
  sent);
- the server's answer (`PLO_PLAYERWARP`, `PLO_LEVELNAME`, board, ...) loads the
  new level through the existing path; the 46 is owed after it for the
  uncached form, and not for the cached one;
- the plain walk update of the same pass is not sent: the request carries the
  position;
- a profile that cannot send the request (its send set lacks the id) logs it
  and keeps the connection; any other send failure ends the connection as
  before;
- offline nothing changes: the level is loaded from the asset folder and the
  player arrives at the given position.

Lab (2026-10-25, private lab, 2.31 and 2.22 Seihai against GServer-v2): a level
NPC with `playertouchsme` and `setlevel2 lv_c.nw,10,20` sent `0` (and `39` in
the 2.22 parity run) and was answered with the new level; an NPC in that level
with `playerx = 40; playery = 22; setlevel lv_n.nw` then sent `30`, `46`, props
on 2.31 and `30`, `39` on 2.22 (the level was cached by then). The real clients never ran level NPCs in the lab (2.22 and 2.31 in the
earlier batch, 2.31 again in this one), so what a real client sends for
a script warp is **unobserved**: see `documentation/script-warps.md`. Seihai's
form is the link form, which is an assumption, not a capture.

**Server warp (178).** The lab text was `labtwo,labtwo,127.0.0.2,14803`. The
2.31 client closed its connection, opened a new one to field 2 (host) and
field 3 (port) and logged in again by itself: a new random key, the same
account and password, no listserver step. Field 0 is unused, field 1 is a name
the client did not need to connect, a swapped host and port gave a failed
connection ("The connection couldn't be established"), and a 3-field text was
ignored. The text came from the open-source listserver reimplementation, so
what a real service sends is still unobserved. 2.22 was not tried, nor a host
name instead of an IP, nor a port above 65535.

Seihai (`layouts::ServerWarp`, `net::login_gserver`): decodes the four fields
(fewer than four: ignored), and unless it is turned off (`--no-serverwarp`, or
`follow_serverwarp=0` in the settings file; `--serverwarp` overrides the file)
closes the
connection and logs in again to field 2 and 3 with a new key, the same
credentials and a fresh session (language, tamper check if enabled, the login
46). A port that is not 1 to 65535 or an empty host counts as a failed
connection, as in the real client. Note that, like the real client, it sends
the stored password to whatever host the server names.

**PLI 24 (SHOWIMGPLAYER) and PLO 32.** Both 2.22 and 2.31 send `24 = GCHAR 9`:
2.31 in its first frame after login and then every 5.11 to 5.17 s (median
5.13 s), idle or walking, in every level; 2.22 only while another player is in
the level (every 5.1 s; 0 sends in 3 solo sessions; the exact condition was
not isolated). Level changes do not trigger or reset it. GServer-v2 forwards
the payload unchanged to the other players in the level as PLO 32 (`GSHORT`
sender id + payload); injected payloads of 0, 1, 5, 14, 58, 95, a file name
and an empty one all went through, and neither client showed any reaction.
What the 9 means is unknown. Seihai sends it only with `--send-showimg`, on the
period and condition in the profile (`protocol.showimg`: 5100 ms with others
needed for 2.22, 5130 ms always for 2.31, nothing for other builds), and
decodes PLO 32 as an id and opaque bytes without drawing it. "Another player is
present" is "Seihai has another player on its list".

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
- **2.22 gmap crossing:** no level-warp packet. Client sets `gr.x`/`gr.y` to the global position (local + 64 × cell). When it wraps, the same props message carries local X/Y = 0 and GMAPLEVELX/Y (props 43/44). Server answers with the new level name, then the gmap name. Seihai handles this now; see "Gmaps".
- **Buddy / ignore:** local files (`buddyaccounts.txt`, `ignoredaccounts.txt`). No listserver traffic in this lab. Mute is `PLI 43` with player id + 1/0.
- **Gen-5 XOR:** all 17 real 2.22 client zlib frames (lengths not multiples of 4) decode with the one-step-per-group rule. No bz2 client send was seen.
- **Missing binaries:** no 2.1.x (2.1–2.16) or 2.3 binaries are recoverable. Those rows stay inferred. Alias 2.1–2.16 to 2.05 or 2.17, and 2.3 to 2.31.

## Gmaps

A gmap is a grid of ordinary 64×64-tile levels (`client/src/gmap.rs`). The
sources are the lab captures of real 2.22 and 2.31 clients on a 2x2 gmap
(Graal_analysis feature spec 06, lab notes of 2026-10-24). Everything is
gated by the version profile: `GmapRules::for_version` reads `levels.gmap`,
the profile's `plo` set (49), the own-prop set (43 and 44), `props.precise_pos`
and the `pli` set (35). A profile without them (2.05 and earlier) never enters
a gmap, sends no cell prop and no 35.

**Server to client (what the session does with it).**

- `PLO_PLAYERWARP2` (49) naming a `.gmap`: a gmap session starts with the cell
  from the message. The gmap name is not a level. The first `PLO_LEVELNAME`
  (6) after it names the player's level.
- The `.gmap` file (`GRMAP001`: `WIDTH`, `HEIGHT`, `LEVELNAMES`, `MAPIMG`,
  `MINIMAPIMG`, `NOAUTOMAPPING`, `LOADFULLMAP`, `LOADATSTART`) gives the grid.
  It is read from the asset folder if it is there, else asked for with
  `WANTFILE` and read when the transfer completes.
- After each level it sends, the server sends a 6 that names the gmap again.
  Seihai reads that as "the player's level again": level data (board, links,
  signs, chests) is parked per level while another level is the one being sent
  (a neighbour the client asked for with 35), so the neighbours' data does not
  land in the player's level. `SessionState::current_level_name` is the level
  at the player's cell.
- Boards are kept by level name (`GmapSession::boards`), which is how
  neighbours are drawn when there is no local level file.
- Own props 43/44 in a `PLO_PLAYERPROPS` move the cell. Another player's 43/44
  give its cell, and its position is drawn relative to ours
  (`OtherPlayer::position_from`).
- `PLO_BOARDMODIFY2` (186) goes onto the level the grid puts at its
  `mapX`/`mapY`, with the column-63 clipping of `BoardModify2::apply_to` and
  the 12-bit tile mask. A cell with no known level drops the block (counted).
  A block for a level whose board has not arrived is applied to an empty
  board. Off a gmap the block applies to the current level only when
  `mapX`/`mapY` are 0. Only the player's own cell was tested against real
  clients; the rest follows the field names.

**Client to server.**

- Position: the walk keeps `player_x`/`player_y` local to the cell. When a
  step takes the sprite's x or y outside `0..64` the player moves into the
  next cell (its level is the loaded neighbour), the position is wrapped, and
  one frame goes out, before the server answers: 46 when the profile sends it
  (2.3, 2.31); the tamper check (39) when the profile re-sends it on level
  changes (2.22) and `--send-tampercheck` is on; then `PLI_PLAYERPROPS` with the
  cell prop (43 for x, 44 for y) followed by the wrapped coordinate (X2/Y2,
  78/79, when the profile has precise positions, else X/Y 15/16). Real 2.31
  bytes from the lab are in the tests: east `43=1, X2=0`, west `43=0, X2=63.5`,
  and the same with 44 and Y2. The server's answer then names the new level;
  the 46 is not owed again.
- `PLI_ADJACENTLEVEL` (35): `GINT5` mod time (0, nothing cached) and the level
  name, once per neighbouring level the server has not yet sent. Seihai sends
  them after the grid is known, not at the login frame as the real client does
  (it has the `.gmap` cached already).
- Walls and water across a seam are read from the neighbour's tiles. A cell
  whose level is not loaded, or the edge of the grid, blocks the walk. A step
  that would cross both edges at once is refused and the walk slides along
  one axis. The real client's behaviour at a corner was not captured.

**Not done or not matching the real client.**

- The `gr.x`/`gr.y` flags come from GServer-v2's `-gr_movement` weapon, not
  from the client (lab, 2026-10-30; `gr-flags.md`). Seihai sends them when that
  weapon runs, but with the position in the current cell, where the real 2.22
  uses the whole-map position.
- The real client sends the south crossing's flag first and the props on the
  next half-step; Seihai sends the props in the crossing step's frame.
- Neighbour levels draw tiles only. Their NPCs, signs, chests and links are
  not drawn or active until the player is in the level. No capture shows how
  the real client draws them across the seam; the only note is from the 2.22
  and 2.31 decompile (the grab lookup also searches the neighbouring map
  parts, Graal_analysis `client-internals/ledges-push-pull.md`). Left open
  until a lab case shows it.
- Cached-level crossings: the lab's `gmap_cross` walk (2.22 and 2.31: east,
  back west, south, back north) re-enters levels the client holds, and the
  real client sends the same props frame as on a first visit. It does **not**
  use the `30`-plus-`46` form of a cached level link on a gmap. The server's
  answer leaves out its mod time (39): level name, an empty board packet, the
  gmap name (Graal_analysis `protocol/lab-checks-2026-10-24.md`, feature spec
  06, `version-profiles/lab-enterlevel-231.tsv`). Seihai already does this:
  `cached_mod_time` is `None` on a gmap, and the answer brings the level back
  from its parked slot (test
  `revisit_answer_without_mod_time_restores_the_parked_level`). The `35` a
  client sends for a neighbour it holds (a non-zero mod time) was never
  captured: the lab only had a cold cache, so Seihai always sends 0.
- `PLO_BIGMAP` (171) feeds the M map (`map-screen.md`). No minimap from `PLO_MINIMAP` (172).
- `PLO_BOARDHEIGHTS` (185) is decoded only. The layout is from the 2.17.1,
  2.22 and 2.31 receive handlers; no capture of a server sending it, and no
  note of what the client does with the heights. Left open.
- Not run against the real server in this change; see the PR for the test
  run.
