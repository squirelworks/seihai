# Listserver login: what real clients send and what Seihai sends

Source: lab captures of real clients against a private loopback listserver (2.05, 2.17.1, 2.22, 2.31), a lab capture of a real 2.00 (client side only), and the public-listserver matrix of 2026-10-25 (Seihai only). The derived note is Graal_analysis `login-frames-by-version`. No account, password, key byte or raw frame is kept here.

The listserver login differs by client **generation**, in four shapes. Inside a generation every observed build sends the same bytes. The per-build token (`GNW24011` for 2.00, `GNW22122` for 2.17.1, and so on) goes to the game server, not the listserver; only shape C carries a token to the listserver, and it is one shared token.

| Shape | Real client (captured) | Frames to the listserver |
|---|---|---|
| A | 2.00, 2.05 | One plain zlib frame: id 0, list name `newmain`, LF. 9 bytes decoded, a 17-byte zlib stream. No account, password, token or key. |
| B | 2.17.1 | One plain zlib frame: id 0 `newmain`, then id 1 (account, password), LF. No token, key or cipher. |
| C | 2.22, 2.31 | A plain zlib frame: id 7, key byte, `GNW30123`, `newmain`, LF. Then a gen-5 frame with id 1 (account, password), XOR-ciphered with the key. |
| D | 3.04 | One plain zlib frame at the default level (`78 9C`): id 7, key byte, `G3D30123`, `newmain`, LF, then id 1 (account, password), LF. The replies are gen 5, as for C. |

The real clients compress at zlib's best level (`78 DA`). Seihai now does the same for the listserver login frames. On the game server, the gen-2 builds (1.33 to 1.39) compress every frame at the best level too, the login and each later one: the real 1.28 beta 2 to 1.39 all did in the lab (2026-10-30), and Seihai 1.33 and 1.39 now match them. Gen 3 and the gen-5 login frame keep the default level (`78 9C`); the real 3.04 sent its game login at that level. Decompression is unchanged.

## What each Seihai profile sends

| Profile | Shape | Basis |
|---|---|---|
| 1.33 to 1.39 | A (no listserver of its own; fetches the list as 2.05) | 2.05's, captured |
| 1.41 | B | **inferred, not captured; no evidence for it** (what Seihai sent before) |
| 1.42 | A | **inferred, not captured** (best guess from 2.00; the build's host and list name are known) |
| 2.00 | A | captured (real 2.00, lab) |
| 2.01, 2.02, 2.02 rev2, 2.03, 2.04 | A | **inferred, not captured** (best guess from 2.00 and 2.05) |
| 2.05 | A | captured (real 2.05, lab) |
| 2.1, 2.12 to 2.16 | B | **inferred, not captured; no evidence for it** (no binary exists; what Seihai sent before) |
| 2.17 | B | captured (real 2.17.1, lab) |
| 2.22, 2.31 | C | captured (real clients, lab) |
| 2.3 | C | inferred from 2.31 (no 2.3 build exists) |
| 3.04 | D | captured (real 3.04 Linux, lab, 2026-10-30): Seihai's frame matches the real one byte for byte once the key, account and password are masked |
| 3.153 | D | taken from 3.04 (the Mac build was never run) |

`Listserver::shape` and `Listserver::shape_basis` in `protocol/profile.rs` hold this. The version-chain table shows it on every row, and the launcher's port-box hover text names it.

## What is not known

- No profile that sends shape A has been run on the public service. Its port is `assumed` (14923) until it is. The matrix of 2026-10-25 ran Seihai's older shape B for 2.00 and the other classic profiles, with the `78 9C` header; 14923 answered those and 14922 stayed silent.
- The matrix's 2.05 rows are not a classic 2.05 login: the saved-version override (fixed here, `--version` now wins) made them run as a gen-5 profile.
- Whether the public service answers shape A, or checks the zlib header, is unobserved. See `public-checks.md`, "Next matrix run".
