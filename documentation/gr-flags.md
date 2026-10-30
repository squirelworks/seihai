# The `gr.x` and `gr.y` flags, and script flags in general

## Where the flags come from

The lab captures of real 2.17 and 2.22 clients showed `PLI_FLAGSET` (18) messages
with `gr.x` and `gr.y`. These hold the player's position as text. Seihai used to send them as
a per-version client habit, through the profile switch `protocol.gr_flags`: once
at login for 2.17, and on every position change for 2.22. That model was wrong.

* GServer-v2 ships a weapon, `-gr_movement`, and its `defaultaccount.txt` gives it
  to every new account. The lab runs that server, so the lab accounts had the weapon
  (Graal_analysis `client-internals/reference-reimplementations/README.md`).
* The weapon is a clientside GS1 script. On `playerenters` and on every `timeout`,
  it runs `setstring gr.x,#v(int((playerx+0.25)*2)/2)` (and the same for `gr.y`),
  then sets `timeout=0.05`.
* The client sends `PLI_FLAGSET` because a script changed a player flag.

Lab check, 2026-10-30:

* Setup: real 2.22 (`Graal_222.exe`), stock GServer-v2 on loopback, the same walk
  (right, then down).
* Account with the weapon (the stock default account): 52 `FLAGSET` messages,
  `gr.x=30`, `gr.y=30.5`, then one per half-tile step.
* Account without the weapon (the `WEAPON -gr_movement` line removed from
  `defaultaccount.txt`, and the weapon file deleted): **no `FLAGSET` at all**.
  The client still sent 28 `PLAYERPROPS`, so it did walk.

On a server without that weapon, which includes the 2001-era official servers, a
real 2.05 or 2.22 client sends no `gr` flags.

## What Seihai does now

Seihai has no position-flag rule of its own any more. When a script changes a player
flag, the flag goes to the server, as the real clients do:

* `set name`, `setstring name,value` and the other string commands queue
  `PLI_FLAGSET` with `name` or `name=value`.
* `unset name` (and a `setstring` to an empty value) queues `PLI_FLAGDEL` (19).
* A flag is queued only when its value changes. The `-gr_movement` weapon sets
  both flags every 50 ms, yet only the axis that moved goes out, which is what the
  captures show.
* Flags that start with `local.` (2.02 and later; Graal_analysis
  `protocol/packet-layouts.md`) or `this.` stay in the client.
* Queued flags ride ahead of the next walk update, in the same frame. If no walk
  update takes them within a tick, they go out on their own. The ones queued
  before the post-login frame go in that frame, between the file checks and the
  process list.
* A version that cannot send id 18 or 19 sends nothing. The connection refuses
  ids outside the version's send set.

Lab check of Seihai 2.22, same server, same walk, 2026-10-30:

* With the weapon, Seihai sent `gr.x=30` and `gr.y=30.5` after login, then one
  flag per changed axis.
* Without the weapon, it sent no `FLAGSET`, against 55 `PLAYERPROPS`.

The profile field `protocol.gr_flags`, `protocol/grflags.rs` and its per-tick
tracker are gone. The flag messages are built in `protocol/flagset.rs`.

## Still different from the real client

* **One tick late.** The real 2.22 runs the weapon's timeout before it sends that
  tick's walk props, so `gr.x=30.5` rides with `X=30.5`. Seihai runs scripts after
  the walk send, so the flag goes one tick later, with the next update.
* **First flags.** The real client put the first `gr.x`/`gr.y` in the post-login
  frame. In the lab run, Seihai sent them a few frames later, because the weapon
  had not run yet when that frame was built.
* **Gmaps.** On a gmap, the real 2.22's flags ran on across a crossing (`63.5`,
  `64`, `64.5`), so that client's `playerx` in a weapon script is the whole-map
  position. Seihai's scripts see the position in the current cell. This was not
  changed here.
* **2.31** sent no `gr` flags with the weapon on its account, and the reason was
  not looked into. Seihai 2.31 runs the weapon like any other.
* **Before 2.02**, `local.` flags may have been sent. Seihai keeps them in the
  client for every version.
