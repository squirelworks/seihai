# Scaffolding from [Grok/[Projects — 2026-10-28

The Drive folder holds zips of prior Graal work. These are the pieces that can be scaffolded into Seihai or a companion server.

## gServers

`Graal gServers.zip` contains Ash, a Python 3 rewrite of the Reborn 2.x server.

- `Ash/ashserver.py` — the server loop
- `Ash/protocol.py` — framing, encryption gens 3–5, gchar packing
- `Ash/servers/ash/` — one world: accounts, config, weapons, world
- `Ash/servers/ash/config/serveroptions.txt` — unstickme, jail, staff, drop rates
- `Ash/servers/ash/config/allowedversions.txt` — GNW tokens for 2.17–2.31
- `Ash/servers/ash/world/onlinestartlocal.nw` — the start level

Ash implements login, walk, chat, PM, warp, and file send. It registers with a listserver. The code is the cleanest starting point for a Seihai-compatible gserver.

`Graal 39sters gServer.zip` is a 2.1-era server with levels and a debug world. Useful for older protocol shapes.

## Scripting

`Graal Scripting.zip` holds the Ash architecture notes for 1520 unique NPC scripts.

- `Ash/00-runtime/architecture.md` — client NPC runtime: events, timeout, toweapons
- `Ash/00-runtime/event-model.md` — event dictionary
- `Ash/00-runtime/state-and-ids.md` — this.*, client.*, save[]
- Packs: larency (HUD/inventory), phoenix (quests/baddies), meh (weapons), UNP (minigames), standalone

These notes map to the Seihai script engine. The event model and state IDs are the parts the parser still needs.

## Level making

`Graal Enigma Playerworld.zip` (223 MB) is a full playerworld. Not downloaded. It holds levels, tilesets, and scripts that can feed the offline reader and the level editor.

## Client analysis

`Graal Client Analysis.zip` and `Graal Client Reform.zip` hold decompile notes and reform material. Already partially imported into shworkingpapers.

## What to scaffold

1. A minimal gserver from Ash: login, walk, chat, level load, file send. Point Seihai at it for offline testing.
2. A listserver stub that answers the three login shapes (A, B, C) and returns a single gserver entry.
3. The Ash event model wired into the Seihai script scheduler, so timeout and toweapons work.
4. A level editor that reads and writes GLEVNW01 and GR-V1, using the formats already in the Seihai reader.
