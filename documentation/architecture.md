# Architecture

## Crates

The repository is a Cargo workspace. The 2.x and 3.x clients share gen-5
framing, the board/gmap formats and almost the same GS1 command set, so
that part is one library and each client line is its own binary.

| Crate                 | Kind             | Owns |
|-----------------------|------------------|------|
| `crates/seihai-core`  | library          | `protocol/` (gen-3/gen-5 codecs, LCG, zlib/bz2, login writers, props, id and layout tables, version profiles and the chain), `assets/` (level, board, gmap, GANI, tiletypes, object-set parsers, the resolver, built-in set), `filecache`, `servercache`, `scripting/` (GS1 lexer, parser, compiler, interpreter, command registry), `offline/` (`OfflineSession`, collision, command applicator) and the game rules it uses (`combat`, `life`, `items`, `throwing`, `bombs`, `arrows`, `pose`, `looks`, `npcart`, `chatlists`), `net/`, `client/` (the game-server session the client lines share: login, server warps, the level store and log, the walk, links and gmap sync), `sound` rules and `.wav` decoding, `difftest` and the capture `viewer`, `dev`, `config` (the settings folder). Nothing here draws, opens a window or opens a sound device. |
| `crates/seihai-mae`   | binary `seihai-mae` | The 1.x and 2.x client (a 6.x pick in its start window also stays here): `bin/main.rs`, `gui.rs` (launcher and start window; a 3.x pick there hands over to `seihai-ato`, and a "Graal3D" pick to `seihai-g3d`, through `ato.rs`), `render/` (tiles, GANI, sprites, HUD, signs, chat), `ui`, `help`, `keys`, `inventory`, `weaponbar`, `stall`, `settings` (the settings file), the sound setup (the rodio backend itself is seihai-core's `audio` feature) and the `difftest` binary. Its library is still named `seihai` and re-exports the core modules, so `seihai::protocol` and the rest still resolve. |
| `crates/seihai-ato`   | binary           | The Torque line, 3.x to 5.x: login, level receive, flat 2D drawing and a local walk (see its README), plus the Torque engine scaffold in `src/torque/engine`, which is off for era 6 and later (`--features era6`). Depends on `seihai-core`. |
| `crates/seihai-issho` | library + binary | The v6 profile. Quattro-shaped shell (draw, sound slot, GS2 shell, `TSocket`, `TServer*` world objects); `--online` logs in with core's `client` session and logs the level. Depends only on `seihai-core`. The launcher starts it for a 6.x `--version`. v7 is an unselectable variant (Unity is the expected engine; no v7 profile). |
| `tools/level-editor`  | library + binary | The level editor. Reads levels and gmaps through `seihai-core`; uses the mae renderer for its tile view. It stays the client's offline editor. |

The `seihai` launcher (`crates/seihai-launcher`) starts `seihai-mae` when
there is no `--version` and for 1.x and 2.x, `seihai-ato` for 3.x to 5.x,
`seihai-g3d` for `graal3d`, and `seihai-issho` for 6.x. A 7.x label starts
nothing. Issho is not on the play path unless v6 was selected.

## Cut line between the client lines

Issue #377. This is where each line starts and stops in the code today,
not a claim about how the original clients were built.

- **Mae / Ato:** Mae is the Delphi-era line (1.x, 2.x). Ato starts with
  the first Torque client, 3.04. 3.x keeps gen-5 framing and the 2.x
  board, gmap and GS1 formats, which is why both use `seihai-core` for
  them.
- **Ato / Issho:** Ato stops where Torque stops. Its engine scaffold is on
  for eras 3, 4 and 5 and off from era 6 (`era6`). Issho starts at 6.x,
  the first build with the Quattro-shaped C++ core and no Torque. Issho
  does not depend on Ato, and has no Torque or Delphi code.
- **After Issho:** 7.x is reserved and unsupported (`EngineProfile::V7`
  is never constructed and the launcher rejects 7.x labels). Issue #377
  proposed that Issho is the core 6.x and 7.x share. Published research
  describes a native Graal layer under Unity in 7.x, but it does not show
  that this layer is the same engine as 6.x. Until it does, the cut line
  stops at 6.x and no v7 profile is added.
- **Exception on the play path:** the launcher sends 6.x to Issho, but a
  6.x pick inside Mae's own start window stays on Mae's flat path
  (`runs_in_ato` in `crates/seihai-mae/src/ato.rs`). 3.x to 5.x picks there
  go to Ato, as from the launcher.
- **Not on any line:** `seihai-g3d` (the February 2000 Graal3D) is its own
  client and protocol, not Ato's 3D.

Build everything with `cargo build --release`. That gives the `seihai`
launcher and the `seihai-mae`, `seihai-ato`, `seihai-issho` and `seihai-g3d`
binaries it starts; keep them in one folder.

## Client Runtime

The client is structured as a set of loosely coupled layers:

| Layer       | Responsibility                                      | Depends on          |
|-------------|-----------------------------------------------------|---------------------|
| Protocol    | TCP connections, packet framing, compression, IDs   | —                   |
| Assets      | Load levels, tilesets, sprites, ganis, sounds       | —                   |
| Scripting   | Execute GS1-compatible (and later extended) scripts | Assets, Protocol    |
| Rendering   | Draw levels, entities, GUI                          | Assets              |
| Core        | Game loop, input, entity positions, camera          | All of the above    |

### Connection Model (from public documentation)

A typical session involves three TCP connections:

- **Client ↔ Listserver** — Account verification, server list, profile updates.
- **Client ↔ Gserver** — Actual gameplay (levels, players, NPCs, chat).
- **Gserver ↔ Listserver** — Server registration and player tracking (not managed by the client).

Optional: RemoteControl and NPC-server connections exist on the server side but are out of scope for the initial client.

### Packet Format (high level)

Packets are length-prefixed or newline-delimited CString-style buffers. The first byte is a packet type ID (0–255). Many packets use a custom compression scheme documented in the open gserver sources. Detailed packet catalogs belong in `protocol.md` and the working papers.

## Seihaiscript Engine

Starts as a GS1-compatible interpreter/VM inspired by OpenGS1, then grows selected GS2 features:

- Unified variable model (numeric + string)
- Functions with parameters and return values
- Basic object/class joining
- Event-driven NPC and weapon scripts
- GUI control bindings (later)

Kept deliberately separate from the renderer so headless tools and tests can use the same engine.

## Tooling

Tools reuse the same asset loaders and (where useful) the scripting engine:

- Level editor
- Gani/animation editor
- Terrain / dungeon / world generators
- File format converters (`.nw`, `.graal`, sprites, etc.)

## Build & Language Targets

Initial target is C++ (matching the bulk of existing open sources). Alternative bindings (Rust, C#, web/WASM) are possible later once the core libraries stabilize.
