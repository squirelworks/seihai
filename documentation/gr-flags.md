# The `gr.x` and `gr.y` flags

Queue item 15(c), from the differential testing (`documentation/difftest.md`,
divergence 5: "2.22: `PLI_FLAGSET` with `gr.x` / `gr.y` is missing").

## What the real clients send

Real clients keep the player's position in two client flags, `gr.x` and `gr.y`, and
send them to the server as `PLI_FLAGSET` (18) with `name=value` text, so server
scripts can read the position. Evidence: the private lab, real clients on a
loopback server (2.22: `login`, `gmap_cross`, `chat`, `door_warp` runs of
2026-10-24 and 2026-10-25; 2.17: the early login capture of 2026-10-24). Nothing
here comes from a public server.

| build | when | what the captures show |
|---|---|---|
| 2.17 | once, in the post-login frame | `gr.x=4.5` and `gr.y=30` between the file checks and the process list. No walk was captured, so nothing more is claimed. |
| 2.22 | login, then every 50 ms tick in which the position changed | at login `gr.x=58`, `gr.y=30.5` (both, `gr.x` first), between the file checks and the process list. Then one flag per step, only the axis that moved: a walk east from 58 shows `gr.x=58.5`, `59`, `59.5` ... one message per 50 ms, in the same frame as the walk props and ahead of them. |
| 2.31 | never | no `gr` flag in any 2.31 run |

The value is the position in tiles as text: a whole number has no decimals (`30`), a
half has `.5` (`30.5`). On a gmap it is the position in the whole map
(`cell * 64 + local`), so it runs on across a crossing: in the 2.22 `gmap_cross`
capture it reads `63.5`, `64`, `64.5`, `65` while the walk props go from `X=63.5` to
`X=0.5`.

## What Seihai does

The profile (`protocol.gr_flags`, set on the 2.17 and 2.22 steps of the version chain)
says which builds send them: `LoginOnly` for 2.17, `Tracking` for 2.22, none otherwise.

* The post-login frame holds both flags, between the file checks and the process list
  (`protocol/login.rs`).
* A tracking build puts the flags that changed ahead of each walk update, in the same
  frame (`Connection::send_walk_sprite`, and the crossing frame, where they come before
  the props message that is last).
* On a gmap the connection knows the player's cell, so the flags hold the whole-map
  position.
* `protocol/grflags.rs` keeps what was last sent and returns only the differences.

## Before and after (lab, 2.22, real client against Seihai)

| run | `FLAGSET` real | Seihai before | Seihai after |
|---|---|---|---|
| `login` | 2 | 0 | 2 (`gr.x=...`, `gr.y=...`, same text) |
| `gmap_cross` | 448 | 0 | 348 |

The `gmap_cross` count differs because Seihai walks slower (divergence 9, about 7.8
tiles per second against 10), so it takes fewer steps in the same time; the flag per
step matches. The ranked report no longer lists `FLAGSET` as a missing id for 2.22.

## Not known

* After a server warp or a level warp the 2.22 client's next tick sets the axes that
  moved. That is how Seihai behaves, from the per-step rule; a warp was not examined
  flag by flag.
* A 2.17 walk was not captured; Seihai sets 2.17's flags only at login.
