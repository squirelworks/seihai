# Protocol & Networking

This document describes the wire protocol used by Graal Reborn listservers and gservers, derived from public sources (primarily GServer-v2 and OpenGraal connection code).

## Connection Sequence

1. Client connects to the listserver (default port historically 14922).
2. Client sends account credentials (username + password). Early clients sent these in the clear; later listservers added optional encryption.
3. Listserver returns the list of registered gservers.
4. User selects a server; client opens a second TCP connection to that gserver.
5. Client and gserver exchange login, level, and entity packets for the duration of the session.
6. Client may keep the listserver connection open for profile updates.

## Packet Framing

- Many packets are terminated by a newline (`\n`) or use an explicit length prefix.
- First byte (or first character after framing) is the packet type ID.
- CString helpers in the server source handle compression, tokenization, and endianness.

## Key Packet Categories (to be fully catalogued)

### Listserver ↔ Client
- Account verification
- Server list population
- Player profile updates
- Chat / IRC bridge packets (optional)

### Gserver ↔ Client
- Login / version handshake
- Level name + tile data
- Player position / movement (including `gr.x` / `gr.y` / `gr.z` smooth movement flags used by later clients)
- NPC create / update / destroy
- Chat messages
- Weapon / item / flag updates
- Sound and effect triggers

Exact IDs and layouts are extracted from the open gserver sources under `../shanalysis/GServer-v2` and recorded in `../shworkingpapers/protocol/` as they are confirmed.

## Compression

The server uses a custom compression scheme (see CString implementation in GServer-v2). The client must implement matching compress/decompress routines for full compatibility.

## Implementation Notes

- Prefer a single `Packet` / `CString`-compatible buffer class that can be shared by both listserver and gserver connections.
- Keep the protocol layer free of rendering or scripting dependencies so it can be unit-tested with recorded captures.
- Version negotiation: different client builds (2.17, 2.22, 2.3x, etc.) send slightly different version strings and feature flags. The initial client should advertise a widely accepted version string.

## Next Steps

- Extract the full packet ID enum from GServer-v2.
- Implement login + server-list flow against a local listserver.
- Implement level-name and basic player-move packets.
