# Torque lineage in Seihai

Graal 3.x ships a Torque-based mode (TCP login to the Graal game server; Torque traffic is UDP on the **listed game port plus one**; see `seihai_ato::torque`).

This repository vendors the **open Torque3D MIT** tree only, under `vendor/Torque3D`, as a git submodule:

- **Remote:** https://github.com/TorqueGameEngines/Torque3D
- **Pinned ref:** tag `v4.0.3` (commit recorded in the submodule checkout)

## Lineage gap

The exact engine revision inside historical Windows `Graal3DEngine.dll` builds is **not** this public tree. That binary is proprietary and is not vendored here. The submodule is the maintained MIT lineage (GarageGames-era Torque 3D, continued as Torque3D) used so the repo contains **real Torque sources**, not a stub.

`seihai-ato` prints the engine version string defined in `vendor/Torque3D/Engine/source/app/version.h` (`TORQUE_GAME_ENGINE_VERSION_STRING`) at startup; integration tests assert that string is non-empty after a normal clone with submodules.

## Clone and build

After `git clone`, run:

`git submodule update --init --recursive`

Then `cargo build` / `cargo test` as usual. CI checks out submodules recursively.

## What is deliberately not in this repo

- Proprietary Graal DLLs or decompiled game code
- GPL engine code (Torque3D MIT is MIT-licensed; see `vendor/Torque3D/LICENSE.md`)
