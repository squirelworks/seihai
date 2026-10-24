# Public-server checks (overnight item 14)

Status: **blocked from the build box; nothing was observed.** This page says
what was tried, what happened, what the sources already say about the
listserver ports, and the exact checklist to run when a network path exists.
No wire code or table row changed because of it.

## What was approved

Read-only checks first (fetch and decode the listserver list), then a log in to
one public game server with a normal account, a few chat lines, a level warp or
two, normal asset downloads and a `PLO_SERVERWARP` if the server sends one. A
few connections in all, a few minutes each at most, no uploads, no account
changes, no scanning, no bypassing anything.

## What was tried (2026-10-25, 03:08 to 03:10 JST)

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
| 2.00, 2.05 | built-in list host, port 14921 (host is the dead original) | binary | needs a redirect for Reborn |
| 2.05 rev 2 on Reborn | 14922 or 14923 | earlier notes | **open**: which of them (or 14921) the public service answers for pre-2.17 clients is not observed |
| 1.41 | 14923 (the only surviving copy is patched) | binary | open |
| 2.17 | list host and port from `arrays2.dat`, default 14922 | binary | profile says 14922; not run against the public service |
| 2.22 | `listserver.graal.in`, then `.graal.cc`, then `.graalonline.biz`, port 14922; token `GNW30123` | binary, lab | matches the lab listserver; not run against the public service |
| 2.31 | as 2.22 | binary | not run publicly |

Seihai's per-profile defaults (14921 for 2.00 to 2.05, 14922 for 2.1 to 2.31,
14923 for 1.41) are unchanged. The pre-2.17 port question stays open, as does
whether the public listserver answers a 2.17 id 0 + id 1 exchange.

## Questions this was meant to settle, and their state

| question | state |
|---|---|
| Real list text for the server list: item 0, names with spaces, `X ` type prefixes (id 178 field order) | open |
| Which port answers for clients older than 2.17 | open |
| What 2.17, 2.22 and 2.31 profile logins get from the public service | open |
| A 2.00-era listserver reply (overnight item 13) | not observed; item 13 skipped |
| Login frames of a public game server: PLO/PLI sets, file update, process list, tamper check, whether the tamper value is enforced | open |
| Chat, warps, normal asset downloads, `PLO_SERVERWARP` on a public server | open |

(Earlier notes in `protocol.md` mention public 2.22 gserver observations from
2026-10-24; they are not repeated or checked here, and this change has no
capture of them.)

## Checklist for when a path exists

Run from a network that reaches the listserver ports. Keep the password in an
environment variable, never on a command line.

1. Capture only the target traffic into a private folder (directory mode 700,
   files mode 600), for example `tcpdump -i IFACE -s 0 -w FILE 'host H and tcp
   port P'`. The pcaps are never committed or uploaded.
2. `seihai --online HOST:PORT --account NAME --password-env VAR --version V
   --list-only` for 2.22, then 2.17 and 2.31. Compare ports 14922 and 14923 for
   a pre-2.17 profile, read-only, one connection each.
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
