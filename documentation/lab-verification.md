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
