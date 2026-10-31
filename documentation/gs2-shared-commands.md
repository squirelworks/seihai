# GS2 shared commands: what Seihai runs

Date: 2026-10-31. Every built-in command the 3.153 client registers for
NPC scripts and as a global function, by name, with the return letter
and the parameter string the client gives it. **[bin]** Only the names
and signatures are taken from the decompile. Tags as in
`gs2-executor.md`.

The client's own letters: `v` no value, `o` an object, `d`/`f` a
number, `s` a string, `i` an integer, `b` a flag, `X` any number of
further arguments, `u` (not read).

The column "Seihai" says how `scripting/gs2/exec.rs` runs the command:

- **machine**: the GS2 machine runs it itself (`gs2-commands.md`).
- **host**: `EffectHost` hands it to the host as the GS1 engine's own
  effect, the same value a GS1 script makes. The name matches in any case
  and with a `this.` or `thiso.` in front.
- **nothing (as GS1)**: GS1 has the command and does nothing with it
  (`vocab.rs`, `NoExec`); GS2 does the same.
- **not run**: Seihai's GS1 engine has the command, but only inside the
  GS1 engine, so the GS2 machine returns "not found". These are stubs.
- **—**: a GS2 command GS1 does not have. Not part of the shared set;
  Seihai returns "not found".

The commands `timeout`, `settimer`, `scheduleevent` and `cancelevents` are
in the client's NPC table under another group and are the machine's own
(Timers in `gs2-commands.md`).

## Shared with GS1 and still a stub

| Command | 3.153 | Why |
|---|---|---|
| `callnpc(id, …)` | `v` `iX` | calls into another NPC's script; Seihai's GS2 machine has one script and no NPC lookup yet |
| `callweapon(id, …)` | `v` `iX` | the same, for a weapon |
| `changeimgmode`, `seteffectmode`, `showani`, `showani2`, `showimg2`, `showpoly`, `showpoly2`, `showtext2`, `findimg` | below | in the 3.153 NPC table next to `showimg` and `changeimgvis`, but not in Seihai's GS1 vocabulary (the 2.05–2.1 set), so there is no GS1 effect to hand them to. Whether a GS1 client has them was not checked **[inf]** |

The GS1 engine's string-list and array commands (`setstring`,
`addstring`, `insertstring`, `replacestring`, `removestring`,
`deletestring`, `setarray`, `tokenize`, `tokenize2`) are not in the 3.153
tables at all. GS2 has its own forms (`gs2-commands.md`, Lists), so
"not found" is what the client does too. `set`, `unset`, `setlevel`,
`setlevel2` and `setplayerprop` are not in the tables either, but
`EffectHost` takes them as GS1 does.

## NPC commands

| Command | Returns | Parameters | Seihai |
|---|---|---|---|
| `blockagain` | `v` |  | host |
| `blockagainlocal` | `v` |  | host |
| `canbecarried` | `v` |  | host |
| `cannotbecarried` | `v` |  | host |
| `canbepushed` | `v` |  | host |
| `cannotbepushed` | `v` |  | host |
| `canbepulled` | `v` |  | host |
| `cannotbepulled` | `v` |  | host |
| `carryobject` | `v` | `s` | host |
| `changeimgcolors` | `v` | `idddd` | host |
| `changeimgmode` | `v` | `ii` | **not run** |
| `changeimgpart` | `v` | `iiiii` | host |
| `changeimgvis` | `v` | `ii` | host |
| `changeimgzoom` | `v` | `id` | host |
| `destroy` | `v` |  | host |
| `dontblock` | `v` |  | host |
| `dontblocklocal` | `v` |  | host |
| `drawaslight` | `v` |  | host |
| `drawoverplayer` | `v` |  | host |
| `drawunderplayer` | `v` |  | host |
| `findimg` | `o` | `i` | **not run** |
| `hide` | `v` |  | host |
| `hidelocal` | `v` |  | host |
| `hideimg` | `v` | `i` | host |
| `hideimgs` | `v` | `ii` | host |
| `hurt` | `v` | `i` | host |
| `lay` | `v` | `s` | host |
| `message` | `v` | `s` | host |
| `move` | `v` | `dddi` | host |
| `setbow` | `v` | `s` | host |
| `setcharani` | `v` | `ss` | host |
| `setchargender` | `v` | `s` | host |
| `setcoloreffect` | `v` | `dddd` | host |
| `seteffectmode` | `v` | `i` | **not run** |
| `setimg` | `v` | `s` | host |
| `setimgpart` | `v` | `siiii` | host |
| `setshape` | `v` | `iii` | host |
| `setshape2` | `v` | `iio` | host |
| `setzoomeffect` | `v` | `d` | host |
| `shootarrow` | `v` | `i` | host |
| `shootball` | `v` |  | host |
| `shootfireball` | `v` | `i` | host |
| `shootfireblast` | `v` | `i` | host |
| `shootnuke` | `v` | `i` | host |
| `show` | `v` |  | host |
| `showani` | `v` | `iddis` | **not run** |
| `showani2` | `v` | `idddis` | **not run** |
| `showcharacter` | `v` |  | host |
| `showimg` | `v` | `isdd` | host |
| `showimg2` | `v` | `isddd` | **not run** |
| `showlocal` | `v` |  | host |
| `showpoly` | `v` | `io` | **not run** |
| `showpoly2` | `v` | `io` | **not run** |
| `showtext` | `v` | `iddsss` | host |
| `showtext2` | `v` | `idddsss` | **not run** |
| `take` | `v` | `s` | host |
| `take2` | `v` | `i` | host |
| `takehorse` | `v` | `i` | host |
| `throwcarry` | `v` |  | host |
| `timereverywhere` | `v` |  | host |
| `toweapons` | `v` | `s` | host |

