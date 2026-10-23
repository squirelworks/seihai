# Seihai

An open client for Graal Reborn listservers and gservers. It plays offline levels and connects to public servers. Written in Rust.

## Prerequisites

Install [Rust](https://rustup.rs/). The build uses `cargo`.

## Build

```bash
git pull origin main
cargo build --release -p seihai
```

The binary is at `target/release/seihai`.

On Windows, if the build appears to hang, run:

```bash
cargo run --release -p seihai
```

## Assets

The client needs a `pics1.png` tile sheet (2048×512). It is not included. Place it in a folder, then enter that folder path in the Assets field of the launcher. Optional files:

- `levels/` — `.nw`, `.graal`, or `.zelda` levels
- `bodies/body.png` and `heads/head0.png` — player sprites
- `tiledefs.txt` or `arrays.dat` — collision data

## Run

With no arguments, the launcher opens:

```bash
./target/release/seihai
```

The launcher has a side panel for the server list. Choose a version, enter account details, and click Connect. Offline play is also available from the launcher.

CLI flags skip the launcher:

```bash
./target/release/seihai --assets /path/to/assets --online listserver.graal.in:14922 --account NAME --password PASS --version 2.05
```

| Flag | Meaning |
|------|---------|
| `--assets`, `-a` | Folder containing `pics1.png` |
| `--level`, `-l` | Level file to load offline |
| `--online`, `-o` | `host:port` of a listserver or gserver |
| `--account` | Account name |
| `--password` | Password |
| `--version` | Client version (`1.39`, `2.05`, `2.17`, `2.22`, `2.31`) |

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
