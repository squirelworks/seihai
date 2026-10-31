# Offline-only builds

Seihai has a version profile for every build from 1.21 to 6.03 rev 7 (see
`version-profiles.md`). The builds before 1.21, and the two NewWorld betas,
have no profile: no Seihai login reaches a server for them. They are listed in
`crates/seihai-core/src/protocol/offline_builds.rs` (`OFFLINE_BUILDS`) and show
in the launcher under **All versions** as "label (date, offline-only)".

Picking one:

- shows why it is offline-only, which level formats it reads, and which profile
  Seihai plays it with (`plays_as`);
- hides the listserver and direct-connect buttons, so only **Play offline** is
  left;
- keeps its own asset folder (`assets.<label>=`), separate from its
  `plays_as` row's folder, so it can point at that build's install;
- starts on the build's own start level where a source names one
  (`level1.zelda` for Zelda Online 1, `level13.graal` for Java Graal).

`seihai --version "5th Graal"` works the same. Given with `--online`, the
address is dropped with a message and the game starts offline.

Nothing here sends a byte. The rows hold no token, packet or prop data.

## The builds

| Label | Archive | Date | Why offline | Reads | Plays as |
|---|---|---|---|---|---|
| Zelda Online 1 | zeldaonline1 | 1998-05 | no networking | Z3-V1.02/1.03 | 1.21 |
| Zelda Online 2 | zeldaonline2 | 1998-07 | GServ relay, room `Z<base>1.6`, port 14801 | Z3-V1.03 | 1.21 |
| 1st Graal | 1stgraal-1 | 1998-09-06 | relay, rooms `Z<base>` + `Zelda2` | Z3-V1.03/1.04 | 1.21 |
| 2nd Graal | 2ndgraal | 1998-09-16 | relay, rooms `Z<base>` + `Zelda2` | + GR-V1.00–1.01 | 1.21 |
| 3rd Graal | 3rdgraal | 1998-09-30 | relay, rooms `G<level>` + `Graal` | GR-V1.00–1.01 | 1.21 |
| 4th Graal | 4thgraal | 1998-10-05 | relay, rooms `G<level>` + `Graal` | GR-V1.00–1.01 | 1.21 |
| 4th Graal alpha | 4thgraalalpha | 1998-10-12 | relay, rooms `G2<level>` + `Graal2` | GR-V1.00–1.01 | 1.21 |
| 5th Graal | 5thgraal | 1998-10-19 | relay, rooms `G3<level>` + `Graal3` | GR-V1.00–1.02 | 1.21 |
| 5th Graal alpha | 5thgraalalpha | 1998-10-29 | relay, rooms `G3<level>` + `Graal3` | GR-V1.00–1.02 | 1.21 |
| 6th Graal (beta 5) | 6thgraal | 1998-11-09 | relay, rooms `G4<level>` + `Graal4` | GR-V1.00–1.02 | 1.21 |
| 7th Graal (beta 7) | 7thgraal | 1998-12-04 | relay, rooms `G5<level>` + `Graal5` | GR-V1.00–1.03 | 1.21 |
| 8th Graal (beta 8) | 8thgraal | 1998-12-08 | relay, rooms `G5<level>` + `Graal5` | GR-V1.00–1.03 | 1.21 |
| 9th Graal alpha | 9thgraalalpha | 1998-12-16 | relay, rooms `G5<level>` + `Graal5` | GR-V1.00–1.03 | 1.21 |
| 9th Graal (beta 9) | 9thgraal | 1998-12-18 | relay, rooms `G5<level>` + `Graal5` | GR-V1.00–1.03 | 1.21 |
| 10th Graal (1.0) | 10thgraal | 1998-12-22 | relay, rooms `G6<level>` + `Graal6` | GR-V1.00–1.03 | 1.21 |
| 11th Graal (1.0) | 11thgraal | 1998-12-23 | relay, rooms `G6<level>` + `Graal6` | GR-V1.00–1.03 | 1.21 |
| 12th Graal (1.0) | 12thgraal | 1999-01-05 | relay, rooms `G6<level>` + `Graal6` | GR-V1.00–1.03 | 1.21 |
| 13th Graal (1.1 beta 1) | 13thgraal | 1999-01-12 | relay, rooms `G7<level>` + `Graal7` | GR-V1.00–1.03 | 1.21 |
| 14th Graal (1.1) | 14thgraal | 1999-01-15 | relay, rooms `G8<level>` + `Graal8` | GR-V1.00–1.03 | 1.21 |
| 15th Graal (1.1) | 15thgraal | 1999-01-25 | relay, rooms `G8<level>` + `Graal8` | GR-V1.00–1.03 | 1.21 |
| 1.1.9 Gold Edition | graal1.1.9 | 1999-02-11 | relay, rooms `G8<level>` + `Graal8` | GR-V1.00–1.03 | 1.21 |
| 1.1 | graal1.1 | 1999-02-14 | relay, rooms `G8<level>` + `Graal8` | GR-V1.00–1.03 | 1.21 |
| Java Graal | javagraal | 2000-07 | relay, room `G2<level>`, port 14802 | GR-V1.00–1.03 | 1.21 |
| NewWorld beta 1 | graalnwb1 | 2001-08-06 | token `G2N15041`, in no row | as 2.05 | 2.05 |
| NewWorld beta 2 | graalnwb2 | 2001-10-03 | token `G2N15041`, in no row | as 2.05 | 2.05 |

