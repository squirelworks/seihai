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
- **two more real scripts, unchanged** (laundry item 10, 2026-10-30), from the Graal_analysis NPC library: the **Hammer** weapon (`npc-0565`) fired facing down puts `wham2.gif` up over the player at `(playerx+0.6, playery-0.4)` on its first 0.05 s step, keeps it up through step 9 and takes it down with `hideimg 1` at step 10; fired facing right it reuses index 1 for `wham3.gif` at `(playerx+1.5, playery-0.2)`. The **rat** of `prtrain.graal` (`npc-0719`) `setgif`s `ratf.gif` when touched (and says sign 1) and `setgif`s `ratb.gif` back when the sign closes. Both are checked in the drawn pixels. They passed with no engine change: the hooks were already right, and these tests pin them to real scripts;
- `setcharani` switches an NPC between `idle` and `walk` with Seihai's own default GANIs: the 8 walk steps are 8 different pictures, they advance every 100 ms and loop, and idle is another picture; an animation that is not loaded falls back to the image; `setimg x.gani`; a non-looping GANI plays once and follows `SETBACKTO`.
- **Smoke test against real scripts** (`#[ignore]`, runs only with `SEIHAI_SCRIPT_CORPUS=<file>`, a text file of one script per line): it runs every script that uses `setimg`, `setgif`, `showimg`, `hideimg`, `setcharani`, `setani`, `showcharacter` or `changeimgpart` as an NPC under the 2.05 and 2.22 vocabularies (created, `playerenters`, a touch, three seconds of ticks, a few chat lines), checks the invariants of the state left behind (image indexes unique and sorted, parts inside the clamps), draws the result, and counts. Nothing of the scripts is printed or stored. On the author's private collection (a 9,516-line file, scripts and blank separators): 183 scripts use these commands, 366 runs, 0 panics, 234 runs changed an NPC image, 22 ended with the NPC hidden; no run left a `showimg` image up and none animated, because the 6 scripts with `showimg` are weapon scripts that fire on weapon events, and none of the collection uses `setcharani` or `setani`. The smoke test now also runs every script that has both `weaponfired` and `showimg` as a **weapon** (fired, then three seconds of ticks) and checks the player's image list. On the Graal_analysis NPC library (`npcs.json`, the same 183 scripts; one script per line): 12 weapon runs, an image went up in 4 (Boomerang and Hammer, under 2.05 and 2.22) and was down again in all 4. The other four need state set elsewhere first (the Joystick's `playing`, the Sticky Bow's `hasstickybow`, the Magic Cape's magic points), or are a single line in the flattened file whose leading `//` comment swallows the whole script (the Cane of Brugara).

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

## Images past the first four requests (laundry list item 14, 2026-10-31)

NPCs still showed nothing on levels that needed many images. The cause was in
the request lists (`seihai-core/src/filecache.rs`, `FileRequester`), not in
the draw path or the cache.

* **What went wrong.** Four `WANTFILE` (and four `UPDATEFILE`) go out at once;
  further names wait and go out one by one as replies arrive
  (`file-cache.md`, "How many at once?"). The lists kept each name in lower
  case, so a waiting name went out lower-cased: a level's fifth image
  `Block.PNG` was asked for as `block.png`. The lab's older GServer-v2 answers
  a name in the wrong case with `PLO_FILESENDFAILED`, the name is then settled
  for the connection, and the NPC draws nothing. The first four went out in
  the right case, which is why some NPCs showed and others did not.
* **The rule now.** A name goes out in the case the level, the script or the
  server gave it, whether it goes out at once or after waiting. Names are
  still compared without regard to case (a second ask in another case is not
  repeated; a reply in any case settles the name).
* **The green box.** The current draw path has no placeholder: an NPC whose
  image is not loaded draws nothing (section *An NPC with no picture draws
  nothing* above, and the `npc_images` tests). A box seen in an older build
  was the removed board-view square.
* **Cache and resolver.** Re-checked with the existing
  `a_downloaded_image_is_found_where_it_was_saved` test (14 kinds of name, four
  save layouts): the store writes where the resolver and the store's own
  lookup find the file. No change there.

Confirmed: the lower-casing, by a test that failed before the change
(`a_waiting_name_goes_out_as_the_server_named_it`). Inferred: that the real
client sends waiting names in their own case (its list code keeps the name
strings; the case of a waiting name was not re-read in the decompile).
Not tested: a lab run with more than four missing NPC images.

## Animated `.mng` images (2026-10-31)

The light fixtures on Graal the Adventure did not show. The lamps by the
stairs next to Cat's Pub are image NPCs whose picture is `bluelampani.mng`
(the 2.22 folder ships it in `levels/images`): a 32 x 64 lamp with three
frames, the yellow light growing, stored with delays of 100, 0 and 0 ticks
and `TERM` repeat for ever. Seihai had no MNG reader, so the file was asked
for, arrived, failed to read ("not a PNG") and the NPC drew nothing. The
real 2.22 client draws the lamp and flickers it: its two screenshots from the
2gta run (Graal_analysis `live-runs/2gta-222-2026-10-30`, `gta-start.png`
and `walk1.png`) catch it on the dim first frame and on the bright last one,
both lamps on the same frame.

* **Reading.** `assets::mng` reads MNG the way the client does (next
  section). `read_png_rgba`
  hands an MNG to it and gets the first frame, so heads, bodies, emoticons,
  item icons and NPC images all show an `.mng` now. A file's claimed size is
  checked like a PNG's (`MAX_IMAGE_PIXELS`), and at most 256 frames are kept.
* **Playing.** `assets::ImageAnimations` keeps the MNGs found in the
  client's image map (`npc_images`) and, before each draw, writes the frame
  due into the map's entry, then asks for a redraw when it changed. So the
  renderer is unchanged, and everything that draws from that map animates:
  NPC images, `showimg` images, weapon images, and heads such as
  `head11.mng` (a blink). A recoloured body is stored under another key and
  stays on its first frame. All animations share one clock from the moment
  they are first seen, so lamps with the same file flicker together.
* **Timing.** See *Graal's MNG is not standard MNG* below: frames count
  down in 50 ms game ticks, so the lamp's 0-delay frames show for 50 ms.
* **Not read.** `LOOP`/`ENDL`, delta images, `JNG`, `BASI`. None of the 28
  MNGs of the 2.22 folder uses them.

Tests: `assets::mng` unit tests (frames, delays, loops, mode 4 and `DEFI`,
the image map swap; `real_bluelampani` is `#[ignore]` and reads the lamp
from `SEIHAI_DATA_222`), and `npc_images::mng_npc_draws_and_animates`, which
draws an NPC with a gif2mng-style lamp through `draw_npcs` and checks the
colour change on the MNG's clock.

Lab check (GServer-v2 beta4 on loopback, 2.22): `onlinestartlocal.nw` with
two `NPC bluelampani.mng` lines and the file in `world/images`. Seihai logged
`requesting bluelampani.mng`, then `image bluelampani.mng`; screenshots 50
ms apart showed three different pictures in turn, both lamps on the same
frame.

## Graal's MNG is not standard MNG (2026-10-31)

Every client from 1.34 to 2.22 (1.34, 1.38, 1.39, 2.05 rev 2 and 2.22
checked: the same `TMNGAnimation` unit, the same step and tick code) and
`gif2mng.exe` read `.mng` with Graal's own loader, not with libmng. Read
from `Graal_222.exe` (the loader, the step to the next frame, the frame
draw, and the game-tick handler that counts frames down) and from
`gif2mng.exe` (the same loader). `assets::mng` now follows it; the first version (PR #322) read the files as standard MNG.

| | Standard MNG | Graal |
|---|---|---|
| Chunks | many | `MHDR`, `FRAM`, `TERM`, `IHDR`, `PLTE`, `tRNS`, `IDAT`, `IEND`, `MEND`; names compared in upper case; `DEFI`, `BACK` and the rest skipped |
| Frame images | any PNG | colour type 3, bit depth 4 or 8 only; anything else and the file does not load ("MNG loading failed!"); no `MHDR`: 100 ticks a second |
| Placement | `DEFI`, framing modes lay images over each other or clear | every frame at (0, 0); its palette indices replace the canvas's; the framing mode only changes the timing |
| Transparency | alpha per palette entry | one index: the first `tRNS` entry that is 0; partial alpha is opaque; a frame's own `tRNS`, else one outside any frame, else none |
| Palette | | a frame's own `PLTE`, else the file's (outside any frame, or the first frame's) |
| Delays | `FRAM` delay for the frame it starts | the loader's own sum (doc of `read_mng`): for gif2mng's files each frame gets the delay of the `FRAM` before it and the last one 1 tick more; then `max(1, centiseconds / 5)` 50 ms game ticks, so a 0 is 50 ms |
| `TERM` | action, delay, count | action not read; count N plays N times then stops on the last frame; 0, or above 2^31, or a short `TERM`: for ever; none: once |

What changed in Seihai: the lamp's frames are now 100, 50 and 50 ms (they
were 100, 100, 100); a 0 delay is one tick, every delay rounds down to
whole ticks (70 ms is 50); the last frame gets the lead tick; frames
replace each other; `DEFI` is ignored; partial alpha is opaque; a non
palette frame fails the file; a `TERM` count of 0 loops for ever.

Corpus: the MNGs of the 1.38, 1.39, 1.41.1, NW beta 1 and 2, 2.05 rev 2 and
2.22 archives in project files, 310 files, 82 different. All are gif2mng
output (palette 8 bit, `MHDR` 1000 ticks, simplicity 1, which claims no
features although they use `FRAM` and `TERM`); 10 carry a stray byte after
`MEND`. Seihai's frames and times for all 1,166 frames were equal to an
independent Python model of the client's loader (Pillow for the pixels).
The newer gif2mng (2004) writes `TERM` 0x7FFFFFFF and framing mode 3, so in
its files (`head11.mng`, `head21.mng`) the long delay falls on the last
frame: 50, 50, 50, 5500 ms.
