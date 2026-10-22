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

Detailed language semantics and edge-case notes go in `../shworkingpapers/scripting/`.