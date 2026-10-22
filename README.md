# Seihai

Open Graal Reborn-compatible client (Rust).

## Run (Windows / Linux / macOS)

No system SDL2 is required. The window uses a software framebuffer.

```bash
cargo run -p seihai -- --assets /path/to/assets --level onlinestartlocal.nw
```

The asset directory should contain:
- `pics1.png` (2048×512 tile sheet)
- `levels/bodies/body.png` and `levels/heads/head.png` (optional; falls back to a marker)
- `tiledefs.txt` or `arrays.dat` (optional)
- `levels/` with `.nw` files
- `gani/` or `levels/ganis/` with `.gani` files (optional)

Arrow keys move the player. Escape closes the window.

## Build

```bash
cargo build --release -p seihai
```

The binary is at `target/release/seihai` (or `seihai.exe` on Windows).

## Status

- Offline level load, collision, and movement
- Gen-3 and gen-5 codecs verified against lab captures
- Version toggle (1.39–2.31)
- Software tile renderer with player body/head sprites
- Chest/sign/NPC markers and simple HUD bars
- Cross-platform window (minifb)

See `documentation/implementation.md` for the full slice.