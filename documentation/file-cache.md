# File transfer and caching rules (queue items 9 and 20)

What the real clients keep, what they ask the server for, and when a local
file counts as "up to date". The code is `client/src/filecache.rs`; the
client binary uses it through `FileSession` (`client/src/bin/main.rs`).

Sources: the 2.00, 2.05 rev 2, 2.17.1, 2.22 and 2.31 clients (read, not run
for this item), the lab captures of 2.17, 2.22 and 2.31 (2026-10-24, login and
gmap runs, decoded with `difftest trace`), and the file-transfer notes in
Graal_analysis. Where a rule was read in one build and the others were only
checked for the same code shape, the table says so. Nothing here came from a
new lab run, and no public server was touched.

## The rules

| Question | Rule | Evidence |
|---|---|---|
| What does the client ask for when it needs a file it does not have? | `PLI_WANTFILE` (23): the name only. | Code in every build; seen in the lab (missing `sword`, `shield`, `body` images, sounds, a gmap). |
| What when it has a copy? | `PLI_UPDATEFILE` (34): the copy's last-write time as a GINT5 (Unix seconds), then the name. The server answers with `PLO_FILE` (a newer copy), `PLO_FILEUPTODATE` (45) or `PLO_FILESENDFAILED` (30). | Seen in the lab: 2.17, 2.22 and 2.31 send it at login for the sword, shield and body images, and 2.22 for a `.gmap` it has. The time matches the file's mtime. |
| When is a copy "up to date"? | The client never decides. It sends its time and the server compares. The client treats a reply of "up to date" like any other reply: the name is settled. | Code and lab. |
| How often is a local file checked? | Once per name per connection. A name that any reply came back for is never asked about again in that session. | Code (the three name lists); the lab clients send each `UPDATEFILE` once. |
| Who asks for what? | Image and GANI loaders: a local file means use it and send `UPDATEFILE`; none means `WANTFILE`. Sounds: `WANTFILE` only when the file is missing (no `UPDATEFILE`). Levels: no file transfer in the real clients. They come as board packets, `LEVELMODTIME` and the weblevel cache. | Code of the loaders; the lab shows no `UPDATEFILE` for a sound. |
| How many at once? | Four of each kind on the wire. Further names wait in order. A reply for one of the first four releases the name now in fourth place. `WANTFILE` and `UPDATEFILE` have separate counts. | Code, identical in all five builds; seen in the lab (four and four before the first reply). |
| A name the server could not send | `PLO_FILESENDFAILED` settles the name. It is not asked for again on that connection. | Lab (2.22 asks for a missing sound once, however often it walks). |
| Builds without `PLI_UPDATEFILE` | 1.33 to 1.41 (profile `protocol.pli` has no 34) never ask whether a copy is current. | Profile table. |
| What does the server's file reply carry? | 2.1 and later: a GINT5 last-write time, then name length, name and data. Before 2.1: no time. Large files come in chunks behind `LARGEFILESTART`/`LARGEFILESIZE`/`LARGEFILEEND` (2.14+). | Code and `profile.files`. |
| Is a download written to disk? | Downloads stay in memory (cached stream) and are written only when `saveimages` is on in `graalconfig.txt` (default on). | Code in 2.05 rev 2, 2.17.1, 2.22 and 2.31. |
| Where is it written? | In place if the client already has a file with that name (found by the name as a path from the game folder, then through its file index). Otherwise: `.wav` to `sounds`; from the build with map folders: `.gmap`, and `.txt`/`.gif`/`.png`/`.mng` whose name contains `map`, to `maps`; everything else under `levels`, by name: `bomys` (`bomy…`, `b_…`), `ganis`, `emoticons`, `hats` (`hat` plus a digit), `heads`, `bodies`, `swords`, `shields`, else `webgifs`. | Code of the save routine in the four builds. 2.05 rev 2 has no `maps` rule. A downloaded 2.22 gmap was saved as `maps/<name>` in the lab. |
| What time does the saved file get? | The server's time (2.1+), so the next `UPDATEFILE` carries the time the server knows. Before 2.1 the time is the time of the download. | Code; lab (the saved gmap kept the server mtime). |
| Names | Compared without regard to case. A name cannot leave the game folder. | Code; Seihai refuses `..`, drive letters and absolute names. |

