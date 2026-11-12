# seihai-mae

Mae is Seihai's original playable client implementation, now housed in its
own crate. It owns the classic 1.x/2.x client and still supplies the flat
play path used for 4.x/5.x. This is Seihai's Rust implementation, not the
original Graal engine or a claim of complete compatibility.

The package and executable are named `seihai-mae`. Its library keeps the
name `seihai`, preserving the existing `seihai::...` imports in the client,
examples and tests. The `seihai` executable is the separate launcher.

## What lives here

- The settings/start window, playable offline and online client, tutorial,
  and client-side integration of the session and scripting effects.
- Flat rendering, player/NPC art, HUD, chat, inventory and map presentation.
- Window/input handling, frame presentation and optional audio output.
- The command-line tools exposed through Mae, including asset checks and
  version-chain output.

Shared profiles, framing/login, resource formats, simulation and scripting
belong to [`seihai-core`](../seihai-core), rather than a second copy here.
The standalone [`seihai-draw`](../seihai-draw) compositor serves tools that
should not depend on the full playable client.

## Boundaries and current routing

[`seihai-launcher`](../seihai-launcher/src/main.rs) starts Mae by default.
It routes 3.x to 5.x profiles to [`seihai-ato`](../seihai-ato/README.md), 6.x
profiles to [`seihai-issho`](../seihai-issho/README.md), and the separate
February 2000 Graal3D mode to [`seihai-g3d`](../seihai-g3d). Ato draws 4.x
and 5.x with its flat 2D fallback; Ato's ownership of the Torque-era engine
surface does not mean that surface is already a playable replacement.

Mae can also run directly. Its settings window hands 3.x to 5.x selections
to Ato and Graal3D selections to the separate Graal3D client. A 6.x
selection inside that window stays in Mae as a flat fallback. The launcher's 6.x routing and Mae's in-window routing are
therefore different. Issho is currently a shell, and v7 remains unsupported.

These are the implemented boundaries, not conclusions that all builds in
an era behave alike. Wire profiles, script capabilities, assets and client
behavior still need exact build/platform evidence. A borrowed server-list
discovery profile does not change the selected game's emulated version.

graal-client-repro documents how the historical clients were shaped. Mae
does not link it or take implementation code from it. Keep changes to the
shared core compatible with its server consumers; do not move client UI,
device handling or renderer dependencies into core.

See the [workspace README](../../README.md) for build/run instructions and
the [Ato orientation](https://github.com/squirelworks/seihai/issues/380) for
the proposed engine direction and unresolved research questions.
