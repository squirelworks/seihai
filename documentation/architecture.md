# Architecture

## Crates

The repository is a Cargo workspace. The 2.x and 3.x clients share gen-5
framing, the board/gmap formats and almost the same GS1 command set, so
that part is one library and each client line is its own binary.

| Crate                 | Kind             | Owns |
|-----------------------|------------------|------|
| `crates/seihai-core`  | library          | `protocol/` (gen-3/gen-5 codecs, LCG, zlib/bz2, login writers, props, id and layout tables, version profiles and the chain), `assets/` (level, board, gmap, GANI, tiletypes, object-set parsers, the resolver, built-in set), `filecache`, `servercache`, `scripting/` (GS1 lexer, parser, compiler, interpreter, command registry), `offline/` (`OfflineSession`, collision, command applicator) and the game rules it uses (`combat`, `life`, `items`, `throwing`, `bombs`, `arrows`, `pose`, `looks`, `npcart`, `chatlists`), `net/`, `sound` rules and `.wav` decoding, `difftest` and the capture `viewer`, `dev`, `config` (the settings folder). Nothing here draws, opens a window or opens a sound device. |
| `crates/seihai-mae`   | binary `seihai`  | The 2.x client: `bin/main.rs`, `gui.rs` (launcher and start window; a 3.x pick there hands over to `seihai-ato` through `ato.rs`), `render/` (tiles, GANI, sprites, HUD, signs, chat), `ui`, `help`, `keys`, `inventory`, `weaponbar`, `stall`, `settings` (the settings file), the rodio sound backend (`audio-rodio` feature) and the `difftest` binary. Its library is still named `seihai` and re-exports the core modules, so `seihai::protocol` and the rest still resolve. |
| `crates/seihai-ato`   | binary (stub)    | The Torque / 3.x line. Depends only on `seihai-core`; prints its version and exits until the 3.x work lands. |
| `tools/level-editor`  | library + binary | The level editor. Reads levels and gmaps through `seihai-core`; uses the mae renderer for its tile view. |

Notes and older documents that cite `client/src/...` mean the same file
under `crates/seihai-core/src/...` (shared modules above) or
`crates/seihai-mae/src/...` (the rest).

Build the client with `cargo build --release -p seihai-mae`; the binary
is still `seihai`.

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
