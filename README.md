# Seihai

An open client for Graal Reborn listservers and gservers. It plays offline levels and connects to public servers. Written in Rust.

## First time setup

1. Install Rust from [rustup.rs](https://rustup.rs/).
2. Clone the repo:

```bash
git clone https://github.com/squirelworks/seihai.git
cd seihai
```

You must be inside the `seihai` folder (the one that contains `Cargo.toml` and `.git`) before any of the commands below. If you are in a parent folder, `git` and `cargo` will fail with "not a git repository" and "could not find Cargo.toml".

## From a downloaded zip (fresh install only)

These steps create a new `~/Seihai/seihai`. They delete any folder already at that path, including an assets folder you put there. Do not use them to update an install you already have.

GitHub's "Download ZIP" of the main branch gives `seihai-main.zip`. It extracts to a folder named `seihai-main`. Put the zip in `~/Seihai` and run:

```bash
mkdir -p ~/Seihai
cd ~/Seihai
rm -rf seihai-main
python3 -m zipfile -e seihai-main.zip ~/Seihai
rm -rf seihai
mv seihai-main seihai
cd seihai
cargo build --release -p seihai
./target/release/seihai
```

You end up in `~/Seihai/seihai`, which is the folder that contains `Cargo.toml`. You must be inside that folder before `cargo`. A zip has no `.git`.

If `~/Seihai/seihai` already exists and is a git clone, leave it alone and use the update steps below. That keeps your assets and your settings.

On Windows (PowerShell), for a fresh extract only:

```powershell
cd seihai
cargo build --release -p seihai
.\target\release\seihai.exe
```

## Update and build (from a git clone)

```bash
cd seihai
git pull origin main
cargo build --release -p seihai
```

This creates the binary at:

- Linux / macOS: `target/release/seihai`
- Windows: `target\release\seihai.exe`

`cargo build` only builds. It does not start the program. To build and run in one step:

```bash
cargo run --release -p seihai
```

`cargo run` uses the same `target` folder. On Windows, if you do not see a `target` directory, check that you are in the `seihai` folder (not a parent folder) and that the build finished without errors.

## Assets

Seihai ships a small set of its own default art, sounds and two starter levels (`assets/default/`, compiled into the program), so a fresh download opens a level and walks with no data folder at all: leave the Assets box empty and use "Play offline". To use your own files, put them in a folder of your choice and type its path into the Assets box. A file your folder lacks comes from the built-in set, file by file. Your folder may hold:

- `levels/` — `.nw`, `.graal`, or `.zelda` levels
- `bodies/body.png` and `heads/head0.png` — player sprites
- `tiledefs.txt` or `arrays.dat` — collision data
- `pics1.png` — a 2048×512 tile sheet

Files a server sends are saved in a per-server cache folder (`~/.config/seihai/cache/<host>_<port>~<name>/`), searched before your folder and before the built-in set. They are never written into your folder or the built-in set. See `documentation/file-cache.md`.

## Run

From the `seihai` folder:

```bash
./target/release/seihai
```

On Windows:

```bash
target\release\seihai.exe
```

The launcher opens. Pick a version, enter your account, and click Connect. Offline play is also available from the launcher.

To skip the launcher:

```bash
./target/release/seihai --assets /path/to/assets --online listserver.graal.in:14923 --account NAME --password PASS --version 2.05
```

To keep the password off the command line, put it in an environment variable and pass the name:

```bash
export SEIHAI_PASSWORD=...
./target/release/seihai --assets /path/to/assets --online listserver.graal.in:14922 --account NAME --password-env SEIHAI_PASSWORD --version 2.22
```

To see what a listserver offers without logging in to anything:

```bash
./target/release/seihai --online listserver.graal.in:14922 --account NAME --password-env SEIHAI_PASSWORD --version 2.22 --list-only
```

The list is fetched as the version you pick, on that version's listserver login: one plain frame for 2.05 to 2.17, a keyed gen-5 login for 2.22 and later. Servers that send the status and URLs in a second frame are handled for both.

Host names are resolved with the system resolver, and every address it returns is tried in turn (5 s each). If your resolver gives wrong or no answers for a name, use the server's IP address in `--online` instead.

| Flag | Meaning |
|------|---------|
| `--assets`, `-a` | Your own asset folder (optional). A file it lacks comes from the built-in default set; with no folder the built-in set is all there is |
| `--cache-dir` | Root of the per-server download caches (default `<settings dir>/cache`; settings file `cache_dir=`). Search order: the server's cache, `--assets`, the built-in set |
| `--level`, `-l` | Level file to load offline |
| `--online`, `-o` | `host:port` of a listserver or gserver |
| `--account` | Account name |
| `--password` | Password (visible in the process list; prefer `--password-env`) |
| `--password-env` | Name of an environment variable that holds the password. Warns on stderr if it is unset or empty |
| `--list-only` | With `--online`, fetch and print the server list from a listserver, then exit. Never prompts and never logs in to a gserver. Exit code 1 if the fetch fails |
| `--check-assets` | Resolve and load the files a version uses from the `--assets` folder (or from the built-in set when none is given) and print what was found (one version with `--version`, else the five main ones). Exit code 1 if a required file is missing or unreadable |
| `--level-info FILE` | Print sizes, section offsets and record counts of a binary `.graal` / `.zelda` level (no content); repeat for more files. Exit code 1 if a file does not parse, or if a `Z3-V1` file does not read to its end with no anomaly |
| `--version-chain` | Print what each client build changed against the build before it (all builds, or the one given with `--version`). `--chain-format text\|md\|tsv` picks the form (`documentation/version-chain.md`) |
| `--version` | Client version (`1.39`, `2.05`, `2.17`, `2.22`, `2.31`). Wins over the version the launcher saved, even when it is the default `2.05`; the saved one applies only when the flag is absent |
| `--no-serverwarp` | Do not follow a server warp (`PLO_SERVERWARP`, 178). By default Seihai reconnects to the host and port the server names and logs in again with the same account and password, as the real client does |
| `--serverwarp` | Follow server warps even if the settings file turns them off |
| `--no-scripts` | Do not run client-side GS1 scripts (level NPCs, weapons) |
| `--no-sound` | Do not play sound effects (the `PLAYSOUND` cues of `walk.gani` and `swim.gani`). By default they play from `sounds/` in the server cache, `<assets>` or the built-in set (first that has the file), if a sound device opens. Otherwise sound is silent and everything else runs the same |
| `--sound` | Play sound effects even if the settings file turns them off |
| `--no-save-downloads` | Keep downloaded files in memory only. By default a file the server sends is also written to the server's cache folder, in the place the real client uses (`documentation/file-cache.md`) |

### Capture viewer

`seihai capture` reads files only (no window, no network). It decodes a lab capture with Seihai's own codecs and prints the messages with their names and field layouts. `documentation/capture-viewer.md` has the details and examples.

| Command | Meaning |
|---------|---------|
| `seihai capture show FILE` | Print the messages of a pcap, or of a `difftest trace` file. The version is read from the login token (or the trace header) unless `--version` is given |
| `seihai capture layout ID\|NAME` | Print one packet's field layout across the version chain, with the fields the tables do not settle marked `?` (unknown) or `~` (partly decoded) |
| `seihai capture tables` | List the tables in use and check them |

| `show` flag | Meaning |
|-------------|---------|
| `--version V` | Client version, for example `2.22` |
| `--tables DIR` | Read `packet-layouts.tsv` (and `version-chain.tsv`) from `DIR` instead of the built-in tables |
| `--dir c2s\|s2c`, `--id N,NAME`, `--name NAME`, `--conn gs0` | Keep only that direction, those packets (by id or name, `PLI_`/`PLO_` prefixes pick a direction) or that connection |
| `--time A..B`, `--frame A..B` | Keep a time range (milliseconds since the connection started) or a range of frame numbers |
| `--limit N` | Stop after N messages |
| `--layouts` | Print the table layout under every message |
| `--raw` | Hex dump of every payload. Undecoded payloads are dumped without this flag |
| `--format text\|tsv\|json` | Output format (default `text`) |
| `--out FILE` | Write to `FILE` with mode 600. Refused if `FILE` already exists with a looser mode and the capture contains a login |
| `--no-mask` | Keep the key, player ids and account name. The password is never shown |

Login frames are redacted by default and their raw bytes are never printed. Keep capture files in a private folder; never commit them.

### Chat commands and list files

In the chat bar (Enter) these are commands, not chat. A name is an account name or a nickname, in any case.

| Command | Effect |
|---------|--------|
| `/ignore NAME`, `/unignore NAME` | Hide that player's bubbles, chat, toalls and PMs. Local only: nothing is sent |
| `/buddy NAME`, `/unbuddy NAME` | Mark the player in the player list (F7). Local only: nothing is sent |
| `/mute NAME`, `/unmute NAME` | As ignore, and sends `PLI_MUTEPLAYER` (43) for a player in the level. Only on versions that send it (2.17 and later); at most 5 sends per session |
| `/lists`, `/lists buddy` | Show the lists |

The lists are plain text files in the settings directory (`~/.config/seihai/`, the folder `seihai.conf` is in): `buddyaccounts.txt`, `ignoredaccounts.txt` and `mutedaccounts.txt`. One account name per line; case does not matter; blank lines, `#` comments and any line ending are fine; a name that is already there is kept once. They hold names only, never a password. A file is replaced whole (written aside, then renamed), and removed when its list becomes empty. Edit them by hand while Seihai is not running; they are read when it connects. `buddyaccounts.txt` and `ignoredaccounts.txt` are the real client's names; the mute file is Seihai's own.

### Settings file

Seihai reads plain `key=value` lines (`#` starts a comment) from `~/.config/seihai/seihai.conf` (on Windows `%USERPROFILE%\.seihai\seihai.conf`). The launcher writes most keys itself: `assets`, `cache_dir`, `account`, `password`, `version`, `listserver`, `last_server`, `last_server_name`, `direct_server`, `offline=1` and `assets.<version>`. A command-line flag always overrides the same option in the file.

Three options are meant to be edited by hand:

```
# Do not follow a server warp (default: 1, follow it)
follow_serverwarp=0
# Do not play sound effects (default: 1)
sound=0
# Do not write downloaded files to disk (default: 1)
save_downloads=0
# Print script diagnostics (default: 0, off)
debug=1
```

`follow_serverwarp` accepts `1`/`0`, `true`/`false`, `yes`/`no` and `on`/`off`. A missing or unreadable value means follow. Precedence is `--serverwarp` or `--no-serverwarp`, then the file, then the default (on). Following sends your stored account and password to the host the server names, so turn it off if you point Seihai at servers you do not trust.

`debug` takes the same values and defaults to off. With it on, every script message (an ignored command, an unknown function, an unhandled effect, a parse error) is written to `_debug/debug-<YYYYMMDD-HHMMSS>/script.log` in the directory you run Seihai from. The folder is named for the time that process started, and the file is opened once. The file is recreated each run and holds one session. Without it, those lines are silent. F3 still toggles the on-screen debug overlay for the current run.

For 1.38 and later, `keyset_<action>_key0` to `_key2` lines (the `graalconfig.txt` form) or `keyset_<action>=S,Space` rebind the game keys. The defaults are the readme's: arrows or I/J/K/L, A action, S or Space sword, D item, M map, Q item list, Tab chat (F7 shows the player list). See `documentation/keys.md`.

`sound` and `save_downloads` take the same values. Precedence is `--sound` or `--no-sound`, then the file, then the default (on). See `documentation/audio.md` for what plays. Audio is on by default; on Linux the build needs the ALSA development files (`libasound2-dev`). To build without it: `cargo build --release -p seihai --no-default-features --features dev`.

## Features

- Offline play with `.nw`, `.graal`, and `.zelda` levels
- Online play against GServer-RS and lab GServer-v2
- Listserver login and server list
- Player sprites, chests, signs, and NPC images (requested if missing)
- Chat bar, PMs, player list, and debug overlay
- Chat bubbles that expire (your own line stays until you move), a fading chat overlay (Seihai's own, timings assumed: `documentation/chat-log.md`), and buddy, ignore and mute lists (chat bar commands `/ignore`, `/buddy`, `/mute`, `/lists`)
- Resizable window, scaled 2× by default

## Known issues

- Tiledefs collision is incomplete
- NPC images arrive after the level loads; a gray placeholder shows until then
- Some packet ids are decoded but not yet applied to gameplay

## Documentation

See `documentation/` for the implementation notes, roadmap, and version profiles.

Level-editor expansion notes (fixtures, mask-aware painting, next slices) are in `documentation/level-editor-expansion.md`.
