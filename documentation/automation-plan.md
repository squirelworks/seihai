# Automating client learning

The differential testing lab already runs the real Wine client and Seihai
through the same scripted sessions and diffs the wire. The gap is that the
scenarios are hand-written and the divergences are hand-fixed. The next step
is to generate the scenarios from captures and to turn the diffs into
patches.

## What already works

- `tools/difftest/lab.sh` runs a scenario against both clients on a private
  namespace, captures both, and diffs them.
- Scenarios exist for login, chat, unstick, gmap crossing, door warp, and
  link geometry.
- The decoder handles gen-3, gen-5, and plain zlib. Masking strips accounts
  and passwords.
- The differ reports missing/extra ids, field mismatches, and timing.

## What to add

### 1. Capture-driven scenarios

A new tool, `seihai capture to-scenario FILE`, reads a pcap of a real client
session and emits a `.steps` file. It records:

- the login version and token
- every level change and the arrival position
- every chat line
- every key press that produces a walk or action frame
- the times between them

The lab runner then replays that `.steps` against Seihai and diffs. This
turns a single real-client capture into a regression test.

### 2. Prop and effect oracle

The lab already captures the wire. Add a second oracle that reads the
client's own log (or a hook) for:

- the current GANI name and step
- the current head, body, shield, and colours
- the current weapon and carried object
- whether a sign is open and which page

Seihai already writes this to `_debug/`. The differ compares the two logs
at each frame. This catches the appearance commands and the carry duck
without waiting for a wire difference.

### 3. Asset request log

The real client writes a file-request log (or the capture shows WANTFILE).
Seihai writes its own. Diff the two sets. Missing requests are the green
boxes. Extra requests are the cache misses.

### 4. Automatic patch suggestion

When the differ finds a field mismatch on a known layout, it can emit a
candidate patch. For example:

- "Seihai sends id 46 before the login burst; the real client sends it
  after." → move the send.
- "Seihai does not send gr.x/gr.y on walk; the real client does." → add
  the flags.

The patch is a comment in the PR, not an automatic commit. A human still
reviews it.

### 5. Continuous lab

A nightly job runs the scenario set against the current main and the real
client. It posts the diff summary to the Seihai issue tracker. New
divergences are new issues. Fixed ones are closed.

## Order

1. `capture to-scenario` — turns existing captures into tests.
2. Prop/effect oracle — catches appearance and carry without wire diffs.
3. Asset request diff — finds the missing images.
4. Patch suggestion — turns diffs into PRs.
5. Nightly lab — keeps the regressions from coming back.

The first three can run on the existing lab. The last two need a machine
that can stay up.
