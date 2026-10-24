# File transfer and caching rules (queue item 9)

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

## What Seihai does

* `FileRequester` keeps the three lists and the window of four, with the rules above.
* `FileSession::need(name)` asks the store for the local copy's mtime and
  returns a `WANTFILE`, an `UPDATEFILE` or nothing. The client binary calls
  it for level files it lacks, for the images of NPCs, and for a `.gmap`
  (`sync_world` returns `gmap_local` when the grid came from a local file).
* `FileSession::process` saves every finished file, sets its mtime, settles
  replies and sends the next waiting requests.
* Profile gating: `protocol.pli` (34), `files.modtime`, and the new
  `files.download_maps`.
* `saveimages` is `save_downloads` in the settings file, and
  `--no-save-downloads` on the command line. Off: downloads stay in memory.

### Seihai's own choices (not what the clients do)

* A downloaded level file (`.nw`, `.graal`, `.gmap`) is always written, and
  goes straight under `levels/`. Seihai loads levels from files; the real
  clients would use the board packet and their level cache.
* The lists and the failure memory are per connection, as in the real client:
  a server warp that reconnects starts them again (`FileSession::new_connection`).
* The local copy is found by the real order (root-relative name, then the
  folders the resolver searches). The resolver's own order for an asset
  lookup is unchanged (`documentation/assets.md`).

## Not settled (left alone)

| Open point | Why |
|---|---|
| The persistent level cache (`GWEBL001`, DES-based). The format is in Graal_analysis. | Needs the key handling and a capture of the level path with the cache on. Not part of file transfer. |
| The login set of default `UPDATEFILE`s and whether they are bundled in one frame. | The lab shows three images (and the gmap) but the full list needs a capture with an empty and a full data folder; item 15. |
| The scan order of the client's file index when two files have the same name in different folders. | Not read in the code that was exported; no capture. The `local_path` order is the one the lookup code shows. |
| Routing of downloads in 1.39 / 2.00, and in 2.1 to 2.16. | 2.1 to 2.16 are inferred from the neighbours (`download_maps` on); 1.39 and 2.00 save routines were not read. |
| Whether pre-2.1 clients re-send `UPDATEFILE` for a file already settled. | The 2.05 code has no time in the reply, so "up to date" has no form there; inferred only. |
| What an empty cached stream after `FILESENDFAILED` does beyond "not asked again". | Not captured. |
| Exact case folding of names. | Case-insensitive assumed. |

No capture is still needed for what is implemented. Items 14 and 15 (lab runs
with Seihai on a private server) are where the open rows would be checked.
