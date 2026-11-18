# Fonts

The 4.x to 6.x clients ask the server for their fonts at login. If the server
does not have them, the client draws no text at all: no chat, no NPC messages
and no tab letters (Graal_analysis `client-internals/hud-603r7-black-box.md`).
The file names are the ones the lab captures show:

| File | Asked for by | What it is here |
|---|---|---|
| `arial.ttf`, `arialbd.ttf` | Windows 4.x to 6.x | Liberation Sans Regular / Bold |
| `helvetica.ttf`, `helveticabd.ttf` | Mac 4.x and 5.x | Liberation Sans Regular / Bold |
| `tempsitc.ttf`, `tempsitcbd.ttf` | 6.03r7 (its default font) | Liberation Sans Regular / Bold |

These are **not** the fonts the names belong to. They are unmodified copies of
Liberation Sans 2.1.5 (metric-compatible with Arial), stored under the file
names the clients request. Liberation Sans is under the SIL Open Font License
1.1 ([LICENSE-OFL.txt](LICENSE-OFL.txt)); the font's own family name inside the
file is unchanged.

Whether a client that loads a font by family name, rather than by file, picks
these up is a lab question; under Wine the system's own font substitution may
hide the difference.

Refresh with:

```
python3 tools/gen_server_assets.py --fonts /usr/share/fonts/truetype/liberation
```