## Where Seihai keeps files: the per-server cache (item 20)

The real clients kept what a server sent them in a cache of their own, and
that is how copies of defunct playerworlds survived. Seihai does the same, and
keeps its own built-in default set out of it. A server's files are never
written into the user's asset folder and never into the built-in set.

### Layout

```text
<cache root>/                       default <settings dir>/cache  (~/.config/seihai/cache)
  play.example.org_14900~Maloria/   <host>_<port>[~<server name>]
    server.txt                      host=, port=, name=   (for people; not read back)
    levels/  levels/ganis/  levels/bodies/  levels/heads/  levels/swords/ ...
    sounds/
    maps/
<settings dir>/builtin/<id>/        the built-in default set, written 0444 from the binary
```

* The **cache root** is `--cache-dir <dir>`, else `cache_dir=` in the settings
  file, else `<settings dir>/cache`. It sits with the settings, not in a system
  cache folder, because what it holds (old copies of worlds) is not disposable.
* The **server folder** is named by the game server's host and port, made
  file-safe: lower case, only `a-z 0-9 . -`, every other character (an IPv6
  colon, a slash, a space) becomes `_`, a leading dot becomes `_`. So
  `Play.Example.ORG:14900` is `play.example.org_14900`. If the server's name is
  known (the launcher saved it, or a server warp named it) it is appended after
  `~`, letters and digits only, runs of anything else as one `-`, 32 characters
  at most: `play.example.org_14900~Maloria`.
* The name is **not** part of the identity. A folder whose name is the key, or
  the key followed by `~` and anything, is that server's folder. A server first
  reached without its name and later with it keeps one folder, and a renamed
  server keeps its cache. A new folder gets the name that is known when it is
  made. (`servercache.rs`; the tests cover the lookups.)
* Inside the folder the **real clients' save-folder rules are unchanged**
  (`download_dir`, the table above): sounds to `sounds`, maps to `maps`, levels
  to `levels`, everything else under `levels/<kind>`. `server.txt` is written
  once, when the first file is saved.
* A **server warp** switches to the new server's folder (the resolver's cache
  layer, the download store and the sound folders all change; the in-memory
  copies of the old server are dropped).

### Search order

A name is looked up in three layers, one after the other. Inside a layer the
folder order of its kind is the one in `documentation/assets.md`.

1. the **server's cache folder** (what this server sent),
2. the user's **asset folder** (`--assets` / `assets=`), if there is one,
3. the **built-in default set** (read-only).

The first layer that has the file wins, so on a server the server's art beats
the user's folder, and the user's folder beats ours. A missing or empty asset
folder, or one that lacks a file, simply falls through to the next layer.

Two guards on the built-in layer:

* **Built-in levels are offline only.** Online, `new1.nw` and `new2.nw` are not
  found in the built-in set: a server's own `new1.nw` must come from the server.
  (Tiles, sheets, GANIs and sounds stay available online.)
* **The built-in tile type table goes with the built-in sheet.** If the user's
  folder has its own `pics1.png` but no `tiledefs.txt`, our table is not applied
  to it.

### What a download does

* It is written to the **cache folder** only (`DownloadStore::store`). A copy of
  the same name already in the cache is replaced where it is. A copy in the
  user's folder or in the built-in set is **left alone**; the new file lands in
  the cache and is found first. A target under the built-in folder is refused
  (`PermissionDenied`), and with only the built-in set and no cache folder there
  is nowhere to save, so a download stays in memory.
* **`save_downloads` / `--no-save-downloads` still gate writing**, as before: off
  means kept in memory, nothing written. The one exception is unchanged: a level
  file (`.nw`, `.graal`, `.zelda`, `.gmap`) is always written, because Seihai
  loads levels from files. It goes to the cache folder.
