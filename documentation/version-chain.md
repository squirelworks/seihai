# Version chain and version tokens

These tables live in seihai-core only. This repo used to keep its own copies,
and they fell behind (the 2026-11-13 copy was missing the 4.04r2 Mac, 5.12 Mac
and 6.00r4 rows).

- The version chain: every client build Seihai can present as, as one step
  against the build before it. Generated and checked by a unit test:
  [seihai-core `documentation/version-chain.md`](https://github.com/squirelworks/seihai-core/blob/main/documentation/version-chain.md)
  and [`version-chain.tsv`](https://github.com/squirelworks/seihai-core/blob/main/documentation/version-chain.tsv).
  `seihai --version-chain [--chain-format text|md|tsv] [--version V]` prints
  the same table from the built-in profiles.
- Every known login token, with or without a Seihai profile, with its evidence
  grade and the builds that send it:
  [seihai-core `documentation/version-tokens.tsv`](https://github.com/squirelworks/seihai-core/blob/main/documentation/version-tokens.tsv)
  (`protocol::KNOWN_TOKENS` in code). It follows Graal_analysis
  [`client-internals/version-token-catalogue.md`](https://github.com/squirelworks/Graal_analysis/blob/main/client-internals/version-token-catalogue.md),
  which holds the evidence.

Code: `crates/seihai-core/src/protocol/chain.rs` (the chain) and
`protocol/tokens.rs` (the tokens) in seihai-core.
