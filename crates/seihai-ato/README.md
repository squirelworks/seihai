# seihai-ato

The Torque / 3.x client line: Graal 3.04 (Linux, Jan 2004) and 3.153 (Mac,
Feb 2005).

The 3.x clients still speak gen-5 framing and use the same board, gmap and
GS1 formats as 2.22 and 2.31, so this crate shares `seihai-core` with the
2.x client (`seihai-mae`). What is 3.x-only lives here.

## Stage 1

- `args`: the command line. `seihai-launcher` passes the user's arguments
  through, so unknown flags are ignored. `--version 3.04` or
  `--version 3.153` (the default); `--account`, `--password-env VAR`,
  `--nick`.
- `wire`: the build's game and listserver tokens and the ids it adds or
  drops against 2.31, all read from the build's row in
  `seihai-core::protocol::chain` (nothing restated).
- `torque`: the Torque connect profile per protocol number (9 for 3.04,
  19 for 3.153 and 4.04r2, 21 for 4.20r6, 24 for 5.12): game string, class
  table, 12-bit class ids, 14-bit length prefixes, and the port rule
  (listed port + 1). Data only. Source: Graal_analysis
  `torque/wire-by-build.md`.
- `start_window`: the 3.x start window, laid out from the clients' own GUI
  files (positions, texts, colours, language list). With `--online`,
  Start hands the form to Stage 2; otherwise the buttons say what the real
  client would do next.

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
  gives the same pace, walls and water as mae.

The walk is local: Ato does not send its position, because what a 3.x
client sends while walking has not been captured. A link the player walks
into is noted on the status bar and dropped.

## Not in yet

The server list, offline mode, the post-login frame the real 3.x client
sends once the board is in (file checks and props, which carry the
nickname and language), sending the walk, level changes, NPCs, other
players, GANI art, and the Torque UDP side. The clients' background images are not shipped,
so the start window's panel is a plain fill.