## Global functions

| Command | Returns | Parameters | Seihai |
|---|---|---|---|
| `findFirstFile` | `s` | `s` | — |
| `findNextFile` | `s` | `s` | — |
| `extractFileExt` | `s` | `s` | — |
| `extractFileBase` | `s` | `s` | — |
| `extractFileName` | `s` | `s` | — |
| `extractFilePath` | `s` | `s` | — |
| `getDataBlockTypes` | `o` |  | — |
| `pushDialog` | `v` | `o` | — |
| `popDialog` | `v` |  | — |
| `isCursorOn` | `b` |  | — |
| `cursorOn` | `v` |  | — |
| `cursorOff` | `v` |  | — |
| `setContentControl` | `v` | `o` | — |
| `selectFileForUpload` | `v` |  | — |
| `selectFileForDownload` | `v` | `s` | — |
| `lightScene` | `b` |  | — |
| `triggerserver` | `v` | `sss` | — |
| `commandToServer` | `v` | `s` | — |
| `setShadowDetailLevel` | `v` | `f` | — |
| `getsurfacetype` | `s` | `ff` | — |
| `setInteriorRenderMode` | `v` | `i` | — |
| `vectoradd` | `s` | `ss` | — |
| `vectorsub` | `s` | `ss` | — |
| `vectorscale` | `s` | `sd` | — |
| `vectornormalize` | `s` | `s` | — |
| `vectordot` | `d` | `ss` | — |
| `vectorcross` | `s` | `ss` | — |
| `vectordist` | `d` | `ss` | — |
| `vectorlen` | `d` | `s` | — |
| `vectororthobasis` | `s` | `s` | — |
| `matrixcreate` | `s` | `ss` | — |
| `matrixcreatefromeuler` | `s` | `s` | — |
| `matrixmultiply` | `s` | `ss` | — |
| `matrixmulvector` | `s` | `ss` | — |
| `matrixmulpoint` | `s` | `ss` | — |
| `getBoxCenter` | `s` | `s` | — |
| `syncTimeOfDay` | `v` | `ff` | — |
| `setSunColors` | `v` | `o` | — |
| `setSkyBandColors` | `v` | `o` | — |
| `setSkyBandSizes` | `v` | `o` | — |
| `setFogColors` | `v` | `o` | — |
| `setTerrainRenderMode` | `v` | `i` | — |
| `mapShapeAnimation` | `b` | `sssf` | — |
| `getObjectAt` | `o` | `ff` | — |
| `getObjectAtMouse` | `o` |  | — |
| `getObjectByRay` | `o` | `ss` | — |
| `addcontrol` | `v` | `o` | — |
| `addtiledef` | `v` | `ssi` | host |
| `addtiledef2` | `v` | `ssii` | host |
| `aindexof` | `i` | `do` | — |
| `attachplayertoobj` | `v` | `ii` | host |
| `callweapon` | `v` | `iX` | **not run** |
| `callnpc` | `v` | `iX` | **not run** |
| `contains` | `b` | `ss` | — |
| `detachplayer` | `v` |  | host |
| `disabledefmovement` | `v` |  | host |
| `disablemap` | `v` |  | — |
| `disablepause` | `v` |  | — |
| `disableselectweapons` | `v` |  | host |
| `disableweapons` | `v` |  | host |
| `enabledefmovement` | `v` |  | host |
| `enablefeatures` | `v` | `u` | — |
| `enablemap` | `v` |  | — |
| `enablepause` | `v` |  | — |
| `enableselectweapons` | `v` |  | host |
| `enableweapons` | `v` |  | host |
| `explodebomb` | `v` | `i` | host |
| `findplayer` | `o` | `s` | — |
| `findplayerbyid` | `o` | `i` | — |
| `findweapon` | `o` | `s` | — |
| `freezeplayer` | `v` | `d` | host |
| `getascii` | `i` | `s` | — |
| `getextension` | `s` | `s` | — |
| `getimgheight` | `i` | `s` | — |
| `getimgwidth` | `i` | `s` | — |
| `getkeycode` | `i` | `s` | — |
| `getmapx` | `i` | `s` | — |
| `getmapy` | `i` | `s` | — |
| `getstringkeys` | `o` | `s` | — |
| `gettextwidth` | `i` | `dsss` | — |
| `gettextheight` | `i` | `dss` | — |
| `getz` | `d` | `dd` | — |
| `hitnpc` | `v` | `iddd` | host |
| `hitobjects` | `v` | `ddd` | host |
| `hideplayer` | `v` | `d` | host |
| `hidesword` | `v` | `d` | host |
| `hitplayer` | `v` | `iddd` | host |
| `isadminguild` | `b` | `s` | — |
| `isObject` | `b` | `s` | — |
| `keydown` | `b` | `i` | — |
| `keydown2` | `b` | `ib` | — |
| `keyname` | `s` | `i` | — |
| `lay2` | `v` | `sdd` | host |
| `loadmap` | `v` | `s` | host |
| `noplayerkilling` | `v` |  | host |
| `onwall` | `b` | `dd` | — |
| `onwall2` | `b` | `dddd` | — |
| `onwater` | `b` | `dd` | — |
| `onwater2` | `b` | `dddd` | — |
| `openurl` | `v` | `s` | host |
| `openurl2` | `v` | `sii` | host |
| `play` | `v` | `s` | host |
| `play2` | `v` | `sddd` | host |
| `playlooped` | `v` | `s` | host |
| `putleaps` | `v` | `idd` | host |
| `removetiledefs` | `v` | `s` | host |
| `replaceani` | `v` | `ss` | host |
| `requesttext` | `v` | `ss` | — |
| `sendtext` | `v` | `sso` | — |
| `requestfiledeletion` | `v` | `s` | — |
| `requestfilerename` | `v` | `ss` | — |
| `requestfilesmove` | `v` | `ss` | — |
| `resetfocus` | `v` |  | nothing (as GS1) |
| `savelog` | `v` | `s` | — |
| `echo` | `v` | `s` | — |
| `say` | `v` | `i` | host |
| `say2` | `v` | `s` | host |
| `screenx` | `i` | `dd` | — |
| `screeny` | `i` | `dd` | — |
| `sendtorc` | `v` | `s` | — |
| `sendrpgmessage` | `v` | `s` | host |
| `serverwarp` | `v` | `s` | — |
| `setani` | `v` | `ss` | host |
| `setbeltcolor` | `v` | `s` | host |
| `setcoatcolor` | `v` | `s` | host |
| `setcursor2` | `v` | `s` | — |
| `seteffect` | `v` | `dddd` | host |
| `setfocus` | `v` | `dd` | nothing (as GS1) |
| `setgender` | `v` | `s` | host |
| `sethead` | `v` | `s` | host |
| `setletters` | `v` | `s` | host |
| `setmap` | `v` | `ssdd` | host |
| `setminimap` | `v` | `ssdd` | host |
| `setmusicvolume` | `v` | `dd` | host |
| `setplayerdir` | `v` | `s` | host |
| `setshield` | `v` | `si` | host |
| `setshoecolor` | `v` | `s` | host |
| `setskincolor` | `v` | `s` | host |
| `setsleevecolor` | `v` | `s` | host |
| `setsword` | `v` | `si` | host |
| `showstats` | `v` | `u` | host |
| `spyfire` | `v` | `ii` | host |
| `stopmidi` | `v` |  | host |
| `stopsound` | `v` | `s` | host |
| `strequals` | `b` | `ss` | — |
| `takeplayercarry` | `v` |  | host |
| `takeplayerhorse` | `v` |  | host |
| `testplayer` | `i` | `dd` | — |
| `tiletype` | `i` | `dd` | — |
| `triggeraction` | `v` | `ddsX` | host |
| `updateboard` | `v` | `iiii` | host |
| `updateterrain` | `v` |  | nothing (as GS1) |
| `worldx` | `d` | `dd` | — |
| `worldy` | `d` | `dd` | — |
| `wraptext` | `o` | `iss` | — |
| `wraptext2` | `o` | `idss` | — |
| `setshootparams` | `v` | `s` | — |
| `reloadFile` | `v` | `s` | — |

