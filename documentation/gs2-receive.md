# GS2 receive path on 3.153 (Stage E)

Date: 2026-10-31. How a server NPC's compiled GS2 script reaches a 3.x
client, and what Ato does with it. The bytecode format is in
`gs2-opcodes.md`, the reader in `scripting/gs2.rs`.

Tags as in `gs2-executor.md`: **[bin]** read from the 3.153 Mac client
(the existing decompile, `graal-decompile/v3x/mac315.c`; no new decompile)
· **[src]** from GServer-v2 or gs2-parser · **[inf]** inferred.

## The packet

`PLO_NPCBYTECODE`, id 131. Of the 3.x builds only 3.153 handles it; 3.04
does not (`Graal_analysis/protocol/v3x-clients.md`). **[bin]**

```
PLO_RAWDATA (100)  GINT3 n  \n
131  GINT3 npc_id  bytecode           (n bytes in all, id byte included)
```

- `bytecode` is the whole file the compiler writes: the four segments, then
  the compiler's own `\n` (`GS2Bytecode::getByteCode`). It has no header:
  GServer-v2 compiles NPC scripts with `GS2Context::compile`, not with
  `CreateHeader`, which only class and weapon scripts get. **[src]**
- The sender is `TLevel::getNpcsPacket`. Each NPC's 131 comes right after
  its `PLO_NPCPROPS` (3). The sender adds a `\n` only if the bytecode does
  not already end in one, so for gs2-parser output it adds none. **[src]**

## What the 3.153 client does with it

The handler at `0x2e0b88` **[bin]**:

1. Needs at least 3 bytes. Reads bytes 1–3 (the string class counts from 1)
   as a GINT3, high 7 bits first, and drops them.
2. An id of 0 or less is dropped. Ids under 1000 are looked up among the
   current level's NPCs, 1000 and up in the global NPC list (`0x2e0c78`).
   An id that is not found is dropped. The NPC has to exist already, which
   is why the server sends 131 after the NPC's props.
3. Hands the rest to the NPC's script object (`0x3ec0f0` → `0x3e2030`),
   which calls the bytecode loader `0x3c9a80` (the one that decodes the
   `F0`–`F6` operands, see `gs2-executor.md`). It then ORs two words of the
   script and of each class in its class list into the NPC (likely the GS1
   event flags; not checked).
4. If any bytes were left, calls a method of the NPC (vtable slot `0x34`).
   What that slot does was not read. **[bin]**

`0x2e0b88` has no direct caller in the decompile: the receive code reaches
it through a table. That 131 lands here is from the receive-switch read in
`v3x-clients.md` together with the layout match (`i3 code, bytecode` in
`protocol/packet-ids.md`). **[inf]**

No capture of a real 131 exists in the project (the lab runs are 3.04,
which has no 131). Nothing here was checked against one.

## GServer-v2 does not send 131 to a 3.x client

`getNpcsPacket` sends 131 only when the client version is
`CLVER_4_0211` or later. For older clients it sends the GS1 script in
`NPCPROP_SCRIPT` as before. **[src]** `GServer-v2/server/src/versions.txt`
lists 4.0.2.11 as `G3D28095`; 3.153 logs in as `G3D28015`. The table that
turns a token into a version id is in gs2lib, which was not in the source
tarball, so where GServer-v2 places 3.153 was not read. It cannot be at or
past 4.0211 by the list's order, so a stock GServer-v2 sends a 3.153 login
GS1 only. Stage F's lab check needs a lab server that sends 131 to this
login (see the PR). **[src]/[inf]**

## What Seihai does

- Core's `SessionState` keeps each 131 by NPC id in `npc_bytecode`, as
  sent, if a `PLO_NPCPROPS` has named that NPC; otherwise it counts it in
  `npc_bytecode_dropped`, as the client drops it. Builds that do not handle
  131 (every classic build and 3.04) leave it in `ignored` as before.
  Classic clients stay on GS1: nothing on their path reads or runs it.
- Ato reads each script of the level's NPCs with `Script::parse` when the
  level is logged and keeps it on the `Arrival` it hands to the view. The
  log gets one line per script (ops, strings, function names) and its
  disassembly. Nothing runs it yet.
- `tests/online_loopback.rs` sends the `shared.gs2bc` fixture as 131 behind
  `PLO_RAWDATA` after NPC 1's props. 3.153 logs the script and its opcodes;
  3.04 does not.
