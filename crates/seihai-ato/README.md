# seihai-ato

The Torque client line: Graal 3.04 (Linux, Jan 2004), 3.153 (Mac,
Feb 2005), and the 4.x and 5.x builds (4.04r2 to 5.31r4), which log in to
a game server named `Login` first and play after its server warp (178).
The 4.x and 5.x start window is 3.153's until their own GUI files are
read.

The 3.x clients still speak gen-5 framing and use the same board, gmap and
GS1 formats as 2.22 and 2.31, so this crate shares `seihai-core` with the
2.x client (`seihai-mae`). What is 3.x-only lives here.

Ato also owns the Torque reimplementation for v3, v4 and v5
(`src/torque/engine`). It is compiled into the default build. It is on
when the selected era is 3, 4 or 5, and off for era 6 and later
(`--features era6`). v3, v4 and v5 are three different module sets.
v3 is the full TGE surface plus `GuiGraalCtrl`. v4 and v5
each enable their own 2D subset and 3D subset. The stub's era defaults
to v3, which is the full set when nobody has picked. graal-client-repro
owns how the original engine was shaped. This crate owns what a new
client runs. They share notes, not code. See `src/torque/engine/README.md`.

## Stage 1

- `args`: the command line. `seihai-launcher` passes the user's arguments
  through, so unknown flags are ignored. `--version` takes any 3.x, 4.x
  or 5.x row (`3.04`, `3.153` the default, `4.04r2`, `4.20r8`, `5.00r7`,
  `5.12`, ...); `--account`, `--password-env VAR`,
  `--nick`, `--no-sound`.
- Sound: walking plays the footstep and water cues of `walk.gani` and
  `swim.gani` from the asset folder (`--assets`), through seihai-core's
  sound engine and device output (the `audio` feature, on by default;
  `--no-default-features` builds Ato without it). A sound that is not on
  disk is asked for from the server (WANTFILE) and plays on a later cue
  once it is saved, as in 2.05.
- `wire`: the build's game and listserver tokens and the ids it adds or
  drops against 2.31, all read from the build's row in
  `seihai-core::protocol::chain` (nothing restated).
- `torque`: the Torque connect profile per protocol number (9 for 3.04,
  19 for 3.153 and 4.04r2, 21 for 4.20r6, 24 for 5.12): game string, class
  table, 12-bit class ids, 14-bit length prefixes, and the port rule
  (listed port + 1). Data only. Source: Graal_analysis
  `torque/wire-by-build.md`.
- `torque::net` and `graal3d`: the 3D side of a Graal3D server, the
  Torque UDP link to the listed port + 1, for 4.20r6 and 5.x (protocols
  21 and 24) and, partly, 3.153 and 4.04r2 (protocol 19): bit stream and strings, handshake, data packet numbering
  and acks, events (the mission start is answered), ghosts and the
  control object. `--graal3d` with `--online host:port` opens it beside
  the TCP login, with or without the window, and logs its progress with a
  `3d:` prefix (login steps, ghosts created, the control object). The
  single launcher passes `--graal3d` for a `3 ` server, and a server warp
  (`PLO_SERVERWARP`) to a `3 ` server opens it too; a warp elsewhere or the
  end of the TCP session closes it. While it is up the window leaves out
  the TCP level boards, as the real clients do. The shapes and Sky textures
  the ghosts name are asked for over TCP through core's session, and
  login step 6 is logged once every request is answered. Graal text over
  BigData (`graal_text`) is applied and logged: flags, `events`, weapons,
  `addnpc`, `setmaterials`, `rpg` and weapon texts; zlib transfers are
  unpacked. The weapon scripts run on core's GS2 VM (`scripts3d`): the
  top-level code and `onCreated` when a script comes, `onReceiveText`
  for a weapon's text, sleeps and timers on the 50 ms frame. Bytecode is
  run as sent; source text is compiled with core's compiler. What a script
  sets (`set`, `client.` variables) and its `triggerserver` calls go back
  as one `events` text (`s` and `t` lines). The world is drawn by `view3d`
  (below). 3.153 and 4.04r2 (protocol 19,
  the stock Torque game layer, `torque::net::tge`) run as far as the
  static reads go and have not met a real protocol 19 server: the three
  mission phases are acked (3.153 echoes the phase sequence, 4.04r2 does
  not), datablocks are counted by class, the stock ghosting and
  datablocks-done connection messages are answered, ghosts are kept by
  class name, and the client sends its position with no moves. The control
  object's own packet data (stock `Player`) is not read yet, so the rest of
  a packet that carries it is skipped. Source: Graal_analysis
  `torque/udp-session.md` (§10 for protocol 19),
  `graal3d/torque-engine-4x.md` and the open (MIT) Torque net layer.
