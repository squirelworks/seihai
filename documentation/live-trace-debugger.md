# Live trace debugger

A window that shows the real client's and Seihai's state side by side while
both are running against the same lab server. The goal is to see a divergence
the moment it happens, not after the capture is decoded.

## What it shows

Two columns. Left is the real client (Wine). Right is Seihai. Each row is a
state that both sides can report:

| row | real client | Seihai |
|---|---|---|
| level | from the window title or a log line | `level_name()` |
| position | from the HUD or a prop frame | `player_x`, `player_y` |
| GANI | from the animation name in the log | current pose |
| head / body | from the prop frame | current images |
| carrying | from the carry prop | carried object id |
| sign | from the sign panel state | `reading` |
| last packet | from the capture | from `_debug/packets.log` |

A row turns red when the two sides differ. The window updates every 200 ms.

## How it gets the real client's state

The real client does not expose an API. The lab already captures the wire.
The debugger reads the capture file as it grows, decodes the frames with
Seihai's own codecs, and extracts the state from the prop and level frames.
This is the same decoder the differential testing lab uses.

Seihai already writes its state to `_debug/`. The debugger reads that log.

## What it does not do

- It does not inject into the real client.
- It does not modify the wire.
- It does not save the password. The capture is masked.

## Where it lives

`tools/difftest/live-trace.rs` is a new binary. It takes the capture path and
the Seihai debug directory. It opens a minifb window and draws the table.

## Order

1. Read the growing capture and decode the prop frames.
2. Read Seihai's `_debug/` log.
3. Draw the table.
4. Highlight the rows that differ.

The first three can share the decoder from `seihai::difftest`. The last one
is new.
