# NPC props from a real server: props 43 and 77 (lab, 2026-10-31)

#319 taught `parse_npc_props` the NPC props past 43 (GANI attributes 6 to 30,
CURLEVEL, X2/Y2) but kept 43 and 77 as unknown props that end the record. A
lab run shows that GServer-v2 beta4 sends both on every NPC record to a 2.22 or
2.31 session, so the reader still stopped before attribute 6.

## Setup

Cloud container only: Wine 9 and Xvfb in a loopback-only namespace, the
`seihai-lab-servers-beta4` pack (GServer-v2 beta4 with the lab patches). A test
level (onlinestartlocal plus two NPCs) where a real `Graal_222.exe` runs
`setcharprop #P1,one` and `setcharprop #P6,six` on a `showcharacter` NPC. The
real 2.22 sends C>S `NPCPROPS` with 36 `one` and 44 `six` (it has no `#P10`
or `#P30`). Seihai 2.22 and 2.31 joined the level, and the server's S>C 3 to
them was captured.

## What the server sends

The end of the character NPC's record (prop id, then value):

```
36 "one"   43 ff   44 "six"   75 26 60 (X2 26.0)   76 26 60 (Y2 26.0)   77 2c 81
```

- **43**: one byte, `ff` on every record seen (both NPCs, both sessions).
  GServer-v2 names it `UNKNOWN43`.
- **77**: two bytes after X2/Y2, the `zprecise` name in beta4's prop list (Z2).

## What changed

`parse_npc_props` skips 43 as one byte and 77 as a `GSHORT`. Reading now stops
only at 48 and at anything past 77. The test
`lab_beta4_record_keeps_attribute_6_past_prop_43` parses the captured record:
attributes 1 and 6 are `one` and `six`, and X2/Y2 give 26, 26, the same as the
classic X/Y. Before the change, the same bytes gave only attribute 1.

## Still open

Seihai's GS1 `#P1` to `#P30` read only what the NPC's own script set with
`setcharprop` (`interp.rs`). The attributes from the server go to the level
NPC's props, which are used for drawing. So a script that reads `#P6` still sees
an empty string on another player's attribute. In the lab, `message seen #P6;`
showed `seen ` before and after this change.