* A resolver without a cache folder (a plain `--assets` run that is not online,
  and the library's `AssetResolver::new`) still saves in its one folder, as
  before item 20. A normal online run always has a cache folder.
* **Change from before item 20:** downloads used to go into the `--assets`
  folder, in place if the file was there. Now they go to the cache. A user who
  had files downloaded into their asset folder keeps using them (layer 2), but
  an updated copy from the server goes to the cache and wins from then on.

### Built-in files and UPDATEFILE / WANTFILE

Decision: **a file that only the built-in set has counts as missing when the
server is asked, and no mod time is ever sent for it.**

* `FileSession::need(name)` sends `WANTFILE` (23) for a name whose only copy is
  built in, `UPDATEFILE` (34) with the copy's mod time for a name the cache or
  the user's folder has, and nothing for a name the server already answered.
* Why `WANTFILE` and not `UPDATEFILE`: an `UPDATEFILE` carries "my copy has
  time T" and the server answers "up to date" when its file is not newer. A
  built-in file's time is whatever it was written at; it is not a copy of the
  server's file, so any time would be a claim that is false, and a server that
  has the file could answer "up to date" and Seihai would never receive it.
  `WANTFILE` is exactly what a client with a clean folder sends, which is also
  what the lab captures of the real clients show (the real 2.17, 2.22 and 2.31
  ask for `idle.gani`, the player images and the icons with `WANTFILE` when
  their folder is empty). With a clean Seihai the login frame is the real
  client's clean-folder frame.
* If the server has the file it is saved in the cache and found before ours. If
  it has not, it answers `FILESENDFAILED`, the name is settled for the
  connection, and the built-in file stays in use.
* Builds without `PLI_UPDATEFILE` (1.33 to 1.41) already never revalidate; for
  them a built-in-only file is `WANTFILE` too (every build has 23).
* For the pre-2.1 builds that give no mod time in the file reply the cache copy
  gets the download time (as before).

### The first-step files (decision of 2026-10-25, t122u)

The real client's clean folder has no `walk.gani`, `steps.wav` or `steps2.wav`,
and it asks for them on its first steps (2.31 `gmap_cross` capture:
`steps.wav` and `walk.gani` in the first step's frame, `steps2.wav` with the
second; `documentation/difftest-leftovers.md`, section 7). Seihai asks for one
**only if it has no copy at all**: a copy in the cache, in the user's folder,
in the built-in set, or already received this session counts as having it
(`FileSession::on_step`, `FIRST_STEP_REQUESTS`). The built-in set has all
three, so a normal run asks for none of them. They are asked for, in the real
order, when the built-in layer is unavailable and nothing else has them. Builds
with no GANIs or no `WANTFILE` ask for none (not captured). The names are asked
for once per connection (the lists are per connection), and a new connection
starts the count again.

This differs on purpose from the login-time files (`idle.gani` where the build
asks for it at login, the player images, the icons): those are part of the real
login frame and keep their request, with `WANTFILE` for a file only we have.
The first-step files are the ones the decision says to skip when present. A
consequence worth knowing: on a server with its own `walk.gani` or step sounds,
Seihai uses its built-in ones unless that server's copy is already in the cache
(an earlier visit, or copied in by hand).

The tamper check and id 24 stay opt-in, and `wbowi1.png` is not requested (the
bomb/bow default-weapon ids are not settled).

## What Seihai does

* `FileRequester` keeps the three lists and the window of four, with the rules above.
* `FileSession::need(name)` asks the store for the local copy's mtime (a
  built-in-only file has none; see above) and
  returns a `WANTFILE`, an `UPDATEFILE` or nothing. The client binary calls
  it for level files it lacks, for the images of NPCs, and for a `.gmap`
  (`sync_world` returns `gmap_local` when the grid came from a local file).
* `FileSession::process` saves every finished file, sets its mtime, settles
  replies and sends the next waiting requests.
* Profile gating: `protocol.pli` (34), `files.modtime`, and the new
  `files.download_maps`.
* `saveimages` is `save_downloads` in the settings file, and
  `--no-save-downloads` on the command line. Off: downloads stay in memory.
* Downloads go to the per-server cache folder (section above), never into the
  user's asset folder or the built-in set.

### Seihai's own choices (not what the clients do)

* A downloaded level file (`.nw`, `.graal`, `.gmap`) is always written, and
  goes straight under `levels/`. Seihai loads levels from files; the real
  clients would use the board packet and their level cache.
* The lists and the failure memory are per connection, as in the real client:
  a server warp that reconnects starts them again (`FileSession::new_connection`).
* The local copy is found by the real order (root-relative name, then the
  folders the resolver searches), in each layer in turn (cache, the user's
  folder, the built-in set). The resolver's folder order inside a layer is
  unchanged (`documentation/assets.md`).

