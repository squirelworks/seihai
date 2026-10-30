# seihai-ato

The Torque / 3.x client line: Graal 3.04 (Linux, Jan 2004) and 3.153 (Mac,
Feb 2005).

The 3.x clients still speak gen-5 framing and use the same board, gmap and
GS1 formats as 2.22 and 2.31, so this crate shares `seihai-core` with the
2.x client (`seihai-mae`). What is 3.x-only lives here.

## What is in (Stage 1)

- `args`: the command line. `seihai-launcher` passes the user's arguments
  through, so unknown flags are ignored. `--version 3.04` or
  `--version 3.153` (the default); `--account`, `--password-env VAR`,
  `--nick`.
- `wire`: the build's game and listserver tokens and the ids it adds or
  drops against 2.31, all read from the build's row in
  `seihai-core::protocol::chain` (nothing restated).
- `torque`: the Torque connect profile per protocol number (9 for 3.04,
  19 for 3.153 and 4.04r2, 21 for 4.20r6, 24 for 5.12): game string, class
  table, 12-bit class ids, 14-bit length prefixes, and the port rule
  (listed port + 1). Data only. Source: Graal_analysis
  `torque/wire-by-build.md`.
- `start_window`: the 3.x start window, laid out from the clients' own GUI
  files (positions, texts, colours, language list). The buttons say what
  the real client would do next; none of it connects yet.

```
cargo run -p seihai-ato -- --version 3.153
cargo run -p seihai-ato -- --version 3.04 --print-profile
```

## Not in yet

The server list, offline mode, the Graal TCP session, any level drawing,
and the Torque UDP side. The clients' background images are not shipped,
so the start window's panel is a plain fill.
