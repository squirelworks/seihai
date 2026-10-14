# Seihai — Project Overview

**Seihai** (聖杯, “Holy Grail”) is a modern, open-source client and tooling suite for Graal Reborn listservers and gservers.

## Goals

- Fully compatible client that can connect to existing Graal Reborn infrastructure.
- Clean-room implementation of networking, asset loading, and scripting.
- Seihaiscript engine that supports GS1 compatibility and selected GS2 improvements.
- Companion tools: level editors, animation tools, generators, converters.
- Long-term: a sustainable replacement for the aging Delphi 2.x clients.

## High-Level Architecture

```
seihai/
├── client/          # Runtime client
│   ├── protocol/    # Packet layer, listserver + gserver connections
│   ├── assets/      # Level, sprite, gani, sound loaders
│   ├── scripting/   # Seihaiscript engine (GS1+ subset)
│   ├── rendering/   # Modern renderer (SDL2/OpenGL or equivalent)
│   └── core/        # Game loop, entity management, input
├── tools/           # Editors, generators, converters
├── documentation/   # This directory
└── examples/
```

## Design Principles

1. **Clean-room first** — Driven by public server sources, observed packet behavior, and documented formats. No direct copying of proprietary client internals.
2. **Protocol compatibility** — Speak the same wire protocol as existing gservers and listservers so the client is immediately useful.
3. **Incremental usefulness** — Networking + basic rendering should work before a full scripting VM or polished editor exists.
4. **Modularity** — Protocol, assets, scripting, and rendering are separable so tools can reuse the same loaders.

## Related Analysis Sources

Public code used for protocol and scripting research lives under `../shanalysis/`:

- `OpenGS1` — GS1 lexer/parser/VM
- `GServer-v2` — Graal Reborn server implementation
- `OpenGraal.Common.Connections-cs` — Client-side connection examples
- `graal-listserver` — C# listserver emulator
- `OpenGraal.LevelEditor-cs` / `OpenGraal.MapGenerator-cs` — Existing tool references

Reverse-engineering notes and Ghidra/IDR work belong in `../shdecompiling/` and `../shworkingpapers/`.

## Status

Scaffold created 2026-10-14. Offline `.nw` play, ASCII/PPM render, provisional protocol types, and a minimal GS1 interpreter are in place. See `implementation.md` for the current slice.