## Leftovers worked through (2026-10-25)

Evidence came from the client binaries (disassembled, not run) and the
existing lab traces; nothing new was captured.

| Point | Result |
|---|---|
| Save folders for 1.39 and 2.00 | **Read and implemented** (`filecache::SaveLayout`). 1.33 to 1.41 (1.33, 1.34, 1.35, 1.36, 1.37, 1.38, 1.39 and 1.41 binaries checked): one folder, `levels/webgifs`, for everything saved; the 1.39 routine saves only names ending `.gif`, `.png`, `.mng` (no sound), deletes an existing file first and sets no modification time. 2.00, 2.00 rev 1 to 3, 2.01 and 2.01 rev 1: `sounds` for `.wav`; under `levels` `bomys` (names starting `bomy` or `b_`), `ganis`, `emoticons`, `heads`, `bodies`, `swords`, `shields`, else `webgifs`; **no `hats` folder** (a `hat0.png` goes to `webgifs`) and no `maps`. Hats appear in 2.02 and stay in 2.02 rev 1/2, 2.03, 2.04, 2.05 and both 2.1 test builds. The 1.42 NewMain builds differ from each other (24 has `ganis`, `heads`, `swords`, `shields`; 26 adds `sounds` and `bomys`): left on the full layout, unread. |
| 2.1 to 2.16 save folders | **Partly settled.** Both 2.1 test builds have the full list with `hats` and **no `maps` rule**; the `maps` folder first shows in 2.17.1. So `download_maps` is not on in the 2.1 test builds; the profile still carries it as on for 2.1 to 2.16 (inferred, left alone so the golden file and the chain table stay as they are). When the rule arrived between the 2.1 tests and 2.17.1 is unknown; no binary exists for 2.1 to 2.16. |
| Empty cached stream after a failed send | **Settled and implemented** (`Stored::Ignored`). All of 1.39, 2.00, 2.05 rev 2, 2.17.1, 2.22 and 2.31 start the save routine with a guard: a missing or zero-length stream returns at once. Nothing is written, an existing copy is not deleted, nothing is remembered. So the empty stream a failed send leaves behind is simply not saved, and the name is settled by the reply, as before. Seihai now does the same for a zero-length `PLO_FILE`. |
| The login set of `UPDATEFILE`s (real vs Seihai) | **Checked on the traces** (2.22 and 2.31, real client against Seihai, `labd15`): after item 15 the sets match for the player images, the icons and the gmap; the one remaining difference is `wbowi1.png`, which the real client asks for and Seihai does not (default-weapon ids, below). The level-file requests Seihai sends in its own frames are not in the real trace because the real client takes levels from the board packet and its level cache; that is a cache-state difference, not a rule. |
| Which default weapon gives `wbowi1.png` | **Evidence only, not wired** (decision: no mapping invented). The server source numbers the default items bow 7, bomb 8; the client's item-name table in 2.22 is in that order; the real 2.22 and 2.31 login asks `wbomb1.png` and then `wbowi1.png`, which is what a client walking default weapons 8 then 7 would do. A capture that names the weapon ids and images in one run would settle it. |

## Not settled (left alone)

| Open point | Why |
|---|---|
| The persistent level cache (`GWEBL001`, DES-based). The format is in Graal_analysis. | Skipped. Seihai loads levels from the board packet and never needs the cache; reading or writing it needs the key handling and a capture of the level path with the cache on (a real client run twice against the same lab level, then the cache folder and the `PLI_LEVELMODTIME` traffic compared). Not part of file transfer. |
| The scan order of the client's file index when two files have the same name in different folders. | Not read in the code that was exported; the lookup shows the `local_path` order, which Seihai uses. A capture would be: the same name placed in `levels/webgifs` and `levels/bodies`, with different bytes, and the one the client draws. |
| Whether pre-2.1 clients re-send `UPDATEFILE` for a file already settled. | The 2.05 code has no time in the reply, so "up to date" has no form there; inferred only. A 2.05 lab run against a server that answers every `UPDATEFILE` with `FILESENDFAILED` would show it. |
| Exact case folding of names. | Case-insensitive assumed. |

No capture is still needed for what is implemented.
