# Architecture

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
