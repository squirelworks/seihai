# #464: protocol 19 (3.153 / 4.04r2) 3D side: what exists to trace against (2026-11-13)

**No live trace was possible.** No lab server runs the protocol 19 3D side, and no archive on lab host B holds one. This folder records three things:
1. why no trace was possible
2. the archive search for an original 3.x-era Graal3D server
3. two read-only results from the 3.15x Linux client that the protocol 19 server work can use

Everything here was read-only. The scripts were decoded from scratch copies, and the archive was left untouched. Nothing found in it was run.

## 1. No lab server runs protocol 19's 3D side

seihai-gserver-v2 with `graal3d = true` (branch `seihai` at f359c227) accepts the protocol 19 connect but gives those clients only the handshake and keep-alive:
- `bin/docs/graal3d.md`:
  - "3.153 and 4.04r2 game packets | **not done.** They use Torque's older game packet (moves, datablocks). Those clients get the handshake and keep-alive only."
  - "3.153, 4.04r2 (the first Graal 3D) | 19 | Handshake and keep-alive only."
- `server/include/graal3d/Graal3DProtocol.h`: `Profile::gameObjectPackets` with "the TGE layout of 3.153 and 4.04r2 (moves, datablocks) is not implemented".

So a real 3.153 or 4.04r2 client would get none of these:
- the datablocks phase
- the ghost phase
- the lighting phase
- `MissionStartPhase1`–`3`
- a control object

seihai-gserver (Rust) and seihai-npcserver have no Graal3D, Torque or UDP code.

## 2. Archive search: no original 3.x-era Graal3D server

Private lab archives were searched by filename (`*.mis`, `*.dso`, `*dedicated*`, `*graal3d*`, `*server*`) and by strings (`dedicated`, `createServer`, `allowConnections`, `MissionStartPhase`, `serverCmd`, `commandToClient`, `DataBlocksDone`, `setNetPort`, `GameConnection`). Host path inventories are omitted here (offline only).

**What was not found:** no dedicated Torque server exe, no `-dedicated` build, no `.mis` file, and no `server/*.cs` or `.dso` set.

**Closest client-side materials (no server scripts):**
- A 2005 Linux 3.15x client payload (Loki installer labeled 3.151; engine build string Jan 27 2005): client scripts only — TGE client half of mission start; no `server/`, `.mis`, or datablock scripts.
- Earlier 2003–2004 Linux client payloads: 2D-era client UI `.dso` only.
- A `Graal3DEngine.dll` that is byte-identical to the **4.20r8** engine: 2-phase mission start (GameObject era, protocol 21), not protocol 19.
- 4.04r2 installers present; no 4.04r2 `Graal3DEngine.dll` located beside an unpacked client in the search.
- 2000-era Graal3D archives: unrelated protocol (seihai-g3d), not TGE.

**Listen or dedicated mode:** no 3.153 or 4.04r2 client build showed one. In stock TGE, `createServer` and `-dedicated` handling are TorqueScript (`common/server/*.cs`), not engine code, and none of those scripts shipped in any install found here.


## 3. The 3.15x client's mission start, decoded

The three scripts were decoded from the 2005 Linux payload's `scripts\*.cs.dso` with Graal_analysis `torque/tools/dsodec.py` (DSO v33, commit 7ef63a9). They were decoded with Graal_analysis `torque/tools/dsodec.py` (DSO v33). The decoded dumps themselves are offline only; the authored results below are the publishable summary. `main3D` and `message` were included. All five files decoded with no errors.

### Server-to-client commands (`clientCmd*`) and the client's answers (`commandToServer`)

| `clientCmd` (args) | What the client does, in order | Answer |
|---|---|---|
| `MissionStartPhase1(%seq, %missionName, %musicTrack)` | `echo("*** New Mission: " @ %missionName)`; `echo("*** Phase 1: Download Datablocks & Targets")`; `onMissionDownloadPhase1(%missionName, %musicTrack)` | `commandToServer('MissionStartPhase1Ack', %seq)` at once |
| `MissionStartPhase2(%seq, %missionName)` | `onPhase1Complete()`; `echo("*** Phase 2: Download Ghost Objects")`; `purgeResources()`; `onMissionDownloadPhase2(%missionName)` | `commandToServer('MissionStartPhase2Ack', %seq)` at once |
| `MissionStartPhase3(%seq, %missionName)` | `onPhase2Complete()`; `StartClientReplication()`; `StartFoliageReplication()`; `echo("*** Phase 3: Mission Lighting")`; `$MSeq = %seq`; `error("Lighting mission....")`; `lightScene("sceneLightingComplete", "")`. If lighting started: `schedule(1, 0, "updateLightingProgress")`, `onMissionDownloadPhase3(%missionName)`, `$lightingMission = 1` | **Later:** when lighting finishes, `sceneLightingComplete()` runs `echo("Mission lighting done")`, `onPhase3Complete()`, `onMissionDownloadComplete()`, then `commandToServer('MissionStartPhase3Ack', $MSeq)` |
| `SyncClock(%time)` (`main3D`) | `HudClock.setTime(%time)` | none |
| `MissionStart(%x)` (`main3D`) | nothing (empty) | none |
| `MissionEnd(%x)` (`main3D`) | `alxStopAll()`; `$lightingMission = 0`; `$sceneLighting::terminateLighting = 1` | none |
| `ChatMessage(%x, %voice, %pitch, %msgString, %a1…%a10)` and `ServerMessage(%msgType, %msgString, %a1…%a10)` (`message`) | stock TGE message handling | none |

