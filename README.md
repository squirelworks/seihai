# Seihai

Open Graal Reborn-compatible client, written in Rust.

## Status (2026-10-14)

Early scaffold. Offline level play works for parsed `.nw` files. Protocol types remain provisional. No live server connection is required for the current slice.

### Implemented
- **Assets** — CRC-32, minimal PNG writer, `.nw` (GLEVNW01) parser, tile-type table (`tiledefs` text or `arrays.dat`), LINK/SIGN/BADDY/CHEST records, binary sign decoder, GANI frame parser and stepper.
- **Offline** — `OfflineSession` loads a level, applies board modifies, and does collision-aware `try_move`. Wall is `>= 20` (blocks). Water is `== 11` (sets `in_water`, does not block, changes display). Directions: 0 south, 1 west, 2 east, 3 north. Optional `walk_gani` stepped on every successful move.
- **Render** — ASCII board and camera views, PPM dump. Player draws as `v < > ^` on land and `V { } A` in water. SDL2 is not wired.
- **Protocol** — `LoginRequest` (classic line locked: prefix + `GNW#####` + length-biased account/password), `LevelRequest`, `PlayerUpdate` (classic ×2, precise ×16, provisional walk frame), `BoardModify`.
- **Scripting** — GS1 bytecode parser, walker, and minimal interpreter (arithmetic, comparison, jump). `Commands` registry with offline stubs: `message`/`say`, `setimg`/`showimg`, `move`, `hideimg`, `setlevel`, `warp`.
- **Tools** — `level-editor` CLI: ASCII preview and re-export of `.nw` files.

### Not yet
- Live listserver/gserver session against a real server
- SDL2/OpenGL renderer
- Full GS1 command set and NPC event dispatch
- Post-login encryption

## Layout
```
seihai/
├── client/          # Runtime crate
│   └── src/
│       ├── assets/  # level, gani, crc32, png
│       ├── offline/ # local session
│       ├── protocol/# provisional packets
│       ├── render/  # ASCII / PPM board view
│       ├── scripting/
│       └── net/
├── tools/level-editor/
├── documentation/
└── examples/
```

## Build
```bash
cargo build
cargo test
```

## Approach
Implement plausible behavior first. Refine layouts as recovery progresses. Clean-room: no proprietary client code is copied.

## References
Protocol and format notes, and the decompile corpus, live outside this repository.
