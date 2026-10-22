# Lab verification (2026-10-22)

These are the results of running the "live protocol lock and codec verification" plan against the cleaned lab export. The lab is offline: GServer-v2, a listserver stub, and 2.05 rev 2 and 2.22 clients under Wine. Only cleaned observations are recorded here. That means no credentials, no account names, and no decompile detail.

The checks run Seihai's own code through the examples in `client/examples/`:

| Example | What it does |
|---------|--------------|
| `lab_replay` | Feeds the exported TCP frames (JSONL) through `Gen3`, `Gen5`, `zlib_decompress`, `bz2_decompress` and `SessionState`. |
| `udp_replay` | Decodes the UDP pcaps with `decode_udp` / `udp_key`. |
| `list_probe` | Probes a lab listserver with the 2.05 `newmain` request and runs `parse_list_reply` on the reply. |
| `session_probe` | Logs a 2.05 client in to a lab gserver and prints a cleaned `SessionState`. |

## 1. Build and offline viewer
- `cargo build -p seihai` failed on `f799d27`, because `protocol/mod.rs` re-exported `frame`, `gen3` and `gen5` twice. That is fixed here.
- With the fix, `cargo test` passes 39 tests.
- `pics1.png` from the 2.05 client is a palette PNG. The viewer refused it, because the reader only took RGB and only knew filters 0–2. The reader now handles 8-bit gray, RGB, palette and alpha images, and all five filters.
- The PNG writer produced invalid files: there were no per-row filter bytes, and a single stored block cannot hold more than 64 KB. It now writes zlib-compressed rows.
- Tiles were drawn from the wrong place on the sheet. `pics1.png` is eight 16-column blocks of 512 tiles, so tile `n` is at column `n % 16 + (n / 512) * 16`, row `(n / 16) % 32`.
- Under Xvfb the window opens on `onlinestartlocal.nw`, shows the correct grass and bush layout, and scrolls when an arrow key is pressed.
- **Open:** a held key moves one tile per frame (about 60 tiles a second), which walks off the 64×64 board within a second. There is no movement speed or edge clamp yet.

## 2. Listserver reply
This was checked against the lab stub's first server frame. A stub is not a real listserver, so the shape still needs a check against one.

- **2.05 (14921):** the client sends a plain zlib frame containing id 0 `newmain`. The reply is a plain zlib frame.
- **2.22 (14922):** the client sends id 7 = GCHAR key + `GNW30123newmain`. The reply is a gen-5 frame, ctype 4, keyed with that key, and its iterator starts at the seed.
- **Body:** the body is id 0 `SVRLIST`: `c count`, then for each server `c 8` followed by 8 GSTRINGs (type+name, language, description, url, version, players, ip, port). After that comes id 2 `STATUS` text.
- **Parser:** the old `parse_list_reply` expected `name,host,port` text lines and found nothing in this reply. It now parses the GSTRING layout above, plus ids 2–5. Both stub replies are unit tests.

## 3. Gen 3 (2.05 client→server)
Seihai's `Gen3`, resumed from each frame's exported iterator state, was run over 9 sessions:

- **Mismatches:** 0 across 704 messages.
- **Index 0:** 116 messages had the junk byte at index 0, before the id byte.
- **Chain:** the iterator after each frame equals the next frame's starting state in every case.
- **Seed:** a fresh `Gen3::new(key)` is right. The login line is clean, and the first iterator step belongs to the first message after the login line.

## 4. Gen 5 (2.22)
- **Group rule:** `Gen5` un-XORs every frame of both 2.22 sessions in both directions, with 0 chain breaks and 0 decompression failures. The group limits are type 2 → ceil(n/4) capped at 12, and types 4/6 → floor(n/4) capped at 4.
- **Seeding:** each direction is seeded on its own at `0x04A80B38`.
- **bz2:** one real ctype 6 frame (server→client, 3275 bytes on the wire) un-XORs with 4 groups and decompresses with `bz2_decompress`. It is GServer that chose bz2 for that body, which was over 0x2000 bytes.
- **Still open:** a client→server bz2 frame, from the client's own ≥ 0x1000 path.

## 5. Level-entry packets
This was checked with a live `session_probe` run against the lab GServer, using a throwaway 2.05 account.

