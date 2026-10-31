# GS2 Stage F: lab check (2026-10-31)

One run in the private lab, on loopback only. A lab GServer-v2 sent one
GS2 script to a 3.153 login, Ato received it as `PLO_NPCBYTECODE` (131),
the reader parsed it, and the log showed the opcodes. The bytecode matches
the `shared.gs2bc` fixture op for op, so no code changes.

## Setup

- Server: GServer-v2 beta4 with two lab patches. The first fixes the
  bad `any_cast` in `Script::getClientByteCode` that crashed on a 3.153
  login. The second lowers the 131 gate in `TLevel::getNpcsPacket` from
  4.0211 to 3.153 (see `gs2-receive.md`: a stock server never sends 131 to
  3.153). The server logged the login with the 3.153 token.
- Level: one level NPC whose `//#CLIENTSIDE` part is the source of
  `tests/fixtures/gs2/shared.gs2`. The server compiles it with its own
  gs2compiler.
- Client: `seihai-ato --version 3.153 --online … --no-window
  --first-level`, built from main at 23988ce.

## Result

Ato logged:

```
npc 300 props: at (30, 28), image "", sprite 2, vis 1, message "", GS1 script 0 bytes
npc 300: GS2 script, 44 ops, 8 strings, functions [onCreated]
```

The 44-op disassembly under that line is the same, op for op and operand
for operand, as the disassembly of `shared.gs2bc`, which gs2-parser
built (`tests/fixtures/gs2/README.md`). The server's gs2compiler and the
gs2-parser build used for the fixtures emit the same bytecode for this
script. **[lab]**

The server did not crash.

## Not checked

- **The real 3.153 client.** The lab machine has no 3.153 client, so there
  is no capture of a real 3.153 receiving 131 to compare against. Ato's
  reading follows the 3.153 handler read in the decompile
  (`gs2-receive.md`). **[bin]**
- **The packet bytes.** The pcap of the session is encrypted, so 131 is
  confirmed by Ato's decode, not by reading the capture. No bytes from it
  are used as a fixture.
- **Running the script.** The script was not run: Ato logs GS2 and does
  not run it.
