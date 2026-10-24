# Version profiles

Picking a version in the launcher makes Seihai act as that client, not only
speak its protocol generation. Each version has one row in
`client/src/protocol/profile.rs` (`PROFILES`), and the rest of the client reads
that row instead of matching on versions.

## What a row holds

| Group | Fields | Used by |
|---|---|---|
| Protocol | client type, encryption generation, listserver (port, token), server→client ids handled, client→server ids sent, language and tampercheck after login, UDP | login, listserver fetch, dispatch, send helpers |
| Props | ids the own-player and other-player prop parsers accept, NPC prop max, old bow form of prop 10, precise positions | prop parsing |
| Files | modtime in `PLO_FILE`, large files, `LARGEFILESIZE`, default image extension | file replies |
| Levels | `.graal`, `.nw`, `.gmap`, level heights | F5 level picker |
| UI | server list style (direct, one list, tabs), Up/Down chat recall, HUD art, gserver minimap, built-in editor, ganis, emoticons, configurable keys, log window, ghost mode, server name in the title | launcher, game loop |

There are 26 rows: 2.02 rev 2, 2.03 and 2.04 are rows of their own, because
each differs on the wire from the build before (LANGUAGE at 2.02 rev 2,
TRIGGERACTION and PLO 48/165 at 2.03, SHOWIMGNPC and prop 42 at 2.04).

A row is built from a generation template (`CLASSIC` for 1.33–1.39, `BASE`
for 2.05, `NEW_2_1` for 2.1–2.17, `GEN5` for 2.22+) with only the differences
spelled out, so a new version is a few lines.

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
  2.3+, and F5 only lists the level formats the version can open.

## Basis

Rows read from a client binary are marked `Basis::Binary`; rows with no binary
(2.1 to 2.16, 2.3) are `Basis::Inferred` and show as "inferred" in the launcher.
Sources: Graal_analysis `cross-version-timeline.md`, `client-versions.md`,
`player-props.md` and `feature-timeline.md`; the 1.33–1.39 version profile
table; GServer-v2 and gs2lib for tokens and the server's version branches.

## Keeping rows in step with Graal_analysis

`client/src/protocol/profile_toml.tsv` holds the per-version data from the
Graal_analysis version-profile TOMLs (token, server and client id sets, prop
limits, tampercheck, listserver port). `tools/gen_profile_fixture.py
PATH/TO/Graal_analysis > client/src/protocol/profile_toml.tsv` regenerates it,
and the unit test `rows_match_version_profile_toml` fails if any row differs.
When the TOMLs change, regenerate the file and fix the rows the test names.
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

- HUD drawing: the row says which `state.png` layout a build ships, but the
  HUD is still Seihai's own bar until the sheet is drawn.
- Key bindings, emoticons, ganis, log window and ghost mode are recorded but
  not implemented.
- Client→server and server→client sets for 2.1–2.16 and 2.3 are inferred (no binary); 2.22 ids 36, 42 and 45 are unmatched in the binary, so 2.22 does not send 36 or 45.
