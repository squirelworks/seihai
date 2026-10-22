# Seihai

Open Graal Reborn-compatible client (Rust).

## Run (Windows / Linux / macOS)

No system SDL2 is required. The window uses a software framebuffer.

```bash
cargo run -p seihai -- --assets /path/to/assets --level start.nw
```

The asset directory should contain:
- `pics1.png` (2048×512 tile sheet)
- `tiledefs.txt` or `arrays.dat` (optional)
- `levels/` with `.nw` files (optional)
- `gani/` with `.gani` files (optional)

Arrow keys move the player. Escape closes the window.

## Build

```bash
cargo build --release -p seihai
```

The binary is at `target/release/seihai` (or `seihai.exe` on Windows).