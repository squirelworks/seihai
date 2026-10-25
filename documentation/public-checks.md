# Public-server checks (overnight item 14)

Status: **list checks done from the user's own machine on 2026-10-25 (two
runs); the game-server checks are still to do.** The build box itself cannot
reach any listserver port. This page says what the first public results were
and what they settle, what was tried from the build box, what the sources say
about the listserver ports, and the checklist for the rest.

## First public results (2026-10-25, 11:50 and 11:53 JST)

The user ran the read-only list check (`--list-only` for 2.22, 2.17, 2.31, 2.05
and 2.00, with a packet capture in the second run) from their own Linux
machine. The account is masked here, and no game-server address or name is
recorded. The derived note with the frame layouts and per-run table is
`protocol/public-listserver-2026-10-25.md` and its `.tsv` in Graal_analysis.

| run | profile | port | login | result |
|---|---|---|---|---|
| both | 2.22, 2.31 | 14922 | gen-5 | list of 8 rows, welcome, site and upgrade lines |
| both | 2.05 (user's binary, gen-5 login) | 14922 | gen-5 | same list |
| 1 | 2.05 (user's binary) | 14923 | gen-5 | failed once: `unknown gen-5 compression byte 0x78` |
| 2 | 2.05 (user's binary) | 14923 | gen-5 | list of 12 rows: 4 legacy rows, then the 8 current rows |
| both | 2.00 | 14923 | classic | legacy list: 4 rows, no welcome, site or upgrade lines |
| both | 2.17 | 14922 | classic | no reply for 5 s: `Resource temporarily unavailable (os error 11)` |
| both | 2.05 | 14921 | gen-5 | connection refused (a TCP reset to the first SYN) |

What this settles:

- **14921 is dead.** The service resets it at once. A refused connection on
  14921 now says so and points at 14923.
- **14923 serves the legacy list.** A classic login (one plain zlib frame with
  id 0 and id 1) gets ONE plain zlib frame back: an id 0 list of 4 rows (count
  byte 4, 8 fields each, all untagged) and no id 2, 3 or 5. This is the first
  real 2.00-era listserver reply (item 13 can now start from it).
- **14922 does not answer a classic login** (the 2.17 profile): the server
  accepts the connection, sends nothing and stays open. Seihai now reports
  "the listserver sent no reply within 5 s" instead of os error 11. Whether a
  real 2.17 client logs in differently is still not known.
- **14922 reply:** two gen-5 zlib frames, a list (id 0) and then ids 2, 3 and
  5. 14923 answers a gen-5 login with the legacy list first (sent right after
  the id 7 frame, before the login message), then those same two frames.
  Seihai keeps both lists in order.
- **"9 server(s) online" over 8 rows is not a dropped row.** The id 0 count
  byte was 8, and exactly 8 complete rows (all with 8 fields) used up the
  payload. The 9 is in the server's status text and does not follow the row
  count (still 9 over 12 rows on 14923).
- **The host `.xxx.xxx.xxx.xx` with missing leading characters is what the
  server sent** (15 bytes, in both lists). The parser keeps it as received.
- **The `???` in a name is on the wire** (three literal `0x3F` bytes).
- **The reply timing matters.** On 14923 the two current frames arrive about
  0.35 s after the legacy list. The wait for more frames after the first list
  was 500 ms; it is now 1.5 s, and a status frame ends it at once.

What stays open:

- The one 2.05 failure on 14923 (a plain zlib frame, first byte 0x78, read by
  the gen-5 reader) did not repeat in run 2, and the run that failed has no
  capture. Cause unknown. As a precaution a listserver reply now accepts a
  plain zlib frame on a gen-5 link; game-server frames are still strict.
- The binary that made these runs sent the gen-5 login for the 2.05 profile.
  The current main sends the classic one for 2.05 (checked on a local mock),
  so the classic 2.05 login against 14923 follows from the matrix result (every
  other classic profile got a list there) but was not run.
- A classic login to 14923 with the 2.17 profile was not tried in these two runs
  (the matrix later got a list), and nothing tells how a real 2.17 client logs in
  to the listserver.
- A real 2.00 client sends only id 0 and the list name (no account). Seihai's
  2.00 profile sends the account too. Whether the legacy list needs it is
  untested.
- No game-server login has been made. Login frames, file requests, the tamper
  check and chat on a public game server are all still unobserved.
- Seihai's default list endpoint now follows the public service, not the
  binary (`documentation/port-rule.md`): `listserver.graal.in`, 14923 for
  profiles that send the classic login and 14922 for the gen-5 login. The
  binary's own host and port are kept as history (`built_in`). An explicit
  `--online HOST:PORT` still wins.

## The port matrix (2026-10-25, 22:43 JST)

The user ran `run_port_matrix.sh` on their own Linux machine: 19 version profiles, each once on port 14922 and once on 14923 of `listserver.graal.in`, one list-only login per run (`--list-only`, no game server, no chat, no warp), each run captured. 38 runs, no ban, rate limit or reset. The account is masked and no frame bytes are kept here; the derived note is `protocol/listserver-login-type-matrix-2026-10-25.md` (and `.tsv`) in Graal_analysis.

| login Seihai sent | profiles | 14922 | 14923 |
|---|---|---|---|
| classic (plain zlib id 0 + id 1) | 1.41, 1.42, 2.00, 2.01, 2.02, 2.02 rev 2, 2.03, 2.04, 2.1, 2.12 to 2.17 (15) | no reply for 5.5 s, connection left open (15 of 15) | legacy list, 2 or 4 rows, no status or site lines (15 of 15) |
| gen-5 (id 7, `GNW30123`) | 2.22, 2.3, 2.31 and the matrix binary's 2.05 | modern list, 8 rows plus status, site and upgrade lines (4 of 4) | modern list, 12 rows (2.31: 17) (4 of 4) |

What it settles:

- **The reply depends on the login type, not on the version or token.** Every classic login got a list on 14923 and nothing on 14922; every gen-5 login got a list on both. So the default port follows the login the profile sends: classic 14923, gen-5 14922 (`documentation/port-rule.md`).
- This **moved 2.1 to 2.17 from 14922 to 14923**: their profiles send the classic login, which 14922 never answers. (This is the user's GUI port box for those versions.) 2.05 moved too, because the current main sends the classic login for it, see below.
- The ports of 15 classic profiles and of 2.22, 2.3 and 2.31 are now `confirmed`, with the meaning "Seihai's login for this profile got a list there". A real client of any of these versions was not captured.

What it does not settle:

- **2.05.** The matrix binary sent the gen-5 login for the 2.05 profile (28-byte frame, same shape as 2.22's); both ports answered it. The current main sends the classic login for 2.05, so that login was not run for 2.05. The 2.05 port is `assumed` (14923); a rerun of the matrix with a build of the change that follows from this result would confirm it.
- The row counts of the legacy list (2 rows in 4 runs, 4 in 11) varied between runs without following the version; the cause was not tested.
- Everything about game servers: login, chat, warps, files.

## The first attempt from the build box

## What was approved

Read-only checks first (fetch and decode the listserver list), then a log in to
one public game server with a normal account, a few chat lines, a level warp or
two, normal asset downloads and a `PLO_SERVERWARP` if the server sends one. A
few connections in all, a few minutes each at most, no uploads, no account
changes, no scanning, no bypassing anything.


**Port confirmation (user, 2026-10-25):** port 14923 is the legacy support port on listserver.graal.in. For pre-2.17 clients use 14923; 14922 serves 2.17 and later. The earlier open question (14921 vs 14923) is settled.

### What was tried (2026-10-25, 03:08 to 03:10 JST)

Seihai built from this branch, version profile 2.22, from the box's own
network. Every attempt to connect to a listserver port got no answer at all.

| # | what | port | profile | result | time |
|---|---|---|---|---|---|
| 1 | `seihai --online <listserver>:14922 --list-only` (packets captured) | 14922 | 2.22 | no reply to the TCP SYN; Seihai gave up after its 5 s connect timeout: `listserver failed: connection timed out` | 5 s |
| 2 | plain TCP connect test, 3 ports in turn (no data sent) | 14922, 14923, 14921 | none | all timed out | 8 s each |
| 3 | plain TCP connect test, packets captured | 14923, 14921 | none | no reply to the SYN, which was retransmitted at about 1 s intervals | 6 s each |

The capture of attempt 1 holds five SYN segments and nothing else, one per
second, never answered. No byte of any protocol, and no login frame, was ever
sent. Because nothing connected there is no gserver list, so no game server was
contacted, no login was made and no chat line, warp, file request or tamper
check exchange took place.

Controls from the same box: an HTTPS request and a raw TCP connection to a
neutral public host on port 443 succeed; a raw connection to port 80 of the
same neutral host times out too. The box's egress looks like a proxy that
passes ordinary web ports and drops other ports; the DNS answers for all the
listserver names pointed at one private-range address that is not the real
host. So the cause is the box's network policy, not the listserver and not
Seihai's handshake. This would also fit the earlier timeout of the 2.22 list
fetch, which had no established cause. It is a reading of the
symptoms, not something the network owner confirmed.

No attempt was made to get round it (no other port, tunnel, address or
machine), because that would be outside what was approved.

## Port facts: what is known and what is not

None of these was observed on a public listserver. All come from client
binaries, sources and the lab.

| client | host and port the client uses | source | status |
|---|---|---|---|
| 2.00, 2.05 | built-in list host, port 14921 (host is the dead original) | binary | 14921 is **refused** on the public service (observed); a classic login is answered on 14923 (observed with the 2.00 profile and 14 others in the matrix) |
| 2.05 rev 2 on Reborn | 14922 or 14923 | earlier notes, public runs | 14923 for a classic login (observed 2.00 profile); 14922 answers only the gen-5 login (observed with the 2.05 profile of the user's binary) |
| 1.41 | 14923 (the only surviving copy is patched) | binary | open |
| 2.17 | list host and port from `arrays2.dat`, default 14922 | binary | classic login on 14922: **no reply** (observed, 3 of 3); on 14923 a **legacy list** (matrix, 2026-10-25) |
| 2.22 | `listserver.graal.in`, then `.graal.cc`, then `.graalonline.biz`, port 14922; token `GNW30123` | binary, lab, public | **answered** on 14922 and 14923 (observed) |
| 2.31 | as 2.22 | binary, public | **answered** on 14922 and 14923 (observed) |

Seihai's per-profile defaults were the ports the client binaries carry (14921
for 2.00 to 2.05). They now follow the public results: `listserver.graal.in`,
14923 for 1.41 to 2.04 and 14922 for 2.05 and later, with the binary's value
kept as `built_in`. Only 2.00, 2.22 and 2.31 were confirmed by a check at that
time. (Superseded by the matrix below: the default follows the login type, and
18 profiles are confirmed.)

## Questions this was meant to settle, and their state

| question | state |
|---|---|
| Real list text for the server list: item 0, names with spaces, `X ` type prefixes (id 178 field order) | list text observed (row layout, `X ` prefixes, silver rows untagged); id 178 field order still needs a server warp on the wire |
| Which port answers for clients older than 2.17 | **settled for the classic login: 14923** (14921 refused, 14922 silent); the matrix confirmed it for all 15 classic profiles |
| What 2.17, 2.22 and 2.31 profile logins get from the public service | list level only: 2.22 and 2.31 get the list on 14922 and 14923; 2.17 (classic login) gets a legacy list on 14923 and no reply on 14922 (the login type decides, see the matrix); game-server logins open |
| A 2.00-era listserver reply (overnight item 13) | **observed** (14923): one plain zlib frame, 4 untagged rows, no status messages. Item 13 is unblocked |
| Login frames of a public game server: PLO/PLI sets, file update, process list, tamper check, whether the tamper value is enforced | open |
| Chat, warps, normal asset downloads, `PLO_SERVERWARP` on a public server | open |

(Earlier notes in `protocol.md` mention public 2.22 gserver observations from
2026-10-24; they are not repeated or checked here, and this change has no
capture of them.)

## Checklist for when a path exists

Steps 1 and 2 were done by the user on 2026-10-25 (see above); the rest are
still to do. Run from a network that reaches the listserver ports. Keep the password in an
environment variable, never on a command line.

1. Capture only the target traffic into a private folder (directory mode 700,
   files mode 600), for example `tcpdump -i IFACE -s 0 -w FILE 'host H and tcp
   port P'`. The pcaps are never committed or uploaded.
2. `seihai --online HOST:PORT --account NAME --password-env VAR --version V
   --list-only` for 2.22, then 2.17 and 2.31. Compare ports 14922 and 14923 for
   a pre-2.17 profile, read-only, one connection each. (Done for every profile in
   the matrix of 2026-10-25, including the 2.17 profile on 14923.)
3. Read the capture with `seihai capture show FILE --version V` (login frames
   are redacted, the account is masked). Record item 0 and the name spelling of
   the list text (id 178 field order) and any prefix on server names.
4. One game-server login per profile, a few minutes at most: note the login
   message set and order, the file update and process list handling, and
   whether the tamper value is checked, without sending anything abnormal.
5. At most five ordinary chat lines in all, one or two level warps through
   normal links, the normal `WANTFILE` / `UPDATEFILE` flow, and the follow of a
   server warp if one arrives. Stop at any ban, rate limit, captcha or error
   that looks like a stop sign.
6. Turn what the captures settle into profile-gated code with synthetic tests,
   update `packet-layouts.tsv` and the version-chain unknowns, and leave the
   rest as notes.

## Applied to the profiles (2026-10-25)

Every profile now defaults to `listserver.graal.in` on the port that gets a list (`documentation/port-rule.md`), by the login type the profile sends: classic login (1.41 to 2.17, including 2.05) on 14923, gen-5 login (2.22, 2.3, 2.31) on 14922; nothing defaults to the dead 14921, which stays as `built_in` history on 2.00 to 2.05. Confirmed by the matrix of 2026-10-25 (Seihai's own login for the profile got a list on that port; no real client was captured): 1.41, 1.42, 2.00 to 2.04, 2.1, 2.12 to 2.17 (14923), 2.22, 2.3, 2.31 (14922). Assumed: 2.05 only, because its classic login was not run (the matrix binary sent the gen-5 login for 2.05). Running the matrix again with a build of this change, `VERSIONS=2.05`, would confirm it. The launcher's port box follows the selected version; Graal_analysis' version-profile TOMLs carry the same fields (`client-internals/version-profiles/list-ports.md`).

