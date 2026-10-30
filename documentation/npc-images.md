# NPC images, animations and `showimg` (item 23)

How a script changes what an NPC looks like, and what Seihai draws. Written for item 23; the code is `scripting/engine/host.rs` (the effects), `npcart.rs` (animation state), `render/npcs.rs` (drawing) and `client/tests/npc_images.rs` (tests).

The base of this branch is item 22 (`chiaki/gani-expansion`, PR #54), because animated NPCs are drawn with `render::draw_gani_frame` from there: **merge in order, #54 first.**

## What each command does

| command | source of the rule | what Seihai does |
|---|---|---|
| `setimg f`, `setgif f` | `gs1-command-reference-205`: "sets the NPC image, shows the NPC, resets the visible part to the whole image" | the NPC's image becomes `f`, it is shown, its part is cleared, and it leaves any animation |
| `setimgpart f,x,y,w,h` | same: "shows only (x, y, w, h)", x and y clamped to 0..16000, w and h to 0..220, then trimmed to the image | as `setimg`, and the NPC draws only that part (`Npc::image_part`, trimmed to the image size when drawn) |
| `hide`, `show` | same: hide and show the NPC | `Npc::visible`; a hidden NPC draws nothing of its own (this already worked; now the renderer is tested) |
| `showimg i,f,x,y` | same: "image *index* at (x, y) ... re-using an index replaces it; the same file only moves" | `Npc::images`, by index (sorted). A used index keeps its entry: the file and place change, the part is cleared and the height goes back to one over the NPC, as the 2.05 code does; colour and zoom stay |
| `changeimgpart i,x,y,w,h` | same | sets the part of image `i` (clamps as above) |
| `hideimg i` | same: "deletes image *index*" | removes image `i` |
| `setcharani name,`, `setani name,` | the command table lists both | the NPC is drawn as that GANI (`Npc::ani`, lower case, no `.gani`) |
| `showcharacter` | the command table | the NPC is drawn as a character, as `idle`, unless it already has an animation |

**The plan text said `showimg` / `hideimg` toggle the NPC's visible flag.** That is not what the command reference says: they put up and delete numbered images, and `hide` / `show` are the visibility commands. Seihai follows the reference: `showimg` / `hideimg` change the NPC's image list, the renderer draws that list, and an image stays up when its NPC is hidden. The old `OfflineSession::handle_command`, which nothing calls, mapped `showimg` / `hideimg` to "show / hide every NPC"; that mapping is gone (2026-10-30), so it handles only `hide` / `show` there.

**Weapons (2026-10-30).** The 2.05 reference says weapon NPCs act as the player: "commands that spawn things (... `showimg` layers ...) use the player's level", and `showimg` puts the image "on the NPC's layer +1 (player's for weapons)" (Graal_analysis `npc-library/gs1-command-reference-205.md`). So a weapon's `showimg` goes into `OfflineSession::player_images`, at height 1 (one over the player), and `hideimg`, `changeimgpart`, `changeimgvis`, `changeimgcolors` and `changeimgzoom` from a weapon act on that list. The renderer draws it with the NPCs' images, the host loads its files with the NPC image pass, and the list is cleared when a level is loaded, since the images belong to the level the player was on. Whether the real client keeps a weapon's images across a level change was not checked; clearing them is Seihai's reading of "the player's level".

## Seihai's own conventions (not documented real-client behaviour)

- **`setimg name.gani`** picks that animation for the NPC instead of an image. Nothing found says the real command does this; it is the plain reading of "setimg changes the NPC's GANI" and costs nothing, since a `.gani` file is not an image. A plain `setimg` afterwards ends the animation.
- **Animation timing:** one step every 100 ms. A `LOOP` animation wraps; any other stays on its last step unless it has `SETBACKTO name`, which then switches the NPC to that animation. The real timing (and what `WAIT` means) is not documented here.
- **Facing:** animated NPCs face down; they have no direction of their own yet.
- **Position of `showimg`:** x and y are level positions in tiles, the unit of an NPC's own x and y. Evidence in the private script collection (derived facts only, no text kept): the `showimg` calls in it are all in weapon scripts and put images at `playerx` and `playery` plus a small offset, which fits tiles; a few HUD scripts use fixed small numbers, which may mean the "screen" image layer (layer 4 in the reference); screen-layer images are not implemented.
- **Layer:** each image has its own height, set by `showimg` and `changeimgvis` (see `npc-draw-order.md`), and the screen layer is drawn over the status bar at screen pixels.
- **Missing picture:** a name that is not loaded draws nothing, and the NPC's image falls back from an animation whose GANI or sheets are not loaded.

## An NPC with no picture draws nothing

The board view used to put a 12×12 gray square under every NPC that had no animation, so an NPC whose image had not arrived (or never would) showed as a box. The 2.05 client's in-game NPC draw has no such placeholder:

- an NPC with no image loaded draws nothing of its own (only its text, if it has any);
- an NPC with a GANI draws the GANI, and a character NPC uses the player's draw;
- the placeholder picture the client has for an NPC with no image is used only in the level editor's view.

So the square is gone: `render_view` draws chests and then nothing for NPCs, and `draw_npcs` draws the image once it is loaded. Requests are unchanged: the NPC image pass asks the server for each missing image when a level arrives, when a script changes a picture, and again after each file arrives. Confirmed from the 2.05 draw path; how the real client batches its own image requests was not checked.

## What the host does

- `SessionScripts::images_changed` is set when a script changed an NPC picture (`setimg`, `showimg`, `hideimg`). `seihai` then runs its NPC image pass (the same one a level arrival uses: ask the server for a missing file, or read the file it has) so the new picture is loaded and drawn on the next frame. A level now runs that pass for the NPCs' images offline as well.
- `OfflineSession::wanted_npc_anis()` lists animation names that NPCs use and that are not loaded; `seihai` loads them with the resolver (the built-in `idle.gani` and `walk.gani` are found there). A name with no file gets an empty animation, so the NPC keeps its image and the lookup is not repeated.
- `tick_npc_anis(ms)` steps the animations on the script clock and asks for a redraw when a step changed.

## Not done

- `hideimgs` (a later command, not in the 2.05 table): recognized and ignored, since no source here says what range it takes.
- Layers other than "over", colours, zoom, the `@text` images, screen-layer images.
- Walk animations of the local player (item 22 left that open too).
- NPC directions. (`setcharprop` heads, bodies, colours and hats are drawn now; see `appearance.md`.)

## Tests

`client/tests/npc_images.rs` (and one ignored test). Each runs a real NPC script through `SessionScripts` and draws the level with `render::draw_npcs`:

- `setimg` changes the pixels drawn (and a picture that is not loaded draws nothing, then shows when it arrives);
- `setimgpart` draws the part and `setimg` resets it;
- `hide` / `show` toggle the drawn NPC;
- `showimg` / `hideimg`: images appear at their level position, an index in use is replaced, the same file only moves, `hideimg` deletes, a hidden NPC keeps its images;
- a **weapon**'s `showimg` is drawn over the player with its part, not as an NPC's image; `hideimg` from the weapon takes it down; a level load clears it;
- **a real NPC script, unchanged** (The Mollusk's chest, `quest_boom2.nw` NPC 1 in `tests/fixtures/npc-scripts/02_phoenix.npc.txt`): `playerenters` `setimg`s the closed chest, a touch `setimg`s the open one and `showimg 300` puts the key up at `x+0.5, y-1`, and after its `sleep 3` `hideimg 300` takes the key down. Each step is checked in the drawn pixels;
- `setcharani` switches an NPC between `idle` and `walk` with Seihai's own default GANIs: the 8 walk steps are 8 different pictures, they advance every 100 ms and loop, and idle is another picture; an animation that is not loaded falls back to the image; `setimg x.gani`; a non-looping GANI plays once and follows `SETBACKTO`.
- **Smoke test against real scripts** (`#[ignore]`, runs only with `SEIHAI_SCRIPT_CORPUS=<file>`, a text file of one script per line): it runs every script that uses `setimg`, `setgif`, `showimg`, `hideimg`, `setcharani`, `setani`, `showcharacter` or `changeimgpart` as an NPC under the 2.05 and 2.22 vocabularies (created, `playerenters`, a touch, three seconds of ticks, a few chat lines), checks the invariants of the state left behind (image indexes unique and sorted, parts inside the clamps), draws the result, and counts. Nothing of the scripts is printed or stored. On the author's private collection (a 9,516-line file, scripts and blank separators): 183 scripts use these commands, 366 runs, 0 panics, 234 runs changed an NPC image, 22 ended with the NPC hidden; no run left a `showimg` image up and none animated, because the 6 scripts with `showimg` are weapon scripts that fire on weapon events, and none of the collection uses `setcharani` or `setani`.

## Images that did not show (laundry list P6, 2026-10-29)

* **GIF.** The image reader took PNG only, so a server's `.gif` (old worlds name
  their images `.gif`, and a pre-2.1 server asks for `.gif` by default) was
  saved and then never drawn. `assets::gif` now reads the first image of a
  GIF87a or GIF89a file (global or local colour table, interlace, the
  transparent colour of a graphic control block), and `read_png_rgba` hands GIF
  data to it, so every image path takes both. Checked against a reference
  decoder on 43 files (40 from the 1.21 archive, 3 made with transparency and
  interlace): the same pixels in every file. Later frames of an animated GIF
  are not read.
* **Names in another case.** A file that arrives is kept under the name the
  server sent. An NPC that names it in another case now gets it too: the level's
  NPC names are looked up again (case-blind) when any image arrives. On the
  lab's older GServer-v2 a request in the wrong case gets `PLO_FILESENDFAILED`;
  beta4 looks names up case-blind and answers with the name as asked.
* **Where a download lands.** Checked by a test that stores 14 kinds of image
  name (heads, bodies, hats, bomys, emoticons, map pictures, `.mng`, a name with
  a folder, upper case) for 1.39, 2.05, 2.22 and 2.31: the resolver and the
  store's own lookup find each one at the path it was written to. No mismatch
  was found there.
* **NPCs that exist only on the server** (`PLO_NPCPROPS`, id 3): the image,
  script and position are read, and an NPC the level file does not have is
  added to the level, drawn and scripted (`ledges-push-pull.md`, *Online*).
  Props past image, script, position and the two flag bytes are not used.

Lab check (GServer-v2 at 127.0.0.1:14802, 2.22): a level with `NPC labgif.gif`
and a 64 x 48 interlaced GIF with transparency. Seihai asked for it, logged
`image labgif.gif`, and drew it at the NPC's place.
