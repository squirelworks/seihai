# Seihai

Seihai is an open client for Graal, written from scratch in Rust. It plays
offline levels and connects to Graal Reborn listservers and game servers, as
the version of the original client you choose. It covers four lines of
clients:

- **Mae** for 1.x and 2.x;
- **Ato** for 3.x to 5.x (it draws the level flat in 2D for now, see
  `crates/seihai-ato/README.md`);
- **Issho** for 6.x (`crates/seihai-issho/README.md`);
- **G3D** for Graal3D, the February 2000 3D client (`documentation/graal3d.md`).

All four are built into one program, `seihai`. It ships with its own small set
of art, sounds and levels, so it runs with no Graal files at all, and it uses
your own Graal files when you point it at them.

Seihai is part of a larger project. Its shared library, with the protocol,
assets, scripting, offline simulation and version profiles, is
[seihai-core](https://github.com/squirelworks/seihai-core). The servers are
[seihai-gserver](https://github.com/squirelworks/seihai-gserver),
[seihai-listserver](https://github.com/squirelworks/seihai-listserver),
[seihai-npcserver](https://github.com/squirelworks/seihai-npcserver),
[seihai-rc](https://github.com/squirelworks/seihai-rc) and
[seihai-servertools](https://github.com/squirelworks/seihai-servertools), plus the
GPL fork [seihai-gserver-v2](https://github.com/squirelworks/seihai-gserver-v2).
The research notes behind all of it are in
[Graal_analysis](https://github.com/squirelworks/Graal_analysis).

## Build and update

1. Install Rust from [rustup.rs](https://rustup.rs/).
2. Clone the repo and build:

```bash
git clone https://github.com/squirelworks/seihai.git
cd seihai
cargo build --release -p seihai
```

Run every command from inside the `seihai` folder, the one with `Cargo.toml` in
it; from a parent folder, `git` and `cargo` fail with "not a git repository" and
"could not find Cargo.toml". The program is then at `target/release/seihai`
(`target\release\seihai.exe` on Windows). `cargo run --release -p seihai`
builds and starts it in one step.

To update later:

```bash
cd seihai
git pull origin main
cargo build --release -p seihai
```

On Linux, sound needs the ALSA development files (`libasound2-dev`) to build;
[documentation/using-seihai.md](documentation/using-seihai.md#settings-file)
says how to build without sound. If you downloaded a zip of the repository
instead of cloning it, the same page has steps for a fresh install from it.

A plain `cargo build --release` also builds each client's own development
binary (`seihai-mae`, `seihai-ato`, `seihai-issho`, `seihai-g3d`) and the
offline level editor in `tools/level-editor`.

## Playing

`seihai` with no arguments opens the start window. Its Version list shows the
main versions Graal Reborn's GServer-v2 knows (1.39, 1.41, 2.05, 2.17, 2.22,
2.31, 3.04, 4.04r2, 4.20r8, 5.00r7, 5.12, 5.31r4 and 6.03r7), grouped by client,
then Graal3D; "All versions" adds every other build, including the betas and
test releases that only play offline.

- **Online:** fill in the account and password, press "Server list", pick a
  server and Join. The version's client logs in at once. A lost connection, or
  closing the game, brings you back to the start window.
- **Offline:** for a 1.x or 2.x version, "Play offline" and "Play the tutorial"
  start Mae with no server.

Each version can have its own asset folder, which wins over the shared one. The
password is kept only for the run; the rest of the form is saved in
`launcher.conf` next to `seihai.conf`.

Each client also runs on its own, from the command line:
`seihai mae|ato|issho|g3d ARGS...`, or `seihai --version V ARGS...` to pick the
client by version. For example, to fetch a server list and join from the
command line, keeping the password in an environment variable:

```bash
export SEIHAI_PASSWORD=...
./target/release/seihai --online listserver.graal.in:14922 --account NAME \
  --password-env SEIHAI_PASSWORD --version 2.22
```

`seihai --help` prints the usage. Every flag, the capture viewer, the chat commands (`/ignore`, `/buddy`, `/mute`)
and the settings file are described in
[documentation/using-seihai.md](documentation/using-seihai.md).

## Assets

Seihai ships a small set of its own default art, sounds, two starter levels and a short tutorial (`assets/default/`, compiled into the program through seihai-core, which keeps its own copy of the folder: after `tools/gen_default_assets.py` regenerates it here, copy the change to [seihai-core](https://github.com/squirelworks/seihai-core) too), so a fresh download opens a level and walks with no data folder at all: leave the Assets box empty and use "Play offline", and you start in the tutorial. "Play the tutorial" in the start window opens it with any Assets folder; the tutorial always uses its own art. See `documentation/tutorial-levels.md`. The built-in player is a squirrel (Seihai's mascot) with an acorn for the money icon; the tiles, hearts and sounds are Seihai's own simple art.

**Using your Graal files instead.** Point Seihai at a Graal data folder and its files replace the squirrel defaults automatically, file by file: type the folder's path into the Assets box (or pass `--assets`, or put `assets=PATH` in the settings file). Nothing is copied or converted. A Graal install folder (for example "Graal Reborn 2.22.0") already has the right layout; for a hand-made folder, put each file at one of these places:

| Graal file | Where in your folder | Replaces |
|--|--|--|
| `pics1.png` | the folder itself | the tile sheet |
| `body.png` (or `.gif`) | `bodies/` or `levels/bodies/` | the squirrel body and tail |
| `head0.png` (or `.gif`) | `heads/` or `levels/heads/` | the squirrel head |
| `sword1.png`, `shield1.png` | `swords/`, `shields/` (or under `levels/`) | the sword and acorn shield |
| `state.png`, `sprites.png` | the folder itself | the HUD (hearts, acorn) and shadows, splashes, carried objects |
| `*.gani` | `gani/` or `levels/ganis/` | the animations |
| `*.wav` | `sounds/` or `levels/sounds/` | the sounds |
| `arrays.dat` or `tiledefs.txt` | the folder itself | the tile types (walls, water) |
| `.nw`, `.graal`, `.zelda` levels | `levels/` or the folder itself | the starter levels |

Names are matched without regard to case. A file you have not placed keeps the squirrel default, so a folder with only `pics1.png` gives Graal tiles and the squirrel player. Builds up to 2.05 read player art as `.gif` only: if your folder has a `bodies` or `heads` folder, those builds use only your GIFs there. `--check-assets --assets PATH` lists which files were found where. The tutorial levels always use their own art.

Files a server sends are saved in a per-server cache folder (`~/.config/seihai/cache/<host>_<port>~<name>/`), searched before your folder and before the built-in set. They are never written into your folder or the built-in set. See `documentation/file-cache.md`.

## What works

- Offline play with `.nw`, `.graal` and `.zelda` levels, and a built-in tutorial
- Online play against Graal Reborn servers, GServer-v2 and seihai-gserver
- Listserver login and server list, for each client version's own login
- Player sprites, chests, signs and NPC images (asked for from the server if missing)
- Client-side scripts in level NPCs and weapons
- Chat bar, chat bubbles, PMs, the player list, and buddy, ignore and mute lists
- A resizable window, scaled 2x by default, and a debug overlay (F3)

## Known issues

- Tiledefs collision is incomplete.
- NPC images arrive after the level loads; a gray placeholder shows until then.
- Some packet ids are decoded but not yet applied to gameplay.
- Ato draws the 3.x to 5.x clients' levels flat, in 2D.

## Documentation

`documentation/` holds the implementation notes, the roadmap and the version
profiles. Notes on the level editor's next steps are in
`documentation/level-editor-expansion.md`.

## macOS builds

`.github/workflows/macos.yml` builds, tests and lints the workspace on GitHub's hosted macOS 15 runners (Apple silicon and Intel), then runs `tools/launcher-smoke.sh`: the release `seihai` start window joins a loopback stand-in with 2.22, 3.04, 5.12, 6.03r7 and Graal3D, runs each client in the same process and comes back to the start window when the connection drops. It runs on demand, weekly, and on a PR that changes the launcher, the smoke or the workflow; `ci.yml` runs Linux and Windows on every PR, and the same smoke on Linux under Xvfb. The lab's Sequoia VM has no Metal device, so Mae's game window cannot open there.

## Versioning

[VERSIONING.md](VERSIONING.md) explains when a release is made, what the version
numbers mean and what 1.0.0 needs. [RELEASES.md](RELEASES.md) has the release
points for each version.

## Licence

Seihai is licensed under either of [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option, the same as
[seihai-core](https://github.com/squirelworks/seihai-core) and the
`license = "MIT OR Apache-2.0"` already in `Cargo.toml`. Unless you say
otherwise, a contribution you submit is licensed the same way.

Seihai is an independent fan project. It is not affiliated with, endorsed by,
or connected to GraalOnline. "Graal" is used only to identify the games and
servers Seihai works with.
