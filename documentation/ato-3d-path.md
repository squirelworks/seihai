# Ato Stage 4: the 3D path (unblocked, not started)

Date: 2026-10-31. Laundry list item 4.

## Status (updated 2026-11-26)

**Unblocked, not started.** Both things this page waited for now exist,
so the 3D path can start whenever it is picked up. Nothing in Ato's code
has changed: `engine/v5/terrain_chunks.rs` and its siblings are still
stubs.

What changed since 2026-10-31:

- **A Torque-speaking lab server.** seihai-gserver (`docs/GRAAL3D.md`) and
  the seihai-gserver-v2 Graal 3D add-on (`bin/docs/graal3d.md`) both speak
  Torque on the listed port + 1: protocol 19 (3.153, 4.04r2, stock TGE with
  three mission phases) and protocols 21 and 24 (4.20r6, 4.20r8, 5.x,
  Graal's `GameObject` model with two phases).
- **Live sessions, not just binaries.** The real 3.153 and 4.04r2 ran the
  full mission start against our server and took a control object
  (Graal_analysis `torque/live-3153-and-404r2-sessions-2026-11-25.md`).
  5.12 was captured end to end (`torque/udp-session.md`), and its whole
  packet inventory, UDP framing, ghost lifecycle and class table are written
  up in `torque/3d-replica-wire-157-160-165-512-2026-11-26.md`
  (T3D-157 to T3D-165).
- **The smallest accepted handshake.** A ~400-line synthetic client linked
  as protocol 24 and was seen by a real 5.12
  (`torque/3d-min-handshake-164-512-2026-11-26.md`, T3D-164). That is the
  "connect only first" step below, already done once outside Ato.
- **Per-build capabilities in the profile.** `VersionProfile::three_d`
  (seihai-core, from Graal_analysis `version-profiles/three-d.toml`) gives
  each build's protocol, class table, game layer, mission phases, control
  objects, terrain and cache rules and ground cover, each tagged with its
  evidence. Ato's gate should read it, not a table of its own.
- **A C++ reference engine.** graal-client-repro's
  `common/cpp/graal3dengine` links to these servers and draws streamed
  terrain the way 5.12 does (T3D-162).

Still missing: 3.04 (TGE protocol 9) has not been run against our server,
so its connect request still comes from the binary only. Start with
3.153 or 4.04r2.

Findings an Ato 3D client has to honour on 5.x, from the lab:

- 5.x lights the mission when `MissionStartPhase2` arrives. Our servers
  hold phase 2 until the TerrainBlock, WaterBlock, Sun and terrain chunks
  are in (T3D-162, seihai-gserver #223, seihai-gserver-v2 #182). A client
  replica should light at that point too, not before.
- Terrain chunks are exchanged as `TerrainRequestUpdatesEvent` (256 stamps,
  one per 16 x 16 chunk) answered by `TerrainChunkEvent`s for the chunks
  whose stamp differs (`torque/terrain-512.md`). The cache is per server.
- Grass draws from a texture, not a shape, and radius alone does not decide
  whether it draws (T3D-048; the radius-8 rule was overstated, cause open).

## The shape, now that there are captures

- **The gate.** The same one as the Stage 2 fields: `torque::of(version)`.
  It applies only when the listed server is a 3D server. A 2D server keeps
  the flat path.
- **Connect only first.** Open the UDP socket and send the connect request
  for the build's profile. Read the accept and check its protocol number
  against `TorqueProfile::protocol`, then log the outcome. No ghosts, moves
  or drawing until the accept is matched byte for byte against the capture.
- **The flat view stays** as the fallback and as the view for 2D servers.