**How `%seq` is passed:**
- The client passes the server's `%seq` back **unchanged**, as the same script value it received. Phase 3 holds it in the global `$MSeq` until lighting is done.
- The script never converts it.
- On the wire, both directions are plain TorqueScript arguments of `commandToClient`/`commandToServer` (stock `RemoteCommandEvent`). The command name goes as a tagged string, `'MissionStartPhase1Ack'` and the other acks. How an argument is encoded is up to the engine's string packing, not the script: in stock TGE 1.x a value that is a plain decimal integer is packed as an integer, and anything else as a string.
- So the ack carries the sequence in whatever form the server sent it, and the server decides int or text.

**The other phase 3 detail:** the Phase 3 ack is not immediate. It waits for `lightScene`'s callback.

**If `lightScene` returns false:** the client never schedules progress and never sets `$lightingMission`. Whether `sceneLightingComplete`, and so the Phase 3 ack, still runs then depends on the engine's `lightScene`; I didn't check that.

### Datablock and ghost callbacks (the engine calls these)

- `onDataBlockObjectReceived(%index, %total)` calls `onPhase1Progress(%index / %total)`. That's the engine-side SimDataBlockEvent receipt; the `index`/`total` match the event's int12 index and total fields.
- `onGhostAlwaysStarted(%ghostCount)` sets `$ghostCount = %ghostCount` and `$ghostsRecvd = 0`.
- `onGhostAlwaysObjectReceived()` sets `$ghostsRecvd += 1`, then calls `onPhase2Progress($ghostsRecvd / $ghostCount)`.
- `onFileChunkReceived(%fileName, %ofs, %size)` (`missionDownload2`) updates the loading GUI's file progress.
- The `onMissionDownloadPhase1..3`, `onPhaseNProgress`, `onPhaseNComplete` and `onMissionDownloadComplete` functions in `missionDownload2` are GUI only: the snowball loading screen, `$missionRunning`, `StartSurfaceReferencer()`, `StartGrassReplication()`. None of them sends anything.

### The connection (`serverConnection`, `main3D`)

**`initClient()`**, in order:
1. `exec`s the GUI and profile scripts, then `missionDownload.cs`, `missionDownload2.cs`, `serverConnection.cs`, `loadingGui.cs`, `playGui.cs` and `centerPrint.cs`.
2. `setNetPort("0")`: an ephemeral UDP port.
3. Shadow, FOV and zoom prefs.
4. `Canvas.setContent("LoadingGui")`.
5. `new GameConnection(ServerConnection)`, then `%conn.connect()` with no address in script; the engine supplies it.

**Callbacks:**
- `GameConnection::onConnectionAccepted(%this)`: only `LagIcon.setVisible(0)`. The client sends nothing on accept; the server drives the mission start from there.
- `GameConnection::initialControlSet(%this)`: `echo("*** Initial Control Object")`, then `Canvas.setContent(PlayGui)` if PlayGui isn't already the content.
- `GameConnection::setLagIcon(%this, %state)`: does nothing for a `"local"` connection; otherwise it sets `LagIcon` visible when `%state $= "true"`.
- `onConnectionTimedOut`, `onConnectionDropped(%msg)` and `onConnectRequestTimedOut`: `disconnectedCleanup()` and a message box.
- `onConnectRequestRejected(%msg)` knows these reason strings:
  - `CR_INVALID_PROTOCOL_VERSION`
  - `CR_INVALID_CONNECT_PACKET`
  - `CR_YOUAREBANNED`
  - `CR_SERVERFULL`
  - `CHR_PROTOCOL`
  - `CHR_CLASSCRC`
  - `CHR_INVALID_CHALLENGE_PACKET`
  - `CHR_PROTOCOL_LESS`: "Please update your Graal, the server is requiring a new Graal version!"
  - `CHR_PROTOCOL_GREATER`: "The server runs an older version of Graal, contact the admins to update it."
