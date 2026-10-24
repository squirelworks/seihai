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

## Update and build

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

The client needs a `pics1.png` tile sheet (2048×512). It is not included. Put it in a folder of your choice, then type that folder path into the Assets box in the launcher. Optional files:

- `levels/` — `.nw`, `.graal`, or `.zelda` levels
- `bodies/body.png` and `heads/head0.png` — player sprites
- `tiledefs.txt` or `arrays.dat` — collision data

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
./target/release/seihai --assets /path/to/assets --online listserver.graal.in:14922 --account NAME --password PASS --version 2.05
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
| `--assets`, `-a` | Folder containing `pics1.png` |
| `--level`, `-l` | Level file to load offline |
| `--online`, `-o` | `host:port` of a listserver or gserver |
| `--account` | Account name |
| `--password` | Password (visible in the process list; prefer `--password-env`) |
| `--password-env` | Name of an environment variable that holds the password. Warns on stderr if it is unset or empty |
| `--list-only` | With `--online`, fetch and print the server list from a listserver, then exit. Never prompts and never logs in to a gserver. Exit code 1 if the fetch fails |
| `--version` | Client version (`1.39`, `2.05`, `2.17`, `2.22`, `2.31`) |
| `--no-serverwarp` | Do not follow a server warp (`PLO_SERVERWARP`, 178). By default Seihai reconnects to the host and port the server names and logs in again with the same account and password, as the real client does |
| `--serverwarp` | Follow server warps even if the settings file turns them off |
| `--no-scripts` | Do not run client-side GS1 scripts (level NPCs, weapons) |
| `--no-sound` | Do not play sound effects (the `PLAYSOUND` cues of `walk.gani` and `swim.gani`). By default they play from `<assets>/sounds`, if the build has the `audio-rodio` feature and a sound device opens. Otherwise sound is silent and everything else runs the same |
| `--sound` | Play sound effects even if the settings file turns them off |

### Settings file

Seihai reads plain `key=value` lines (`#` starts a comment) from `~/.config/seihai/seihai.conf` (on Windows `%USERPROFILE%\.seihai\seihai.conf`). The launcher writes most keys itself: `assets`, `account`, `password`, `version`, `listserver`, `last_server`, `last_server_name`, `direct_server`, `offline=1` and `assets.<version>`. A command-line flag always overrides the same option in the file.

Two options are meant to be edited by hand:

```
# Do not follow a server warp (default: 1, follow it)
follow_serverwarp=0
# Do not play sound effects (default: 1)
sound=0
```

`follow_serverwarp` accepts `1`/`0`, `true`/`false`, `yes`/`no` and `on`/`off`. A missing or unreadable value means follow. Precedence is `--serverwarp` or `--no-serverwarp`, then the file, then the default (on). Following sends your stored account and password to the host the server names, so turn it off if you point Seihai at servers you do not trust.

`sound` takes the same values. Precedence is `--sound` or `--no-sound`, then the file, then the default (on). See `documentation/audio.md` for what plays and for the optional `audio-rodio` build (`cargo build --release -p seihai --features audio-rodio`, needs the ALSA development files on Linux).

## Features

- Offline play with `.nw`, `.graal`, and `.zelda` levels
- Online play against GServer-RS and lab GServer-v2
- Listserver login and server list
- Player sprites, chests, signs, and NPC images (requested if missing)
- Chat bar, PMs, player list, and debug overlay
- Resizable window, scaled 2× by default

## Known issues

- Tiledefs collision is incomplete
- NPC images arrive after the level loads; a gray placeholder shows until then
- Some packet ids are decoded but not yet applied to gameplay

## Documentation

See `documentation/` for the implementation notes, roadmap, and version profiles.