- **What came through:** 6 frames. `level_name` = `onlinestartlocal.nw`, 1 board line, 3 chests and one `PLO_PLAYERWARP` (14).
- **Login line:** `write_login` wrote the client type and key as raw bytes. Both are GCHARs, so 2.05 now opens with `0x20` + (key + 0x20). The lab server accepted the corrected line.
- **Player props:** `PLO_PLAYERPROPS` used to be read as raw `(prop, value)` byte pairs, with x/y/dir at props 0/1/2. It is actually a GCHAR prop stream with variable-length values. `parse_player_props` now walks it using the PLPROP encoding table.
- **Prop 10:** GServer sends a 2.05 login a plain GSTRING gani name (`idle`), not the pre-2.1 bow form. Reading it as the bow form made the parse drift after prop 10. The cleaned login burst (props 1–25, cut before the account prop) is now a unit test. APCOUNTER (25) is a GSHORT, and it reads 60 s.
- **Position:**
  - The login burst carries no X/Y. The start position arrives in `PLO_PLAYERWARP` (`c x*2, c y*2, level`), and `SessionState` now takes `player_x`/`player_y` from it.
  - Direction comes from SPRITE (17) mod 4.
  - Later X/Y updates come through props 15/16.
- **Frequent unhandled ids:** 32, 34, 42 `NEWWORLDTIME` and 43.

## 6. UDP peer channel (2.05)
- **Format:** a datagram is `seq` + GCHAR id + fields, and the whole datagram is XORed with `recipientId XOR 1`. The old scaffold sent `[21, x, y, dir]` with no seq, no XOR and no +0x20.
- **New functions:** `encode_udp`, `decode_udp`, `udp_key` and `encode_udp_move`, which builds the id 0 movement message of six GCHARs.
- **Sequence counter:** all 222 datagrams decode with `seq` counting from 0 for each sender→recipient pair.
- **Player ids in run 4:** they changed on reconnect, to 14899 = 3 and 14898 = 4.
- **Ids seen:** 0 movement (8 bytes), 10 bomb add, 15 showimg, 20 item delete and 26 trigger action.
- **Port:** 14897, 14898 and 14899 were all used, so the peer port is configurable (`fixedudpport`) rather than fixed.
- **Id numbering:** ids 21, 23 and 24 are hit, level name and entered level, not position, props and board. The constants are renamed to match. None of the three appears in the captures.

## 7. Sprites and HUD (d1c499c)
This check uses the player sheets from the lab 2.05 client folder: `body.png`, `head0.png`, `sprites.png`, and `idle.gani` for the offsets. It is run with `cargo run -p seihai --example render_sprites -- <assets> onlinestartlocal.nw <out>`, from the start position 30, 30.5.

Problems found in d1c499c:
- **PNG reading:** the sheets are 4-bit palette PNGs with `tRNS`. The reader only took 8-bit images and dropped alpha, so the sprites could not load. `read_png_rgba` now reads 1/2/4/8-bit palettes and applies `tRNS`, and the sprites blit as RGBA.
- **Direction order:** the offline session used 0 south, 1 west, 2 east, 3 north. Graal's order, in the ganis, on the 2.05 wire and in `scripting::DIR_*`, is 0 up, 1 left, 2 down, 3 right. With the old order the player faced away when walking down. `offline::DIR_*` and the ASCII view now use Graal order.
- **Player placement:** body and head are now placed with the `idle.gani` offsets in the 48×48 player box, per direction, with the shadow from `sprites.png`. Before, they were at fixed offsets from the screen centre.
- **Camera:** the camera now centres on the middle of the player box, to the pixel. It used to snap to whole tiles from the player's top-left.

Result next to the real 2.05 screenshot (`screens/client205-onlinestartlocal.png`):
- **Tiles:** the tile layout matches. The horizontal alignment matches to the pixel. Vertically Seihai sits about 12 px lower, because the real client centres on the play area above its 24 px chat bar.
- **Player:** the player is recognisable, and the four step images show the back, right, front and left of the head and body.
- **Chests:** the three chests appear as 2×2-tile markers in the right places. They are not chest art yet.
- **HUD:** the HUD bar shows at the top left. It is a placeholder, not the real hearts and counters.
- **Fallback:** without `body.png`/`head0.png` the player falls back to a coloured marker, with no crash.

Not drawn yet: shield, hat, sword, the nickname tag, and gani animation.

## 8. Gen-5 wire and file reply (1bbcb13)
`1bbcb13` was checked against the lab 2.22 frames (`run1`, `run6`) and the GServer-v2 source. Three of its changes do not match the wire, and the file reply used the wrong ids.

