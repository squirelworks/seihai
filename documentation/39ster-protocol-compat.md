# 39ster-era protocol compatibility notes

Short reference for anyone expanding Seihai into historical or lab servers that speak the 2006 39ster 2.1 dialect, or when validating loaders against the original corpus.

## Framing (unchanged from modern)

- TCP, default port **14822**.
- Frame = `uint16` big-endian compressed length + zlib payload.
- Inflated payload = one or more messages separated by `0x0A`.
- Outgoing: append `\n`, zlib when buffer ≥ 4 KiB or on flush.

## PLAYER scramble (exact 2006 LCG)

First login line is **not** scrambled. Subsequent PLAYER packets drop one byte:

```
iterator starts at 0x04A80B38
iterator = iterator * 0x8088405 + key
pos = (iterator & 0xFFFF) % count + 1
drop the byte at index pos-1
```

RC clients skip this entirely. The `key` is the second GCHAR of the login line.

## GCHAR

All multi-byte integers and most single-byte values that go through the classic buffer helpers are biased by +32 and packed in 7-bit groups. Frame lengths and raw tile blobs are **not** GCHAR.

## Start map and data

Stub account loader used `level13.graal`. Full original levels, GIFs, Weapons.txt, and accounts are in the Drive archive `Graal 39sters gServer.zip` under `[Grok/[Projects/`.

## Suggested Seihai uses

- Optional historical mode or documentation note when connecting to pure 39ster servers.
- Offline validation suite that loads a subset of the original `.graal` files to confirm the GR-V1 loader.
- Cross-check remaining tiledefs / collision rules against the 2006 level data and Ash notes (now also in Graal_analysis `ash/`).

See also:
- Graal_analysis `ash/ALGORITHMS.md` and `ash/BLUEPRINT.md` (PR #53).
- Drive archive for the complete museum copy and Phoenix reference implementation.

No code changes in this note; pure documentation for the problem-resolution expansion.
