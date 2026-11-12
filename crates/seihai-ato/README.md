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
  nothing. NPC props that come after the level (moves, new images) are
  not applied yet, and missing images are not asked for from the server.

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
place, drawn past the cell's edges (tiles only, not their NPCs), and the
walk crosses into them; a neighbour that is not on disk leaves its edge a
wall. A plain level link leaves the gmap.

```
cargo run -p seihai-ato -- --version 3.04 --assets ~/graal --level onlinestartlocal.nw
```

## Not in yet

The NPCs of neighbouring gmap levels, board changes (`PLO_BOARDMODIFY`)
in the view, NPC scripts and NPC animations, NPC changes after
the level arrives, other players' looks (they show as plain markers),
GANI art, and the Torque UDP side. The clients' background images are not shipped,
so the start window's panel is a plain fill.