- `view3d`, `three_ds` and `dts`: the 3D world as the link keeps it, drawn
  flat shaded by a small software rasteriser. Shapes are `.3ds` and Torque
  `.dts` (versions 23 to 25; the rest pose of the first detail level that is
  not a billboard, placed by the nodes' default transforms; skins from their
  initial vertices). The Sky's six textures (PNG, GIF or BMP) are drawn as a
  box around the eye, else its solid colour; shapes fade into the fog colour
  from half the visible distance and are not drawn past it. Nicknames (in
  their colour) and chat go over each object, written by the window. Not
  drawn yet: `.dif` interiors, `.ft` shapes, textures on shapes, and
  terrain (no server sends `TerrainBlock` or terrain chunks yet).
- `start_window`: the 3.x start window, laid out from the clients' own GUI
  files (positions, texts, colours, language list). Start opens the
  server list; with `--online`, it hands the form straight to Stage 2.
  Tutorial and Options say they are not in Ato yet.
- `server_list`: Start logs in at the version's listserver with core's
  `Connection::fetch_server_list` (`listserver.graal.in` on the row's
  port, the `G3D30123` token, login shape D as the real 3.04 and 3.153
  sent it in the lab; `--listserver host:port` for a lab listserver),
  shows the servers, and logs in at the one picked as `--online` does.

## Stage 2

- `online`: log in at a game server (`--online host:port`) with core's
  login, which already carries the 3.04 and 3.153 login frames from the
  lab run of the real 3.04, and read every frame with the core readers.
- `level`: keep the level the way `seihai-mae` does (a local copy at the
  server's mod time if `--assets` has one, otherwise the server's board,
  with the server's links, chests and signs on it) and log it once the
  server goes quiet: the level name, the start position and the NPC count.

`--no-window` logs in without the start window; `--first-level` hangs up
after the first level is logged (for lab runs).

```
cargo run -p seihai-ato -- --version 3.153
GRAAL_PW=... cargo run -p seihai-ato -- --version 3.04 --listserver 127.0.0.1:14922 \
    --account lab --password-env GRAAL_PW
cargo run -p seihai-ato -- --version 3.04 --print-profile
GRAAL_PW=... cargo run -p seihai-ato -- --version 3.04 --online 127.0.0.1:14900 \
    --account lab --password-env GRAAL_PW --no-window --first-level
```

The output of a loopback run (`tests/online_loopback.rs`) looks like:

```
logged in at 127.0.0.1:14900 as lab (3.04)
level lab3x.nw: from the server's board, start (30.5, 31), 1 NPCs (1 from the server, 0 in the file)
```

## Stage 3

The loop and the renderer below live in the `seihai-play` crate, shared
with `seihai-issho` (6.x). Ato keeps the start window, the server list,
the 3.x window size and its Torque field lines.

- `render`: the flat 2D fallback. It draws the level's tiles as
  `seihai-mae` does (16-pixel tiles from `pics1.png`, from the asset
  folder or the built-in sheet; the camera centred on the player's box;
  chests from their records) and the player as a plain marker. The tile
  maths is copied from mae, not shared, so the two lines stay apart.
- `game`: after Start, the 640x480 window shows the level the server sent.
  The arrow keys walk it with core's `OfflineSession::try_move`, which
  gives the same pace, walls and water as mae. Other players the server
  places in the level (`PLO_OTHERPLPROPS`) show as yellow markers, with
  their nickname under the marker when the server sent one.
- NPCs: the level file's NPCs and the ones the server sent for the level
  (`PLO_NPCPROPS`, joined through core's `take_server_npc`, so a file NPC
  at the same place with the same image is one NPC) are drawn as their
  PNG or GIF image from the asset folder, with `setimgpart` parts and the
  clear pixels left out. An image not in the asset folder is drawn as a
  purple outline and named once on stderr; an NPC with no image draws
  nothing. NPC props that come after the level (moves, new images, a new
  NPC) are applied as they arrive, through core's `Live::npc_changes`, to
  the level the view is on; missing images are not asked for from the
  server. A `PLO_NPCDEL2` delete takes the NPC off the level it names;
  `PLO_NPCDEL` (29) and `PLO_HIDENPCS` (151) are not applied yet.

The walk is sent: the view hands its pose to the network thread, which
sends it once a 50 ms tick after the post-login frame. 3.04 and 4.04r2
walk on core's captured cadence (`protocol::walkcadence::WalkCadence`:
changed props only, GANI `walk`/`idle`); the other builds send their whole
walk message (`net::walk_message_for`: X2/Y2 where the profile has precise
positions) when the pose changes, as `seihai-mae` does for them. A link
the player walks into is asked for once with the build's own request
(core's `net::link_request_frame`: `PLI_LEVELWARP`, or the cached form
where the build has one); the walk holds until the server's level comes,
and the player arrives where the link said unless the server says
otherwise. A new level owes the builds that send it a 46
(`EnterLevelSender`), with the arrival props after it.