- **Gserver login:** in the 2.22 capture the client sends exactly one frame before the first ciphered one, and that frame's successor starts with the iterator at the seed. So the login line itself is the plain zlib frame. The id 7 `GNW30123newmain` handshake is the listserver's first packet, not the gserver's. GServer reads the first frame as the login, so a handshake there is taken as a login of client type 7. Seihai now sends the plain zlib login to the gserver, and `login_listserver` sends id 7 then a gen-5 id 1 (account, password) to the listserver.
- **XOR group count:** the iterator steps once per 4-byte group the body really has, capped at 12 (`0x02`) or 4 (`0x04`/`0x06`). Stepping a fixed 12 or 4 times breaks the chain after the first short uncompressed frame. Over 610 lab gen-5 frames the per-group count decodes all of them, and the fixed count decodes 14.
- **`PLO_RAWDATA`:** the header is `GCHAR 100` + `GINT3` (three GCHARs, high bits first) + `\n`, and the next *n* bytes are one message. The lab board header reads 8194 = 1 + 64·64·2 + 1. `1bbcb13` read three raw little-endian bytes and kept the `\n`. A raw block cut at a frame boundary is now finished by the next frame.
- **File reply:** `PLO_FILE` is id 102 and always sits behind `PLO_RAWDATA`. For 2.1+ it is `GINT5 modtime` + `GCHAR name length` + name + data + `\n`; for 2.05 there is no modtime and no `\n`. The other ids are 30 `FILESENDFAILED`, 45 `FILEUPTODATE`, 68/69 `LARGEFILESTART`/`END` and 84 `LARGEFILESIZE`. Seihai had 45 as the file and 84/86 as start/end.
- **1.39 login:** GServer picks gen 2 when the token follows the client type directly, so the gen-2 login has no key byte.
- **Disconnect text:** `PLO_DISCMESSAGE` (16) is kept, and the "connection closed" error shows the server's reason when it sent one. A public server that refuses a login (version not allowed, login server offline, bad account) says why in this message before it closes, which a bare "failed to fill whole buffer" hides.

Checks: unit tests replay lab 2.22 frames (client frame 1 from the seed; server frames 2–4 across a 7-byte uncompressed frame), and loopback tests cover the gen-5 login, both listserver logins and the gen-2 login. A loopback gserver that follows GServer-v2's framing logged Seihai in as 2.22, sent a missing `new1.nw` through `PLO_RAWDATA`/`PLO_FILE` (byte-identical on disk), and decoded every walk frame.

Still open: a public gserver run. `session_probe` now takes a version (`... session_probe -- <host:port> 10 2.22`) and prints frame sizes, ids, the level name and the disconnect text.

## 9. Chat, player list and overlays
Layouts are from GServer-v2 and the 2.05 notes (`chat-pm-rc.md`); they still need the lab check.

- **Chat bar (Enter):** a line goes out as CURCHAT (prop 12) in `PLI_PLAYERPROPS`, which is the bubble. A line that starts with `toall:` goes out as `PLI_TOALL` (6) instead, as 2.05 does. Before, Enter sent a fixed "hello" as a raw, unframed `PLI_TOALL`, which a gen-3 or gen-5 server cannot read.
- **PM (F2):** `PLI_PRIVATEMESSAGE` (28) is a GSHORT count, GSHORT player ids, then the gtokenized text, to the nearest player. Before, it sent `name:text` unframed.
- **Incoming:** `PLO_TOALL` (13), `PLO_PRIVATEMESSAGE` (37, `"type","text"` list), `PLO_RC_ADMINMESSAGE` (35) and other players' CURCHAT go into a 200-line history. PgUp/PgDn scroll it. Text is single-byte (Latin-1).
- **Other players:** the id in `PLO_OTHERPLPROPS` and `PLO_DELPLAYER` is a GSHORT, not the first byte. Updates are partial, so a prop that is absent keeps its value (a chat-only update used to reset the player to 0,0). PCONNECTED (51) or JOINLEAVELVL 0 removes the player. X2/Y2 are read too.
- **Overlays:** nickname tags and bubbles use a 5×7 bitmap font drawn for Seihai. F4 toggles the player list, F3 a debug overlay (fps, position, level, frames and messages, players), F12 saves a PNG to `screenshots/`. Esc closes the chat bar, and quits only when it is closed.

Loopback check (2.22 framing, a scripted second player): the server got the CURCHAT, toall and PM with the right layouts. The bubble, nickname, toall, PM, history, list and debug overlay all showed.
