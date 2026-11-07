# Torque engine

Ato owns the Torque reimplementation for v3, v4 and v5. This module is
what a new client runs. graal-client-repro owns how the original engine
was shaped. The two projects share notes, not code. This crate does not
link that repro and does not copy it.

The product default follows the selected era. The reimplementation is
**on** when the era is 3, 4 or 5, and **off** when the era is 6 or later.
`Era::default()` is `Era3`, so a caller who has not picked an era gets
the full v3 surface.

The default build of `seihai-ato` compiles this module. A v6 build does
not. Pass `--features era6`. That leaves `torque::engine` out of the
binary. v6 is `quattro_lib`. `GuiGraalCtrl` names may remain on that
line as Graal GUI. That GUI is native Seihai, not Ato, and it is not
this module.

`torque_enabled(era)` is true for `Era3`, `Era4` and `Era5` only.
`modules_for(era)` lists the submodules that are on. Era 6 returns an
empty list. An `era6` build does not compile the list at all.

Nothing here speaks a protocol, opens a socket, or stores a byte layout.
The connect-profile table in the parent `torque` module is unchanged.
The playable `--version` flag still chooses a 3.x session (3.04 or
3.153). Both sit on era 3. `Era` is the engine gate.

## v3

`Era3` enables every submodule of the full TGE surface, plus Graal-only
`GuiGraalCtrl`:

- console and `.dso`
- `SimObject`
- `ActionMap`
- GUI, `GBitmap`, and math
- terrain, interiors, and DTS
- `ShapeBase`, `Player`, and vehicles
- `GameConnection` as the Torque demo protocol
- `GuiGraalCtrl`

Math is part of that full surface. The later 2D engine keeps the same
library on its own cut.

## v4

`Era4` enables two subsets of its own.

2D subset: GUI, bitmap, math, `GuiGraalCtrl`.

3D subset: `SimObject`, `GameConnection`, `GameObject` (a `SceneObject`),
terrain chunks, interiors.

It does not enable `ShapeBase`, `Player`, vehicles, the Torque script
console, or `.dso`. It also leaves off `ActionMap`, the v3 terrain
block, and DTS.

The 2D engine keeps Torque GUI and math and has no `SimObject` console
and no `.dso`. The 3D side keeps the 3D subset above and drops
`ShapeBase` and Torque's `Player`. The launcher is Delphi plus
`graal_set` callbacks. That launcher is not Ato and is not compiled
here.

## v5.0

`Era5` is the 5.0 cut. It enables the same two subsets as v4: 2D GUI,
bitmap, math, `GuiGraalCtrl`; 3D `SimObject`, `GameConnection`,
`GameObject`, terrain chunks, interiors. It does not enable `ShapeBase`,
`Player`, vehicles, the Torque script console, or `.dso`.

The subset names match v4. The cut does not. v5 owns `v5::differences`,
which names the 5.0.0.7 facts and does not implement them:

- the 3D protocol check accepts only 24
- the launcher resolves 95 `graal_set` names
- the engine defines 118 `graal_set` names

No setter is called and no accept is parsed. The three-program split
matches v4: a Delphi launcher, a 2D engine, and a 3D DLL. The launcher
is not this module.

Unpacked 5.30 and 5.31 are a later cut. They drop `graal_set`,
`SimObject` and `GameConnection`, so the 3D Torque link is gone. Do not
fold 5.30 into this 5.0 set. `Era5` is not a bucket for every 5.x build.
The same note is a comment in `v5/mod.rs`.

## v6 and later

`torque_enabled(Era6)` is false and `modules_for(Era6)` is empty. A
build that passes `era6` does not compile this directory. Torque is not
called. Any `GuiGraalCtrl` name that remains is Graal GUI in native
Seihai, not this reimplementation.
