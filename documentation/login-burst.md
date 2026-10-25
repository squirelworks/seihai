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

2.17 and 2.22 also send `WANTFILE idle.gani` in the first frame after the login
line, before the burst arrives. 2.31 does not.

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
Seihai does the same for weapon images (it first asked for `-`). What is still
different: the real clients also ask for `wbowi1.png`, which no weapon of the lab
server names (the server sends the default-weapon ids 7 and 8 as well); which
default weapon gives which image is not known, so Seihai does not ask for it.
Evidence for a later decision, not used: the server source numbers the default
items bow 7 and bomb 8, the 2.22 item-name table is in the same order, and the
real 2.22 and 2.31 logins ask `wbomb1.png` before `wbowi1.png`, as a walk over
default weapons 8 then 7 would. No mapping is wired (`file-cache.md`).

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
