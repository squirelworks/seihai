# Protocol & Networking

This document describes the wire protocol used by Graal Reborn listservers and gservers, derived from public sources (primarily GServer-v2 and OpenGraal connection code) and cleaned observations of client 2.05 rev 2 behaviour. No decompile material.

## Connection Sequence

1. Client connects to the listserver (2.05: `www.graalonline.com:14921`, list name `newmain`; later clients use 14922/14923).
2. Client sends account credentials. In 2.05 the first line is client type `0` + key (`Random & 0x7F`) + version token (`GNW03081`) + length-prefixed account and password.
3. Listserver returns the list of registered gservers.
4. User selects a server; client opens a second TCP connection to that gserver.
5. Client and gserver exchange login, level, and entity packets for the duration of the session.
6. Client may keep the listserver connection open for profile updates.

## Packet Framing

- Logical message: `chr(id+32)` + payload + `"\n"`.
- Several messages are zlib-compressed together under a `u16be` length prefix.
- Binary payloads may be wrapped behind a raw-data escape.

## Encryption generations (cleaned)

| Gen | Builds | Behaviour |
|-----|--------|-----------|
| 1 | through ~1.1.9 | No compression |
| 2 | ~1.21–1.39 | zlib only |
| 3 | ~1.41 onward (including 2.05) | zlib + one LCG-inserted byte per client→server message after the first |
| 4/5 | later clients | full-frame partial XOR with the same LCG family |

**2.05 gen-3 insertion:** iterator seeded `0x04A80B38`, stepped `it = it * 0x08088405 + key`. Inserts `chr(((it>>16)&0x7F)+32)` at 1-based position `(it & 0xFFFF) mod len + 1` (len includes the trailing newline). Position can be 1 (junk byte before the id). Server→client is plain. Login frame is clean.

## Key Packet Categories

### Listserver ↔ Client
- Account verification
- Server list population
- Player profile updates

### Gserver ↔ Client
- Login / version handshake
- Level name + tile data
- Player position / movement (half-tile props 15/16 in 2.05; precise ×16 in later clients)
- NPC create / update / destroy
- Chat messages (including `toall:` prefix → id 6)
- Weapon / item / flag updates
- Sound and effect triggers

### Peer-to-peer UDP (2.05)
- Fixed port 14899. IP/port exchanged via player props 30/31.
- Datagram = sequence byte + payload, XOR’d with low byte of `(peerId ^ ownId)`. No zlib, no gen-3.
- Id 21: remote hit (from GS1 `hitplayer`).
- Id 24: entered level at x,y (3×3 neighbourhood).
- Id 23: receive-only in 2.05 (rewritten as current-level prop).

Exact IDs and layouts are recorded in `../shworkingpapers/protocol/` as they are confirmed.

## Implementation Notes

- Prefer a single `Packet` / buffer class that can be shared by both listserver and gserver connections.
- Keep the protocol layer free of rendering or scripting dependencies so it can be unit-tested with recorded captures.
- Version negotiation: advertise a widely accepted token (`GNW03081` for 2.05-era servers, later tokens for modern Reborn).

## Next Steps

- Implement gen-3 encode/decode against a local capture.
- Implement login + server-list flow against a local listserver.
- Implement level-name and basic player-move packets.