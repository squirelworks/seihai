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
| `WANTFILE -.gif` | The 2.22 and 2.31 clients also asked for a file named `-.gif` in this frame (non-gmap logins) or the next (2.22 gmap login). What triggers it is not known; the captures never vary it. It stays a known difference. |
| the order of the gmap file and `ADJACENTLEVEL` messages inside the frame | Seihai sends those in their own frames; bundling is the next change (frame bundling, divergence 10). |
| `PLI_ENTERLEVEL` (46) and the `gr.x`/`gr.y` flags inside the frame | Separate items (15 c and d). |

## Tests

`protocol::login::tests` pin the image order, the icon names, the process list text,
the byte layout of the 2.22 props (including the `TEXTCODEPAGE` 1252 bytes seen in the
capture), the 2.17 cut-off at `PLANGUAGE`, that the frame goes out once and only after 182,
and which builds send it.
