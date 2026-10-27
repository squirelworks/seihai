# Claude Code laundry list — Seihai testing and Graal_analysis

Do not wait for a merge. Open the PR for one item, then start the next on the same branch or a new one. The reviewer catches up. Work offline and against the lab first. Do not touch a public server unless the item says so and the user has approved that run.

Password never goes on a command line. Use `--password-env` or the settings file. Do not put account names, passwords, hosts or IPs in notes that go to Graal_analysis.

## How to check Seihai is right

For each item, prefer this order:

1. Read the real client's behaviour from a lab capture or from the cleaned notes in Graal_analysis. Do not invent bytes.
2. Write the expected rule in a short note under `documentation/` in Seihai, or under `client-internals/` in Graal_analysis if it is a new fact.
3. Add a test that fails before the change and passes after.
4. Run the same action in Seihai and, where possible, in the real Wine client on the lab. Diff the wire or the visible result.
5. Open a PR. List what is confirmed, what is inferred, and what was not tested.

## Immediate — broken or missing

### 1. Sword while carrying

Using the sword while holding an object is wrong. Find out what the real client does (2.05, 2.22, 2.31): does a swing drop the object, throw it, refuse the swing, or play a different animation? Capture it in the lab if the notes do not already say. Then make Seihai match. Today Seihai likely lets both happen or ignores one.

- Output: a note in `documentation/cut-and-lift.md` or `documentation/combat.md`, a test, and the fix.
- Graal_analysis: if the rule is new, a short note under `client-internals/` with no addresses or function names.

### 2. Inventory (Q)

Q opens the inventory in the real client. Seihai has no inventory. Find the real client's inventory screen: what it shows (items, weapons, rupees, hearts, keys), how it is opened and closed, and which packets it sends or receives. Then implement a first slice in Seihai: Q toggles an inventory panel that lists what the player is carrying and what is in the default weapon slots. It does not have to be pixel-perfect. It has to exist and not crash.

- Output: `documentation/inventory.md`, the panel, a test that Q toggles it offline and online.
- Graal_analysis: the key, the layout, and any packet ids the real client uses.

### 3. Carry pose and other players

The carried object is drawn as a cut-out, but the carrying pose was not read, and the lift prop is not sent to other players. Read whether the real client changes the body GANI while carrying, and which player prop carries the sprite id. Send that prop online so the other client can draw it.

- Output: note, send path, a loopback or lab check that the second player sees the object.

## Gameplay gaps already noted

These are listed as open in the existing docs. Confirm each against a capture or the decompile notes before coding.

### 4. Thrown object hits

A thrown object does not hit NPCs, players or baddies. The wall test may also miss extra object checks. Read the real client's flight hit rule and implement the ones that are confirmed.

### 5. Knockback and packet 40

A sword hit should push the victim. The notes say 5 tiles in the lab, but the exact rule was not fully read. Confirm and apply. Packet 40's per-player field is still open.

### 6. Bombs, arrows, baddies

Neither bombs nor arrows send hit packets; the victim's client judges. Blast shape and the arrow's last-byte rule were not found. Baddies were not tested. Capture or read, then implement the confirmed parts. Leave the rest marked open.

### 7. Corner nudge and flush snap

#81 added the real client's walking rule. Walk Seihai against a field of rocks and against a doorway the real client passes. If Seihai still sticks, capture the real client's position each step and diff.

### 8. Benches and beds

Standing on a chair tile draws `sit.gani`. Confirm against the real client. Beds and sleeping are not done.

### 9. Cut bush second effect

The second cut effect has no art in the built-in set. Either generate a small original effect or document that it stays blank.

## Protocol and login

### 10. List login matrix with the new frame

#79 made 2.00 and 2.05 send the real `newmain`-only frame and switched listserver frames to `78 DA`. The matrix has not been re-run. Run `VERSIONS="2.00 2.05 2.17" ./run_port_matrix.sh` from a machine that can reach the public listserver, with the user's approval. Record which ports answer. Update `port_status` from assumed to confirmed only where the new frame actually got a list.

### 11. Game-server login burst

The difftest leftovers still list missing login player props, file update (34) and process list (44). Capture a real 2.22 and 2.31 login and make Seihai send the same set, version-gated.

### 12. Gmaps

Ids 49, 185, 186 and cell props 43/44 are partly in. Walk a 2x2 gmap in the lab with 2.22 and 2.31 and confirm edge crossings, the 46/39 order, and that the neighbour draws. Diagonal corners are still refused.

### 13. Cached level links

#25 did the in-memory cache. It does not read the real client's disk cache, and gmap crossings are still open. Confirm with a lab capture.

## Assets and rendering

### 14. Default weapon images

#74 asks for ids 7, 12, 20–23 at login. Confirm the real client draws them when equipped, and that Seihai shows the image once the server sends it. Cover weapons that arrive after login.

### 15. NPC images and signs

NPCs still fall back to a green box when the image is missing. Signs have no art. Confirm the real client's draw path and request missing images the way the real client does.

### 16. HUD

Hearts, rupees, keys and the sword/shield icons. The real client draws these. Seihai's HUD is partial. Match the layout for 2.22 first.

### 17. Water

Swimming shows the head only in the real client. Confirm Seihai's water draw matches, including the splash when a thrown object lands in water.

## Script engine

### 18. Script-to-GANI

`setimg`, `showimg`, `hideimg` should change what is drawn. Cover one real NPC script that uses them.

### 19. Board and tiles reads

`board[]` and `tiles[]` currently return 0. Read the real client's meaning and return the tile id.

### 20. String lists, with, join, weapon-key input

These are listed as not done. Implement the ones a real script in the library uses.

### 21. Script warps online

`setlevel` / `setlevel2` from an NPC should go out as a level request. Confirm against the lab; the real client did not run the lab's scripts in an earlier attempt.

## Level editor

### 22. Next slice

The CLI can open, list, paint and save a `.nw`. Next: binary `.nw` if the format is known, then link/sign/chest editing, then a minimal GUI. Keep the version selector. 2.22 first.

## Graal_analysis — facts still open

Write these as cleaned notes. No decompiler names, addresses, hosts or passwords.

- Which build tries `.zelda` first for a bare name, and whether a missing `.zelda` falls back to `.graal`.
- The 2.00-era listserver reply from a real 2.00 client (the synthetic one is noted).
- Id 24 meaning beyond the weapon-image list.
- SERVERWARP field order from a real service (the lab text is from the open-source listserver).
- Whether any public server enforces the tamper-check value.
- The damage formula for bombs, arrows and baddies.
- Status bit 0x20.
- The respawn entry-save rule for 2.00–2.04.
- Per-build UDP receiver ids and the long-handler field layouts for 2.00–2.04.

## Order

1. Sword while carrying.
2. Inventory (Q).
3. Carry pose and other players.
4. The gameplay gaps (4–9) in any order.
5. Protocol and login (10–13).
6. Assets and rendering (14–17).
7. Script engine (18–21).
8. Level editor slice.
9. Graal_analysis facts as they come up.

Open the PR, then start the next. Do not wait.
