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
| Zelda Online 2 | zeldaonline2 | 1998-07 | GServ relay, room `Z<level>1.6`, port 14801 | Z3-V1.03 | 1.21 |
| 1st Graal | 1stgraal-1 | 1998-09-06 | relay, room `Zelda2` | Z3-V1.03/1.04 | 1.21 |
| 2nd Graal | 2ndgraal | 1998-09-16 | relay, room `Zelda2` | + GR-V1.00–1.01 | 1.21 |
| 3rd Graal | 3rdgraal | 1998-09-30 | relay, room `Graal` | GR-V1.00–1.01 | 1.21 |
| 4th Graal | 4thgraal | 1998-10-05 | relay, room `Graal` | GR-V1.00–1.01 | 1.21 |
| 4th Graal alpha | 4thgraalalpha | 1998-10-12 | relay, room `Graal2` | GR-V1.00–1.01 | 1.21 |
| 5th Graal | 5thgraal | 1998-10-19 | relay, room `Graal3` | GR-V1.00–1.02 | 1.21 |
| 5th Graal alpha | 5thgraalalpha | 1998-10-29 | relay, room `Graal3` | GR-V1.00–1.02 | 1.21 |
| 6th Graal (beta 5) | 6thgraal | 1998-11-09 | relay, room `Graal4` | GR-V1.00–1.02 | 1.21 |
| 7th Graal (beta 7) | 7thgraal | 1998-12-04 | relay, room `Graal5` | GR-V1.00–1.03 | 1.21 |
| 8th Graal (beta 8) | 8thgraal | 1998-12-08 | relay, room `Graal5` | GR-V1.00–1.03 | 1.21 |
| 9th Graal alpha | 9thgraalalpha | 1998-12-16 | relay, room `Graal5` | GR-V1.00–1.03 | 1.21 |
| 9th Graal (beta 9) | 9thgraal | 1998-12-18 | relay, room `Graal5` | GR-V1.00–1.03 | 1.21 |
| 10th Graal (1.0) | 10thgraal | 1998-12-22 | relay, room `Graal6` | GR-V1.00–1.03 | 1.21 |
| 11th Graal (1.0) | 11thgraal | 1998-12-23 | relay, room `Graal6` | GR-V1.00–1.03 | 1.21 |
| 12th Graal (1.0) | 12thgraal | 1999-01-05 | relay, room `Graal6` | GR-V1.00–1.03 | 1.21 |
| 13th Graal (1.1 beta 1) | 13thgraal | 1999-01-12 | relay, room `Graal7` | GR-V1.00–1.03 | 1.21 |
| 14th Graal (1.1) | 14thgraal | 1999-01-15 | relay, room `Graal8` | GR-V1.00–1.03 | 1.21 |
| 15th Graal (1.1) | 15thgraal | 1999-01-25 | relay, room `Graal8` | GR-V1.00–1.03 | 1.21 |
| 1.1.9 Gold Edition | graal1.1.9 | 1999-02-11 | relay, room `Graal8` | GR-V1.00–1.03 | 1.21 |
| 1.1 | graal1.1 | 1999-02-14 | relay, room `Graal8` | GR-V1.00–1.03 | 1.21 |
| Java Graal | javagraal | 2000-07 | relay, room `G2<level>`, port 14802 | GR-V1.00–1.03 | 1.21 |
| NewWorld beta 1 | graalnwb1 | 2001-08-06 | token `G2N15041`, in no row | as 2.05 | 2.05 |
| NewWorld beta 2 | graalnwb2 | 2001-10-03 | token `G2N15041`, in no row | as 2.05 | 2.05 |

Dates, archives and level formats are from
`graal-decompile/archive-survey/build-catalogue.md` and Graal_analysis
`client-internals/precursors-and-fan-builds.md`. The Delphi room names were read
from each beta's `Graal.exe` strings (`N<room>#def`, the GServ join line) on
2026-10-31. Every Delphi beta pre-fills `127.0.0.1:14801`; from 3rd Graal on the
connect dialog also has a 14802 field (not the 4th and 5th alphas).

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

No, not as shipped. Both talk to Knorr's GServ relay lobby, and Java Graal's
`D` payload set is Zelda Online 2's (minus `S`, plus `L`), which the 1st Graal
kept. But GServ only links players who joined the same room name:

- 5th Graal joins the fixed room `Graal3` (one room for every level).
- Java Graal joins `G2` + the level file name, one room per level.

So on one relay they never see each other. No Delphi beta joins a `G2` room.
The near miss is Java Graal's unused fixed-room mode, `Graal2`: that is exactly
4th Graal alpha's room, but the applet never selects it. Builds that *do* share
a room are the ones with the same fixed name above (5th Graal with 5th Graal
alpha, 3rd with 4th, 7th to 9th, and so on).

A bridge would be a relay that renames rooms on join (for example `G2<level>`
and `Graal3` into one room). Even then, mixed sessions would share only the
payload types both sides know (movement, chat, board changes, baddies, items,
arrows; Java players would keep default colours). That is inferred from the
sources and has not been run. Seihai has no relay client or server, so it is a
follow-up, not part of these rows. The unit test
`java_graal_and_5th_graal_do_not_share_a_room` records the room rule.
