# The post-login frame

Queue item 15(b), from the differential testing (`documentation/difftest.md`,
divergences 2, 3, 4 and part of 7 and 10). A real client answers the server's
login burst with one frame. Seihai used to answer with nothing: no image
checks, no process list, no login props.

Evidence: real 2.17, 2.22 and 2.31 clients in the private lab (2026-10-24 and
2026-10-25). The 2.17 capture is the early login capture; 2.22 and 2.31 are the
`login`, `gmap_cross`, `chat` and `door_warp` scenarios. Nothing here comes from
a public server.

## What the real clients send

The server's burst ends with `PLO_LISTPROCESSES` (182). About 60 ms later the
client sends one frame:

| order | message | content |
|---|---|---|
| 1 | `PLI_UPDATEFILE` (34) or `PLI_WANTFILE` (23), one per image | sword image, shield image, head image, body image (the images in the server's own-player props, in prop order 8, 9, 11, 35), then the image of each weapon in the order the server added them. A file the client has gets `UPDATEFILE` with its mod time; a missing one gets `WANTFILE`. |
| 2 | `PLI_PROCESSLIST` (44) | the answer to 182. The text is the client's list of running programs, gtokenized, one per line; GServer-v2 parses it with `guntokenize` and splits on newlines. |
| 3 | `PLI_PLAYERPROPS` (2) | `NICKNAME`, `HEADGIF`, `COLORS`, `STATUS`, `UDPPORT`, `ADDITFLAGS`, `PLANGUAGE`, and in 2.22 and 2.31 `OSTYPE` and `TEXTCODEPAGE`. Head, colours and status are the values the server gave. |

2.17 sends the same three parts; its props stop at `PLANGUAGE`.

Then the status icons: one `WANTFILE plisticon<status>.png` per
`PLO_STATUSLIST` (180) entry, lower-cased (`No PMs` is `plisticonno pms.png`).
They go out four at a time, as the request queue of `documentation/file-cache.md`
says: the first four are in flight and the rest follow as replies arrive.

2.17 and 2.22 also check `idle.gani`; 2.31 does not. The 2026-10-30 Windows lab
(stock beta4, decoded message by message) put the 2.22 `WANTFILE idle.gani` first
in the post-login frame, about 240 ms after the level; Seihai now sends it there,
or alone at that time when the frame is not due (`difftest-leftovers.md`, section
7). The earlier reading, "in the first frame after the login line, before the
burst arrives", came from the patched lab and is replaced.

## What Seihai does now

`protocol/login.rs` builds the frame, and the profile (`protocol.login`, set on the
2.17, 2.22 and 2.31 steps of the version chain) says which builds send it. Builds
without a recorded frame send nothing.

* The image names come from the server's own `PLO_PLAYERPROPS` and the weapon list;
  the file checks go through the request rules of `filecache.rs`, so a file Seihai has
  gets `UPDATEFILE` with its mod time and a missing one `WANTFILE`, with four in flight.
* `PLI_PROCESSLIST` carries one entry, `seihai`. The real clients list every running
  program; Seihai does not read the process table, and the server only parses the text.
* The login props are built from the server's props as they came, with the account as
  `NICKNAME` and `ADDITFLAGS` 0.

### Left out on purpose

| what | why |
|---|---|
| `UDPPORT` (31) in the props | The real clients bind a UDP socket and name its port. Seihai binds none (the UDP channel is not wired into the game loop), and a port nobody listens on would be a false claim. |
| `OSTYPE` value | The real clients said `wind`. Seihai says `wind` as well: it is the one value seen, and servers know it. A Linux or macOS value was never observed. |
| the real process list | See above. |
| the order of the gmap file and `ADJACENTLEVEL` messages inside the frame | Seihai sends those in their own frames; bundling is the next change (frame bundling, divergence 10). |
| `PLI_ENTERLEVEL` (46) and the `gr.x`/`gr.y` flags inside the frame | Separate items (15 c and d). |

The `-.gif` request seen in these frames is a weapon image: the lab server gave a
weapon the image `-`, and the clients add `.gif` to an image name with no extension.
Seihai does the same for weapon images (it first asked for `-`). The real clients
also ask for `wbowi1.png`, which no weapon of the lab server names: it comes from
the default-weapon ids the server sends (8 then 7). Seihai asks the image of each
default weapon in arrival order: id 7 `wbowi1.png` (lab, real 2.22 with the ids
filtered to 7 alone) and id 8 `wbomb1.png` (the 2.05 rev 2 item table), which
gives the real 2.22 and 2.31 order, `wbomb1.png` before `wbowi1.png`
(`file-cache.md`, `weapon-images.md`).

## Without `PLO_LISTPROCESSES`

Stock GServer beta4 never sends 182. Lab, 2026-10-30 (Windows, real 2.22 and 2.31,
two runs each, decoded message by message): the client sent its frame anyway,
236 to 246 ms after the level frame. Before it the server had sent the
signature, the start messages, `STATUSLIST`, the full own props, the weapon
messages and the level, all within 5 ms; the client had sent only `LANGUAGE`
and `TAMPERCHECK`. The frame:

| order | message | content |
|---|---|---|
| 1 | `WANTFILE idle.gani` | 2.22 only |
| 2 | `UPDATEFILE` / `WANTFILE`, one per status icon | the four in flight |
| 3 | `PLI_ENTERLEVEL` (46) | 2.31 only |
| 4 | `PLI_PLAYERPROPS` (2) | `NICKNAME`, `HEADGIF`, `COLORS`, `STATUS`, `UDPPORT` 0, `ADDITFLAGS`, `PLANGUAGE`, `OSTYPE`, `TEXTCODEPAGE` |
| 5 | `SHOWIMGPLAYER` (24) | |

No `PLI_PROCESSLIST` and no image checks in the frame. The other icons, then the
sword, shield and body images and the weapon images (`wbomb1.png`, `wbowi1.png`)
followed in later frames through the four-in-flight queue. **The head image was
never checked.**

Seihai does the same (`LoginBurst::due_at`, `late_frame`): when no 182 has come
240 ms after the first level (`FRAME_AFTER_LEVEL`, timed from the moment the
level's frame was read off the socket), it sends this shape. Images the window
lets out at once follow the props. `SHOWIMGPLAYER` stays opt-in (`--send-showimg`),
as for the 182 frame, and `UDPPORT` stays out (see below). A 182 that comes first
keeps the frame above. Real 2.17.1 was captured on beta4 later the same day
(`difftest-217.md` item 2): the same shape (status icons, the `gr` flags, the props,
`SHOWIMGPLAYER`, no process list), but 96 to 112 ms after the board rather than
240 ms, so 2.17 waits 96 ms (`LoginInfo::frame_after_level`).

## Gen-2 builds (1.21 to 1.39)

These builds have no post-login frame and no process list. Their first
`PLI_PLAYERPROPS` goes out once the first level has loaded, with `NICKNAME`
(0), `HEADGIF` (11), `COLORS` (13), `STATUS` (18), `UDPPORT` (31) and
`ADDITFLAGS` (33) (Graal_analysis `version-profiles/1.33-1.39.md`).

Lab check, 2026-10-30. The real 1.33 and 1.39 logged into a gen-2 stub 13
times (zlib frames, the server's props, the level and its board, file requests
answered with `FILESENDFAILED`). Each run changed the server's head or status:

| server sent | real 1.39 sent | real 1.33 sent |
|---|---|---|
| nick `stubnick` | `stubnick` (the server's, not the account) | the same |
| head `head3.png` | `head3.png` | `c(0)` |
| head `head1.gif`, `head5.gif` | `c(1)` | `c(0)` |
| head `c(5)` | `c(5)` | `c(5)` |
| colours | as sent | as sent |
| status 0, 1, 2, 4, 8, 32 | 0 | 0 (status 0 tried) |
| status 16, 20 | 16 | 16 (status 16, 20 tried) |
| status 64 | 64 | not tried |
| then | `UDPPORT` (a new port each run), `ADDITFLAGS` 0 | the same |

Seihai builds the message in `classic_login_props_message` with those rules.
The test `classic_login_props_match_the_real_1_33_and_1_39` holds the real
bytes of nine of the runs. Against the same stub, Seihai 1.33 and 1.39 sent the
real clients' bytes exactly, apart from `UDPPORT`: Seihai binds no UDP socket,
so it leaves that prop out, as for the later builds.

Still different in those runs (not changed here):

* Seihai asks for the level file (`WANTFILE onlinestartlocal.nw`) before the
  props. The real clients do not.
* The real 1.39 asks for the head (`head3.png`) twice, before the props.
  Seihai asks once, after them. The real 1.33 asks for `head1` for
  `head1.gif`.

Not done:

* The default weapon registration: id 25 (the full weapon) up to 1.37, and
  id 33 (`c(0)` + default weapon id) from 1.38. The stub sent no weapons, and
  neither client sent either id.
* 1.34 to 1.38 are taken to behave as 1.33. They were not run.

### 1.21 to 1.32

Lab check, 2026-10-30. The real 1.28 beta 2 (`graal128b2/Graal.exe`), 1.3 test
(`graal13b/Graal13.exe`), 1.31 and 1.32 logged in to the same gen-2 stub under
Wine (server head `head3.png`, status 20). Each one's login line is the 1.33
one with its own `GSERV` token. Each asked `WANTFILE head3`, then sent its props
once the level was in:

| build | props after the nickname, head `c(0)` and colours |
|---|---|
| 1.28 beta 2 | `UDPPORT` only: no `STATUS`, no `ADDITFLAGS` |
| 1.3 test | the same |
| 1.31 | `STATUS` 16, `UDPPORT` |
| 1.32 | `STATUS` 16, `UDPPORT`, `ADDITFLAGS` 0 (as 1.33) |

`classic_login_props_message` sends `STATUS` from 1.31 and `ADDITFLAGS` where
the build's own props reach 33 (1.32 on). Against the stub, Seihai 1.28 beta 2,
1.3 test, 1.31 and 1.32 sent each real client's login line (account and
password masked) and props byte for byte, apart from `UDPPORT`. The test
`classic_login_props_match_the_real_1_28_to_1_32` holds those bytes.

* **1.31 and the burst order.** The real 1.31 stalled at "Loading account from
  server (1/5)" when the level name came before the board. With GServer13's order
  (props, warp, raw-data size; the board frame; level name, level mod time,
  leader) it went in. 1.32 to 1.39 take either order.
* **1.21** was not run: its archive holds only an InstallShield setup. It takes
  1.28 beta 2's props, without `UDPPORT` (it has no UDP).
* **zlib level.** Every real build from 1.28 beta 2 to 1.39 compresses its frames
  at zlib's best level (`78 DA`); Seihai's gen-2 frames use the default (`78 9C`).
  Not changed here.

## 3.04 (Linux)

Lab check, 2026-10-30. The real 3.04 (the Linux installer's own executable,
run natively with i386 libraries on Xvfb) and Seihai `--version 3.04` each
logged in to the lab's GServer-v2 and lab listserver, walked right and swung
the sword. Graal_analysis `version-profiles/lab-304-linux-2026-10-30.md` has
the setup.

What the real 3.04 sends, and what Seihai now sends:

| step | real 3.04 | Seihai 3.04 |
|---|---|---|
| listserver | one plain zlib frame (`78 9C`): id 7 (key, `G3D30123`, `newmain`) and id 1 | the same; byte for byte once the key, account and password are masked |
| game login | one plain zlib frame: login (type 5, `G3D03014`), `LANGUAGE`, `UPDATEFILE bigmap.txt`, `UPDATEFILE minimap.txt` | the same, byte for byte with key, account, password and mod times masked. A map file Seihai does not have (in `<assets>/maps/` or `<assets>/`) gets `WANTFILE` |
| tamper check, process list, id 24 | none in the whole session | none |
| post-login frame | once the board is in (182 is not handled): `UPDATEFILE` head image, `wbomb1.png`, `wbowi1.png`; the `gr.x`/`gr.y` flags; the status icons; then props | the same order. No sword, shield, body or `-.gif` checks, as in the real one |
| login props | `NICKNAME`, `STATUS`, `UDPPORT`, `ADDITFLAGS` 0, `PLANGUAGE`, `OSTYPE` `linux`, `TEXTCODEPAGE` 1250; no `HEADGIF` or `COLORS` | the same without `UDPPORT`; the bytes after the nickname match except `STATUS` (below) |

Still different:

* **Nickname.** The real client sends the start window's nickname field; Seihai
  has no such field and sends the account.
* **Status.** The real one sent 21 where the server gave 20 (bit 1 set). Seihai
  sends the server's value. Which client state sets bit 1 was not traced.
* **`UDPPORT`**, left out as for every build.
* **Status icons.** The real client asked only the icons it lacks, with
  `WANTFILE`, plus `chest.png` for the level's chests. Seihai checks every
  icon of the list through the usual file rules and does not ask the chest
  image.
* **First `gr.y`.** At the same start (30, 30.5) the real 3.04 sent `gr.y=30`, the
  real 2.22 and Seihai `gr.y=30.5`. Not traced.
* **Walk and sword GANI files.** The real client sent `UPDATEFILE walk.gani` with
  its first step and `UPDATEFILE sword.gani` with its first swing. Seihai asks a
  GANI only when it has no copy (`documentation/file-cache.md`).

The walk and sword props follow the 2.22 cadence (`protocol/walkcadence.rs`):
`walk` + X + `SPRITE`, then X every 100 ms, with the `gr.x` flag every tick; the
real 3.04 sent `idle` one tick after the last step, with its position, and the
sword GANI with no `SPRITE`, `idle` 249 ms later (Seihai 244 ms).

## 3.153 (Mac OS X)

Never run. Its frame is 3.04's with `OSTYPE` `mac`, the value its binary
writes; the code page (1250), the first-frame files and the image checks are
3.04's. What is locked and what is carried is in `3x-session.md`.

## Tests

`protocol::login::tests` pin the image order, the icon names, the process list text,
the byte layout of the 2.22 props (including the `TEXTCODEPAGE` 1252 bytes seen in the
capture), the 2.17 cut-off at `PLANGUAGE`, that the frame goes out once and only after 182,
and which builds send it. A weapon image without an extension gets `.gif`.

## Before and after (lab, real client against Seihai, `login` scenario)

Ranked difftest report, default Seihai options (no opt-in senders):

| | real client | Seihai before | Seihai after |
|---|---|---|---|
| 2.22 client-to-server messages | 24 | 3 | 21 |
| 2.22 divergences (high / medium / low) | | 5 / 7 / 2 | 2 / 4 / 2 |
| 2.31 client-to-server messages | 37 | 4 | 21 |
| 2.31 divergences (high / medium / low) | | 5 / 8 / 2 | 2 / 5 / 2 |

The messages the real client still sends that Seihai does not are the opt-in
`TAMPERCHECK` and `SHOWIMGPLAYER` (2.31: 9 and 6 in the run), the `gr` flags and the
`PLI_ENTERLEVEL` of the next items, and `UDPPORT`. What is left in the medium rows is
the process list text (real 285 bytes, Seihai 8), the missing `UDPPORT` prop, a
`wbowi1.png` request, and Seihai's own request for the level file. The missing
`UPDATEFILE`/`FLAGSET`/`PROCESSLIST`/`PLAYERPROPS` ids no longer appear in the report, and the server's
`FILEUPTODATE` and `FILESENDFAILED` replies now arrive as they do for the real client.
