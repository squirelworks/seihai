# Script-initiated warps: what is known and what is not

Item 2 of the overnight queue. The wiring is in `net::start_online_link` and
the main loop; the protocol side is in `documentation/protocol.md`, "Level
changes" and "Script-initiated warps". This note is the part that could not be
finished.

## Known

- `setlevel2 level,x,y` and `setlevel level` (position = the player's current
  one, including any `playerx`/`playery` the script set first) are in the
  2.00-era vocabulary and end the script run (Graal_analysis, GS1 command
  reference; Seihai's own command table agrees).
- Seihai sends the link request for them: `PLI_LEVELWARP` (0), or
  `PLI_LEVELWARPMOD` (30) for a cached level on profiles with cached links,
  plus what a link frame carries on that profile (2.22: the tamper check; 2.31:
  46 and the props with the cached form). This is the lab-observed form of a
  *link*, reused.
- Seihai lab run (private namespace `graallabdsw`, displays :41/:42, a copy of
  the difftest lab, placeholder account): two level NPCs, one in `lv_n.nw`
  (`NPC - 20 32`, `setshape 1,32,32`, `playertouchsme` -> `setlevel2
  lv_c.nw,10,20`) and one in `lv_c.nw` (`NPC - 6 21`, `playertouchsme` ->
  `playerx = 40; playery = 22; setlevel lv_n.nw`). Seihai 2.31 and 2.22 walked
  into them and the frames were as described in `protocol.md`; the server
  answered each with the new level, and the second warp used the cache.

## Not observed

- **What a real client sends for a script warp.** The real 2.22/2.31 clients
  never ran a level NPC script in this lab (the earlier lab batch saw the same
  on 2.22 and 2.31). Three attempts on 2.31 in this run: (1) an NPC with no
  image and a `playertouchsme` trigger, with the player walking through its
  rectangle (22 walk updates, no level request); (2) an NPC with an image
  and a `playerenters` event that calls `setlevel2`, the image missing on
  the server (the client asked for it and got `FILESENDFAILED`); (3) the same
  with the image on the server (the client asked for it and received it). In
  all three the client logged in, entered the level and sent no level
  request. The server does send the NPC (a `PLO_NPCPROPS` arrives right after
  the level name), so whether the client does not run the script, or runs it
  and refuses the warp, is
  unknown. No bytes are guessed here.
- Whether a real client sends 46 and the props with a script warp to a cached
  level the way it does for a link.
- What the real client does with a `setlevel2` to the level it is already in,
  or with a level the server does not have. Seihai sends the request for the
  first and lets the server answer; a warp request that gets no answer is
  given up after 4 s (as for links).
- A level name that carries a path or a missing extension. Seihai sends it as
  the script wrote it.

## Seihai's choices that are not captures

- A second script warp while one is pending is dropped.
- The plain walk update of the pass in which a script warp went out is left
  out, because the request carries the position.
- A profile that cannot send the request (id not in its send set) logs the
  warp and keeps the connection.

To close the gap: find a build and a script form that the real client does run
(for example a weapon script, or an NPC the lab sends in a different way), then
capture its warp and compare it with the frames above.
