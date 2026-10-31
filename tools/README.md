# Tools

- `editors/` — level editor, gani editor, etc.
- `generators/` — terrain, dungeon, world generators
- `converters/` — format conversion utilities

## Present

- `gen_default_assets.py` — writes the original default asset set `assets/default/` (see `documentation/default-assets.md`); `--check` verifies the committed tree, `--preview` renders a contact sheet.
- `capture-run.sh` — one online Seihai run with a `tcpdump` capture (Linux): `tools/capture-run.sh --target HOST:PORT [--version V] [--account A] [--password-file F]`. The target is required; there is no default server. It captures ports 14800–14999 plus the target's port, copies the run's `_debug` folder (with `debug=1`), and packs everything in a private `_debug/run-*.tar.gz`. The password comes from a file, `GRAAL_PW` or a hidden prompt, and never reaches the command line. Decode with `seihai capture show`.
- `capture-run.ps1` — the same for Windows (dumpcap from Wireshark, or pktmon in an elevated PowerShell), with the same options. Keep the two in step.
- `run_gserver_check.sh` — a shorter one-login check against one game server with `tcpdump`: `tools/run_gserver_check.sh /path/to/seihai HOST:PORT [version]` (account `guest` unless `ACCOUNT=` is set; hidden password prompt; needs sudo).