- `addMessageCallback('MsgConnectionError', handleConnectionErrorMessage)` stores `%msgError` in `$ServerConnectionErrorMessage`.

## 4. The 2005 engine, strings only

An earlier lab had already unpacked the 2005 Linux `graalexe` (ELF i386). This pass scanned server-related strings only (listing kept offline). The engine build string is "Jan 27 2005 at 18:05:22".

| String | Count | Notes |
|---|---|---|
| `createServer` | 0 | a script function in stock TGE (`common/server/server.cs`), so its absence in the engine is expected |
| `dedicated` | 0 | the `-dedicated` flag is script-handled in stock TGE |
| `allowConnections` | 1 | **present**: the console function that lets the NetInterface accept connect requests |
| `serverCmd` | 1 | **present**: the prefix the engine uses to dispatch `commandToServer` on a server |
| `MissionStartPhase` | 0 | sent as tagged strings from script only, so not in the engine |
| `commandToClient` | 2 | present, with its usage string |
| `SimDataBlockEvent` | 2 | present (class rep) |
| `onDataBlocksDone` | 1 | present: the server-side callback after the datablocks are sent |
| `GhostAlwaysSet`, `GhostAlwaysObjectEvent`, `onGhostAlwaysStarted`, `onGhostAlwaysObjectReceived`, `onGhostAlwaysObjectsReceived` | present | |
| `LocalClientConnection` | 1 | present: the stock name for a listen server's own client connection |
| `Binding server port to %s`, `Binding server port to default IP`, `Unable to open listen socket: %s`, `Unable to listen on port %d: %s` | present | the server-side port binding |
| `.mis` | 1 | present |
| `CHR_PROTOCOL`, `CHR_PROTOCOL_LESS`, `CHR_PROTOCOL_GREATER`, `Graal3D` | present | the connect checks |

**Reading:**
- The 3.15x engine still carries stock TGE's **server-side network plumbing**: accepting connections, `serverCmd` dispatch, sending datablocks, `onDataBlocksDone`, the ghost-always set, a local client connection, and server port binding.
- What's missing is the **server scripts**: `createServer`, mission loading, the datablock definitions, a `.mis`, and the `GameConnection::onConnect`/`onDataBlocksDone`/`MissionStartPhaseNAck` handlers. None shipped.
- So a listen server inside this client is possible in principle, by supplying stock-TGE-style server scripts, but nothing archived provides them.
- I did not test it, and nothing was run.


## 5. Re-check, 2026-11-14

- I re-decoded `missionDownload.cs.dso`, `missionDownload2.cs.dso` and `serverConnection.cs.dso` from fresh scratch copies. The output is byte-for-byte the same as the earlier decode (CRLF aside).
- Source sha256 values:
  - `missionDownload.cs.dso` `eb6966928f483c2b9422ab1a6d3d2cd2fe921ca40508100e094afea57afe45c9`
  - `missionDownload2.cs.dso` `1a498bda44049fd76c3d83e5df3317d9e54fc1f0cd289fc580af37cfc2671e96`
  - `serverConnection.cs.dso` `c10f51d2d4737a183d840436d2f6d5277c414ba3069fea9193b0578ef2238097`
- `setSnowBallPosition` has a `?:` that `dsodec.py` shows as an empty `goto`. From the disassembly (offsets 46-77) it is `%snowprogress = %progress > 0.2 ? 0.05 + (%progress - 0.2) * 0.95 / 0.8 : %progress / 4.0`. It is GUI only.
- Both `%screenw` and `%screenh` read `getWord(LoadingGui.extent, 0)`. That is in the original bytecode, not a decode error.
- `clientCmdClearCenterPrint` and `clientCmdclearBottomPrint`, called from `disconnectedCleanup()`, are defined in `centerPrint.cs.dso` (checked).
- `disconnectedCleanup()` order: `HudMessageVector.clear()`, `alxStopAll()`, `MusicPlayer.stop()` if it exists, hide `LagIcon`, clear bottom and center print, `clearTextureHolds()`, `purgeResources()`, `Canvas.setContent(MainMenuGui2D)`.
- The UPX step was skipped again. Section 4 uses the copy that an earlier lab unpacked.

## Capture summary

Decoded TorqueScript dumps (`.cs.txt` / `.dis.txt`), engine string listings, and private archive path inventories are **not** in this tree. Authored conclusions stay above. Offline: Share pack nested `seihai-lab-464-p19-trace`. Do not re-commit decoded client scripts or host path lists.
