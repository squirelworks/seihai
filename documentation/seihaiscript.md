# Seihaiscript

Seihaiscript is the scripting engine for the seihai client and tools. It aims for practical GS1 compatibility while adding selected GS2-quality improvements.

## Design Goals

- Run existing GS1 NPC and weapon scripts with minimal changes.
- Provide a cleaner variable model, functions, and basic object support inspired by GS2.
- Stay clean-room: driven by public documentation, OpenGS1, observed server behaviour, and cleaned client observations rather than decompiled official VM code.
- Usable both inside the client (client-side scripts) and in tools (validation, generators, editors).

## Language Features (target)

### GS1 baseline (from OpenGS1, public docs, and cleaned 2.05 observations)
- Literals (numbers, strings, lists)
- Arithmetic, comparison, logical operators
- `if` / `else`, `for`, `while`
- Commands (see below)
- Built-in functions
- Dot-operator identifiers (`this.`, `player.`, `level.`, …)
- Basic arrays / string lists

### 2.05 command set (cleaned)
The 2.05 client uses a **parse-tree interpreter**, not bytecode. Command opcode = 600 + table index. The table contains **161** commands (opcodes 600..760). `show`/`hide` are 759/760. A small set of commands compile but have no executor case in 2.05 (`seturllevel`, `setfocus`, `resetfocus`, `setbacktile`, `setbacktile2`, `canwarp`, `canwarp2`, `drawovertrees`, `join`, `freezeplayer2`, `unfreezeplayer`, `setz`, `updateterrain`). Script-facing directions are 0 up / 1 left / 2 down / 3 right. Full cleaned list is in the Graal_analysis notes; Seihai should implement the subset needed for offline play first.

### Planned extensions (GS2-inspired)
- Functions with parameters and return values
- Unified numeric/string variable access
- Multi-dimensional arrays
- Object/class joining (`join` / `leave`)
- Event handlers (`onCreated`, `onPlayerChats`, …)
- Timeout / scheduling primitives
- GUI control objects (later phase)

## Implemented GS1 engine

`client/src/scripting/engine/` is a first working GS1 interpreter. It is written from observed behaviour and its own design.

**Pipeline.** `parse(src, era)` never fails: it returns a `Script` plus diagnostics, and an unknown or not-yet-introduced command becomes an `Unknown` statement that is logged and ignored when run. The tree compiles to a flat node list with jump targets. Every event runs that list from the top; `if (created)`-style blocks test the event flag. `sleep` remembers the next node and the next `timeout` event resumes there.

**Language.** `if`/`else`, `while`, `for`, `break`/`continue`/`return`, assignment and `++`/`--`, the usual numeric, comparison and logical operators (a single `=` in a condition compares), numeric and string variables, arrays (`setarray`), message codes (`#c #n #a #L #F #v #s`, and from 2.02 `#t #e #T`, from 2.03 `#p`), and the built-in functions `abs int sin cos arctan random strequals strlen startswith indexof ascii arraylen onwall playersays playersays2`. Scopes: `this.` is per object, `level.` clears on a level change, `server.` is global, `client.`/`local.`/plain names are flags.

**Text join and the chat attribute (2.1 and later).** `a @ b` joins two texts; `SPC`, `NL` and `TAB` join with a space, a line break or a tab between (the four operators of the GS2 grammar of the public GServer, where `@` binds looser than the comparisons and tighter than `&&`). `chat` (an object's bubble text, the same as the `message` command) and `player.chat` (the player's chat text, like `setplayerprop #c`) can be read and written; `player.account` and `player.nick` can be read. A level NPC script of a server that runs NPCs (`chat = player.account SPC "touched me";`) parsed with `expected ;` before. Open: scripts that are really GS2 (`temp.` variables, `this.x` methods, `{}` object literals) are still not run; and when the server runs the NPCs (`PLO_HASNPCSERVER`) a level file's NPC script is still run locally.

**Commands.** About 150 are recognised, each with a signature and the version that introduced it. Commands that change the world are returned as `Effect` values for the host to apply (messages, warps, sounds, images, weapon grants, player movement, flags, `sendtoserver`); the offline client applies the ones it can and prints the rest to stderr. The visual-effect commands do nothing when online without an NPC server, as the original does.

**Version gating.** `Era` follows the client version: 2.00 baseline, then 2.01 (`hitobjects`, `playerdies`...), 2.02 (string functions, `local.`), 2.03 (`triggeraction`, `action*` events, `move` with time), 2.04 (`replaceani`, tile defs, `attachplayertoobj`...), 2.05 (`loadmap`, `setbacktile2`, `updateterrain`) and 2.1 and later (`ascii`, `playerhp`...). A command or event from a later era is parsed as unknown on an earlier one.

**Scheduler.** Ticks are 50 ms. Timeouts round up to a tick and fire in order of tick, then creation. The same script and the same seed produce the same run. Limits: 10,000 loop passes, 200,000 steps per run, call depth 64, expression depth 400, parser depth 200. `callnpc`, `callweapon`, `triggeraction` and `movementfinished` are queued and run after the caller finishes (at most 1,000 per drain).

**Touch.** `playertouchsme` is edge-triggered against the NPC's rectangle (2 by 2 tiles by default; `setshape 1,w,h` is in pixels), tested with the player's foot point.

**Host.** `scripting::engine::host::SessionScripts` wires an offline session to the engine. Scripts run by default; `--no-scripts` turns them off.

**Not done.** `board[x,y]` and `tiles[x,y]` reads return 0; no string lists or `with`/`join`; online NPC script props are not decoded; no weapon-key input for `weaponfired`; `move` is instant; script warps online use the link request (see `documentation/script-warps.md`); sound and network effects are only logged; image effects (`setimg`, `setimgpart`, `showimg`, `hideimg`, `changeimgpart`, `setcharani`, `setani`, `showcharacter`) change the NPC and are drawn (`documentation/npc-images.md`). Assumptions that still need the lab: case-insensitive `==`/`strequals`, quote stripping, the touch geometry, and `playersays` matching rules.

## Architecture

```
scripting/
├── lexer
├── parser
├── compiler (optional bytecode)
├── vm / interpreter
├── builtins/          # Commands and functions registered by the host
└── host/              # Client or tool bindings
```

The engine exposes a host API so the client can register movement, drawing, networking, and flag commands, while a headless tool can register a reduced set.

## Relationship to OpenGS1

OpenGS1 (`../shanalysis/OpenGS1`) is the primary reference implementation for the GS1 subset. Seihaiscript may start by wrapping or reimplementing it, then diverge as GS2 features are added. Note that OpenGS1 is a reimplementation; the 2.05 client itself does not use its bytecode.

## Documentation Sources

- Graal Bible pages on GScript / GS1→GS2 migration (public)
- OpenGS1 source and tests
- Server-side script handling in GServer-v2
- Cleaned 2.05 command observations (Graal_analysis notes)
- Archived Classic scripting corpus (see `documentation/legacy-scripts.md` for patterns useful for host effects and open items)

Detailed language semantics and edge-case notes go in `../shworkingpapers/scripting/`.
