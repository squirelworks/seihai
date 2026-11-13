# #464: protocol 19 (3.153 / 4.04r2) 3D side: what exists to trace against (2026-11-13)

**No live trace was possible.** No lab server runs the protocol 19 3D side, and no archive on Mirae holds one. This folder records three things:
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

**Where I looked:**
- `D:\[2026-11-03]` (UGD, Graal 1Go, Graalians Archive Project, New Folder 5)
- `D:\[2026-11-13]`
- `D:\[2026-11-14]` (the deduped set)
- `C:\Users\brian\Claude\Graal`
- `Documents\ChatGPT\Graal`
- the earlier engine teardowns

**How:** by filename (`*.mis`, `*.dso`, `*dedicated*`, `*graal3d*`, `*server*`) and by strings (`dedicated`, `createServer`, `allowConnections`, `MissionStartPhase`, `serverCmd`, `commandToClient`, `DataBlocksDone`, `setNetPort`, `GameConnection`).

**What I didn't find:** no dedicated Torque server exe, no `-dedicated` build, no `.mis` file, and no `server/*.cs` or `.dso` set anywhere.

| Candidate | Size, date | What it is |
|---|---|---|
| `D:\[2026-11-14]\Websites\Wayback (client-hunt)\work\wb\graalinstall-20050213\setup.data\release\` | `graalexe` 2,531,504 B (2005-01-29, UPX-packed); `main.cs` 340 B (2005-01-28); `scripts\*.dso` | The Linux 3.15x client payload; the Loki installer labels it 3.151. The engine was built Jan 27 2005. **Client scripts only:** the TGE client half of the mission start (section 3), and no `server/`, `.mis` or datablock scripts. The closest thing to protocol 19 on Mirae. |
| `…\wb\graalinstall-20030524`, `-20030718`, `-20040114` | 2003–2004 | 2D-era Linux client payloads; client UI `.dso` only |
| `D:\[2026-11-03]\Graal\[Graal]\Graal 1Go\Graal4_client_Unpacked\Graal3DEngine.dll` | 1,417,216 B | Byte-identical to the **4.20r8** engine (sha256 `afc288bf…`; the same file is in `Graal 1Go\…\v4\Graal v4.20r8[Windows][Unpacked]` and `Claude\login-lab\stage\420r8`). Keeps TGE `allowConnections`, `setNetPort` and `commandToClient`, but only a 2-phase mission start (`MissionStartPhase1/2`): the GameObject era, protocol 21, not 19. |
| `Graal 1Go\Apps\Clients\Main\v4\Graal v4.04r2[Windows][Installer]\Graal4Setup.exe` | 3,913,032 B (2014-04-25) | The 4.04r2 Windows installer; 7-Zip can't open it. The only unpacked 4.04r2 on Mirae, `Claude\Graal\4.04r2\Graal4.exe` (977,920 B), has no `Graal3DEngine.dll` beside it, and I found no 4.04r2 copy of that DLL. The Linux `.bin` and Mac `.dmg` are installers that I didn't open. |
| `Graal 1Go\…\v3\Graal v3.15r1[Linux][Broken].sh` | 9,229,454 B | A Loki setup shell only; the payload was downloaded at install time (the Wayback payload above) |
| `Graal 1Go\…\v3\Graal v3.04[Linux][Fixed].tgz` | 6,612,356 B | A 3.04 client: `main.cs`, `common/main.cs.dso` and client UI. No server scripts. |
| `Graal3D.zip`/`.rar`/`graal3d.7z` (several archives), `Claude\Graal\graal3d\Graal3D.exe` | | The 2000 Graal3D: an unrelated protocol (seihai-g3d's), not TGE |

**Listen or dedicated mode:** no 3.153 or 4.04r2 client build showed one. In stock TGE, `createServer` and the `-dedicated` handling are TorqueScript (`common/server/*.cs`), not engine code, and none of those scripts shipped in any install found here.

## 3. The 3.15x client's mission start, decoded

The three scripts were decoded from the 2005 Linux payload's `scripts\*.cs.dso` with Graal_analysis `torque/tools/dsodec.py` (DSO v33, commit 7ef63a9). Output is in `decoded/`: `*.cs.txt` is the source view and `*.dis.txt` the disassembly. `main3D` and `message` are included too. All five files decode with no errors.

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

## 4. The 2005 engine, unpacked

**About the unpacked copy:**
- No `upx` binary is installed on Mirae.
- The 2005 `graalexe` was already unpacked by an earlier lab. That copy is `graalinstall-20050213.unpacked` in `Claude\client-harness-2026-11-13\runs\f222min\b1tok\upx.tgz`: ELF i386, 6,942,828 B, sha256 `d9a9932a…`. The packed original's sha256 is `3c35f280…`.
- I extracted it to scratch and scanned its strings (`graalexe-2005-server-strings.txt`, with line numbers).
- The engine build string is "Jan 27 2005 at 18:05:22".

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

## Files

- `decoded/missionDownload.cs.txt`, `missionDownload2.cs.txt`, `serverConnection.cs.txt`, `main3D.cs.txt`, `message.cs.txt`: decompiled source view, plus `*.dis.txt` disassembly
- `graalexe-2005-server-strings.txt`: the server-related strings from the unpacked 2005 engine, with line numbers in the full strings dump
