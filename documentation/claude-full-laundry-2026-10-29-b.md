# Claude Full Laundry List — addendum 2026-10-29

From a live run on 2gta.home.eevul.net:14913. Level: usdarkdragonhouse2.nw.

## Priority 1 — First-load warps

Walking a link to a level that is not local lands at the link's `playerx`/`playery` instead of the link's new-x/new-y. The client applies the arrival before the board arrives, then the board overwrites the position. On a cached level the arrival is correct.

The log shows:
```
link to westocean_n-17.nw at 7.5 61
level westocean_n-17.nw not loaded (No such file or directory (os error 2)); requesting
level from the server's board
loaded westocean_n-17.nw
```
Then the player is at 7.5, 61 instead of the link's stated arrival.

Fix: hold the arrival until the board is applied, or re-apply the link's new-x/new-y after the board lands.

## Priority 2 — Asset cache

Downloads go to `~/.config/seihai/cache/<host>_<port>~<server>/`. They should also go to `_cache/<host>_<port>/` in the Seihai directory, so a fresh run does not re-request every image. The config cache is fine as a secondary; the game-dir cache is the one that should be checked first.

## Priority 3 — Appearance commands

The log is full of:
```
effect not applied: Command { obj: 199, name: "setbody", args: [Str("body13.png")] }
effect not applied: Command { obj: 199, name: "sethead", args: [Str("head927.png")] }
effect not applied: Command { obj: 199, name: "setskincolor", args: [Num(18.0)] }
effect not applied: Command { obj: 199, name: "setcoatcolor", args: [Num(15.0)] }
effect not applied: Command { obj: 199, name: "setsleevecolor", args: [Num(11.0)] }
effect not applied: Command { obj: 199, name: "setshoecolor", args: [Num(11.0)] }
effect not applied: Command { obj: 199, name: "setbeltcolor", args: [Num(18.0)] }
```
These are the clothes-changer NPCs. Apply them to the NPC and to the local player when the object is the player. Also apply them from chat (`set head head927`, `set body body13`) the way `unstick me` is applied.

## Priority 4 — Carry and signs

- **Duck while carrying.** Holding an object should let the player duck under it (hide the head). Seihai only throws.
- **Sign input.** While a sign is open, sword and movement should scroll the sign, not act. The A key is already reserved for the sign; S should be too.

## Priority 5 — Chat repeat

Holding a key should repeat the character. "gaaaaaaaaaaaaaaaaaaaay" should be typeable without releasing the key.

## Priority 6 — Pots and missing images

- Pots in the house are not lift-able. They are objects in the level; the lift table from arrays.dat should cover them.
- Images still fail to load (green box in the screenshot, missing NPC sprites). The request path is there; the cache and the resolver need to agree on where a downloaded file lands.

## Priority 7 — Big map neighbours

The big map (`bigmap2.txt`) is read. Neighbouring levels should be requested when the player is near an edge, so the crossing is seamless. The log shows the request only after the link is walked.

## What the log still shows

- `strcontains`, `onmapx`, `onmapy`, `lindexof` — unknown functions
- `play2` — unknown command
- `deletestring` — unknown command
- `sendrpgmessage` — unknown command
- Unmatched `}` and `else` without `if` in weapon scripts (parser leftovers from the 444)

These are not blocking. The appearance commands and the first-load warp are.
