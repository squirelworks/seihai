# #400 recheck: 5.x nicknames with seihai-core v0.1.8 (#509)

This rechecks finding 1 of the [#400 run](README.md): a 5.x player showed as `default` to 4.x clients.

**Setup:**
- `seihai-ato` from main `1f428c8`, which already has #509 merged (seihai-core / seihai-draw v0.1.8).
- The same private stack as the #400 run: seihai-listserver plus Lab Modern on seihai-gserver-v2 `101a2cce`, in a loopback-only namespace.
- Both clients ran on Xvfb in the Linux lab VM.

**Each pair:**
- A 5.x Ato with `--account X --nick X`, and a 4.20r8 Ato as the viewer, both logged in to Lab Modern with `--online 127.0.0.1:14805`.
- The 5.x client walked right, then the viewer's window was captured.

## Result: pass, all four checks

| Check | 5.12 | 5.00r7 |
|---|---|---|
| Ato log | `sent the nickname "lab509five"` (after `logged in at 127.0.0.1:14805 as lab509five (5.12)`) | `sent the nickname "lab509fivezero"` (after `logged in at 127.0.0.1:14805 as lab509fivezero (5.00r7)`) |
| The gserver holds it | `PlayerProp::NICKNAME: value: *lab509five` | `PlayerProp::NICKNAME: value: *lab509fivezero` |
| The 4.20r8 window's label | `*lab509five` (`f512-420-view.png`) | `*lab509fivezero` (`f500-420-view.png`) |

**About the gserver values:**
- The `*` prefix is the gserver's own marking, the same as 4.x gets (`*lab400d` in the first run).
- `default` still shows up in the gserver's dump, but only in the props it sends at login, before the client's nickname packet arrives.
- After that packet, the 5.x players carry their own nickname (`nickname-dump.txt`), and the 4.20r8 viewers draw it.

## Files (`recheck-509/`)

- `g-f512-stdout.txt` and `g-f500-stdout.txt`: the 5.x clients' stdout
- `g-f512v-stdout.txt` and `g-f500v-stdout.txt`: the 4.20r8 viewers' stdout
- `nickname-dump.txt`: the gserver's player props carrying the new nicknames
- `f512-420-view.png` and `f500-420-view.png`: the 4.20r8 windows
