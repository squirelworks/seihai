# Version profiles

Picking a version in the launcher makes Seihai act as that client, not only
speak its protocol generation. Each version has one row in
`client/src/protocol/profile.rs` (`PROFILES`, resolved from `chain.rs`), and the rest of the client reads
that row instead of matching on versions.

The builds before 1.21 and the NewWorld betas have no row; they are listed as
offline-only builds (`offline-only-builds.md`).

## What a row holds

| Group | Fields | Used by |
|---|---|---|
| Protocol | client type, encryption generation, listserver (port, token), server→client ids handled, client→server ids sent, language after login, tampercheck (opt-in: constant and re-send rule), UDP | login, listserver fetch, dispatch, send helpers |
| Props | ids the own-player and other-player prop parsers accept, NPC prop max, old bow form of prop 10, precise positions | prop parsing |
| Files | modtime in `PLO_FILE`, large files, `LARGEFILESIZE`, default image extension | file replies |
| Levels | `.graal`, `.zelda` (Z3-V1, every build), `.nw`, `.gmap`, level heights, cached level links (`PLI 30`) | F5 level picker |
| Health | the client-side death sequence (`life.death`: timer, input lock, dead bit, respawn hearts by AP, `PLI 14` claim) | death and respawn; see `health-death.md` |
| UI | server list style (direct, one list, tabs), Up/Down chat recall, HUD art, gserver minimap, built-in editor, ganis, emoticons, configurable keys, log window, ghost mode, server name in the title | launcher, game loop |

There are 26 rows: 2.02 rev 2, 2.03 and 2.04 are rows of their own, because
each differs on the wire from the build before (LANGUAGE at 2.02 rev 2,
TRIGGERACTION and PLO 48/165 at 2.03, SHOWIMGNPC and prop 42 at 2.04).

The rows are the resolved **version chain** (`client/src/protocol/chain.rs`):
1.33 is written out in full, and every later build is a small delta against the
build before it, so a new version is one step with only what changed. A
generated per-step handler-change table (`seihai --version-chain`) lists those
changes, each step's source and what is still unknown. See `version-chain.md`.
The 3.x login and framing, locked against unknown, is in `3x-session.md`.

## What changes when you pick a version

- **Login and framing**: token, client type, key byte and cipher come from the row.
- **Server traffic**: an id the version has no handler for is counted in
  `SessionState::ignored` and dropped, as that client does. A prop stream stops
  at the first id the version's parser does not know, so 2.05 never reads props
  above 42, and another player's MAXPOWER (1) ends that player's stream.
- **Sends**: a message the version could not send is refused. 2.05 and later
  send `PLI_LANGUAGE` after login; 1.3x builds do not.
- **Launcher**: 1.33–1.39 have no listserver, so the launcher offers a direct
  game-server field (the original default was `www.graalonline.com:14900`).
  Each version can point at its own asset folder (`assets.<label>=` in the
  settings file), so it can use that install's images.
- **Game window**: the title follows the version (2.05+ add the server name).
  The minimap appears from 2.1 (set by the game server). Up/Down recall of
  typed chat lines appears in 1.39. F4/F5 (editor and level loading) are gone in
  2.3+, and F5 only lists the level formats the version can open. While online F4 and F5 are refused with a chat line (see `implementation.md`).

## Basis

Rows read from a client binary are marked `Basis::Binary`; rows with no binary
(2.1 to 2.16, 2.3) are `Basis::Inferred` and show as "inferred" in the launcher.
Sources: Graal_analysis `cross-version-timeline.md`, `client-versions.md`,
`player-props.md` and `feature-timeline.md`; the 1.33–1.39 version profile
table; GServer-v2 and gs2lib for tokens and the server's version branches.

## Keeping rows in step with Graal_analysis

`client/src/protocol/profile_toml.tsv` holds the per-version data from the
Graal_analysis version-profile TOMLs (token, server and client id sets, prop
limits, tampercheck, listserver endpoint). `tools/gen_profile_fixture.py
PATH/TO/Graal_analysis > client/src/protocol/profile_toml.tsv` regenerates it,
and the unit test `rows_match_version_profile_toml` fails if any row differs.
When the TOMLs change, regenerate the file and fix the rows the test names.
Default list endpoint (`documentation/port-rule.md`): every profile dials
**`listserver.graal.in`**, and 14921 is dead, so none defaults to it. The public
listserver answers by login type, not by version or token, so the default
follows the login the profile sends: profiles that send the classic login
(`kind` legacy: 1.41 to 2.17, including 2.05) default to **14923**; profiles
that send the gen-5 login (`kind` supported: 2.22, 2.3, 2.31) default to
**14922**. The host and port the binary carries are kept as
`built_in_host` / `built_in_port` (`Listserver::built_in`): history, not what
Seihai dials. Each profile also carries `status`: **confirmed** where a public
check with Seihai's login for that profile got a list on its port (the matrix of
2026-10-25: the 15 classic profiles on 14923, 2.22, 2.3 and 2.31 on 14922; a
real client of those versions was not captured), and **assumed** for 2.05,
whose classic login was not run. The per-profile table is in
`client-internals/version-profiles/list-ports.md` in Graal_analysis.
Defaults are `ClientVersion::listserver_host`, `listserver_port` and
`listserver_addr`. `--online host:port` is used exactly as given (a bare
`--online listserver.graal.in` gets the version's default port). The launcher's
port box shows the selected version's default and changes with the version
selector until the user types a port (or the settings file holds one), which it
then keeps and marks as set by the user; "Use default" hands it back.
Known exception: 2.16 keeps the 2.16 prop limits its TOML note describes
(54–74), though the TOML's `props_max` still says 45.

## Lab note

The lab GServer's gs2lib patch maps 2.05 (`GNW03081`) onto 2.1 and drops 2.1's
own token. A stock server treats 2.05 as an old client, which is what real 2.05
expects: prop 10 in the old bow form and file replies without a modtime. Seihai
now reads 2.05 that way, so against the patched lab 2.05 loses props after 10
(the direction, among them). Run the lab without the patch to see real 2.05
behaviour.

## Not yet per version

Graal_analysis `version-profiles/matrix/` has the cross-era matrix (1998 to
6.11) and `seihai-gaps.md` there lists what these rows do not model yet.

- HUD drawing: the row says which `state.png` layout a build ships, but the
  HUD is still Seihai's own bar until the sheet is drawn.
- Emoticons, log window and ghost mode are recorded but not implemented.
  Builds without GANIs (1.33 to 1.39) draw their built-in player poses from
  the stock GANI files of the asset folder (`builtin-animations-139.md`).
- Client→server and server→client sets for 2.1–2.16 and 2.3 are inferred (no binary); 2.22 ids 36, 42 and 45 are unmatched in the binary, so 2.22 does not send 36 or 45.
- 2.31 keeps PLI 44 in its send set (a real 2.31 client answered the server's process-list request with it); Seihai never sends it.
- 2.3 (inferred) has no tamper-check constant, so `--send-tampercheck` sends nothing for it.
