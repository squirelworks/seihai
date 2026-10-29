# Appearance: head, body and colours

The clothes-changer NPCs, the chat commands and prop 13 (COLORS), and what
Seihai does with them.

## What 2.05 does (derived from the decompile)

| Fact | Status |
|---|---|
| `sethead name`, `setbody name` and `setskincolor`, `setcoatcolor`, `setsleevecolor`, `setshoecolor`, `setbeltcolor` change the **local player**, whichever NPC or weapon runs them. The NPC itself is not changed (an NPC's own look is `setcharprop #3` and the like) | derived |
| The colour commands take a colour (a name of the 20-colour list, or a number), rounded, into colour slot 0 to 4: skin, coat, sleeves, shoes, belt | derived |
| A body picture is a palette image. Before it is drawn, palette entry 5 gets the skin colour, 6 the coat, 4 the sleeves, 3 the shoes, 1 the belt, and entry 0 always gets colour 18 (black, the outline) | derived |
| The 20 colours, as RGB: white 255,255,255; yellow 255,255,0; orange 255,173,107; pink 255,132,132; red 255,0,0; darkred 206,24,41; lightgreen 132,255,132; green 0,255,0; darkgreen 0,198,0; lightblue 132,132,255; blue 0,0,255; darkblue 0,0,198; brown 132,0,0; cynober 0,255,255; purple 255,0,255; darkpurple 132,0,132; lightgray 206,206,206; gray 132,132,132; black 0,0,8; transparent 0,0,0 | derived |
| A new player starts as orange, darkred, black, darkred, black, with `body.png` and `head0.gif` | derived |
| The stock bodies (2.22 files) follow that layout: entry 2 is the clear colour | checked on `body.png` and `body13.png` |

## The chat commands (GServer-v2 beta4)

Graal Reborn's server runs these itself from a chat line, and sends the new
props back to the player: `sethead name`, `setbody name` (it adds `.png`,
`.mng` or `.gif` when the file is found that way), and `setskin`, `setcoat`,
`setsleeves`, `setshoes`, `setbelt` with a colour name (`grey` is `gray`).
These are server commands, not client ones.

## What Seihai does

- The script commands set `OfflineSession::looks` (head, body, five
  colours). Online, Seihai then sends PLAYERPROPS with HEADGIF (11),
  BODYIMG (35) and COLORS (13), so the server and the other players see it.
- **Chat:** the GServer forms above, and the spaced forms `set head head927`
  and `set body body13`. They are checked after the `playerchats` scripts,
  the way `unstick me` is. Offline, Seihai applies them (adding `.png` when
  there is no extension). Online, the line goes out in the server's form
  (`set head head927` is sent as `sethead head927`); the server answers
  with the props.
- Own head, body and colours from the server's PLAYERPROPS are taken as
  our looks.
- **Drawing:** our own player is drawn with its head and body, the body in
  its colours. Other players' bodies are drawn in their prop 13 colours (the
  2.05 defaults until one arrives). A picture not here yet is asked for
  from the server and the default is drawn meanwhile.
- Seihai's own `body.png` was re-laid to the 2.05 palette layout, so the
  colours land on the right parts (`default-assets.md`).

## Not done

- Bodies drawn without a GANI (the plain fallback with no pose loaded) are
  not recoloured.
- Only PNG bodies and heads are drawn; a `.gif` or `.mng` name is asked
  for but not decoded.
- Not run against a server. The online send of props 11, 35 and 13 is
  inferred from the PLAYERPROPS layout, not from a capture of 2.05 sending
  it.
