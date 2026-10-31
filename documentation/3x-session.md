# 3.x session: what is locked and what is not

Date: 2026-10-30. Covers the two readable 3.x clients, 3.04 (Linux, `G3D03014`)
and 3.153 (Mac OS X, `G3D28015`), and only their 2D Graal TCP session: login
and framing. The rows themselves are the `3.04` and `3.153` steps in
`client/src/protocol/chain.rs`.

Sources, all already in Graal_analysis (no new decompile):

- `version-profiles/3.04-3.153.md` and `.toml` (both builds read statically)
- `protocol/v3x-clients.md` (listserver, cipher, handled ids)
- `version-profiles/lab-304-linux-2026-10-30.md` (the real 3.04 on the wire)
- `torque/network.md` (only to say what the 3D side is; not used for bytes)

Tags: **cap** seen on the wire from the real 3.04 · **bin** read from the
binary · **3.04** a 3.153 value taken from 3.04.

## Locked

| Item | 3.04 | 3.153 | Seihai |
|---|---|---|---|
| Client type, key byte | 5, `rand & 0x7F` (bin, cap) | same (bin) | yes |
| Game token | `G3D03014` (bin, cap) | `G3D28015` (bin) | yes |
| Listserver token | `G3D30123` + `newmain` in id 7 (bin, cap) | same (bin) | yes |
| Account and password | plain strings on the wire (bin, cap). The 2.17+ `~b − 10` map is in-memory only, so nothing changes on the wire | same (bin) | yes |
| Cipher | gen 5: LCG seed `0x04A80B38`, mul `0x08088405`, add key, one iterator per direction (bin, cap) | same (bin) | yes |
| Framing | `u16be(len + 1)` + type + body; raw under 0x30, zlib under 0x1000, bz2 from 0x1000; XOR caps 12 / 4 / 4 dwords (bin, cap) | same (bin) | yes |
| Listserver frame | shape D: id 7 and id 1 in one plain zlib frame at `78 9C`, gen-5 reply (cap) | **3.04** | yes; 3.153 now marked inferred |
| Game login frame | one plain zlib frame: 5, 37 `English`, 34 `bigmap.txt`, 34 `minimap.txt` (cap) | **3.04** | yes |
| Tampercheck (39), process list (44), TCP id 24 | none (bin, cap) | none (bin) | none sent |
| Post-login frame | head image, the default weapons' images, then props 0, 18, 31, 33, 52, 75, 76 (cap) | **3.04** | yes, without 31 (no UDP socket) |
| `OSTYPE` (75) | `linux` (bin, cap) | `mac` (bin) | **3.153 changed in this PR** (it sent `linux` before) |
| `TEXTCODEPAGE` (76) | 1250 (cap) | not in the notes | 3.153 keeps 1250, flagged |
| Handled and sent ids, prop parser ranges | bin (and cap for 3.04) | bin, sent set incomplete | already in the rows |

## Unknown, left as it is

- **3.153 on the wire.** It was never run. Its listserver shape, game login
  frame, post-login order and code page are 3.04's. The step's `unknown`
  entries say so, and the listserver shape basis is now `Inferred` rather
  than `Captured`.
- **3.153 sent ids** 2, 3, 15, 24, 29 and 80: carried from 3.04 (the PPC scan
  did not resolve those call sites).
- **Public listserver port** for a `G3D30123` login: only the lab listserver
  answered it. Port status stays `assumed`. Trying it is an outward action.
- **Nickname.** The real 3.04 sends the start window's nickname; Seihai sends
  the account (`login-burst.md`).
- **`STATUS` bit 1** in the real 3.04's first props, and its first `gr.y`:
  not traced.
- **UDP channel** (2.05 P2P codec, id 15 every 5.1 s, ids 0, 22, 26): known
  for 3.04 (cap) but Seihai has no UDP socket for any build, so it is not sent.

## Not started (out of scope)

- **The 3D side of 3.153.** A `3 `-prefixed server makes the client open
  Torque on UDP port + 1 alongside the normal TCP login (bin,
  `torque/network.md`). The Torque handshake was only seen live from 5.12,
  so for 3.x it is not locked and is not modelled.
- **Other-player prop set.** The notes give 0–76 (3.04) and 0–80 (3.153); the
  rows keep 2.31's set without 78–80. That is a prop-parser change, not
  login or framing, so it was not touched here.
- The Torque front end (start window, themes, chat drawing).

## Lab run, 2026-10-31: 3.153 on the wire

Ato 3.153 (main `7203690`) logged in to the patched lab GServer-v2 (port
14900, the gs2lib patch that adds `G3D28015`) and loaded `gs2test.nw`. It
printed `level gs2test.nw: from the server's board, start (30, 32), 1 NPCs`
and read NPC 300's GS2 script (19 ops, `onCreated`). The two frames that
carry the player and the level are in
`crates/seihai-ato/tests/fixtures/lab-3153-gs2test.txt`, and
`tests/lab_3153_gs2test.rs` replays them. What they show:

- The level comes as LEVELNAME, an empty LEVELBOARD (0), the board behind
  RAWDATA, LEVELMODTIME and LEVELNAME again, then NPCPROPS (3) for NPC 300
  and its 131 behind RAWDATA, all in one frame.
- The first PLAYERPROPS runs from prop 1 to 80 (X2, Y2, Z2 included) and
  ends with 82 (`guest`). 3.153's own-prop set stops at 80, so Seihai stops
  reading there, as the client would. Whether the real 3.153 reads 82 is
  not known.
- This settles nothing about the listserver shape or the post-login frame
  of the real 3.153: the client on the wire was Seihai.

Ato now logs these fields under the level line (`seihai-ato/src/fields.rs`),
for builds with a Torque profile only.
