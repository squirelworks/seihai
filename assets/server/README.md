# Seihai server art pack

Everything a game server hands to a real Graal client so it can draw: the
tile sheet, the player sheets, the HUD, the ganis, the sounds, the base
update package and the fonts. Real 4.x to 6.x builds bundle none of these
and ask the server for them at login; without them a real 5.24r4 or 6.03r7
draws a black screen. Older 2.x builds ship their own, but take a server's
copy over it.

All the art is Seihai's own (the squirrel), drawn and synthesised by
`tools/gen_default_assets.py` and `tools/gen_server_assets.py`. No file of a
Graal client or world was read to make it. The fonts are Liberation Sans
under the client's font names (see `fonts/README.md`). The licence is the
repository's (`MIT OR Apache-2.0`), except the fonts (SIL OFL 1.1).

## Using it

Copy the whole folder into the world. Both servers find a file by its name
in any subfolder:

- **seihai-gserver:** `<world>/files/` (for example `<world>/files/server-art/`).
- **seihai-gserver-v2:** a folder its `foldersconfig.txt` serves to clients,
  for example `bin/servers/<name>/world/server-art/` with a `file
  server-art/*` line (or the usual per-kind lines for `.png`, `.gani`,
  `.wav`, `.ttf`, `.gupd`).

A world's own file of the same name wins if both are present, so a world
can override any part of the pack.

For a 4.x to 6.x login, serve the pack on the **login server too**, and let
its first file requests finish before the server warp: the clients ask for
`pics1.png` only once, on the login server (Graal_analysis
`client-internals/v5v6-expiry-and-black-screen.md`).

## Contents

| Folder | Files |
|---|---|
| `images/` | `pics1.png` (2048x512, the standard tile layout), `sprites.png`, `state.png`, `chest.png`, `wbowi1.png`, `wbomb1.png` (+ `.gif` copies), `tiledefs.txt` |
| `bodies/` | `body.png`, `body2.png`, `body3.png` (+ `.gif`) |
| `heads/` | `head0.png` to `head3.png` (+ `.gif`) |
| `swords/` | `sword1.png` to `sword4.png` (+ `.gif`), one per sword power |
| `shields/` | `shield1.png` to `shield3.png` (+ `.gif`), one per shield power |
| `ganis/` | `idle`, `walk`, `swim`, `sword`, `sit`, `sleep`, `hurt`, `dead`, `carry`, `carrystill`, `push`, `pull`, `grab`, `lift`, `spin`, `hidden`, `hiddenstill` |
| `sounds/` | `steps`, `steps2`, `sword`, `water`, `water2`, `crush`, `lift`, `lift2`, `put`, `item`, `item2`, `extra`, `chest`, `stonemove`, `jump`, `arrow`, `arrowon`, `bomb`, `fire`, `thunder`, `frog`, `swordon`, `axe`, `dead`, `compudead`, `sign`, `nextpage` |
| `packages/` | `basepackage.gupd`: a minimal `GRPKG001` (`NAME`, `VERSION`, `PLATFORM any`, `MODE required`), no `FILE` lines |
| `fonts/` | `arial`, `arialbd`, `helvetica`, `helveticabd`, `tempsitc`, `tempsitcbd` (`.ttf`) |

The names come from the files the lab captures show 2.x to 6.x clients
asking for (Graal_analysis `protocol/v5v6-lab-play.md`,
`client-internals/hud-603r7-black-box.md`,
`version-profiles/lab-mac-tiger-2026-10-31.md`). Sizes follow the real
sheets (`documentation/asset-reference.md`). The ganis past the default set
are built from the default body poses; their steps and timing are Seihai's
own, not measured from the stock files.

## Not covered

- GUI images the 6.x login scripts ask for (`guigames_arcmessage2.png`,
  `defaulttextedit.png`, `defaultbutton.png`, `playerlist_icons.png`): they
  belong to a world's GUI scripts, and a miss only drops that widget.
- The 2.x baddy sheets, `letters.png`, `bomb1.png`/`bomb2.png`, maps and
  `tileobjects/`.
- Which `.gupd` `FILE` lines a client acts on is still open
  (Graal_analysis `protocol/update-packages-focus-status.md`), so the
  package lists none.

## Regenerating

```
python3 tools/gen_server_assets.py            # writes assets/server/
python3 tools/gen_server_assets.py --check    # exit 1 if it differs
```

`cargo test -p seihai-mae --test server_assets` loads every file with
Seihai's readers.
