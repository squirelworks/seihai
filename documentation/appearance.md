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
- **GANI attributes (hats).** A GANI's `ATTR1` … `ATTR5` sprites draw
  from the player's attributes 1 to 5, or from the GANI's `DEFAULTATTR1` …
  `DEFAULTATTR5` while the player has none. Attribute 1 is the hat and is
  looked for in `hats/`; the stock `idle.gani` names `hat0.png` as its
  default. An attribute set to nothing draws no picture. They come from
  props 37 to 41 (GATTRIB1 to 5, which 2.05 reads for other players), for
  us and for others, and from `setplayerprop #P1,<image>` … `#P5`. Online,
  the attributes we have set go out as props 37 and up with the looks.
- **Character NPCs** (`showcharacter`). `setcharprop` codes are drawn
  as 2.05 lists them: `#3` head, `#8` body, `#2` shield, `#1` sword,
  `#C0` … `#C4` colours (name or number) and `#P1` … `#P5` attributes.
  The body is recoloured in the NPC's colours. A colour the NPC has not set
  is the new-player one (`DEFAULT_COLORS`). A picture not here yet is
  asked for from the server, and the default is drawn meanwhile.
- **Checked in the Wine lab (2.05 rev 2, offline, 2026-10-30).** A level
  with three `showcharacter` NPCs, run in the real client and in Seihai:
  - No `setcharprop`: the body is orange skin, dark red coat and shoes,
    black sleeves and belt (pixels 255,173,107 / 206,24,41 / 0,0,8), the
    same as the player. So the new-player colours are right for characters.
  - `#3 head5.png`, `#8 body2.png`, `#C0` … `#C4` blue, green, red, yellow,
    white, `#P1 hat2.png`, `#2 shield2.png`: the real client draws that
    head and body, skin 0,0,255, coat 0,255,0, sleeves 255,0,0, shoes
    255,255,0, belt 255,255,255, and the hat over the head. Seihai draws
    the same, and the hat sits within a pixel of the real one against the
    body.
  - `#P1 hat1.png` alone: the hat is drawn on the default figure in both.
  - The real client also draws the shield (`#2`), from the `SHIELD`
    sprites of the stock `idle.gani`. Seihai's own `idle`, `walk` and
    `sword` GANIs now have them too, at the stock places against the head,
    step by step (measured on the 2.22 GANIs; behind the body, or in front
    facing down; to the side during the swing, none facing left or right).
    Sit, swim, hurt, carry and dead have none, as the stock ones. The lab
    picture of that NPC then matches the real one, shield included.
- **Per build.** GANI attributes exist from 2.02, the build that added
  props 37 to 41 (`chain.rs`; Graal_analysis `141-217.md`). For 2.00 and
  2.01 the `ATTR` sprites draw nothing, no attribute picture is asked for,
  and `send_looks` leaves props 37 to 41 out (it sends only the attributes
  in the build's own prop set). GServer-v2 sends no props past 36 to a
  pre-2.1 client, so online on 2.02 to 2.05 other players' hats come only
  from a server that does.
- **Seihai's own GANIs** now place the hat (`ATTR1`, a 48x48 cell of a
  192x144 sheet) after every head, at the stock offset from the head for
  its facing: (-8, -18) up, (-12, -18) left, (-8, -18) down, (-4, -18)
  right. Those numbers were measured on the 2.22 player GANIs, where every
  facing-head row of idle, walk, sword, sit, swim and carry has them. Seihai
  ships no hat and no `DEFAULTATTR1`, so a player wears one only when
  prop 37 or `#P1` gives it.
- `setplayerprop` also takes the looks codes `#3` (head), `#8` (body) and
  `#C0` … `#C4` (colours, by name or number), the same as the commands.
- Seihai's own `body.png` was re-laid to the 2.05 palette layout, so the
  colours land on the right parts (`default-assets.md`).
- A `.gif` body takes the colours the same way: the same palette entries of
  the GIF's colour table are replaced before it is decoded
  (`read_gif_rgba_recolored`). The built-in `body.gif` has the PNG's palette,
  and both come out pixel-equal in any colours. That a real `.gif` body uses
  the same entries is inferred: no old body GIF was in the corpus to check.

## Not done

- Bodies drawn without a GANI (the plain fallback with no pose loaded) are
  not recoloured.
- `.mng` bodies and heads are asked for but not decoded. PNG and GIF are
  (GIF: the first image only).
- Attributes other than the hat are looked for where other images are;
  where 2.05 looks for them is not checked. Seihai's default GANIs have
  only the hat (`ATTR1`).
- Attributes 6 to 30 (2.14 and later: `#P1` … `#P9`, then props 54 to 74
  from 2.16) are not kept; only 1 to 5 are.
- The stock `dead.gani` puts a lying hat (cell 48) on a lying head; Seihai's
  own lying step keeps the upright head, so it keeps the upright hat.
- Not run against a server. The online send of props 11, 35 and 13 is
  inferred from the PLAYERPROPS layout, not from a capture of 2.05 sending
  it.
