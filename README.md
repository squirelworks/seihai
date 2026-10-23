## Menu

Run with no arguments for a terminal menu:

```bash
./target/release/seihai
```

Choose offline or online, then enter the asset path, level or server, account, and version. All CLI flags still work and skip the menu.

## Build

Make sure you have everything you need to build a Rust program. Essentially, "cargo".

```bash
cargo build --release -p seihai
```

However, on Windows, it doesn't seem to do anything so I just straight up go and do this:

```bash
cargo run --release -p seihai

## Assets

Also, you will probably need to bring your own pics1.png so, uh, sorry but that is not included in this repository.

Make a folder in the main seihai directory and, in the GUI (I am unsure of the CLI method), type the folder name in the "Assets" textbox.

## Bugs

Tiledefs are not working properly. The start-up GUI needs some work since it does not resize the window for the incoming server list. Also, the game GUI does not have a way to resize itself, either.

Work-in-progress.
