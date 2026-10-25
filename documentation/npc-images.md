# NPC images, animations and `showimg` (item 23)

How a script changes what an NPC looks like, and what Seihai draws. Written for item 23; the code is `scripting/engine/host.rs` (the effects), `npcart.rs` (animation state), `render/npcs.rs` (drawing) and `client/tests/npc_images.rs` (tests).

The base of this branch is item 22 (`chiaki/gani-expansion`, PR #54), because animated NPCs are drawn with `render::draw_gani_frame` from there: **merge in order, #54 first.**

## What each command does

| command | source of the rule | what Seihai does |
|---|---|---|
| `setimg f`, `setgif f` | `gs1-command-reference-205`: "sets the NPC image, shows the NPC, resets the visible part to the whole image" | the NPC's image becomes `f`, it is shown, its part is cleared, and it leaves any animation |
| `setimgpart f,x,y,w,h` | same: "shows only (x, y, w, h)", x and y clamped to 0..16000, w and h to 0..220, then trimmed to the image | as `setimg`, and the NPC draws only that part (`Npc::image_part`, trimmed to the image size when drawn) |
| `hide`, `show` | same: hide and show the NPC | `Npc::visible`; a hidden NPC draws nothing of its own (this already worked; now the renderer is tested) |
| `showimg i,f,x,y` | same: "image *index* at (x, y) ... re-using an index replaces it; the same file only moves" | `Npc::images`, by index (sorted); a new file at a used index replaces the entry, the same file only moves it |
| `changeimgpart i,x,y,w,h` | same | sets the part of image `i` (clamps as above) |
| `hideimg i` | same: "deletes image *index*" | removes image `i` |
| `setcharani name,`, `setani name,` | the command table lists both | the NPC is drawn as that GANI (`Npc::ani`, lower case, no `.gani`) |
| `showcharacter` | the command table | the NPC is drawn as a character, as `idle`, unless it already has an animation |

**The plan text said `showimg` / `hideimg` toggle the NPC's visible flag.** That is not what the command reference says: they put up and delete numbered images, and `hide` / `show` are the visibility commands. Seihai follows the reference: `showimg` / `hideimg` change the NPC's image list, the renderer draws that list, and an image stays up when its NPC is hidden. (The old `OfflineSession::handle_command` still maps `hideimg` to "hide every NPC"; nothing calls it any more.)

## Seihai's own conventions (not documented real-client behaviour)

- **`setimg name.gani`** picks that animation for the NPC instead of an image. Nothing found says the real command does this; it is the plain reading of "setimg changes the NPC's GANI" and costs nothing, since a `.gani` file is not an image. A plain `setimg` afterwards ends the animation.
- **Animation timing:** one step every 100 ms. A `LOOP` animation wraps; any other stays on its last step unless it has `SETBACKTO name`, which then switches the NPC to that animation. The real timing (and what `WAIT` means) is not documented here.
- **Facing:** animated NPCs face down; they have no direction of their own yet.
- **Position of `showimg`:** x and y are level positions in tiles, the unit of an NPC's own x and y. Evidence in the private script collection (derived facts only, no text kept): the `showimg` calls in it are all in weapon scripts and put images at `playerx` and `playery` plus a small offset, which fits tiles; a few HUD scripts use fixed small numbers, which may mean the "screen" image layer (layer 4 in the reference); screen-layer images are not implemented.
- **Layer:** all shown images draw after every NPC. The reference gives layers 0 to 4 (`changeimgvis`); only "over" is drawn.
- **Missing picture:** a name that is not loaded draws nothing, and the NPC's image falls back from an animation whose GANI or sheets are not loaded.

## What the host does

- `SessionScripts::images_changed` is set when a script changed an NPC picture (`setimg`, `showimg`, `hideimg`). `seihai` then runs its NPC image pass (the same one a level arrival uses: ask the server for a missing file, or read the file it has) so the new picture is loaded and drawn on the next frame. A level now runs that pass for the NPCs' images offline as well.
- `OfflineSession::wanted_npc_anis()` lists animation names that NPCs use and that are not loaded; `seihai` loads them with the resolver (the built-in `idle.gani` and `walk.gani` are found there). A name with no file gets an empty animation, so the NPC keeps its image and the lookup is not repeated.
- `tick_npc_anis(ms)` steps the animations on the script clock and asks for a redraw when a step changed.

## Not done

- `showimg` in **weapon** scripts (all the `showimg` calls found in the private collection are in weapon scripts): the weapon object has no image list, so those calls are accepted and nothing is drawn.
- Layers other than "over", colours, zoom, the `@text` images, screen-layer images.
- Walk animations of the local player (item 22 left that open too).
- NPC directions, `setcharprop` heads and bodies.

## Tests

`client/tests/npc_images.rs`, 12 tests (and one ignored). Each runs a real NPC script through `SessionScripts` and draws the level with `render::draw_npcs`:

- `setimg` changes the pixels drawn (and a picture that is not loaded draws nothing, then shows when it arrives);
- `setimgpart` draws the part and `setimg` resets it;
- `hide` / `show` toggle the drawn NPC;
- `showimg` / `hideimg`: images appear at their level position, an index in use is replaced, the same file only moves, `hideimg` deletes, a hidden NPC keeps its images;
- `setcharani` switches an NPC between `idle` and `walk` with Seihai's own default GANIs: the 8 walk steps are 8 different pictures, they advance every 100 ms and loop, and idle is another picture; an animation that is not loaded falls back to the image; `setimg x.gani`; a non-looping GANI plays once and follows `SETBACKTO`.
- **Smoke test against real scripts** (`#[ignore]`, runs only with `SEIHAI_SCRIPT_CORPUS=<file>`, a text file of one script per line): it runs every script that uses `setimg`, `setgif`, `showimg`, `hideimg`, `setcharani`, `setani`, `showcharacter` or `changeimgpart` as an NPC under the 2.05 and 2.22 vocabularies (created, `playerenters`, a touch, three seconds of ticks, a few chat lines), checks the invariants of the state left behind (image indexes unique and sorted, parts inside the clamps), draws the result, and counts. Nothing of the scripts is printed or stored. On the author's private collection (a 9,516-line file, scripts and blank separators): 183 scripts use these commands, 366 runs, 0 panics, 234 runs changed an NPC image, 22 ended with the NPC hidden; no run left a `showimg` image up and none animated, because the 6 scripts with `showimg` are weapon scripts that fire on weapon events, and none of the collection uses `setcharani` or `setani`.