Dates, archives and level formats are from
`graal-decompile/archive-survey/build-catalogue.md` and Graal_analysis
`client-internals/precursors-and-fan-builds.md`. The rooms were read from each
beta's `Graal.exe` strings and confirmed live on 2026-10-31 for 1st, 3rd, 4th
alpha and 5th Graal against Knorr's own GServ: every Delphi beta opens **two**
relay connections, one to a room per level (`<level>` is the file name,
`<base>` the name without extension) and one to a fixed game-wide room. Every
Delphi beta pre-fills `127.0.0.1:14801`; from 3rd Graal on the server box is
greyed out at Cyberjoueur's `204.85.8.128:14802` (the 4th and 5th alphas keep an
editable `127.0.0.1:14801`).

1.21 is the `plays_as` row for the 1998–1999 builds because it is the oldest
profile and reads both `.graal` and `.zelda` with no `.nw`. Its HUD, editor and
drop rules are 1.21's, not the betas' own; the betas' differences are not
modelled. Seihai's Z3-V1 reader takes 1.03 and 1.04 only, so Zelda Online 1's
`Z3-V1.02` levels do not open yet.

Not listed:

- **2.1 test 2 and 3** and **GraalX**: they log in with 2.05's and 2.01's
  tokens, so the existing 2.05 and 2.01 rows already cover their wire.
- **Graal3D**: its own protocol and no Graal level loader, so there is nothing
  for the 2D offline game to play.
- The Game Boy Color demo and 39ster's 2014 remake are not Graal PC clients.

## Can Java Graal and 5th Graal meet in one room?

Not on a stock relay, yes through a room-renaming bridge. Both were run live
against Knorr's GServ on 2026-10-31 (write-up, tool and captures: Graal_analysis
`client-internals/relay-room-bridge.md`).

- Java Graal joins `G2<level>`; 5th Graal joins `G3<level>` (plus `Graal3`).
  GServ only links players in the same room, so they never see each other.
- A proxy in front of GServ that renames `G2` to `G3` in the join line put
  both in `G3level13.graal`. Each client then drew the other player; nickname,
  position and chat went both ways, and Java Graal's level change (out to a
  house and back) rejoined the bridged room.
- **4th Graal alpha needs no bridge**: it joins `G2<level>` and `Graal2`,
  exactly Java Graal's two room modes, and the two met on a stock GServ with
  chat both ways. No other build here shares a room with Java Graal.

Seihai itself still has no relay client, so these rows stay offline-only. The
unit test `java_graal_meets_4th_graal_alpha_not_5th_graal` records the room
rule.