Gmaps online: the network thread keeps the gmap the server has the player
on with core's `gmap::sync_world` (the `.gmap` from the asset folder or
asked for from the server, the neighbours from the server's boards or the
local files the file rules allow, `PLI_ADJACENTLEVEL` for the rest) and
hands each new snapshot to the view. The view enters the gmap once it
holds the level of the snapshot's cell, and draws and walks into the
neighbours as offline. A crossing goes back to the network thread, which
sends it as the build does: with a walk cadence (3.04, 4.04r2) the 46 opens
the next tick's frame and the cell props ride the cadence; otherwise
core's whole crossing frame (`Connection::send_gmap_crossing`). The
server's copy of the new cell replaces the view's without moving the
player; a level of the gmap at another cell moves the view there, and a
level off the gmap leaves it.

Board changes the server sends (`PLO_BOARDMODIFY`, 7, and the 186
blocks) are passed to the view, which puts them on the level it is on and
on the gmap neighbours it holds (core's `gmap::apply_board_mods_to`). A
change for a level the view does not hold is dropped.

## Offline

`--level NAME` (`-l`, as in `seihai-mae`) with no `--online` plays level
files with no server. The name is looked up in `--assets` (its `levels`
folder and the build's level extensions); with no asset folder it is read
as a path. The window opens on that level at (30, 30). A link loads its
level from the same place and puts the player where the link says; a link
whose level is not there says so in the status bar and the player stays.
Nothing is sent anywhere.

A `.gmap` name (on the command line or as a link's level) opens the gmap
offline with core's gmap walk. Without a position the player starts at
(30, 30) of the first cell that names a level; a link into a gmap gives a
position on the whole map, which picks the cell (64 tiles each) and the
place in it. The levels next to the player's cell are loaded from the same
place, drawn past the cell's edges (tiles and file NPCs), and the
walk crosses into them; a neighbour that is not on disk leaves its edge a
wall. A plain level link leaves the gmap.

```
cargo run -p seihai-ato -- --version 3.04 --assets ~/graal --level onlinestartlocal.nw
```

## Not in yet

The server NPCs of neighbouring gmap levels (their file NPCs are drawn), NPC scripts and NPC animations, `PLO_NPCDEL` and `PLO_HIDENPCS`,
other players' looks (they show as plain markers),
GANI art, and, on the Torque UDP side, textures and the other shape formats, walking and chat. The clients' background images are not shipped,
so the start window's panel is a plain fill.
